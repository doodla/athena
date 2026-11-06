//! Stress test for [`athena_testkit::write_executable`]: many threads each write a stand-in program and run it
//! straight away, so some thread is always forking while another has just written. That is the shape of the
//! workspace's test suite, and the shape that fails with `ETXTBSY` ("Text file busy") when a stand-in is
//! written in-process.
//!
//! Two variants run the same load:
//!
//! - `the_helper_never_meets_a_busy_file` writes through the helper and asserts the busy count is 0.
//! - `the_in_process_write_is_shown_for_comparison` writes with `std::fs::write` and `set_permissions`, the way
//!   the tests used to. It asserts nothing about the busy count, because that depends on scheduling and
//!   machine load, and may be 0 on a quiet machine. It prints the count to standard error with a direct
//!   `io::stderr()` write, which the test harness does not capture, so the count shows in the output of a
//!   passing run:
//!
//!   ```text
//!   cargo test --locked -p athena-testkit --test stand_in_stress
//!   ```
//!
//! This file is the one test file the scan in `no_in_process_exec_writes.rs` skips, because the comparison
//! variant is written the old way on purpose.

use athena_testkit::write_executable;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

const THREADS: usize = 8;
const ROUNDS: usize = 60;

/// Runs the load with `write` as the way to make a stand-in, and returns how many runs failed with `ETXTBSY`.
/// Any other failure, or a stand-in that prints something other than expected, panics.
fn busy_runs(write: fn(&Path, &str) -> std::io::Result<()>) -> usize {
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|worker| {
                scope.spawn(move || {
                    let dir = tempfile::tempdir().unwrap();
                    let mut busy = 0;
                    for round in 0..ROUNDS {
                        let path = dir.path().join(format!("stand-in-{round}"));
                        write(&path, &format!("#!/bin/sh\necho {worker}-{round}\n")).unwrap();
                        match Command::new(&path).output() {
                            Ok(output) => {
                                assert!(output.status.success());
                                assert_eq!(
                                    String::from_utf8_lossy(&output.stdout),
                                    format!("{worker}-{round}\n")
                                );
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                                busy += 1;
                            }
                            Err(e) => panic!("running {} failed: {e}", path.display()),
                        }
                    }
                    busy
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().unwrap())
            .sum::<usize>()
    })
}

fn through_the_helper(path: &Path, script: &str) -> std::io::Result<()> {
    write_executable(path, script)
}

fn in_process(path: &Path, script: &str) -> std::io::Result<()> {
    std::fs::write(path, script)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[test]
fn the_helper_never_meets_a_busy_file() {
    let busy = busy_runs(through_the_helper);
    // A direct write to the stderr handle, which the harness does not capture, so it shows in a passing run.
    writeln!(
        std::io::stderr(),
        "ETXTBSY runs with write_executable: {busy} of {}",
        THREADS * ROUNDS
    )
    .unwrap();
    assert_eq!(busy, 0, "the helper must never leave a stand-in busy");
}

#[test]
fn the_in_process_write_is_shown_for_comparison() {
    let busy = busy_runs(in_process);
    writeln!(
        std::io::stderr(),
        "ETXTBSY runs with an in-process write: {busy} of {} (scheduling-dependent; may be 0)",
        THREADS * ROUNDS
    )
    .unwrap();
}
