//! `opctl`: the operator's side of the Athena build experiment.
//!
//! It owns one thing: the single-operator lease in `operator/LEASE.toml`, which records which
//! session operates this repository and since when, so two operators never push at once.
//!
//! The lease of record is the one on `origin/main`. A session may push only while it holds that
//! lease, or while the lease there is released or expired and its own working copy claims it (the
//! push that carries the claim). `opctl install-hook` links `.git/hooks/pre-push` to this binary,
//! so every push from the clone, the operator's or Athena's, passes that check.
//!
//! This tool is sandbox-side. Athena never calls it and knows nothing about it.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const LEASE_PATH: &str = "operator/LEASE.toml";
const DEFAULT_HOURS: u64 = 4;
const SESSION_ENV: &str = "CLAUDE_CODE_REMOTE_SESSION_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Held,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lease {
    state: State,
    holder: String,
    since_unix: u64,
    expires_unix: u64,
}

impl Lease {
    fn is_live(&self, now: u64) -> bool {
        self.state == State::Held && now < self.expires_unix
    }

    fn render(&self) -> String {
        let state = match self.state {
            State::Held => "held",
            State::Released => "released",
        };
        format!(
            "# Single-operator lease, managed by operator/opctl (see operator/HANDOFF.md).\n\
             # The copy on origin/main is the lease of record.\n\
             state = \"{state}\"\n\
             holder = \"{holder}\"\n\
             since = \"{since}\"\n\
             since_unix = {since_unix}\n\
             expires = \"{expires}\"\n\
             expires_unix = {expires_unix}\n",
            holder = self.holder,
            since = rfc3339(self.since_unix),
            since_unix = self.since_unix,
            expires = rfc3339(self.expires_unix),
            expires_unix = self.expires_unix,
        )
    }

    fn parse(text: &str) -> Result<Lease, String> {
        let mut state = None;
        let mut holder = None;
        let mut since_unix = None;
        let mut expires_unix = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("malformed lease line: {line}"));
            };
            let value = value.trim().trim_matches('"').to_string();
            match key.trim() {
                "state" => {
                    state = Some(match value.as_str() {
                        "held" => State::Held,
                        "released" => State::Released,
                        other => return Err(format!("unknown lease state: {other}")),
                    })
                }
                "holder" => holder = Some(value),
                "since_unix" => since_unix = Some(parse_u64(&value)?),
                "expires_unix" => expires_unix = Some(parse_u64(&value)?),
                _ => {} // human-readable mirrors such as `since` and `expires`
            }
        }
        Ok(Lease {
            state: state.ok_or("lease has no state")?,
            holder: holder.ok_or("lease has no holder")?,
            since_unix: since_unix.ok_or("lease has no since_unix")?,
            expires_unix: expires_unix.ok_or("lease has no expires_unix")?,
        })
    }
}

fn parse_u64(s: &str) -> Result<u64, String> {
    s.parse::<u64>().map_err(|e| format!("not a number: {s} ({e})"))
}

/// Why a push is allowed or refused. Pure, so the whole decision table is testable.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    AllowHolder,
    AllowClaim,
    Refuse(String),
}

fn push_verdict(of_record: Option<&Lease>, local: Option<&Lease>, me: &str, now: u64) -> Verdict {
    match of_record {
        Some(l) if l.is_live(now) && l.holder == me => Verdict::AllowHolder,
        Some(l) if l.is_live(now) => Verdict::Refuse(format!(
            "session {} holds the lease until {}; wait for it to release or expire",
            l.holder,
            rfc3339(l.expires_unix)
        )),
        // Released, expired, or no lease of record yet: only a push that carries our own claim.
        _ => match local {
            Some(l) if l.is_live(now) && l.holder == me => Verdict::AllowClaim,
            _ => Verdict::Refuse(
                "no live lease is held by this session; run `opctl lease acquire` and commit it"
                    .to_string(),
            ),
        },
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Formats unix seconds as an RFC 3339 UTC timestamp (civil-from-days, Howard Hinnant's method).
fn rfc3339(unix: u64) -> String {
    let days = (unix / 86_400) as i64;
    let secs = unix % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        secs / 3_600,
        (secs % 3_600) / 60,
        secs % 60
    )
}

fn repo_root() -> Result<PathBuf, String> {
    let out = git(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(out.trim()))
}

fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn me() -> Result<String, String> {
    std::env::var(SESSION_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("{SESSION_ENV} is not set; cannot tell which session this is"))
}

fn read_local(root: &Path) -> Result<Option<Lease>, String> {
    match std::fs::read_to_string(root.join(LEASE_PATH)) {
        Ok(text) => Lease::parse(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {LEASE_PATH}: {e}")),
    }
}

/// The lease of record: `operator/LEASE.toml` on a freshly fetched `origin/main`.
fn read_of_record() -> Result<Option<Lease>, String> {
    git(&["fetch", "--quiet", "origin", "main"])?;
    match git(&["show", &format!("origin/main:{LEASE_PATH}")]) {
        Ok(text) => Lease::parse(&text).map(Some),
        Err(_) => Ok(None), // no lease on main yet
    }
}

fn write_local(root: &Path, lease: &Lease) -> Result<(), String> {
    std::fs::write(root.join(LEASE_PATH), lease.render())
        .map_err(|e| format!("cannot write {LEASE_PATH}: {e}"))
}

fn hours_arg(args: &[String]) -> Result<u64, String> {
    match args.iter().position(|a| a == "--hours") {
        Some(i) => parse_u64(args.get(i + 1).ok_or("--hours needs a value")?),
        None => Ok(DEFAULT_HOURS),
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let invoked_as = args
        .first()
        .map(|a| Path::new(a).file_name().unwrap_or_default().to_string_lossy().into_owned())
        .unwrap_or_default();
    if invoked_as == "pre-push" {
        return check();
    }
    let words: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    match words.as_slice() {
        ["lease", "status", ..] => status(),
        ["lease", "acquire", ..] => acquire(hours_arg(args)?),
        ["lease", "renew", ..] => renew(hours_arg(args)?),
        ["lease", "release", ..] => release(),
        ["lease", "check", ..] => check(),
        ["install-hook", ..] => install_hook(),
        _ => Err(USAGE.to_string()),
    }
}

const USAGE: &str = "usage: opctl lease status|acquire [--hours N]|renew [--hours N]|release|check\n       opctl install-hook";

fn status() -> Result<(), String> {
    let root = repo_root()?;
    let now = now_unix();
    let show = |label: &str, lease: Option<Lease>| match lease {
        Some(l) => println!(
            "{label}: {:?} by {} since {} until {} ({})",
            l.state,
            l.holder,
            rfc3339(l.since_unix),
            rfc3339(l.expires_unix),
            if l.is_live(now) { "live" } else { "not live" }
        ),
        None => println!("{label}: none"),
    };
    show("origin/main", read_of_record()?);
    show("working copy", read_local(&root)?);
    Ok(())
}

fn acquire(hours: u64) -> Result<(), String> {
    let root = repo_root()?;
    let me = me()?;
    let now = now_unix();
    if let Some(l) = read_of_record()? {
        if l.is_live(now) && l.holder != me {
            return Err(format!(
                "session {} holds the lease until {}; do not take over",
                l.holder,
                rfc3339(l.expires_unix)
            ));
        }
    }
    let lease = Lease {
        state: State::Held,
        holder: me,
        since_unix: now,
        expires_unix: now + hours * 3_600,
    };
    write_local(&root, &lease)?;
    println!("claimed in {LEASE_PATH}; commit it and land it on main to make it the lease of record");
    Ok(())
}

fn renew(hours: u64) -> Result<(), String> {
    let root = repo_root()?;
    let me = me()?;
    let mut lease = read_local(&root)?.ok_or("no lease in the working copy; acquire first")?;
    if lease.holder != me || lease.state != State::Held {
        return Err("this session does not hold the lease; acquire first".to_string());
    }
    lease.expires_unix = now_unix() + hours * 3_600;
    write_local(&root, &lease)?;
    println!("renewed until {}", rfc3339(lease.expires_unix));
    Ok(())
}

fn release() -> Result<(), String> {
    let root = repo_root()?;
    let mut lease = read_local(&root)?.ok_or("no lease in the working copy")?;
    lease.state = State::Released;
    lease.expires_unix = now_unix();
    write_local(&root, &lease)?;
    println!("released; commit it and land it on main before starting the next session");
    Ok(())
}

fn check() -> Result<(), String> {
    let root = repo_root()?;
    let me = me()?;
    match push_verdict(read_of_record()?.as_ref(), read_local(&root)?.as_ref(), &me, now_unix()) {
        Verdict::AllowHolder | Verdict::AllowClaim => Ok(()),
        Verdict::Refuse(why) => Err(format!("push refused by the operator lease: {why}")),
    }
}

fn install_hook() -> Result<(), String> {
    let root = repo_root()?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate opctl: {e}"))?;
    let hooks = PathBuf::from(git(&["rev-parse", "--git-path", "hooks"])?.trim());
    let hooks = if hooks.is_absolute() { hooks } else { root.join(hooks) };
    std::fs::create_dir_all(&hooks).map_err(|e| format!("cannot create {}: {e}", hooks.display()))?;
    let hook = hooks.join("pre-push");
    let _ = std::fs::remove_file(&hook);
    std::os::unix::fs::symlink(&exe, &hook)
        .map_err(|e| format!("cannot link {}: {e}", hook.display()))?;
    println!("{} -> {}", hook.display(), exe.display());
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("opctl: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lease(state: State, holder: &str, expires_unix: u64) -> Lease {
        Lease { state, holder: holder.into(), since_unix: 0, expires_unix }
    }

    #[test]
    fn rfc3339_matches_known_instants() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339(1_791_623_897), "2026-10-10T09:18:17Z");
    }

    #[test]
    fn render_then_parse_round_trips() {
        let l = Lease { state: State::Held, holder: "cse_a".into(), since_unix: 10, expires_unix: 20 };
        assert_eq!(Lease::parse(&l.render()).unwrap(), l);
    }

    #[test]
    fn parse_rejects_unknown_state_and_missing_fields() {
        assert!(Lease::parse("state = \"maybe\"").is_err());
        assert!(Lease::parse("state = \"held\"\nholder = \"x\"").is_err());
    }

    #[test]
    fn holder_of_a_live_lease_may_push() {
        let rec = lease(State::Held, "me", 100);
        assert_eq!(push_verdict(Some(&rec), None, "me", 50), Verdict::AllowHolder);
    }

    #[test]
    fn others_are_refused_while_a_lease_is_live() {
        let rec = lease(State::Held, "other", 100);
        let mine = lease(State::Held, "me", 200);
        assert!(matches!(push_verdict(Some(&rec), Some(&mine), "me", 50), Verdict::Refuse(_)));
    }

    #[test]
    fn a_released_or_expired_lease_can_be_claimed_by_a_push_carrying_the_claim() {
        let mine = lease(State::Held, "me", 200);
        let released = lease(State::Released, "other", 100);
        let expired = lease(State::Held, "other", 40);
        assert_eq!(push_verdict(Some(&released), Some(&mine), "me", 50), Verdict::AllowClaim);
        assert_eq!(push_verdict(Some(&expired), Some(&mine), "me", 50), Verdict::AllowClaim);
        assert_eq!(push_verdict(None, Some(&mine), "me", 50), Verdict::AllowClaim);
    }

    #[test]
    fn a_push_without_a_claim_is_refused_when_nobody_holds_the_lease() {
        let released = lease(State::Released, "other", 100);
        assert!(matches!(push_verdict(Some(&released), None, "me", 50), Verdict::Refuse(_)));
        let stale_claim = lease(State::Held, "me", 40);
        assert!(matches!(push_verdict(None, Some(&stale_claim), "me", 50), Verdict::Refuse(_)));
    }
}
