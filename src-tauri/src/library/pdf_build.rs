//! Observable, cancellable Source metadata work. Never stores source text in
//! progress, logs or job history. Parsing runs without the database lock.
use super::{adapters::pdf, original, pdf_structure};
use crate::{
    db::{library, pdf_structures, Database},
    error::{AppError, AppResult, UserMessage},
};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub stage: String,
    pub milliseconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub status: String,
    pub stage: String,
    pub done: usize,
    pub total: usize,
    pub elapsed_ms: u64,
    pub timings: Vec<Timing>,
    pub heading_count: Option<usize>,
    pub indexed_links: Option<usize>,
    pub error_code: Option<String>,
    pub can_cancel: bool,
}

struct State {
    progress: Progress,
    changed: Instant,
    cancelled: bool,
}
pub struct Job {
    started: Instant,
    state: Mutex<State>,
}
fn problem(code: &'static str) -> AppError {
    AppError::InvalidInput(UserMessage::new(code, "Source heading analysis stopped. The previous structure and reference links are unchanged. You can retry."))
}
impl Default for Job {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            state: Mutex::new(State {
                changed: now,
                cancelled: false,
                progress: Progress {
                    status: "running".into(),
                    stage: "file".into(),
                    done: 0,
                    total: 0,
                    elapsed_ms: 0,
                    timings: vec![],
                    heading_count: None,
                    indexed_links: None,
                    error_code: None,
                    can_cancel: true,
                },
            }),
        }
    }
}
impl Job {
    pub fn snapshot(&self) -> Progress {
        let mut p = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress
            .clone();
        if p.status == "running" {
            p.elapsed_ms = self.started.elapsed().as_millis() as u64;
        }
        p
    }
    pub fn cancel(&self) -> bool {
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !s.progress.can_cancel {
            return false;
        }
        s.cancelled = true;
        true
    }
    pub fn check(&self) -> AppResult<()> {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.cancelled {
            return Err(problem("library.structure.cancelled"));
        }
        if self.started.elapsed() > DEADLINE {
            return Err(problem("library.structure.timeout"));
        }
        Ok(())
    }
    pub fn step(&self, stage: &str, done: usize, total: usize) -> AppResult<()> {
        self.check()?;
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.progress.stage != stage {
            let timing = Timing {
                stage: s.progress.stage.clone(),
                milliseconds: s.changed.elapsed().as_millis() as u64,
            };
            s.progress.timings.push(timing);
            s.changed = Instant::now();
            s.progress.stage = stage.into();
        }
        if stage == "indexed" {
            s.progress.indexed_links = Some(done);
        }
        s.progress.done = done;
        s.progress.total = total;
        Ok(())
    }
    /// Cancellation and this commit gate are serialised. A cancellation
    /// accepted before the gate always rolls back; after it we finish commit.
    pub fn begin_commit(&self) -> AppResult<()> {
        self.step("commit", 0, 0)?;
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.cancelled {
            return Err(problem("library.structure.cancelled"));
        }
        s.progress.can_cancel = false;
        Ok(())
    }
    pub fn finish(&self, result: &AppResult<pdf_structures::View>) {
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let timing = Timing {
            stage: s.progress.stage.clone(),
            milliseconds: s.changed.elapsed().as_millis() as u64,
        };
        s.progress.timings.push(timing);
        s.progress.elapsed_ms = self.started.elapsed().as_millis() as u64;
        s.progress.can_cancel = false;
        match result {
            Ok(v) => {
                s.progress.status = if v.headings.is_empty() {
                    "empty"
                } else {
                    "succeeded"
                }
                .into();
                s.progress.heading_count = Some(v.headings.len());
            }
            Err(e) => {
                let code = match e {
                    AppError::InvalidInput(m) => m.code.unwrap_or("library.structure.failed"),
                    _ => "library.structure.failed",
                };
                s.progress.status = if code == "library.structure.cancelled" {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                s.progress.error_code = Some(code.into());
            }
        }
        // No error text, title, original path or document contents in logs.
        for t in &s.progress.timings {
            log::info!(
                "PDF analysis stage={} ms={} status={}",
                t.stage,
                t.milliseconds,
                s.progress.status
            );
        }
    }
}

#[derive(Default)]
pub struct Jobs(Mutex<HashMap<String, Arc<Job>>>);
impl Jobs {
    pub fn start(&self, id: &str) -> AppResult<Arc<Job>> {
        let mut jobs = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if jobs.values().any(|j| j.snapshot().status == "running") {
            return Err(problem("library.structure.busy"));
        }
        if jobs.len() >= 8 {
            jobs.clear();
        }
        let job = Arc::new(Job::default());
        jobs.insert(id.into(), Arc::clone(&job));
        Ok(job)
    }
    pub fn get(&self, id: &str) -> Option<Arc<Job>> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned()
    }
}

pub fn rebuild(
    db: &Database,
    sources: &Path,
    id: &str,
    job: &Job,
) -> AppResult<pdf_structures::View> {
    job.step("file", 0, 0)?;
    let (relative, format) = db.with_conn(|c| library::original_record(c, id))?;
    if format != "pdf" {
        return Err(problem("library.structure.invalid"));
    }
    let bytes = original::read_managed(sources, id, &relative, 200 * 1024 * 1024)?;
    let hash = super::files::content_hash(&bytes);
    job.step("segments", 0, 0)?;
    let stored = db.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT position,page_number,content,char_start,char_end FROM source_segments WHERE source_id=?1 ORDER BY position")?;
        let rows = stmt.query_map([id], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,Option<u32>>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<u32>>(3)?,r.get::<_,Option<u32>>(4)?)))?.collect::<Result<Vec<_>,_>>()?;
        Ok(rows)
    })?;
    let pieces: Vec<_> = stored
        .iter()
        .filter_map(|(position, page, text, start, end)| {
            Some(pdf_structure::Piece {
                position: *position,
                page: (*page)?,
                text,
                byte_start: *start,
                byte_end: *end,
            })
        })
        .collect();
    job.step("pdf", 0, 0)?;
    let pdf = pdf::read_pages_with_progress(&bytes, |done, total| job.step("pdf", done, total))?;
    let document =
        pdf_structure::extract_with_progress(&bytes, &pdf, &pieces, |stage, done, total| {
            job.step(stage, done, total)
        })?;
    job.step("store", 0, 0)?;
    db.with_conn(|conn| {
        pdf_structures::replace_observed(conn, id, &hash, &document, |stage, done, total| {
            if stage == "commit" {
                job.begin_commit()
            } else {
                job.step(stage, done, total)
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_limit_is_checked_before_any_next_stage_or_commit() {
        let job = Job {
            started: Instant::now() - DEADLINE - Duration::from_secs(1),
            ..Job::default()
        };
        let error = job.step("pdf", 1, 400).unwrap_err();
        assert!(
            matches!(error, AppError::InvalidInput(m) if m.code==Some("library.structure.timeout"))
        );
        assert!(job.begin_commit().is_err());
    }
}
