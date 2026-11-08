//! Replay of accepted readings through the base and candidate reading rules.

use super::*;

// ---------------------------------------------------------------------------------------------------------
// Replay of accepted readings through the base and candidate reading rules

/// The most accepted readings one replay uses: the newest ones.
pub const REPLAY_WINDOW: usize = 50;

/// A candidate that newly refuses more than this share of the replayed readings is held, unless it declares a
/// share of its own ([`replay_declaration`]). Exactly this share is not held.
pub const REPLAY_HOLD_PERCENT: usize = 10;

/// Why a replay of an empty history did not run.
pub const NO_HISTORY_REASON: &str = "no accepted readings in the history";

/// What the reading rules say about one reading and its request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadingCheck {
    Accept,
    /// Refused, with the reason on one line.
    Refuse(String),
}

impl ReadingCheck {
    /// The one line `athena check-reading` prints: `accept`, or `refuse: <reason>`.
    pub fn line(&self) -> String {
        match self {
            ReadingCheck::Accept => "accept".to_string(),
            ReadingCheck::Refuse(reason) => format!("refuse: {reason}"),
        }
    }

    /// Parses the first line of what `athena check-reading` printed.
    pub fn parse(output: &str) -> Result<ReadingCheck> {
        let line = output.lines().next().unwrap_or("").trim();
        if line == "accept" {
            return Ok(ReadingCheck::Accept);
        }
        match line.strip_prefix("refuse:") {
            Some(reason) if !reason.trim().is_empty() => {
                Ok(ReadingCheck::Refuse(reason.trim().to_string()))
            }
            _ => bail!("check-reading printed {line:?}, not `accept` or `refuse: <reason>`"),
        }
    }
}

/// Runs the reading rules ([`check_reading_acceptance`]) on one stored reading and request. There are no
/// earlier attempts, so the retry rule (`check_prior_result_and_scope`) sees a first attempt. This is what
/// `athena check-reading` runs, and no model is called. There is no repository to compare with, so the
/// words are checked and the deliverable paths are not.
pub fn check_reading_text(reading: &Reading, request: &str) -> ReadingCheck {
    match check_reading_acceptance(reading, request, None, &[], None) {
        Ok(()) => ReadingCheck::Accept,
        Err(error) => ReadingCheck::Refuse(
            error
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        ),
    }
}

/// Something that applies one build's reading rules to a stored reading. It is given the request and the
/// reading and nothing else, so it cannot reach a model.
pub trait ReadingChecker {
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck>;
}

impl<F> ReadingChecker for F
where
    F: Fn(&str, &Reading) -> Result<ReadingCheck>,
{
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck> {
        self(request, reading)
    }
}

/// The base checker and the candidate checker of one replay.
pub type ReplayCheckers = (
    std::rc::Rc<dyn ReadingChecker>,
    std::rc::Rc<dyn ReadingChecker>,
);

/// A build of Athena, run as `<bin> check-reading --request <file> --reading <file>`. It holds a path and
/// nothing else: no driver, no agent handle.
#[derive(Debug, Clone)]
pub struct BinaryChecker {
    bin: PathBuf,
}

impl BinaryChecker {
    pub fn new(bin: PathBuf) -> BinaryChecker {
        BinaryChecker { bin }
    }
}

impl ReadingChecker for BinaryChecker {
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck> {
        let dir = tempfile::tempdir().context("making a scratch directory for check-reading")?;
        let request_file = dir.path().join("request.txt");
        let reading_file = dir.path().join("reading.json");
        std::fs::write(&request_file, request)?;
        std::fs::write(
            &reading_file,
            AnyRecord::Reading(Box::new(reading.clone())).to_json(),
        )?;
        let mut cmd = std::process::Command::new(&self.bin);
        cmd.arg("check-reading")
            .arg("--request")
            .arg(&request_file)
            .arg("--reading")
            .arg(&reading_file);
        let output = athena_agent::procs::retry_busy(|| cmd.output())
            .with_context(|| format!("running {} check-reading", self.bin.display()))?;
        // Exit 0 is accept and 2 is refuse; both print the one line. Anything else is a failure to check.
        if !matches!(output.status.code(), Some(0 | 2)) {
            bail!(
                "{} check-reading exited with {}: {}",
                self.bin.display(),
                output.status,
                tail(&String::from_utf8_lossy(&output.stderr), 300)
            );
        }
        ReadingCheck::parse(&String::from_utf8_lossy(&output.stdout))
    }
}

/// One stored request and its accepted reading.
#[derive(Debug, Clone)]
pub struct ReplayCase {
    pub job: JobId,
    pub request: String,
    pub reading: Reading,
}

/// The newest [`REPLAY_WINDOW`] jobs that have an accepted reading, newest first, with their request and
/// that reading. `exclude` (the running job) is left out, so a history of only that job is empty.
pub fn replay_cases(records: &Records, exclude: Option<&JobId>) -> Vec<ReplayCase> {
    records
        .request_jobs()
        .into_iter()
        .filter(|j| Some(j.id()) != exclude)
        .filter_map(|j| {
            let n = j.job.as_ref()?.accepted_reading?;
            Some(ReplayCase {
                job: j.id().clone(),
                request: j.request.as_ref()?.text.clone(),
                reading: j.reading(n)?.clone(),
            })
        })
        .take(REPLAY_WINDOW)
        .collect()
}

/// How many percentage points above its declared share a candidate may newly refuse before it is held.
/// Exactly declared plus this many points is not held.
pub const REPLAY_DECLARED_MARGIN_POINTS: usize = 5;

/// A declared share above this percentage counts only with a stated reason (see [`replay_declaration`]).
/// Exactly this share needs no reason.
pub const REPLAY_REASON_ABOVE_PERCENT: u32 = 25;

/// The most characters of a stated reason that the replay keeps for the verifier.
pub(super) const REPLAY_REASON_CHARS: usize = 300;

/// Words, lowercase, one of which a criterion or out_of_scope item must contain to count as a reason for a
/// declared share above [`REPLAY_REASON_ABOVE_PERCENT`] percent. Whole phrases, matched by `has_phrase`.
pub(super) const REPLAY_REASON_MARKERS: [&str; 3] = ["because", "now asks", "new required"];

/// Words, lowercase, one of which the same item must also contain, so the sentence is about past readings or
/// refusals and not about something else. Whole phrases, matched by `has_phrase`.
pub(super) const REPLAY_REASON_ANCHORS: [&str; 9] = [
    "past readings",
    "previous readings",
    "earlier readings",
    "older readings",
    "refuse",
    "refuses",
    "refused",
    "refusal",
    "refusals",
];

/// What a reading declares about the replay: the share of newly refused readings it expects, and the reason
/// it gives for a share above [`REPLAY_REASON_ABOVE_PERCENT`] percent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayDeclaration {
    /// The declared whole percent, 0 to 100, when a criterion states one.
    pub share: Option<u32>,
    /// The first criterion or out_of_scope item that gives a reason, cut to a fixed length.
    pub reason: Option<String>,
}

impl ReplayDeclaration {
    /// The share the replay holds the candidate to: the declared share when it is at most
    /// [`REPLAY_REASON_ABOVE_PERCENT`] percent or has a reason; otherwise none, so the
    /// [`REPLAY_HOLD_PERCENT`] limit applies.
    pub fn effective_share(&self) -> Option<u32> {
        self.share
            .filter(|s| *s <= REPLAY_REASON_ABOVE_PERCENT || self.reason.is_some())
    }

    /// The declared share when it is above [`REPLAY_REASON_ABOVE_PERCENT`] percent and has no reason, so it
    /// was not applied.
    pub fn unsupported_share(&self) -> Option<u32> {
        self.share
            .filter(|s| *s > REPLAY_REASON_ABOVE_PERCENT && self.reason.is_none())
    }
}

/// Lowercase text with runs of white space made single spaces.
pub(super) fn normalised(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The percent that `text` (normalised) declares for newly refused readings: a whole number from 0 to 100
/// and `%`, then optionally `of` and `the`, then `newly refused` ("expected 30% of newly refused readings").
/// A decimal, a number inside a word, or a percentage that is not tied to newly refused readings declares
/// nothing, so a quoted example does not count. The first match wins.
pub(super) fn declared_percent(text: &str) -> Option<u32> {
    for (at, _) in text.match_indices('%') {
        let digits_start = at
            - text[..at]
                .bytes()
                .rev()
                .take_while(u8::is_ascii_digit)
                .count();
        let digits = &text[digits_start..at];
        if digits.is_empty() || digits.len() > 3 {
            continue;
        }
        let before = text[..digits_start].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '.') {
            continue;
        }
        let mut rest = text[at + 1..].trim_start();
        for word in ["of ", "the "] {
            rest = rest.strip_prefix(word).unwrap_or(rest);
        }
        let tied = rest
            .strip_prefix("newly refused")
            .is_some_and(|after| !after.chars().next().is_some_and(char::is_alphanumeric));
        match digits.parse::<u32>() {
            Ok(percent) if tied && percent <= 100 => return Some(percent),
            _ => {}
        }
    }
    None
}

/// What the criteria and out_of_scope items declare about the replay. A share is declared only as
/// "N% of newly refused readings" ([`declared_percent`]), the first in criteria order. The reason is the
/// first criterion, then out_of_scope item, that has one of [`REPLAY_REASON_MARKERS`] and one of
/// [`REPLAY_REASON_ANCHORS`]. This is a text floor, not proof of a good reason: the verifier is shown the
/// sentence beside the replay list and judges it.
pub fn replay_declaration(criteria: &[Criterion], out_of_scope: &[String]) -> ReplayDeclaration {
    let share = criteria
        .iter()
        .find_map(|c| declared_percent(&normalised(&c.text)));
    let reason = criteria
        .iter()
        .map(|c| c.text.as_str())
        .chain(out_of_scope.iter().map(String::as_str))
        .find(|item| {
            let text = normalised(item);
            REPLAY_REASON_MARKERS.iter().any(|m| has_phrase(&text, m))
                && REPLAY_REASON_ANCHORS.iter().any(|a| has_phrase(&text, a))
        })
        .map(|item| {
            item.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(REPLAY_REASON_CHARS)
                .collect()
        });
    ReplayDeclaration { share, reason }
}

/// What the reading declares about the replay, from its acceptance criteria and out_of_scope. The gate and the
/// verifier's view both read it from here.
pub fn replay_declaration_of(reading: &Reading) -> ReplayDeclaration {
    replay_declaration(&reading.acceptance_criteria, &reading.out_of_scope)
}

/// Whether `newly` refusals among `total` replayed readings hold the job. With a declared share, more than
/// that share plus [`REPLAY_DECLARED_MARGIN_POINTS`] points holds it; with none, more than
/// [`REPLAY_HOLD_PERCENT`] percent does. Integer math, so exactly the limit is not held.
pub fn replay_is_held(total: usize, newly: usize, declared: Option<u32>) -> bool {
    newly * 100 > total * replay_limit_percent(declared)
}

/// The share of newly refused readings, in whole percent, that a candidate may reach without being held:
/// the declared share plus [`REPLAY_DECLARED_MARGIN_POINTS`] points, or [`REPLAY_HOLD_PERCENT`] when none is
/// declared. The hold at verification and the replay inside the producer's loop both read it from here, so the
/// producer is told the very limit the hold applies.
pub fn replay_limit_percent(declared: Option<u32>) -> usize {
    declared.map_or(REPLAY_HOLD_PERCENT, |d| {
        d as usize + REPLAY_DECLARED_MARGIN_POINTS
    })
}

/// Whether a change list can alter the compiled reading rules: any path under `crates/`, or the workspace's
/// `Cargo.toml` or `Cargo.lock`. A change of documents only cannot, so the replay inside the producer's loop
/// skips it. The gate is deliberately wide: which crates `check-reading` links is not a list worth keeping in
/// step with the code, and a missed helper change is the failure this replay exists to catch.
pub fn touches_reading_rules(paths: &[String]) -> bool {
    paths
        .iter()
        .any(|p| p.starts_with("crates/") || p == "Cargo.toml" || p == "Cargo.lock")
}

/// The result of one replay.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayRun {
    pub total: usize,
    /// Refused by the candidate and accepted by the base, newest request first.
    pub newly_refused: Vec<ReplayRefusal>,
    /// Whether a declared share applied to this replay and the job is not held for it.
    pub intended: bool,
    pub held: bool,
    /// Why no replay ran (`no replay: <reason>`). Never a pass.
    pub no_replay: Option<String>,
}

impl ReplayRun {
    /// A replay that did not run, for `reason`.
    pub fn none(reason: &str) -> ReplayRun {
        ReplayRun {
            total: 0,
            newly_refused: vec![],
            intended: false,
            held: false,
            no_replay: Some(format!("no replay: {reason}")),
        }
    }

    /// The record of this replay.
    pub fn record(&self, job: &JobId, base_commit: &str, candidate_commit: &str) -> Replay {
        Replay {
            job: job.clone(),
            base_commit: base_commit.to_string(),
            candidate_commit: candidate_commit.to_string(),
            window: REPLAY_WINDOW as u32,
            total: self.total as u32,
            newly_refused: self.newly_refused.clone(),
            held: self.held,
            intended: self.intended,
            no_replay: self.no_replay.clone(),
        }
    }
}

/// Replays `cases` through the base and the candidate rules and lists the requests the candidate refuses and
/// the base accepted. `declared` is the share of newly refused readings the running job declares
/// ([`ReplayDeclaration::effective_share`]); see [`replay_is_held`]. No cases, or a checker that fails, gives
/// a replay that did not run, never a pass.
pub fn replay(
    cases: &[ReplayCase],
    base: &dyn ReadingChecker,
    candidate: &dyn ReadingChecker,
    declared: Option<u32>,
) -> ReplayRun {
    if cases.is_empty() {
        return ReplayRun::none(NO_HISTORY_REASON);
    }
    let mut newly_refused = Vec::new();
    for case in cases {
        let before = match base.check(&case.request, &case.reading) {
            Ok(check) => check,
            Err(error) => {
                return ReplayRun::none(&format!(
                    "the base build failed on {}: {error:#}",
                    case.job
                ));
            }
        };
        let after = match candidate.check(&case.request, &case.reading) {
            Ok(check) => check,
            Err(error) => {
                return ReplayRun::none(&format!(
                    "the candidate build failed on {}: {error:#}",
                    case.job
                ));
            }
        };
        if let (ReadingCheck::Accept, ReadingCheck::Refuse(reason)) = (before, after) {
            newly_refused.push(ReplayRefusal {
                job: case.job.clone(),
                reason,
            });
        }
    }
    // `Replay::validate` rejects a record that is held and intended, so a declared share that is exceeded is held only.
    let held = replay_is_held(cases.len(), newly_refused.len(), declared);
    ReplayRun {
        total: cases.len(),
        held,
        newly_refused,
        intended: declared.is_some() && !held,
        no_replay: None,
    }
}
