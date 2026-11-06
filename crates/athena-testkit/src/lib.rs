//! Test-only helpers shared by the tests of this workspace. Nothing here is used by shipped code: depend on
//! this crate under `[dev-dependencies]` only.
//!
//! # Why stand-in scripts are written by a child process
//!
//! Tests often write a small executable script (a stand-in for `claude`, `gh`, `cargo` or a build of
//! `athena`) and then run it, sometimes through the code under test. On Linux, `execve` fails with `ETXTBSY`
//! ("Text file busy") when any process still holds the file open for writing. A test process that writes the
//! script itself holds such a handle for a moment, and a *different* test thread that forks in that moment
//! gives its child a copy of the handle, which stays open until the child's own `exec` closes it. Running the
//! script during that window fails, and retrying at each run site cannot cover runs made inside the code
//! under test.
//!
//! [`write_executable`] closes the window at its source: it starts a short-lived `sh` child that receives the
//! bytes on its standard input, writes them to a temporary file next to the target, makes the file
//! executable, and renames it onto the target. The test process only ever holds a pipe, never a handle to
//! the script, so no fork in another thread can inherit one, and the target path is never visible
//! half-written or without its executable bit.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// The shell program the child runs. `$1` is the temporary path and `$2` the final path; `&&` stops at the
/// first failure, so a failed `cat` or `chmod` never moves a partial file into place.
const WRITE_PROGRAM: &str = r#"cat > "$1" && chmod 755 "$1" && mv "$1" "$2""#;

/// Distinguishes the temporary files of concurrent calls in one process.
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Writes `bytes` to `path` as an executable file (mode 755), replacing anything there, without this process
/// ever opening the file for writing. See the crate documentation for why.
///
/// The directory of `path` must exist. The temporary file is created in that same directory (so the final
/// rename stays on one filesystem) and is removed if the write fails. Returns an error if the child cannot
/// be started or exits unsuccessfully.
pub fn write_executable(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the path has no file name"))?;
    let mut temp_name = name.to_os_string();
    temp_name.push(format!(
        ".stand-in-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let temp: PathBuf = path.with_file_name(temp_name);

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(WRITE_PROGRAM)
        .arg("stand-in-writer")
        .arg(&temp)
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()?;
    // The pipe is the only thing this process writes to. A child that exits early closes it, which shows
    // here as a broken pipe; the exit status below is the better report, so that error is set aside.
    let sent = {
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(bytes.as_ref())
    };
    let status = child.wait()?;
    if !status.success() {
        let _ = std::fs::remove_file(&temp);
        return Err(io::Error::other(format!(
            "writing the stand-in {} failed: {status}",
            path.display()
        )));
    }
    sent
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn writes_the_bytes_executable_and_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        write_executable(&path, "#!/bin/sh\necho hi\n").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"#!/bin/sh\necho hi\n");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
        let out = Command::new(&path).output().unwrap();
        assert_eq!(out.stdout, b"hi\n");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn replaces_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        std::fs::write(&path, "old").unwrap();
        write_executable(&path, "#!/bin/sh\nexit 0\n").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"#!/bin/sh\nexit 0\n");
    }

    #[test]
    fn a_missing_directory_is_an_error_and_leaves_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("tool");
        assert!(write_executable(&path, "x").is_err());
        assert!(!path.exists());
    }
}
