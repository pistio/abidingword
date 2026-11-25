//! A.4.4 (#19): AI writing help on a **block** teaching, with DeepSeek
//! configured, a German interface and Korean writing. A.3 sent a block's
//! body as a "point" part without a point and failed with the generic
//! "This item could not be found" before any provider was called. A scripted
//! service stands in for DeepSeek: no network, no key leaves the test, no paid
//! call. All text is invented.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::pin;
use std::sync::Mutex;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};

use abiding_word_lib::assistant::config::{self, AssistantConfig, ProviderKind};
use abiding_word_lib::assistant::context::{AssistAction, AssistInput, Part};
use abiding_word_lib::assistant::http::{HttpCall, HttpReply, Transport};
use abiding_word_lib::assistant::secrets::{self, MemoryStore, SecretStore};
use abiding_word_lib::assistant::{self, AssistResult};
use abiding_word_lib::db::{projects, studies, teaching, teaching_blocks, workspaces, Database};
use abiding_word_lib::domain::{
    CreateProject, CreateStudy, CreateTeaching, NewTeachingBlock, UpdateTeachingBlock,
};
use abiding_word_lib::error::{AppError, AppResult};
use serde_json::json;

fn block_on<F: Future>(future: F) -> F::Output {
    fn raw() -> RawWaker {
        fn clone(_: *const ()) -> RawWaker {
            raw()
        }
        fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    let waker = unsafe { Waker::from_raw(raw()) };
    let mut cx = TaskContext::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            return v;
        }
    }
}

struct Scripted {
    replies: Mutex<VecDeque<HttpReply>>,
    calls: Mutex<Vec<HttpCall>>,
}

impl Scripted {
    fn new(replies: Vec<serde_json::Value>) -> Self {
        Self {
            replies: Mutex::new(
                replies
                    .into_iter()
                    .map(|v| HttpReply {
                        status: 200,
                        body: serde_json::to_vec(&v).unwrap(),
                    })
                    .collect(),
            ),
            calls: Mutex::default(),
        }
    }
    fn calls(&self) -> Vec<HttpCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl Transport for Scripted {
    async fn send(&self, call: HttpCall) -> AppResult<HttpReply> {
        self.calls.lock().unwrap().push(call);
        Ok(self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("no more scripted replies"))
    }
}

fn says(content: serde_json::Value) -> serde_json::Value {
    json!({"choices": [{"message": {"content": content.to_string()}}]})
}

const KEY: &str = "sk-deepseek-test-0123456789";
const KOREAN: &str = "우리는 열매를 맺기 위하여 더 열심히 노력하지 않는다. 우리는 머문다";

struct World {
    db: Database,
    secrets: MemoryStore,
    prep: String,
    erkennen: String,
    verstehen: String,
}

/// An EVA-shaped block teaching (sections with one writing block each).
fn world() -> World {
    let db = Database::open_in_memory().unwrap();
    let secrets = MemoryStore::default();
    let (prep, erkennen, verstehen) = db
        .with_conn(|conn| {
            let ws = workspaces::get_default(conn)?;
            let project = projects::create(
                conn,
                &CreateProject {
                    workspace_id: ws.id,
                    title: "Hauskreis".into(),
                    description: None,
                },
            )?;
            let study = studies::create(
                conn,
                &CreateStudy {
                    project_id: project.id.clone(),
                    title: "요한복음 15".into(),
                    ..Default::default()
                },
            )?;
            let block =
                |key: &str, parent: Option<&str>, block_type: &str, title: &str| NewTeachingBlock {
                    key: Some(key.into()),
                    parent_key: parent.map(String::from),
                    block_type: block_type.into(),
                    title: title.into(),
                    body: String::new(),
                    participant_facing: Some(false),
                    role: None,
                    guidance_key: None,
                };
            let prep = teaching::create(
                conn,
                &CreateTeaching {
                    project_id: None,
                    study_id: study.study.id.clone(),
                    title: "Hauskreis · 요한복음 15".into(),
                    kind: Some("bible_study".into()),
                    points: Vec::new(),
                    template_key: Some("eva".into()),
                    template_version: Some(2),
                    blocks: vec![
                        block("s1", None, "section", "Erkennen"),
                        block("b1", Some("s1"), "writing", "Erkennen"),
                        block("s2", None, "section", "Verstehen"),
                        block("b2", Some("s2"), "writing", "Verstehen"),
                    ],
                },
            )?;
            let id_of = |title: &str, block_type: &str| {
                prep.blocks
                    .iter()
                    .find(|b| b.title == title && b.block_type == block_type)
                    .unwrap()
                    .id
                    .clone()
            };
            let erkennen = id_of("Erkennen", "writing");
            let verstehen = id_of("Verstehen", "writing");
            teaching_blocks::update(
                conn,
                &UpdateTeachingBlock {
                    id: erkennen.clone(),
                    body: Some(KOREAN.into()),
                    ..Default::default()
                },
            )?;
            teaching_blocks::update(
                conn,
                &UpdateTeachingBlock {
                    id: verstehen.clone(),
                    body: Some("가지는 포도나무에 붙어 있다.".into()),
                    ..Default::default()
                },
            )?;
            config::set(
                conn,
                AssistantConfig {
                    enabled: true,
                    provider: ProviderKind::Deepseek,
                    model: "deepseek-chat".into(),
                    ..AssistantConfig::default()
                },
            )?;
            Ok((prep.id, erkennen, verstehen))
        })
        .unwrap();
    secrets
        .set(&secrets::provider_key_name("deepseek"), Some(KEY))
        .unwrap();
    World {
        db,
        secrets,
        prep,
        erkennen,
        verstehen,
    }
}

impl World {
    /// What the A.4.4 interface sends for a block's body (its block, no point).
    fn input(&self, block: Option<&str>) -> AssistInput {
        AssistInput {
            preparation_id: self.prep.clone(),
            action: Some(AssistAction::Grammar),
            part: Some(Part::PointExplanation),
            point_id: None,
            block_id: block.map(String::from),
            text: KOREAN.into(),
            ui_language: "de".into(),
            ..Default::default()
        }
    }
    fn ask(&self, input: &AssistInput, service: &Scripted) -> AppResult<assistant::AssistReply> {
        let prepared = self.db.with_conn(|conn| assistant::prepare(conn, input))?;
        let prepared = assistant::unlock(prepared, &self.secrets)?;
        block_on(assistant::run(service, &prepared))
    }
    fn body(&self, id: &str) -> String {
        self.db
            .with_conn(|conn| {
                Ok(teaching::get(conn, &self.prep)?
                    .blocks
                    .into_iter()
                    .find(|b| b.id == id)
                    .unwrap()
                    .body)
            })
            .unwrap()
    }
}

fn code<T: std::fmt::Debug>(result: AppResult<T>) -> String {
    match result {
        Err(AppError::Assistant(m)) => m.code.unwrap_or_default().to_string(),
        other => panic!("expected an assistant refusal, got {other:?}"),
    }
}

#[test]
fn korean_correction_of_a_block_reaches_deepseek_and_is_only_a_suggestion() {
    let w = world();
    let corrected = "우리는 열매를 맺기 위하여 더 열심히 노력하지 않습니다. 우리는 머뭅니다.";
    let service = Scripted::new(vec![says(
        json!({"suggestion": corrected, "changes": "높임말로 맞추고 마침표를 더했습니다."}),
    )]);
    let reply = w.ask(&w.input(Some(&w.erkennen)), &service).unwrap();
    match reply.result {
        AssistResult::Proposal {
            suggestion,
            placement,
            ..
        } => {
            assert_eq!(suggestion, corrected);
            assert_eq!(placement, "replace");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(reply.assistant, "DeepSeek · deepseek-chat");
    // The suggestion never replaces the person's words by itself.
    assert_eq!(w.body(&w.erkennen), KOREAN);
    // One request, to DeepSeek, carrying the Korean text, the block's name and its
    // neighbour as context; the key is never in the body.
    let calls = service.calls();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].url.starts_with("https://api.deepseek.com/v1"),
        "{}",
        calls[0].url
    );
    let body = calls[0]
        .body
        .as_ref()
        .map(|b| b.to_string())
        .unwrap_or_default();
    assert!(
        body.contains("우리는 열매를"),
        "the Korean text is sent as written"
    );
    assert!(
        body.contains("Erkennen"),
        "the block is named, not a missing point"
    );
    assert!(
        body.contains("가지는 포도나무에"),
        "the neighbouring block is context"
    );
    assert!(!body.contains(KEY));
}

#[test]
fn the_preview_names_the_block_and_sends_nothing() {
    let w = world();
    let preview =
        w.db.with_conn(|conn| {
            Ok(assistant::preview(&assistant::prepare(
                conn,
                &w.input(Some(&w.erkennen)),
            )?))
        })
        .unwrap();
    let text = serde_json::to_string(&preview).unwrap();
    assert!(text.contains("Erkennen"));
    assert!(text.contains("DeepSeek"));
    assert!(!text.contains(KEY));
}

#[test]
fn a3s_request_without_a_block_is_refused_with_an_actionable_message_not_not_found() {
    let w = world();
    let service = Scripted::new(vec![]);
    // Exactly what the A.3 interface sent: a point part, no point, no block.
    assert_eq!(
        code(w.ask(&w.input(None), &service)),
        "assistant.staleTarget"
    );
    assert!(service.calls().is_empty(), "nothing is sent");
}

#[test]
fn a_block_deleted_after_asking_is_refused_and_nothing_is_sent_or_applied() {
    let w = world();
    w.db.with_conn(|conn| teaching_blocks::delete(conn, &w.verstehen))
        .unwrap();
    let service = Scripted::new(vec![]);
    assert_eq!(
        code(w.ask(&w.input(Some(&w.verstehen)), &service)),
        "assistant.staleTarget"
    );
    assert!(service.calls().is_empty());
    assert_eq!(w.body(&w.erkennen), KOREAN);
}

#[test]
fn a_block_from_another_teaching_is_not_accepted() {
    let w = world();
    let mut input = w.input(Some("not-a-block-of-this-teaching"));
    input.text = KOREAN.into();
    let service = Scripted::new(vec![]);
    assert_eq!(code(w.ask(&input, &service)), "assistant.staleTarget");
    assert!(service.calls().is_empty());
}

#[test]
fn block_writing_counts_as_the_persons_material_for_a_free_request() {
    let w = world();
    let mut input = w.input(None);
    input.action = Some(AssistAction::Free);
    input.part = Some(Part::Whole);
    input.text = String::new();
    input.request = "이 부분을 더 짧게 할 수 있을까요?".into();
    let service = Scripted::new(vec![says(
        json!({"comments": [{"text": "짧게 할 수 있습니다.", "basis": "general"}]}),
    )]);
    let reply = w.ask(&input, &service);
    assert!(
        !matches!(&reply, Err(AppError::Assistant(m)) if m.code == Some("assistant.writeFirst")),
        "{reply:?}"
    );
}
