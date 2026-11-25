//! The Bible importer runs PDF parsing in a child of the application.
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::error::{AppError, AppResult, UserMessage};
use crate::scripture::documents::{self, DocumentFile};

#[cfg(any(target_os = "linux", windows))]
const MEMORY_LIMIT: usize = 1536 * 1024 * 1024;
const OUTPUT_LIMIT: u64 = 128 * 1024 * 1024;
const TIME_LIMIT: Duration = Duration::from_secs(120);
const WORKER_ARGUMENT: &str = "--bible-pdf-text-worker";

fn failed() -> AppError {
    AppError::Import(UserMessage::new(
        "bibleImport.pdfWorkerFailed",
        "The PDF text could not be read within the extraction limits.",
    ))
}

pub fn cancelled() -> AppError {
    AppError::Import(UserMessage::new(
        "bibleImport.cancelled",
        "Bible import cancelled.",
    ))
}

/// Called before the GUI starts. The parent sends permission only after
/// installing limits. EOF, including if the parent exits early, refuses it.
pub fn run_if_requested() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(WORKER_ARGUMENT)) {
        return None;
    }
    let result = (|| {
        let path = args.next()?;
        let position = args.next()?.to_str()?.parse().ok()?;
        if args.next().is_some() {
            return None;
        }
        let mut start = [0];
        std::io::stdin().read_exact(&mut start).ok()?;
        if start != [1]
            || !Path::new(&path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
        {
            return None;
        }
        let file = documents::read(Path::new(&path), position).ok()?;
        serde_json::to_writer(std::io::stdout().lock(), &file).ok()
    })();
    Some(if result.is_some() { 0 } else { 1 })
}

pub(super) fn read(
    path: &Path,
    position: u16,
    is_cancelled: &dyn Fn() -> bool,
) -> AppResult<DocumentFile> {
    read_with_executable(
        &std::env::current_exe().map_err(|_| failed())?,
        path,
        position,
        is_cancelled,
    )
}

/// Use the same child protocol when testing the built application binary.
pub fn read_with_executable(
    executable: &Path,
    path: &Path,
    position: u16,
    is_cancelled: &dyn Fn() -> bool,
) -> AppResult<DocumentFile> {
    if is_cancelled() {
        return Err(cancelled());
    }
    let mut command = Command::new(executable);
    command
        .arg(WORKER_ARGUMENT)
        .arg(path)
        .arg(position.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    configure(&mut command)?;
    let mut child = command.spawn().map_err(|_| failed())?;
    let result = (|| {
        #[cfg(windows)]
        let _job = windows::limit(&child)?;
        let mut input = child.stdin.take().ok_or_else(failed)?;
        let output = child.stdout.take().ok_or_else(failed)?;
        input.write_all(&[1]).map_err(|_| failed())?;
        drop(input);
        let output = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            output.take(OUTPUT_LIMIT + 1).read_to_end(&mut bytes)?;
            Ok::<_, std::io::Error>(bytes)
        });
        let deadline = Instant::now() + TIME_LIMIT;
        let status = loop {
            if is_cancelled() || Instant::now() >= deadline {
                break None;
            }
            if let Some(status) = child.try_wait().map_err(|_| failed())? {
                break Some(status);
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        if status.is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let bytes = output.join().map_err(|_| failed())?.map_err(|_| failed())?;
        if is_cancelled() {
            return Err(cancelled());
        }
        if !status.is_some_and(|status| status.success()) || bytes.len() as u64 > OUTPUT_LIMIT {
            return Err(failed());
        }
        let mut file: DocumentFile = serde_json::from_slice(&bytes).map_err(|_| failed())?;
        file.format = "pdf";
        file.path = path.to_path_buf();
        Ok(file)
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[cfg(target_os = "linux")]
fn configure(command: &mut Command) -> AppResult<()> {
    use std::os::unix::process::CommandExt;
    // The pre-exec hook uses only async-signal-safe system calls on success.
    unsafe {
        command.pre_exec(|| {
            let limit = libc::rlimit {
                rlim_cur: MEMORY_LIMIT as libc::rlim_t,
                rlim_max: MEMORY_LIMIT as libc::rlim_t,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0
                || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(())
}

#[cfg(windows)]
fn configure(command: &mut Command) -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    command.creation_flags(CREATE_NO_WINDOW);
    Ok(())
}

#[cfg(not(any(target_os = "linux", windows)))]
fn configure(_: &mut Command) -> AppResult<()> {
    Err(failed())
}

#[cfg(windows)]
mod windows {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use windows_sys::Win32::{Foundation::*, System::JobObjects::*};

    pub(super) struct Job(HANDLE);

    impl Drop for Job {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    pub(super) fn limit(child: &Child) -> super::AppResult<Job> {
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(super::failed());
            }
            let job = Job(handle);
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
            limits.BasicLimitInformation.ActiveProcessLimit = 1;
            limits.ProcessMemoryLimit = super::MEMORY_LIMIT;
            if SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            ) == 0
                || AssignProcessToJobObject(job.0, child.as_raw_handle()) == 0
            {
                return Err(super::failed());
            }
            Ok(job)
        }
    }
}
