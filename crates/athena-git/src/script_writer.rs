//! Writing an executable stand-in script so that running it right away cannot fail with `ETXTBSY`.
//!
//! Tests write small scripts and run them, sometimes through production code. On Linux, `exec` of a file
//! fails with "Text file busy" (`ETXTBSY`) while any process holds that file open for writing. A test that
//! writes the script itself holds a write handle for a moment, and another test thread that forks in that
//! moment copies the handle into its child, which keeps it open until the child execs. The first test's own
//! run of the script then fails, though nothing is wrong with it.
//!
//! [`write_executable`] avoids the window: a short-lived `sh` child writes the bytes to a temporary file next
//! to the target, sets mode 755, and renames it into place; the caller waits for the child. The calling
//! process never opens the script for writing, so no fork in it can carry a write handle to the script. The
//! child's own handle closes before the rename, so the final path is never open for writing in any process.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// The shell program the child runs. The target path is `$1`; the bytes arrive on standard input.
const PROGRAM: &str = r#"cat > "$1.writing" && chmod 755 "$1.writing" && mv "$1.writing" "$1""#;

/// Writes `bytes` to `path` as an executable file (mode 755), through a child process, and waits for it.
///
/// An existing file at `path` is replaced. Panics if the child cannot be run or fails, which is the failure
/// a test wants. Unix only: it needs `sh`, `cat`, `chmod` and `mv`.
pub fn write_executable(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
    let path = path.as_ref();
    let mut child = Command::new("sh")
        .args(["-c", PROGRAM, "sh"])
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn sh to write a script");
    // Dropping the handle after the write closes the pipe, which ends `cat`.
    let mut stdin = child.stdin.take().expect("child stdin");
    stdin.write_all(bytes.as_ref()).expect("send script bytes");
    drop(stdin);
    let status = child.wait().expect("wait for the script writer");
    assert!(
        status.success(),
        "writing {} failed: {status}",
        path.display()
    );
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn the_script_is_written_executable_and_replaces_an_earlier_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        std::fs::write(&path, "old").unwrap();
        write_executable(&path, "#!/bin/sh\necho hi\n");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "#!/bin/sh\necho hi\n"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!dir.path().join("tool.writing").exists());
        let out = Command::new(&path).output().unwrap();
        assert_eq!(out.stdout, b"hi\n");
    }

    #[test]
    fn many_write_then_exec_cycles_succeed_while_other_threads_fork() {
        let stop = Arc::new(AtomicBool::new(false));
        let forkers: Vec<_> = (0..4)
            .map(|_| {
                let stop = Arc::clone(&stop);
                std::thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        Command::new("true").status().unwrap();
                    }
                })
            })
            .collect();
        let writers: Vec<_> = (0..4)
            .map(|t| {
                std::thread::spawn(move || {
                    let dir = tempfile::tempdir().unwrap();
                    for i in 0..100 {
                        let path = dir.path().join(format!("stand-in-{i}"));
                        write_executable(&path, format!("#!/bin/sh\necho {t}-{i}\n"));
                        let out = Command::new(&path)
                            .output()
                            .unwrap_or_else(|e| panic!("exec of cycle {t}-{i} failed: {e}"));
                        assert!(out.status.success());
                        assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{t}-{i}\n"));
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
    }
}
