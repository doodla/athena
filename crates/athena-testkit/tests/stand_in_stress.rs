//! A stress test of the `ETXTBSY` race that `write_executable` removes.
//!
//! Writer threads write a script and run it at once, while forker threads start short processes. The helper
//! variant must meet no busy file. The in-process variant is the control: it writes the old way, and prints how
//! many busy files it met (the count depends on timing, so it is reported, not asserted). The two variants run
//! one at a time so one's forkers do not skew the other's count.

use std::io::ErrorKind;
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const WRITERS: usize = 4;
const CYCLES: usize = 200;
const FORKERS: usize = 4;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Runs `write` then the script, `WRITERS * CYCLES` times, with forkers running; returns the busy count.
fn run(write: fn(&std::path::Path, &str)) -> usize {
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let stop = AtomicBool::new(false);
    let busy = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let forkers: Vec<_> = (0..FORKERS)
            .map(|_| {
                scope.spawn(|| {
                    while !stop.load(Ordering::Relaxed) {
                        Command::new("true").status().unwrap();
                    }
                })
            })
            .collect();
        let writers: Vec<_> = (0..WRITERS)
            .map(|t| {
                let busy = &busy;
                scope.spawn(move || {
                    let dir = tempfile::tempdir().unwrap();
                    for i in 0..CYCLES {
                        let path = dir.path().join(format!("stand-in-{i}"));
                        write(&path, &format!("#!/bin/sh\necho {t}-{i}\n"));
                        match Command::new(&path).output() {
                            Ok(out) => {
                                assert!(out.status.success());
                                assert_eq!(
                                    String::from_utf8_lossy(&out.stdout),
                                    format!("{t}-{i}\n")
                                );
                            }
                            Err(e) if e.kind() == ErrorKind::ExecutableFileBusy => {
                                busy.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => panic!("exec of cycle {t}-{i} failed: {e}"),
                        }
                    }
                })
            })
            .collect();
        for w in writers {
            w.join().unwrap();
        }
        stop.store(true, Ordering::Relaxed);
        for f in forkers {
            f.join().unwrap();
        }
    });
    busy.load(Ordering::Relaxed)
}

#[test]
fn helper_variant_never_meets_a_busy_script() {
    let hits = run(|path, body| athena_testkit::write_executable(path, body));
    eprintln!(
        "helper variant: {hits} ETXTBSY in {} cycles",
        WRITERS * CYCLES
    );
    assert_eq!(hits, 0);
}

#[test]
fn in_process_variant_reports_its_busy_count() {
    use std::os::unix::fs::PermissionsExt;
    let hits = run(|path, body| {
        std::fs::write(path, body).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    });
    eprintln!(
        "in-process variant: {hits} ETXTBSY in {} cycles",
        WRITERS * CYCLES
    );
}
