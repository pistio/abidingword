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
    let result = read_child(&mut child, path, is_cancelled, TIME_LIMIT);
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

/// The production child protocol, also exercised by bounded Windows subprocess
/// tests. The public importer always passes TIME_LIMIT; tests can prove the
/// deadline path without holding a worker open for two minutes.
fn read_child(
    child: &mut std::process::Child,
    path: &Path,
    is_cancelled: &dyn Fn() -> bool,
    time_limit: Duration,
) -> AppResult<DocumentFile> {
    #[cfg(windows)]
    let _job = windows::limit(child)?;
    let mut input = child.stdin.take().ok_or_else(failed)?;
    let output = child.stdout.take().ok_or_else(failed)?;
    input.write_all(&[1]).map_err(|_| failed())?;
    drop(input);
    let output = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output.take(OUTPUT_LIMIT + 1).read_to_end(&mut bytes)?;
        Ok::<_, std::io::Error>(bytes)
    });
    let deadline = Instant::now() + time_limit;
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

    /// These exercise real Windows kernel handles and child processes. They
    /// are part of the native pre-packaging gate; Linux cannot stand in for
    /// them. The helper and allocation probe do not exist in production.
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::error::AppError;
        use crate::scripture::pdf_worker::{configure, read_child, MEMORY_LIMIT};
        use std::io::{Read, Write};
        use std::process::{Command, ExitStatus, Stdio};
        use std::time::{Duration, Instant};
        use windows_sys::Win32::System::Memory::{
            VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
        };

        struct Fixture(Child);

        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }

        fn fixture(mode: &str) -> Fixture {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "scripture::pdf_worker::windows::tests::worker_fixture",
                    "--ignored",
                    "--nocapture",
                ])
                .env("AW_TEST_PDF_WORKER_MODE", mode)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            configure(&mut command).unwrap();
            Fixture(command.spawn().unwrap())
        }

        fn permit(child: &mut Child) {
            let mut input = child.stdin.take().unwrap();
            input.write_all(&[1]).unwrap();
        }

        fn wait(child: &mut Child) -> ExitStatus {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    return status;
                }
                assert!(Instant::now() < deadline, "limited worker did not exit");
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn message_code<T>(result: crate::error::AppResult<T>) -> &'static str {
            match result {
                Err(AppError::Import(message)) => message.code.unwrap(),
                Err(error) => panic!("unexpected worker error: {error}"),
                Ok(_) => panic!("the bounded worker should have stopped"),
            }
        }

        #[test]
        #[ignore = "only launched as a bounded Windows PDF worker fixture"]
        fn worker_fixture() {
            let Ok(mode) = std::env::var("AW_TEST_PDF_WORKER_MODE") else {
                return;
            };
            // Like the application worker, do nothing until the parent has
            // assigned the Job Object and sent its permission byte.
            let mut byte = [0];
            std::io::stdin().read_exact(&mut byte).unwrap();
            assert_eq!(byte, [1]);
            match mode.as_str() {
                "wait" => std::thread::sleep(Duration::from_secs(10)),
                "memory" => unsafe {
                    // Committing address space charges the job's memory limit;
                    // no pages are touched and no 1.5 GiB buffer is filled.
                    // First show that a small allocation is possible.
                    let small = VirtualAlloc(
                        std::ptr::null(),
                        64 * 1024,
                        MEM_RESERVE | MEM_COMMIT,
                        PAGE_READWRITE,
                    );
                    assert!(!small.is_null(), "small in-limit allocation failed");
                    assert_ne!(VirtualFree(small, 0, MEM_RELEASE), 0);
                    let excessive = VirtualAlloc(
                        std::ptr::null(),
                        MEMORY_LIMIT + 64 * 1024,
                        MEM_RESERVE | MEM_COMMIT,
                        PAGE_READWRITE,
                    );
                    if !excessive.is_null() {
                        // Free any unexpected success before failing the test.
                        let _ = VirtualFree(excessive, 0, MEM_RELEASE);
                    }
                    assert!(
                        excessive.is_null(),
                        "Job Object did not refuse excess commitment"
                    );
                },
                _ => panic!("unknown bounded worker fixture"),
            }
        }

        #[test]
        fn actual_job_limits_and_process_assignment_match_the_importer() {
            let mut child = fixture("wait");
            let job = limit(&child.0).unwrap();
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            let mut returned = 0;
            let mut assigned = 0;
            unsafe {
                assert_ne!(
                    QueryInformationJobObject(
                        job.0,
                        JobObjectExtendedLimitInformation,
                        (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                        std::mem::size_of_val(&limits) as u32,
                        &mut returned,
                    ),
                    0,
                );
                assert_ne!(
                    IsProcessInJob(child.0.as_raw_handle(), job.0, &mut assigned),
                    0
                );
            }
            assert_ne!(assigned, 0);
            let required = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
            assert_eq!(limits.BasicLimitInformation.LimitFlags & required, required);
            assert_eq!(limits.BasicLimitInformation.ActiveProcessLimit, 1);
            assert_eq!(limits.ProcessMemoryLimit, MEMORY_LIMIT);
            // Keep stdin open without sending permission, so the fixture cannot
            // finish naturally. Job closure does not promise a particular exit code.
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "worker must still be alive before closing the job"
            );
            drop(job);
            let _ = wait(&mut child.0);
        }

        #[test]
        fn actual_job_refuses_allocation_above_the_production_memory_cap() {
            let mut child = fixture("memory");
            let _job = limit(&child.0).unwrap();
            permit(&mut child.0);
            let status = wait(&mut child.0);
            let mut output = String::new();
            child
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut output)
                .unwrap();
            assert!(status.success(), "memory-limit probe failed: {output}");
        }

        #[test]
        fn native_cancel_stops_and_reaps_the_actual_job_child() {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("invented.pdf");
            let bytes = b"invented bytes preserved by cancellation";
            std::fs::write(&path, bytes).unwrap();
            let mut child = fixture("wait");
            let start = Instant::now();
            let result = read_child(
                &mut child.0,
                &path,
                &|| start.elapsed() >= Duration::from_millis(150),
                Duration::from_secs(5),
            );
            assert_eq!(message_code(result), "bibleImport.cancelled");
            assert!(child.0.try_wait().unwrap().is_some());
            assert!(start.elapsed() < Duration::from_secs(5));
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }

        #[test]
        fn native_deadline_stops_and_reaps_the_actual_job_child() {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("invented.pdf");
            let bytes = b"invented bytes preserved by a timeout";
            std::fs::write(&path, bytes).unwrap();
            let mut child = fixture("wait");
            let start = Instant::now();
            let result = read_child(&mut child.0, &path, &|| false, Duration::from_millis(150));
            assert_eq!(message_code(result), "bibleImport.pdfWorkerFailed");
            assert!(child.0.try_wait().unwrap().is_some());
            assert!(start.elapsed() < Duration::from_secs(5));
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }
}
