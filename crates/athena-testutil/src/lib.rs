//! Test support: executable stand-in scripts that do not fail with `ETXTBSY`.
//!
//! # Why this crate exists
//!
//! Many tests write a small shell script, mark it executable and run it in place of a real tool
//! (`claude`, `gh`, `cargo`, a build of `athena`). On Linux that races with the test harness itself:
//!
//! 1. The test opens the script for writing. The open file descriptor is a writer of that file.
//! 2. Another test thread, running at the same time, forks a child process (any `Command::spawn`).
//!    The child inherits every open descriptor, including the writer, until it calls `exec`.
//! 3. The first thread closes its descriptor and runs the script. The kernel refuses to execute a file that
//!    any process still holds open for writing, and fails with `ETXTBSY` ("Text file busy").
//!    The only holder is the forked child, which has not yet reached `exec`.
//!
//! The failure is rare and depends on timing, so it shows up as a flaky test.
//!
//! # What this crate does
//!
//! [`write_executable`] never opens the final path for writing. It writes the content to a temporary
//! name, closes it, sets the mode, and renames it over the final path. A rename is atomic, so a reader never
//! sees a half-written script, and a script that is already running is replaced, not rewritten.
//! The rename alone does not close the race: a file stays busy for as long as any forked child holds a
//! writer on its inode, and the rename keeps the inode. The child holds it only until its `exec`, which is
//! a few milliseconds, so [`run`] and [`retry_busy`] try again, with a short sleep, while the error is
//! `ETXTBSY`. Use `run` (or wrap the spawn in `retry_busy`) for every stand-in a test starts.
//!
//! No crate fits this need, and the code is a few lines of `std`, so it is built here.

use std::io;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// How many times a busy run is tried again before its error is returned.
const RETRIES: u32 = 100;
/// The wait between tries.
const RETRY_WAIT: Duration = Duration::from_millis(20);

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `contents` to `path` as an executable file (mode 0755), through a temporary name in the
/// same directory: write, close, `chmod`, then rename over `path`. An existing file at `path` is replaced.
///
/// Panics on an I/O error, because it is for tests.
pub fn write_executable(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = path.parent().expect("the script path has a directory");
    let name = path.file_name().expect("the script path has a name");
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(
        ".{}.{}.{n}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));
    // The write handle is closed when `write` returns.
    std::fs::write(&tmp, contents).expect("write the temporary script");
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
        .expect("make the script executable");
    std::fs::rename(&tmp, path).expect("move the script into place");
}

/// True if `message` is the text of an `ETXTBSY` error ("Text file busy").
pub fn is_busy(message: &str) -> bool {
    message.contains("Text file busy")
}

/// Calls `attempt` until it succeeds or fails with something other than `ETXTBSY`, at most
/// [`RETRIES`] extra times. The error is judged by its text, so it works for `io::Error`, `anyhow::Error`
/// and any other error that shows the operating system's message.
pub fn retry_busy<T, E: std::fmt::Display>(
    mut attempt: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    let mut result = attempt();
    for _ in 0..RETRIES {
        match &result {
            Err(e) if is_busy(&format!("{e:#}")) => {
                std::thread::sleep(RETRY_WAIT);
                result = attempt();
            }
            _ => break,
        }
    }
    result
}

/// Runs the command to completion and returns its output, trying again while it fails with `ETXTBSY`.
/// `build` makes a fresh `Command` for each try, because a `Command` cannot be reused after it is spawned.
pub fn run(mut build: impl FnMut() -> Command) -> io::Result<Output> {
    retry_busy(|| build().output())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn a_written_script_runs_and_a_rewrite_replaces_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        write_executable(&path, "#!/bin/sh\necho one\n");
        let out = run(|| Command::new(&path)).unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "one\n");
        write_executable(&path, "#!/bin/sh\necho two\n");
        let out = run(|| Command::new(&path)).unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "two\n");
        // No temporary file is left behind.
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn retry_busy_stops_on_other_errors_and_retries_busy_ones() {
        let mut calls = 0;
        let r: Result<(), String> = retry_busy(|| {
            calls += 1;
            Err("no such file".to_string())
        });
        assert!(r.is_err());
        assert_eq!(calls, 1);
        let mut calls = 0;
        let r: Result<u32, String> = retry_busy(|| {
            calls += 1;
            if calls < 3 {
                Err("Text file busy (os error 26)".to_string())
            } else {
                Ok(calls)
            }
        });
        assert_eq!(r, Ok(3));
    }

    /// Writes and runs many scripts while other threads fork all the time, and counts `ETXTBSY` failures.
    /// This is the race the crate exists for: a forked child holding a writer descriptor on a script.
    /// Each script is run with [`run`], so a file that stays busy beyond the retries fails the test.
    #[test]
    fn scripts_run_without_etxtbsy_while_other_threads_fork() {
        const WRITERS: usize = 8;
        const PER_WRITER: usize = 50; // 400 scripts in all
        let stop = Arc::new(AtomicBool::new(false));
        let forkers: Vec<_> = (0..4)
            .map(|_| {
                let stop = Arc::clone(&stop);
                std::thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        let _ = Command::new("true").output();
                    }
                })
            })
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let writers: Vec<_> = (0..WRITERS)
            .map(|w| {
                let dir = dir.path().to_path_buf();
                std::thread::spawn(move || {
                    let mut busy = 0;
                    for i in 0..PER_WRITER {
                        let path = dir.join(format!("script-{w}-{i}"));
                        write_executable(&path, &format!("#!/bin/sh\necho {w}-{i}\n"));
                        match run(|| Command::new(&path)) {
                            Ok(out) => assert_eq!(
                                String::from_utf8_lossy(&out.stdout).trim(),
                                format!("{w}-{i}")
                            ),
                            Err(e) if is_busy(&e.to_string()) => busy += 1,
                            Err(e) => panic!("{e}"),
                        }
                    }
                    busy
                })
            })
            .collect();
        let busy: usize = writers.into_iter().map(|h| h.join().unwrap()).sum();
        stop.store(true, Ordering::Relaxed);
        for f in forkers {
            f.join().unwrap();
        }
        assert_eq!(
            busy,
            0,
            "{busy} of {} runs failed with ETXTBSY",
            WRITERS * PER_WRITER
        );
    }
}
