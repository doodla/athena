//! Athena's setup and CI tasks (design §11).
//!
//! - `cargo run -p xtask -- setup [--ci] [--force] [--allow-dirty]`: takes a machine to a working Athena.
//!   Every step is idempotent. `./setup.sh` installs rustup if needed and then runs this.
//! - `cargo run -p xtask -- ci`: fmt, clippy and tests in sequence, runnable locally. CI itself runs them as
//!   parallel jobs (see `.github/workflows/ci.yml`), the tests split by `test-shard`.
//! - `cargo run -p xtask -- test-shard <k> <n>`: one of `n` disjoint slices of the workspace's tests (see `shard`),
//!   balanced by the recorded seconds in `xtask/shard-timings.txt`.
//! - `cargo run -p xtask -- shard-timings`: runs every test unit and rewrites that timings file.
//! - `cargo run -p xtask -- walltime`: wall time per verified result from the job records.
//! - `cargo run -p xtask -- fingerprint` and `ci-cache`: let the CI gate job reuse an earlier pass for
//!   unchanged governed content (see `ci_cache`).
//! - `cargo run -p xtask -- build-inputs`: prints the globs the athena binary is built from
//!   (`BUILD_INPUT_GLOBS`). `setup --ci` skips the build when a stamp beside the binary equals their digest, and
//!   the `athena-bin` cache key in `.github/workflows/ci.yml` hashes the same globs.
//!
//! Setup reads every pin from `athena.toml`, reads credentials only to report whether they work, and never
//! stores them.

use anyhow::{Context, Result, anyhow, bail};
use athena_data::{Config, paths};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

mod ci_cache;
mod shard;
mod site;
mod walltime;

const SITE_USAGE: &str = "cargo run -p xtask -- site [--out <dir>]\n       \
     writes <dir>/index.html; the default <dir> is site/ in the cargo target directory ($CARGO_TARGET_DIR, else target/),\n       \
     so building never changes the work tree (a verifier's snapshot stays unchanged); an --out inside the work tree is refused";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("setup") => setup(&Flags::parse(&args[1..])),
        Some("ci") => ci(),
        Some("test-shard") => shard::command(&args[1..], &repo_root()),
        Some("shard-timings") => shard::timings_command(&repo_root()),
        Some("site") => site_command(&args[1..]),
        Some("walltime") => walltime::command(&args[1..], &repo_root()),
        Some("fingerprint") => ci_cache::fingerprint_command(),
        Some("ci-cache") => ci_cache::command(&args[1..]),
        Some("build-inputs") => build_inputs_command(),
        _ => {
            eprintln!(
                "usage: cargo run -p xtask -- setup [--ci] [--force] [--allow-dirty]\n       cargo run -p xtask -- ci\n       cargo run -p xtask -- build-inputs\n       {}\n       {SITE_USAGE}\n       {}\n       {}",
                shard::USAGE,
                walltime::USAGE,
                ci_cache::USAGE
            );
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// `site [--out <dir>]`: renders the explainer page to `<dir>/index.html`. The default is `site/` in the
/// cargo target directory (`$CARGO_TARGET_DIR`, else `target/`), so building never writes into the work
/// tree and a verifier's snapshot stays unchanged. An `--out` inside the work tree (other than under the
/// target directory) is refused.
fn site_command(args: &[String]) -> Result<()> {
    let root = repo_root();
    let cwd = std::env::current_dir().context("cannot read the current directory")?;
    let env = std::env::var_os("CARGO_TARGET_DIR");
    let target = site::target_dir(&root, env.clone(), &cwd);
    let out = match args {
        [] => site::default_out(&root, env, &cwd),
        [flag, dir] if flag == "--out" => PathBuf::from(dir),
        _ => bail!("{SITE_USAGE}"),
    };
    site::check_out(&root, &out, &target, &cwd)?;
    site::build(&root, &out)
}

#[derive(Debug, Default)]
struct Flags {
    ci: bool,
    force: bool,
    allow_dirty: bool,
}

impl Flags {
    fn parse(args: &[String]) -> Self {
        Flags {
            ci: args.iter().any(|a| a == "--ci"),
            force: args.iter().any(|a| a == "--force"),
            allow_dirty: args.iter().any(|a| a == "--allow-dirty"),
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the repository")
        .to_path_buf()
}

fn step(name: &str) {
    println!("\n== {name}");
}

fn run(cmd: &mut Command) -> Result<()> {
    let shown = format!("{cmd:?}");
    let status = cmd
        .status()
        .with_context(|| format!("could not run {shown}"))?;
    if !status.success() {
        bail!("{shown} failed with {status}");
    }
    Ok(())
}

fn output(cmd: &mut Command) -> Result<String> {
    let shown = format!("{cmd:?}");
    let out = cmd
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("could not run {shown}"))?;
    if !out.status.success() {
        bail!("{shown} failed with {}", out.status);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn found(program: &str, arg: &str) -> Option<String> {
    let out = Command::new(program)
        .arg(arg)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status.success().then(|| {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string()
    })
}

fn setup(flags: &Flags) -> Result<()> {
    refuse_inside_a_job(std::env::var_os("ATHENA_NESTED"))?;
    let root = repo_root();
    let config = Config::parse(&std::fs::read_to_string(root.join("athena.toml"))?)
        .map_err(|e| anyhow!("athena.toml: {e}"))?;

    step("1. Prerequisites");
    let git = found("git", "--version")
        .ok_or_else(|| anyhow!("git is required; install git 2.40 or later"))?;
    let version = git.split_whitespace().nth(2).unwrap_or("0.0");
    let mut parts = version.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    if (major, minor) < (2, 40) {
        bail!(
            "git {version} is too old: Athena needs 2.40 or later (check-attr --source, merge-tree --merge-base)"
        );
    }
    println!("git {version}");
    let cc = found("cc", "--version").ok_or_else(|| {
        anyhow!("a C linker (cc) is required by cargo; install your platform's build tools")
    })?;
    println!("cc: {cc}");
    let curl = found("curl", "--version")
        .ok_or_else(|| anyhow!("curl is required to download pinned tools"))?;
    println!("{}", curl.split(' ').take(2).collect::<Vec<_>>().join(" "));

    step("2. Toolchain and crates");
    println!(
        "{}",
        output(Command::new("rustc").arg("--version").current_dir(&root))?
    );
    run(Command::new("cargo")
        .args(["fetch", "--locked"])
        .current_dir(&root))?;

    step("3. gitleaks");
    install_gitleaks(&config)?;

    step("4. sccache");
    if skip_sccache(flags.ci) {
        println!(
            "--ci: skipping sccache; CI sets no RUSTC_WRAPPER, so it would only add build time"
        );
    } else {
        install_sccache(&config)?;
    }

    step("5. Athena");
    let binary = build_and_adopt(&root, flags)?;

    if flags.ci {
        println!("\n--ci: skipping hooks, Claude Code, and credentials.");
        return Ok(());
    }

    step("6. Git hooks");
    install_hooks(&root, &binary, flags.force)?;

    step("7. Claude Code");
    let claude = ensure_claude(&config)?;

    step("8. Credentials");
    check_credentials(&claude);

    step("Ready");
    println!(
        "athena {} adopted at {}",
        env!("CARGO_PKG_VERSION"),
        binary.display()
    );
    println!("next: athena ask \"<a one-line request>\"   (or: athena status)");
    Ok(())
}

/// Setup changes the machine: it installs tools and adopts the binary the git hooks run. Agents and checks
/// run with the nested-run marker set, and none of them may do that (acceptance verification 1, m4).
fn refuse_inside_a_job(marker: Option<std::ffi::OsString>) -> Result<()> {
    match marker.filter(|v| !v.is_empty()) {
        Some(job) => bail!(
            "setup does not run inside an Athena job (ATHENA_NESTED={}): it installs tools and adopts the \
             binary that the git hooks run, for the whole machine",
            job.to_string_lossy()
        ),
        None => Ok(()),
    }
}

/// The path of the `athena` binary in cargo's `--message-format=json` output, wherever the target
/// directory is (`CARGO_TARGET_DIR` included; acceptance verification 1, m5).
fn built_binary(messages: &str) -> Option<PathBuf> {
    messages
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|m| m["reason"] == "compiler-artifact" && m["target"]["name"] == "athena")
        .filter_map(|m| m["executable"].as_str().map(PathBuf::from))
        .next_back()
}

/// Whether the gitleaks at `target` exists and reports exactly the pinned version. A copy restored from the CI
/// cache passes only if it is this version, so a stale entry is never trusted.
fn gitleaks_present(target: &Path, version: &str) -> bool {
    target.exists()
        && found(target.to_str().unwrap_or(""), "version").is_some_and(|v| v.trim() == version)
}

/// Everything the athena binary is built from, as repository-relative paths. An entry ending in `/**` is a
/// directory and every file below it; any other entry is one file. This is the single list: `inputs_digest` hashes
/// it, `xtask build-inputs` prints it, and `xtask/tests/policy.rs` requires every `athena-bin` cache key in
/// `.github/workflows/ci.yml` to hash exactly these globs and to cover every file the binary compiles in.
///
/// The directories are whole, not just `*.rs`, because the binary embeds prompts (`include_str!` of
/// `crates/*/prompts/*.md`) and `docs/system-facts.md`. `docs/` as a whole is not listed: the other documents are
/// not compiled in, and listing them would discard the cache on every documentation change.
const BUILD_INPUT_GLOBS: [&str; 9] = [
    "crates/**",
    "xtask/**",
    "rust-toolchain.toml",
    "Cargo.toml",
    "Cargo.lock",
    "athena.toml",
    "defaults.toml",
    "setup.sh",
    "docs/system-facts.md",
];

/// `xtask build-inputs`: prints `BUILD_INPUT_GLOBS`, one per line.
fn build_inputs_command() -> Result<()> {
    for glob in BUILD_INPUT_GLOBS {
        println!("{glob}");
    }
    Ok(())
}

/// The files under `root` that `BUILD_INPUT_GLOBS` names, as sorted repository-relative paths. A file that does
/// not exist is simply absent, which changes the digest like any other difference.
fn input_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if path.is_file() {
                out.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    for glob in BUILD_INPUT_GLOBS {
        match glob.strip_suffix("/**") {
            Some(dir) => walk(root, &root.join(dir), &mut files)?,
            None if root.join(glob).is_file() => files.push(PathBuf::from(glob)),
            None => {}
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// A SHA-256 over the OS, the architecture, and each input file's path and content (length-prefixed), in path
/// order. Any change to a compiled-in file, a manifest or a pin changes it.
fn inputs_digest(root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update([0]);
    hasher.update(std::env::consts::ARCH.as_bytes());
    hasher.update([0]);
    for rel in input_files(root)? {
        let content = std::fs::read(root.join(&rel))
            .with_context(|| format!("reading {} for the inputs digest", rel.display()))?;
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(&content);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// The stamp beside the adopted binary: the digest of the inputs it was built from.
fn stamp_path(home: &Path) -> PathBuf {
    home.join("bin/athena.inputs")
}

/// Whether `setup --ci` may skip the build: only with `--ci`, an adopted binary that is a file, and a stamp equal
/// to `digest`. Without `--ci` the build always runs, so a local setup keeps its dirty-tree check.
fn can_skip_build(ci: bool, home: &Path, digest: &str) -> bool {
    ci && home.join("bin/athena").is_file()
        && std::fs::read_to_string(stamp_path(home)).is_ok_and(|s| s.trim() == digest)
}

/// Runs `build` with the stamp removed, and writes the stamp (holding `digest`, taken before the build) only if
/// `build` succeeded. A failed build, or one that stops partway, therefore never leaves a stamp that vouches
/// for a binary built from other inputs.
fn rebuild_stamped<T>(home: &Path, digest: &str, build: impl FnOnce() -> Result<T>) -> Result<T> {
    let stamp = stamp_path(home);
    match std::fs::remove_file(&stamp) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("removing {}", stamp.display())),
    }
    let built = build()?;
    std::fs::create_dir_all(stamp.parent().expect("stamp has a parent"))?;
    std::fs::write(&stamp, digest)?;
    Ok(built)
}

fn install_gitleaks(config: &Config) -> Result<()> {
    let pin = &config.tools.gitleaks;
    let target = paths::tool_path("gitleaks", &pin.version);
    if gitleaks_present(&target, &pin.version) {
        println!(
            "gitleaks {} already installed at {}; skipped the install",
            pin.version,
            target.display()
        );
        return Ok(());
    }
    let platform =
        paths::platform_key().ok_or_else(|| anyhow!("no gitleaks pin for this platform"))?;
    let expected = pin
        .sha256
        .get(platform)
        .ok_or_else(|| anyhow!("athena.toml pins no gitleaks checksum for {platform}"))?;
    let url = format!(
        "https://github.com/gitleaks/gitleaks/releases/download/v{v}/gitleaks_{v}_{platform}.tar.gz",
        v = pin.version
    );
    let dir = tempfile::tempdir()?;
    let archive = dir.path().join("gitleaks.tar.gz");
    println!("downloading {url}");
    run(Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(&archive)
        .arg(&url))?;
    let bytes = std::fs::read(&archive)?;
    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if &actual != expected {
        bail!(
            "gitleaks archive checksum mismatch: expected {expected}, got {actual}; refusing to install"
        );
    }
    println!("sha256 verified: {actual}");
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
    let mut installed = false;
    for entry in tar.entries()? {
        let mut entry = entry?;
        if entry.path()?.file_name().is_some_and(|n| n == "gitleaks") {
            std::fs::create_dir_all(target.parent().expect("tool path has a parent"))?;
            let partial = target.with_extension("partial");
            entry.unpack(&partial)?;
            set_executable(&partial)?;
            std::fs::rename(&partial, &target)?;
            installed = true;
            break;
        }
    }
    if !installed {
        bail!("the gitleaks archive has no gitleaks binary");
    }
    println!("installed gitleaks {} at {}", pin.version, target.display());
    Ok(())
}

/// Whether setup leaves sccache out: CI never uses it, and compiling it roughly doubles the CI time.
fn skip_sccache(ci: bool) -> bool {
    ci
}

/// Installs the pinned sccache with `cargo install --locked`, when `athena.toml` pins one. The install root
/// is `tools/sccache/<version>` in the install directory, so there is no archive and no digest to keep.
fn install_sccache(config: &Config) -> Result<()> {
    let Some(pin) = &config.tools.sccache else {
        println!("no sccache pinned in athena.toml; skipping");
        return Ok(());
    };
    install_sccache_at(&paths::athena_home(), &pin.version, run)
}

/// The install step with the command runner injected, so tests need no network.
fn install_sccache_at(
    home: &Path,
    version: &str,
    runner: impl FnOnce(&mut Command) -> Result<()>,
) -> Result<()> {
    let binary = paths::sccache_binary_in(home, version);
    if binary.is_file() {
        println!(
            "sccache {version} already installed at {}",
            binary.display()
        );
        return Ok(());
    }
    let root = paths::sccache_root(home, version);
    println!("building sccache {version} with cargo install --locked (this can take minutes)");
    runner(
        Command::new("cargo")
            .args(["install", "--locked", "--version", version, "--root"])
            .arg(&root)
            .arg("sccache"),
    )
    .with_context(|| {
        format!("installing sccache {version} failed; fix the error above, or remove [tools.sccache] from athena.toml to go without it")
    })?;
    if !binary.is_file() {
        bail!(
            "cargo install reported success but {} is missing",
            binary.display()
        );
    }
    println!("installed sccache {version} at {}", binary.display());
    Ok(())
}

/// Builds and adopts the athena binary, or with `--ci` reuses a cached one built from the same inputs.
///
/// The skip needs `--ci`, an adopted binary and a stamp equal to the digest of `BUILD_INPUT_GLOBS`. It runs
/// before the dirty-tree refusal, which CI does not need because its checkout is clean; a local setup never
/// skips. Every other path removes the stamp, builds, and writes the stamp only after the build and adoption
/// succeeded, so a later non-CI build cannot leave a stale stamp. A skipped build keeps the binary of the run that
/// built it, so `athena --version` there names that run's commit.
fn build_and_adopt(root: &Path, flags: &Flags) -> Result<PathBuf> {
    let home = paths::athena_home();
    let digest = inputs_digest(root)?;
    if can_skip_build(flags.ci, &home, &digest) {
        let adopted = home.join("bin/athena");
        set_executable(&adopted)?;
        println!(
            "--ci: skipped the athena build; the cached binary at {} was built from these inputs (digest {}). \
             The dirty-tree check does not run on this path",
            adopted.display(),
            &digest[..12]
        );
        link_local_bin(&adopted)?;
        return Ok(adopted);
    }
    rebuild_stamped(&home, &digest, || build_fresh(root, flags))
}

fn build_fresh(root: &Path, flags: &Flags) -> Result<PathBuf> {
    let commit = output(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root),
    )?;
    let status = output(
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root),
    )?;
    let dirty = !status.is_empty();
    if dirty && !flags.allow_dirty {
        bail!(
            "the working tree has uncommitted changes, so this build would not name a commit (D45). \
             Commit or stash them, or adopt anyway with --allow-dirty"
        );
    }
    let messages = output(
        Command::new("cargo")
            .args([
                "build",
                "--release",
                "--locked",
                "-p",
                "athena-cli",
                "--message-format=json-render-diagnostics",
            ])
            // These two feed `athena --version` through wiring::build_info().
            .env("ATHENA_BUILD_COMMIT", &commit)
            .env("ATHENA_BUILD_DIRTY", if dirty { "true" } else { "false" })
            .current_dir(root),
    )?;
    let built = built_binary(&messages)
        .ok_or_else(|| anyhow!("cargo built athena but did not say where"))?;
    let adopted = paths::athena_home().join("bin/athena");
    std::fs::create_dir_all(adopted.parent().expect("bin has a parent"))?;
    let same = std::fs::read(&adopted).ok() == Some(std::fs::read(&built)?);
    if same {
        println!(
            "athena at {} is already this build ({})",
            adopted.display(),
            &commit[..12]
        );
    } else {
        let partial = adopted.with_extension("partial");
        std::fs::copy(&built, &partial)?;
        set_executable(&partial)?;
        std::fs::rename(&partial, &adopted)?;
        println!(
            "adopted athena built from {}{} at {}",
            &commit[..12],
            if dirty { " (dirty)" } else { "" },
            adopted.display()
        );
    }
    link_local_bin(&adopted)?;
    Ok(adopted)
}

/// Points `~/.local/bin/athena` at the adopted binary.
fn link_local_bin(adopted: &Path) -> Result<()> {
    if let Some(home) = std::env::var_os("HOME") {
        let local_bin = PathBuf::from(home).join(".local/bin");
        std::fs::create_dir_all(&local_bin)?;
        let link = local_bin.join("athena");
        let points_here = std::fs::read_link(&link).ok().as_deref() == Some(adopted);
        if !points_here {
            if link.symlink_metadata().is_ok() {
                std::fs::remove_file(&link)?;
            }
            symlink(adopted, &link)?;
        }
        let on_path = std::env::var_os("PATH")
            .is_some_and(|p| std::env::split_paths(&p).any(|d| d == local_bin));
        println!(
            "{} -> {}{}",
            link.display(),
            adopted.display(),
            if on_path {
                ""
            } else {
                "  (add ~/.local/bin to PATH)"
            }
        );
    }
    Ok(())
}

fn install_hooks(root: &Path, binary: &Path, force: bool) -> Result<()> {
    let common = output(
        Command::new("git")
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .current_dir(root),
    )?;
    let hooks = PathBuf::from(common).join("hooks");
    std::fs::create_dir_all(&hooks)?;
    for name in ["pre-commit", "commit-msg"] {
        let hook = hooks.join(name);
        match std::fs::read_link(&hook) {
            Ok(target) if target == binary => {
                println!("{name}: already linked");
                continue;
            }
            Ok(_) | Err(_) if hook.symlink_metadata().is_ok() && !force => {
                bail!(
                    "{} exists and is not Athena's; leaving it alone. Re-run with --force to replace it",
                    hook.display()
                );
            }
            _ => {}
        }
        if hook.symlink_metadata().is_ok() {
            std::fs::remove_file(&hook)?;
        }
        symlink(binary, &hook)?;
        println!("{name} -> {}", binary.display());
    }
    Ok(())
}

fn ensure_claude(config: &Config) -> Result<PathBuf> {
    let pin = &config.agents.claude_version;
    let local = paths::athena_home().join("claude-code").join(pin);
    let private = local.join("node_modules/.bin/claude");
    for candidate in [PathBuf::from("claude"), private.clone()] {
        if let Some(text) = found(candidate.to_str().unwrap_or("claude"), "--version")
            && text.split_whitespace().next() == Some(pin.as_str())
        {
            let resolved = if candidate == Path::new("claude") {
                which("claude").unwrap_or(candidate)
            } else {
                candidate
            };
            write_machine_config(&resolved)?;
            println!("Claude Code {pin} at {}", resolved.display());
            return Ok(resolved);
        }
    }
    let npm = found("npm", "--version").ok_or_else(|| {
        anyhow!("Claude Code {pin} is not installed, and npm is needed to install it privately; install Node.js and npm")
    })?;
    println!("installing Claude Code {pin} privately with npm {npm}");
    std::fs::create_dir_all(&local)?;
    run(Command::new("npm")
        .args(["install", "--no-fund", "--no-audit", "--prefix"])
        .arg(&local)
        .arg(format!("@anthropic-ai/claude-code@{pin}")))?;
    write_machine_config(&private)?;
    Ok(private)
}

fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(program))
        .find(|p| p.is_file())
}

fn write_machine_config(claude: &Path) -> Result<()> {
    let file = paths::athena_home().join("config.toml");
    let text = format!(
        "# Machine-local settings written by setup. Not committed.\nclaude_bin = {:?}\n",
        claude.to_string_lossy()
    );
    if std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
        std::fs::create_dir_all(file.parent().expect("config has a parent"))?;
        std::fs::write(&file, text)?;
    }
    Ok(())
}

fn check_credentials(claude: &Path) {
    // `claude auth status` reports whether Claude Code can authenticate, without a model call (E20).
    let out = Command::new(claude)
        .args(["auth", "status"])
        .stderr(Stdio::null())
        .output();
    let logged_in = out
        .ok()
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
        .and_then(|v| v.get("loggedIn").and_then(serde_json::Value::as_bool));
    match logged_in {
        Some(true) => println!("Claude Code can authenticate."),
        Some(false) => println!(
            "Claude Code is not logged in: run `claude` once to log in, or set ANTHROPIC_API_KEY."
        ),
        None => println!(
            "could not read `claude auth status`; Athena's agents may fail to authenticate."
        ),
    }
}

fn ci() -> Result<()> {
    let root = repo_root();
    for (name, args) in [
        ("fmt", vec!["fmt", "--all", "--check"]),
        (
            "clippy",
            vec![
                "clippy",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("test", vec!["test", "--locked"]),
    ] {
        step(name);
        run(Command::new("cargo").args(&args).current_dir(&root))?;
    }
    println!("\nci: fmt, clippy, and tests passed");
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn symlink(_target: &Path, _link: &Path) -> Result<()> {
    bail!("Athena's git hooks are symlinks, which need a Unix-like system")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_refuses_inside_a_job() {
        assert!(refuse_inside_a_job(Some("20261010-153012-k7q2".into())).is_err());
        assert!(refuse_inside_a_job(Some("".into())).is_ok());
        assert!(refuse_inside_a_job(None).is_ok());
    }

    #[test]
    fn ci_skips_sccache_and_normal_setup_does_not() {
        assert!(skip_sccache(true));
        assert!(!skip_sccache(false));
    }

    #[test]
    fn sccache_install_skips_when_present_and_fails_clearly() {
        let home = tempfile::tempdir().unwrap();
        let binary = paths::sccache_binary_in(home.path(), "9.9.9");
        // Failure of the install is reported with the version named.
        let err = install_sccache_at(home.path(), "9.9.9", |_| bail!("boom")).unwrap_err();
        assert!(format!("{err:#}").contains("installing sccache 9.9.9 failed"));
        // Success without the binary is also an error.
        assert!(install_sccache_at(home.path(), "9.9.9", |_| Ok(())).is_err());
        // The install command targets the versioned root.
        let mut seen = String::new();
        let _ = install_sccache_at(home.path(), "9.9.9", |cmd| {
            seen = format!("{cmd:?}");
            Ok(())
        });
        assert!(seen.contains("--locked") && seen.contains("tools/sccache/9.9.9"));
        // Already installed: the runner is never called.
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, "").unwrap();
        install_sccache_at(home.path(), "9.9.9", |_| bail!("must not run")).unwrap();
    }

    #[test]
    fn skip_only_with_ci_matching_stamp_and_binary() {
        let home = tempfile::tempdir().unwrap();
        let h = home.path();
        // No binary and no stamp: build.
        assert!(!can_skip_build(true, h, "abc"));
        std::fs::create_dir_all(h.join("bin")).unwrap();
        std::fs::write(h.join("bin/athena"), "x").unwrap();
        // A binary without a stamp: build.
        assert!(!can_skip_build(true, h, "abc"));
        std::fs::write(stamp_path(h), "abc\n").unwrap();
        // A binary, a matching stamp, and --ci: skip.
        assert!(can_skip_build(true, h, "abc"));
        // Built from other inputs: build.
        assert!(!can_skip_build(true, h, "def"));
        // Without --ci: build, whatever the stamp says.
        assert!(!can_skip_build(false, h, "abc"));
        // A stamp whose binary is gone: build.
        std::fs::remove_file(h.join("bin/athena")).unwrap();
        assert!(!can_skip_build(true, h, "abc"));
    }

    #[test]
    fn a_non_ci_build_leaves_no_stale_stamp() {
        let home = tempfile::tempdir().unwrap();
        let h = home.path();
        std::fs::create_dir_all(h.join("bin")).unwrap();
        std::fs::write(h.join("bin/athena"), "old").unwrap();
        std::fs::write(stamp_path(h), "D1").unwrap();
        assert!(can_skip_build(true, h, "D1"));
        // A successful build: the stamp is gone while it runs, and holds the new digest afterwards.
        let ran = rebuild_stamped(h, "D2", || {
            assert!(!stamp_path(h).exists(), "the old stamp is removed first");
            Ok(7)
        })
        .unwrap();
        assert_eq!(ran, 7);
        assert_eq!(std::fs::read_to_string(stamp_path(h)).unwrap(), "D2");
        assert!(!can_skip_build(true, h, "D1"));
        assert!(can_skip_build(true, h, "D2"));
        // A failed build leaves no stamp, so it cannot vouch for whatever binary is there.
        let failed: Result<()> = rebuild_stamped(h, "D3", || bail!("cargo failed"));
        assert!(failed.is_err());
        assert!(!stamp_path(h).exists());
        assert!(!can_skip_build(true, h, "D2"));
        assert!(!can_skip_build(true, h, "D3"));
    }

    /// A temp tree holding one file for every entry of `BUILD_INPUT_GLOBS`, with the compiled-in kinds a plain
    /// `*.rs` filter would miss: a prompt under `crates/` and `docs/system-facts.md`.
    fn input_tree() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        for glob in BUILD_INPUT_GLOBS {
            let rel = match glob.strip_suffix("/**") {
                Some("crates") => "crates/athena-context/prompts/producer.md".to_string(),
                Some(dir) => format!("{dir}/src/main.rs"),
                None => glob.to_string(),
            };
            let path = root.path().join(&rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, format!("content of {rel}")).unwrap();
        }
        std::fs::write(root.path().join("CHANGELOG.md"), "log").unwrap();
        root
    }

    #[test]
    fn the_inputs_digest_changes_when_a_compiled_in_file_is_edited() {
        let tree = input_tree();
        let r = tree.path();
        let before = inputs_digest(r).unwrap();
        assert_eq!(before, inputs_digest(r).unwrap());
        // A prompt, which the binary embeds with include_str!.
        let prompt = r.join("crates/athena-context/prompts/producer.md");
        std::fs::write(&prompt, "a changed prompt").unwrap();
        let after_prompt = inputs_digest(r).unwrap();
        assert_ne!(before, after_prompt);
        // docs/system-facts.md, also embedded.
        std::fs::write(r.join("docs/system-facts.md"), "changed facts").unwrap();
        let after_facts = inputs_digest(r).unwrap();
        assert_ne!(after_prompt, after_facts);
        // A new file under crates/, and each exact file.
        std::fs::write(r.join("crates/new.txt"), "new").unwrap();
        assert_ne!(after_facts, inputs_digest(r).unwrap());
        for glob in BUILD_INPUT_GLOBS.iter().filter(|g| !g.ends_with("/**")) {
            let base = inputs_digest(r).unwrap();
            std::fs::write(r.join(glob), "edited").unwrap();
            assert_ne!(base, inputs_digest(r).unwrap(), "{glob}");
        }
        // A file outside the list does not matter.
        let base = inputs_digest(r).unwrap();
        std::fs::write(r.join("CHANGELOG.md"), "other log").unwrap();
        assert_eq!(base, inputs_digest(r).unwrap());
    }

    #[test]
    fn input_files_follow_the_shared_list() {
        let tree = input_tree();
        let files = input_files(tree.path()).unwrap();
        assert_eq!(files.len(), BUILD_INPUT_GLOBS.len());
        assert!(files.contains(&PathBuf::from("crates/athena-context/prompts/producer.md")));
        assert!(files.contains(&PathBuf::from("docs/system-facts.md")));
        assert!(!files.contains(&PathBuf::from("CHANGELOG.md")));
    }

    #[cfg(unix)]
    #[test]
    fn a_cached_gitleaks_is_skipped_only_when_it_reports_the_pinned_version() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("gitleaks");
        // Missing: install.
        assert!(!gitleaks_present(&fake, "8.1.0"));
        std::fs::write(&fake, "#!/bin/sh\necho 8.1.0\n").unwrap();
        set_executable(&fake).unwrap();
        // Present and the pinned version: skip.
        assert!(gitleaks_present(&fake, "8.1.0"));
        // Present but another version: install.
        assert!(!gitleaks_present(&fake, "9.0.0"));
    }

    #[test]
    fn the_built_binary_is_found_wherever_cargo_put_it() {
        let messages = concat!(
            r#"{"reason":"compiler-artifact","target":{"name":"athena_cli","kind":["lib"]},"executable":null}"#,
            "\n",
            r#"{"reason":"compiler-artifact","target":{"name":"athena","kind":["bin"]},"executable":"/elsewhere/release/athena"}"#,
            "\n",
            r#"{"reason":"build-finished","success":true}"#,
        );
        assert_eq!(
            built_binary(messages),
            Some(PathBuf::from("/elsewhere/release/athena"))
        );
        assert_eq!(built_binary(r#"{"reason":"build-finished"}"#), None);
    }
}
