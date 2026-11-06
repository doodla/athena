//! Test-only helpers shared by the tests of every Athena crate. Nothing here is used by the shipped binary:
//! other crates list `athena-testkit` under `[dev-dependencies]` only.
//!
//! # Why [`write_executable`] exists
//!
//! Many tests stand in for an external program (`claude`, `gh`, `gitleaks`, `cargo`, an `athena` build) with
//! a small shell script, write it to a temporary directory, and run it. On Linux that races when it is done
//! naively. A test thread that writes the script keeps the file open for writing until it closes it. The test
//! harness runs tests on many threads, and any other thread that starts a child process (`fork`) in that window
//! gives the child a copy of the open descriptor. Running the script then fails with `ETXTBSY` ("Text file
//! busy"), because the kernel refuses to execute a file that some process still has open for writing, and the
//! copy lives on in the other thread's child until that child calls `exec`. The failure is rare and
//! depends on scheduling, which makes it a flaky test.
//!
//! The fix is to never open the script for writing in the test process. [`write_executable`] hands the bytes to
//! a short-lived `sh` child, which writes, marks the file executable, and renames it into place, and the test
//! waits for that child to exit. The only process that ever had the file open for writing is gone before the
//! function returns, and the rename means the final path never names a half-written file.
//!
//! Tests must not retry around `ETXTBSY` at the place that runs the script, and must not write an executable
//! in-process. The scan test `tests/no_in_process_exec_writes.rs` fails on any test code that calls
//! `set_permissions`, `Permissions::from_mode` or `PermissionsExt::set_mode`, and `tests/stand_in_stress.rs`
//! shows the difference between the two ways of writing.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// The shell program the child runs. `$1` is a temporary path next to the final one and `$2` the final path.
/// `cat` writes the script from standard input, `chmod 755` makes it executable, and `mv` renames it into place.
const WRITE_SCRIPT: &str = r#"cat > "$1" && chmod 755 -- "$1" && mv -- "$1" "$2""#;

/// Makes the temporary names of concurrent calls (in one process) differ.
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Writes `contents` to `path` as an executable file (mode 755), through a child process.
///
/// The child is `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'`: the bytes go to its standard input, it
/// writes them to a temporary file in the same directory as `path`, and it renames that file to `path`. This
/// process never opens the file for writing, so no thread of the test can hand a writable copy of it to a
/// forked child, and running `path` afterwards cannot fail with `ETXTBSY` because of this write. An existing
/// file at `path` is replaced.
///
/// The directory of `path` must exist. Returns an error if the child cannot be started or exits non-zero; the
/// error text carries the child's standard error.
pub fn write_executable(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref();
    let temporary = temporary_beside(path);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(WRITE_SCRIPT)
        .arg("athena-testkit-write") // $0, the name the shell reports in errors
        .arg(&temporary)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().expect("stdin was requested as a pipe");
    let sent = stdin.write_all(contents.as_ref());
    // Closing the pipe is what tells `cat` the script is complete.
    drop(stdin);
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "writing the executable {} failed ({}): {}",
            path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    sent
}

/// A path in the directory of `path` that nothing else uses, for the child to write before the rename.
fn temporary_beside(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let unique = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(".{name}.tmp-{}-{unique}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn the_file_is_executable_has_the_exact_bytes_and_runs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stand-in");
        write_executable(&path, "#!/bin/sh\necho hello\n").unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"#!/bin/sh\necho hello\n".to_vec()
        );
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
        let output = Command::new(&path).output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
    }

    #[test]
    fn an_existing_file_is_replaced_and_no_temporary_file_is_left() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stand-in");
        write_executable(&path, "#!/bin/sh\necho one\n").unwrap();
        write_executable(&path, "#!/bin/sh\necho two\n").unwrap();
        let output = Command::new(&path).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), "two\n");
        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["stand-in"]);
    }

    #[test]
    fn a_missing_directory_is_an_error_that_names_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("stand-in");
        let error = write_executable(&path, "#!/bin/sh\n").unwrap_err();
        assert!(error.to_string().contains("stand-in"), "{error}");
    }

    #[test]
    fn a_path_with_spaces_and_quotes_is_passed_through_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a b 'c' \"d\"");
        write_executable(&path, "#!/bin/sh\nexit 0\n").unwrap();
        assert!(Command::new(&path).status().unwrap().success());
    }

    #[test]
    fn contents_larger_than_a_pipe_buffer_arrive_whole() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big");
        let body = format!("#!/bin/sh\n# {}\nexit 0\n", "x".repeat(1 << 20));
        write_executable(&path, &body).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), body.into_bytes());
    }
}
