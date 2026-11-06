//! Stress test for [`athena_testkit::write_executable`]: many write-then-exec cycles while other threads fork.
//!
//! The race (Linux): a script written by this process is held open for writing for a moment. If another
//! thread forks in that moment, the child inherits the handle until its own `exec`, and running the script
//! then fails with `ETXTBSY` ("Text file busy", `io::ErrorKind::ExecutableFileBusy`).
//!
//! - `helper_never_hits_text_file_busy_while_other_threads_fork` writes with the helper, which hands the bytes
//!   to a child process, so this process never holds a write handle. It must see zero `ETXTBSY`.
//! - `in_process_write_hits_text_file_busy_while_other_threads_fork` is the contrast: the same cycles with
//!   `std::fs::write`. It prints how many runs hit `ETXTBSY` and does not assert a count, because whether a
//!   fork lands in the window depends on the machine's scheduling. It stops at the first hit once it has run a
//!   minimum number of cycles, or after a cap, and says which. Run it with `--nocapture` to see the count:
//!   `cargo test -p athena-testkit --test stand_in_stress -- --nocapture`.

use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;

const SCRIPT: &[u8] = b"#!/bin/sh\nexit 0\n";
const WRITERS: usize = 4;
const FORKERS: usize = 4;

/// The result of a batch of write-then-exec cycles.
struct Outcome {
    cycles: usize,
    busy: usize,
    other_errors: Vec<io::Error>,
}

/// Runs `cycles_per_writer` write-then-exec cycles on each of `WRITERS` threads, with `FORKERS` threads
/// spawning short-lived processes the whole time. `stop_on_busy_after` ends a writer early once it has run
/// that many cycles and some writer has seen `ETXTBSY`.
fn run_cycles(
    write: fn(&Path, &[u8]) -> io::Result<()>,
    cycles_per_writer: usize,
    stop_on_busy_after: Option<usize>,
) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let done = AtomicBool::new(false);
    let busy = AtomicUsize::new(0);
    let cycles = AtomicUsize::new(0);
    let mut other_errors = Vec::new();

    thread::scope(|scope| {
        for _ in 0..FORKERS {
            scope.spawn(|| {
                while !done.load(Ordering::Relaxed) {
                    Command::new("true").status().unwrap();
                }
            });
        }
        let writers: Vec<_> = (0..WRITERS)
            .map(|w| {
                let (dir, busy, cycles) = (dir.path(), &busy, &cycles);
                scope.spawn(move || {
                    let mut errors = Vec::new();
                    for i in 0..cycles_per_writer {
                        if let Some(min) = stop_on_busy_after
                            && i >= min
                            && busy.load(Ordering::Relaxed) > 0
                        {
                            break;
                        }
                        let path = dir.join(format!("script-{w}-{i}"));
                        let result =
                            write(&path, SCRIPT).and_then(|()| Command::new(&path).status());
                        cycles.fetch_add(1, Ordering::Relaxed);
                        match result {
                            Ok(status) => assert!(status.success()),
                            Err(e) if e.kind() == io::ErrorKind::ExecutableFileBusy => {
                                busy.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => errors.push(e),
                        }
                    }
                    errors
                })
            })
            .collect();
        for writer in writers {
            other_errors.extend(writer.join().unwrap());
        }
        done.store(true, Ordering::Relaxed);
    });

    Outcome {
        cycles: cycles.load(Ordering::Relaxed),
        busy: busy.load(Ordering::Relaxed),
        other_errors,
    }
}

#[test]
fn helper_never_hits_text_file_busy_while_other_threads_fork() {
    let outcome = run_cycles(
        |path, bytes| athena_testkit::write_executable(path, bytes),
        150,
        None,
    );
    eprintln!(
        "helper write: {} cycles, {} ETXTBSY",
        outcome.cycles, outcome.busy
    );
    assert!(
        outcome.other_errors.is_empty(),
        "{:?}",
        outcome.other_errors
    );
    assert_eq!(outcome.cycles, WRITERS * 150);
    assert_eq!(outcome.busy, 0, "the helper must never hit ETXTBSY");
}

#[test]
fn in_process_write_hits_text_file_busy_while_other_threads_fork() {
    let outcome = run_cycles(
        |path, bytes| {
            std::fs::write(path, bytes)?;
            // Make it executable the way a test would without the helper.
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        },
        2000,
        Some(150),
    );
    eprintln!(
        "in-process write: {} cycles, {} ETXTBSY on this machine (not asserted: it depends on scheduling)",
        outcome.cycles, outcome.busy
    );
    assert!(
        outcome.other_errors.is_empty(),
        "{:?}",
        outcome.other_errors
    );
}
