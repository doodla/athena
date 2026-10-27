//! The transport under the host client: the `gh` program behind [`Runner`], the retry of transient failures,
//! the `curl` program behind [`HttpPost`], and the in-memory log of refused calls.

use crate::Ui;
use anyhow::{Context, Result, bail};
use athena_data::records::{HostError, HostOperation, MessageClass, WorkPhase};
use athena_data::{JobId, RecordId};
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Runs the host client with the given arguments and returns its standard output.
pub trait Runner {
    fn gh(&self, args: &[String]) -> Result<String>;
}

/// [`Runner`] over the installed `gh` program. It never reads or stores a credential: `gh` holds it.
#[derive(Debug, Default)]
pub struct GhCli;

impl Runner for GhCli {
    fn gh(&self, args: &[String]) -> Result<String> {
        let out = Command::new("gh")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .context("running gh; is the GitHub client installed?")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(GhFailure {
                call: args.get(3).cloned().unwrap_or_else(|| "api".to_string()),
                status_code: parse_status(&stderr),
                text: stderr.trim().to_string(),
            }
            .into());
        }
        String::from_utf8(out.stdout).context("gh printed non-text output")
    }
}

/// A call that `gh` ran and the host refused. It is the error a [`Runner`] returns for that case, so the client
/// can read the status without matching text again. `text` is for the person reading the error; it is never
/// stored in a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhFailure {
    /// What was called (the REST path).
    pub call: String,
    /// The HTTP status, when `gh` reported one.
    pub status_code: Option<u16>,
    /// The client's own words, as printed.
    pub text: String,
}

impl std::fmt::Display for GhFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gh {} failed: {}", self.call, self.text)
    }
}

impl std::error::Error for GhFailure {}

/// The HTTP status in the text `gh` prints for a refused call, which has the form `... (HTTP 405)`. The first
/// `HTTP ` followed by three digits that make a status from 100 to 599 counts; no such text gives `None`.
pub fn parse_status(stderr: &str) -> Option<u16> {
    stderr.match_indices("HTTP ").find_map(|(at, marker)| {
        let digits = &stderr[at + marker.len()..];
        let code = digits.get(..3)?;
        if !code.bytes().all(|b| b.is_ascii_digit())
            || digits[3..].starts_with(|c: char| c.is_ascii_digit())
        {
            return None;
        }
        code.parse().ok().filter(|n| (100..=599).contains(n))
    })
}

/// Whether a status is a failure worth asking again: a server that did not answer well, not one that refused
/// the request.
pub fn is_transient(status: Option<u16>) -> bool {
    matches!(status, Some(500 | 502 | 503 | 504))
}

impl GhFailure {
    /// Why this failure is worth asking again, or `None` when it is not. An HTTP status decides on its own
    /// (5xx of [`is_transient`] yes; 4xx and auth errors no). Without a status, `gh`'s text is read for a timeout
    /// or a connection reset; "not logged in" and the like never match.
    pub fn transient_reason(&self) -> Option<String> {
        if let Some(status) = self.status_code {
            return is_transient(Some(status)).then(|| format!("HTTP {status}"));
        }
        transient_text(&self.text)
    }
}

/// Why a failure with no HTTP status is worth asking again, read from the client's own words: a timeout or a
/// connection reset. Anything else, such as "not logged in", is not.
pub fn transient_text(text: &str) -> Option<String> {
    let text = text.to_ascii_lowercase();
    if text.contains("timeout") || text.contains("timed out") {
        Some("timeout".to_string())
    } else if text.contains("connection reset") || text.contains("econnreset") {
        Some("connection reset".to_string())
    } else {
        None
    }
}

/// How a call is repeated after a transient failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    /// How many times the call is made in all, at least once.
    pub attempts: u32,
    /// The wait before the second call; the wait before the third is twice that, and so on. Jitter is added
    /// on top (see [`Retry::wait`]).
    pub delay: Duration,
}

impl Retry {
    /// The wait after the failed attempt number `attempt` (counting from 1): `delay` doubled for each earlier
    /// failure, plus up to half of that again. `entropy` picks the jitter (any number; it is taken modulo 1000),
    /// so a test can fix it.
    pub fn wait(&self, attempt: u32, entropy: u32) -> Duration {
        let base = self.delay * 2u32.saturating_pow(attempt.saturating_sub(1));
        base + base * (entropy % 1000) / 2000
    }
}

/// A jitter source without a new dependency: the sub-second part of the clock.
pub(super) fn clock_entropy() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos())
}

impl Retry {
    /// Three calls in all, with a short wait between them.
    pub const STANDARD: Retry = Retry {
        attempts: 3,
        delay: Duration::from_secs(2),
    };
}

/// Runs `call` and repeats it while it fails transiently, up to the bound of `retry`. `transient` says whether
/// an error is worth asking again, and why; any other error, and the last transient one, is returned. Each
/// repeat is reported to `ui` as one line naming `what`, the reason and the attempt. The wait between calls is
/// [`Retry::wait`].
pub fn retry_transient<T, E>(
    retry: Retry,
    ui: Option<&dyn Ui>,
    what: &str,
    mut call: impl FnMut() -> Result<T, E>,
    transient: impl Fn(&E) -> Option<String>,
) -> Result<T, E> {
    let attempts = retry.attempts.max(1);
    let mut attempt = 1;
    loop {
        match call() {
            Ok(value) => return Ok(value),
            Err(error) => {
                let Some(reason) = transient(&error) else {
                    return Err(error);
                };
                if attempt >= attempts {
                    return Err(error);
                }
                let wait = retry.wait(attempt, clock_entropy());
                if let Some(ui) = ui {
                    ui.line(&format!(
                        "retrying {what} after {reason} (attempt {attempt} of {attempts} failed; waiting {} ms)",
                        wait.as_millis()
                    ));
                }
                std::thread::sleep(wait);
                attempt += 1;
            }
        }
    }
}

/// What an HTTP server answered: the status and the body, whether or not it is a success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpReply {
    pub status: u16,
    pub body: String,
}

/// Posts a JSON body and returns the reply, whatever its status. An `Err` means no reply was received (the
/// connection failed, timed out, or the client program could not run). The bearer token, when there is one, is
/// the caller's secret: an implementation never writes it to a file, a log or an error.
///
/// `Send + Sync` so that a client holding one (the Jev client inside [`crate::Athena`]) can be shared by the
/// workers of a parallel drain.
pub trait HttpPost: Send + Sync {
    fn post_json(&self, url: &str, bearer: Option<&str>, body: &str) -> Result<HttpReply>;
}

/// [`HttpPost`] over the installed `curl` program, the same way [`GhCli`] runs `gh`: no HTTP crate is added to
/// the workspace. The request settings, including the token header, go to the program on its standard input,
/// so the token is never in its argument list; the body goes through a temporary file that is removed after.
/// A proxy named by the usual environment variables is used by the program itself.
#[derive(Debug, Clone, Copy)]
pub struct CurlHttp {
    /// The most seconds one call may take.
    pub max_seconds: u64,
}

impl Default for CurlHttp {
    fn default() -> Self {
        CurlHttp { max_seconds: 60 }
    }
}

impl HttpPost for CurlHttp {
    fn post_json(&self, url: &str, bearer: Option<&str>, body: &str) -> Result<HttpReply> {
        use std::io::Write;
        let unsafe_text = |s: &str| s.contains(['"', '\\', '\n', '\r']);
        if unsafe_text(url) || bearer.is_some_and(unsafe_text) {
            bail!("the request has a character that cannot be sent safely");
        }
        let mut body_file = tempfile::NamedTempFile::new().context("preparing the request body")?;
        body_file
            .write_all(body.as_bytes())
            .context("preparing the request body")?;
        let mut config = format!(
            "silent\nshow-error\nrequest = \"POST\"\nheader = \"Content-Type: application/json\"\n\
             max-time = {}\nwrite-out = \"\\n%{{http_code}}\"\nurl = \"{url}\"\ndata-binary = \"@{}\"\n",
            self.max_seconds,
            body_file.path().display()
        );
        if let Some(token) = bearer {
            config.push_str(&format!("header = \"Authorization: Bearer {token}\"\n"));
        }
        let mut child = Command::new("curl")
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("running curl; is it installed?")?;
        child
            .stdin
            .take()
            .context("curl has no standard input")?
            .write_all(config.as_bytes())
            .context("sending the request settings to curl")?;
        let out = child.wait_with_output().context("waiting for curl")?;
        let scrub = |text: &str| match bearer {
            Some(token) if !token.is_empty() => text.replace(token, "<key>"),
            _ => text.to_string(),
        };
        if !out.status.success() {
            bail!(
                "the request failed: {}",
                scrub(String::from_utf8_lossy(&out.stderr).trim())
            );
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let (body, code) = text
            .rsplit_once('\n')
            .context("the reply has no status line")?;
        let status = code
            .trim()
            .parse()
            .context("the reply has no HTTP status")?;
        Ok(HttpReply {
            status,
            body: body.to_string(),
        })
    }
}

impl CurlHttp {
    /// The text of the page at an `http` or `https` URL, following redirects. An `Err` means no page: the
    /// URL is not http, the program could not run, the call timed out, or the server answered with an error
    /// status. No credential is sent.
    pub fn get_text(&self, url: &str) -> Result<String> {
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            bail!("{url:?} is not an http or https URL");
        }
        let out = Command::new("curl")
            .args(["--fail", "--silent", "--show-error", "--location"])
            .args(["--max-time", &self.max_seconds.to_string(), "--"])
            .arg(url)
            .output()
            .context("running curl; is it installed?")?;
        if !out.status.success() {
            bail!(
                "could not fetch {url}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// The refusals of one landing run, kept in memory until the caller commits them as records. The client writes
/// one for every refused call, retried or not.
#[derive(Debug)]
pub struct HostErrorLog {
    job: JobId,
    phase: WorkPhase,
    entries: RefCell<Vec<HostError>>,
}

impl HostErrorLog {
    pub fn new(job: JobId, phase: WorkPhase) -> Self {
        HostErrorLog {
            job,
            phase,
            entries: RefCell::new(Vec::new()),
        }
    }

    /// Notes one refused call. The client does this for every refusal; it is public so a test of the landing
    /// trigger can stand in for the client.
    pub fn record(&self, operation: HostOperation, status_code: Option<u16>) {
        let mut entries = self.entries.borrow_mut();
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while entries.iter().any(|e| e.id == id) {
            id = RecordId::generate(now);
        }
        entries.push(HostError {
            id,
            at: now,
            status_code,
            operation,
            job: self.job.clone(),
            phase: self.phase,
            message_class: MessageClass::of_status(status_code),
            derived: false,
            source_intervention: None,
        });
    }

    /// The refusals so far, oldest first.
    pub fn entries(&self) -> Vec<HostError> {
        self.entries.borrow().clone()
    }
}
