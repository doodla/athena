//! The validation rules of design §5.4, as pure functions over a [`Study`].
//!
//! Eight rules are *evidence* rules: nothing waives them. Ten are *model* rules: a `waiver` finding clears
//! one only when it names the rule and every offending id, carries a non-empty reason, and cites a user
//! intervention record that exists (see [`validate`]). Git lookups go through [`GitLookup`], so the rules
//! never touch a network; [`FixtureRepo`] is the in-memory repository the tests use.

use crate::model::Model;
use crate::records::Pin;
use crate::records::{
    Claim, Finding, FindingKind, Provenance, Rationale, RuleResult, Run, Source, SourceKind,
    VALIDATION_SCHEMA, Validation,
};
use athena_data::{Provenance as QuantityProvenance, Quantity};
use std::collections::{BTreeMap, BTreeSet};

/// Whether a failed rule can be waived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Non-waivable.
    Evidence,
    /// Waivable by a user-written waiver.
    Model,
}

/// The git questions the rules ask. Real and fixture repositories both answer them.
pub trait GitLookup: std::fmt::Debug {
    /// The lines of `path` at commit `sha`, or `None` when the commit or the path does not exist.
    fn file_lines(&self, sha: &str, path: &str) -> Option<Vec<String>>;
    /// True when `commit` exists and is reachable from `from`.
    fn reachable(&self, commit: &str, from: &str) -> bool;
}

/// A small in-memory repository for tests and fixtures.
#[derive(Debug, Default, Clone)]
pub struct FixtureRepo {
    files: BTreeMap<(String, String), Vec<String>>,
    reachable: BTreeMap<String, BTreeSet<String>>,
}

impl FixtureRepo {
    pub fn new() -> FixtureRepo {
        FixtureRepo::default()
    }

    /// Adds a file at a commit.
    pub fn add_file(&mut self, sha: &str, path: &str, text: &str) {
        self.files.insert(
            (sha.to_string(), path.to_string()),
            text.lines().map(String::from).collect(),
        );
    }

    /// Records that `commit` is reachable from `from`.
    pub fn add_reachable(&mut self, from: &str, commit: &str) {
        self.reachable
            .entry(from.to_string())
            .or_default()
            .insert(commit.to_string());
    }
}

impl GitLookup for FixtureRepo {
    fn file_lines(&self, sha: &str, path: &str) -> Option<Vec<String>> {
        self.files
            .get(&(sha.to_string(), path.to_string()))
            .cloned()
    }

    fn reachable(&self, commit: &str, from: &str) -> bool {
        let known = self.reachable.get(from);
        (commit == from && known.is_some()) || known.is_some_and(|s| s.contains(commit))
    }
}

/// Everything the rules look at.
#[derive(Debug, Clone, Copy)]
pub struct Study<'a> {
    pub pin: &'a Pin,
    pub claims: &'a [Claim],
    pub runs: &'a [Run],
    pub rationales: &'a [Rationale],
    pub model: &'a Model,
    pub findings: &'a [Finding],
    /// Ids of user intervention records. A waiver must cite one of them.
    pub user_interventions: &'a BTreeSet<String>,
    /// Provenances for which the renderer's template can reach a card's `authors-say` field.
    pub template_voice: &'a [Provenance],
    pub git: &'a dyn GitLookup,
}

/// One rule failure: the rule, the offending record ids, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub ids: Vec<String>,
    pub detail: String,
}

fn violation(rule: &'static str, ids: &[&str], detail: impl Into<String>) -> Violation {
    Violation {
        rule,
        ids: ids.iter().map(|s| s.to_string()).collect(),
        detail: detail.into(),
    }
}

/// A rule: its id, its class, and its check.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub class: Class,
    pub check: fn(&Study) -> Vec<Violation>,
}

/// Every rule of design §5.4, in the order of its table.
pub const RULES: &[Rule] = &[
    Rule {
        id: "uncited-claim",
        class: Class::Evidence,
        check: uncited_claim,
    },
    Rule {
        id: "wrong-sha",
        class: Class::Evidence,
        check: wrong_sha,
    },
    Rule {
        id: "bad-citation",
        class: Class::Evidence,
        check: bad_citation,
    },
    Rule {
        id: "quote-mismatch",
        class: Class::Evidence,
        check: quote_mismatch,
    },
    Rule {
        id: "stated-without-source",
        class: Class::Evidence,
        check: stated_without_source,
    },
    Rule {
        id: "inferred-shown-as-stated",
        class: Class::Evidence,
        check: inferred_shown_as_stated,
    },
    Rule {
        id: "incomplete-card",
        class: Class::Evidence,
        check: incomplete_card,
    },
    Rule {
        id: "unsupported-number",
        class: Class::Evidence,
        check: unsupported_number,
    },
    Rule {
        id: "uncited-element",
        class: Class::Model,
        check: uncited_element,
    },
    Rule {
        id: "unreachable-state",
        class: Class::Model,
        check: unreachable_state,
    },
    Rule {
        id: "dead-end",
        class: Class::Model,
        check: dead_end,
    },
    Rule {
        id: "dangling-target",
        class: Class::Model,
        check: dangling_target,
    },
    Rule {
        id: "missing-guard",
        class: Class::Model,
        check: missing_guard,
    },
    Rule {
        id: "undeclared-counter",
        class: Class::Model,
        check: undeclared_counter,
    },
    Rule {
        id: "unbounded-retry",
        class: Class::Model,
        check: unbounded_retry,
    },
    Rule {
        id: "unexercised-path",
        class: Class::Model,
        check: unexercised_path,
    },
    Rule {
        id: "divergence-unrecorded",
        class: Class::Model,
        check: divergence_unrecorded,
    },
    Rule {
        id: "missing-limit-or-list",
        class: Class::Model,
        check: missing_limit_or_list,
    },
];

/// Every source of the study, with the id of the claim or card that holds it.
fn sources<'a>(s: &Study<'a>) -> Vec<(&'a str, &'a Source)> {
    let claims = s
        .claims
        .iter()
        .flat_map(|c| c.sources.iter().map(move |src| (c.id.as_str(), src)));
    let cards = s
        .rationales
        .iter()
        .flat_map(|r| r.sources.iter().map(move |src| (r.id.as_str(), src)));
    claims.chain(cards).collect()
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

// ---- evidence rules ----------------------------------------------------------------------------------------

/// A claim has no source; its mark does not match its source kinds; or a `both` claim lacks a file or run source.
pub fn uncited_claim(s: &Study) -> Vec<Violation> {
    use crate::records::Mark;
    let mut out = Vec::new();
    for c in s.claims {
        let runs = c
            .sources
            .iter()
            .filter(|x| x.kind == SourceKind::Run)
            .count();
        let files = c
            .sources
            .iter()
            .filter(|x| x.kind == SourceKind::File)
            .count();
        let why = if c.sources.is_empty() {
            Some("no source")
        } else {
            match c.mark {
                Mark::Read if runs > 0 => Some("marked read but cites a run"),
                Mark::Observed if runs == 0 => Some("marked observed but cites no run"),
                Mark::Both if runs == 0 || files == 0 => {
                    Some("marked both but lacks a file source or a run source")
                }
                _ => None,
            }
        };
        if let Some(why) = why {
            out.push(violation("uncited-claim", &[&c.id], why));
        }
    }
    out
}

/// A file citation names a SHA other than the pin, or a cited run's `sha` is not the pin.
pub fn wrong_sha(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for (owner, src) in sources(s) {
        match src.kind {
            SourceKind::File if src.sha.as_deref() != Some(s.pin.sha.as_str()) => {
                out.push(violation(
                    "wrong-sha",
                    &[owner],
                    "a file citation names a SHA other than the pin",
                ));
            }
            SourceKind::Run => {
                let run = s.runs.iter().find(|r| Some(&r.id) == src.run_id.as_ref());
                if run.is_some_and(|r| r.sha != s.pin.sha) {
                    out.push(violation(
                        "wrong-sha",
                        &[owner],
                        "a cited run is of a tree other than the pin",
                    ));
                }
            }
            _ => {}
        }
    }
    out
}

/// The cited lines of a file source at the pin, when the path and range exist.
fn cited_lines(s: &Study, src: &Source) -> Option<String> {
    let lines = s.git.file_lines(&s.pin.sha, src.path.as_deref()?)?;
    let (start, end) = (src.start_line? as usize, src.end_line? as usize);
    if start < 1 || end < start || end > lines.len() {
        return None;
    }
    Some(lines[start - 1..end].join("\n"))
}

/// A path or range is missing at the pin; a commit is missing or unreachable; a run id has no record; or a
/// run timestamp lies outside the run.
pub fn bad_citation(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for (owner, src) in sources(s) {
        let why = match src.kind {
            SourceKind::File => cited_lines(s, src)
                .is_none()
                .then_some("the path or line range does not exist at the pin"),
            SourceKind::Commit | SourceKind::PullRequest => {
                let reachable = src
                    .sha
                    .as_deref()
                    .is_some_and(|c| s.git.reachable(c, &s.pin.sha));
                if src.kind == SourceKind::PullRequest && src.id.is_none() {
                    Some("a pull-request citation has no identifier")
                } else {
                    (!reachable)
                        .then_some("the commit does not exist or is not reachable from the pin")
                }
            }
            SourceKind::Run => match s.runs.iter().find(|r| Some(&r.id) == src.run_id.as_ref()) {
                None => Some("the run id has no run record"),
                Some(r) => match src.timestamp {
                    Some(t) if r.start <= t && t <= r.end => None,
                    _ => Some("the timestamp lies outside the run"),
                },
            },
        };
        if let Some(why) = why {
            out.push(violation("bad-citation", &[owner], why));
        }
    }
    out
}

/// A file source's `quote` is not a substring of the cited lines at the pin. (A citation whose lines do not
/// exist is left to `bad-citation`.)
pub fn quote_mismatch(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for (owner, src) in sources(s) {
        if src.kind != SourceKind::File {
            continue;
        }
        let Some(text) = cited_lines(s, src) else {
            continue;
        };
        match src.quote.as_deref() {
            Some(q) if !blank(q) && text.contains(q) => {}
            Some(q) if !blank(q) => {
                out.push(violation(
                    "quote-mismatch",
                    &[owner],
                    "the quote is not in the cited lines",
                ));
            }
            _ => out.push(violation(
                "quote-mismatch",
                &[owner],
                "a file source has no quote",
            )),
        }
    }
    out
}

/// A `stated` card cites no source whose `quote` field is present (its own, or those of the claims it cites).
pub fn stated_without_source(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for card in s
        .rationales
        .iter()
        .filter(|r| r.provenance == Some(Provenance::Stated))
    {
        let cited = s
            .claims
            .iter()
            .filter(|c| card.claims.contains(&c.id))
            .flat_map(|c| c.sources.iter());
        let quoted = card
            .sources
            .iter()
            .chain(cited)
            .any(|x| x.quote.as_deref().is_some_and(|q| !blank(q)));
        if !quoted {
            out.push(violation(
                "stated-without-source",
                &[&card.id],
                "a stated card cites no quoted source",
            ));
        }
    }
    out
}

/// A card whose provenance is not `stated` has an `authors-say` field, or the renderer's template can reach
/// that field for such a card.
pub fn inferred_shown_as_stated(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for card in s
        .rationales
        .iter()
        .filter(|r| r.provenance != Some(Provenance::Stated))
    {
        let reached = card
            .provenance
            .is_some_and(|p| s.template_voice.contains(&p));
        if card.authors_say.is_some() || reached {
            out.push(violation(
                "inferred-shown-as-stated",
                &[&card.id],
                "the authors' voice is reachable for a card that is not stated",
            ));
        }
    }
    out
}

/// A rationale card has an empty decision, alternatives, tradeoff, reversal cost, or provenance.
pub fn incomplete_card(s: &Study) -> Vec<Violation> {
    s.rationales
        .iter()
        .filter(|r| {
            blank(&r.decision)
                || r.alternatives.iter().all(|a| blank(a))
                || blank(&r.tradeoff)
                || blank(&r.reversal_cost)
                || r.provenance.is_none()
        })
        .map(|r| violation("incomplete-card", &[&r.id], "a card field is empty"))
        .collect()
}

/// A `Quantity` field has neither a measurement nor an estimate label (that is, it is still pending).
pub fn unsupported_number(s: &Study) -> Vec<Violation> {
    let unsupported = |q: &Quantity| q.provenance() == QuantityProvenance::Pending;
    s.runs
        .iter()
        .filter(|r| unsupported(&r.cap) || r.cost.iter().any(unsupported))
        .map(|r| {
            violation(
                "unsupported-number",
                &[&r.id],
                "a run quantity is neither measured nor estimated",
            )
        })
        .collect()
}

// ---- model rules -------------------------------------------------------------------------------------------

/// A state, counter, transition, retry edge, unhandled-failure list, or concurrency rule lists no claim.
pub fn uncited_element(s: &Study) -> Vec<Violation> {
    let m = s.model;
    let mut out = Vec::new();
    let mut check = |id: &str, claims: &[String]| {
        if claims.is_empty() {
            out.push(violation(
                "uncited-element",
                &[id],
                "the element lists no supporting claim",
            ));
        }
    };
    for x in &m.states {
        check(&x.id, &x.claims);
    }
    for x in &m.counters {
        check(&x.id, &x.claims);
    }
    for x in &m.transitions {
        check(&x.id, &x.claims);
        if let Some(r) = &x.retry {
            check(&x.id, &r.claims);
        }
    }
    if let Some(u) = &m.unhandled {
        check("unhandled", &u.claims);
    }
    if let Some(c) = &m.concurrency {
        check("concurrency", &c.claims);
    }
    out.dedup();
    out
}

/// A state cannot be reached from any `initial` state.
pub fn unreachable_state(s: &Study) -> Vec<Violation> {
    use crate::model::StateKind;
    let m = s.model;
    let seen = m.reachable_from(
        m.states
            .iter()
            .filter(|x| x.kind == StateKind::Initial)
            .map(|x| x.id.as_str()),
    );
    m.states
        .iter()
        .filter(|x| !seen.contains(x.id.as_str()))
        .map(|x| violation("unreachable-state", &[&x.id], "no initial state reaches it"))
        .collect()
}

/// A non-terminal state has no outgoing transition.
pub fn dead_end(s: &Study) -> Vec<Violation> {
    let m = s.model;
    m.states
        .iter()
        .filter(|x| !x.kind.is_terminal())
        .filter(|x| !m.transitions.iter().any(|t| t.from == x.id))
        .map(|x| violation("dead-end", &[&x.id], "a non-terminal state has no way out"))
        .collect()
}

/// A transition names a source or target that is not a declared state.
pub fn dangling_target(s: &Study) -> Vec<Violation> {
    let declared: BTreeSet<&str> = s.model.states.iter().map(|x| x.id.as_str()).collect();
    s.model
        .transitions
        .iter()
        .filter(|t| !declared.contains(t.from.as_str()) || !declared.contains(t.to.as_str()))
        .map(|t| {
            violation(
                "dangling-target",
                &[&t.id],
                "a source or target is not a declared state",
            )
        })
        .collect()
}

/// A transition has no guard field (`none` is allowed; absence is not).
pub fn missing_guard(s: &Study) -> Vec<Violation> {
    s.model
        .transitions
        .iter()
        .filter(|t| t.guard.as_deref().is_none_or(blank))
        .map(|t| violation("missing-guard", &[&t.id], "the guard field is absent"))
        .collect()
}

/// A guard uses a counter the model does not declare.
pub fn undeclared_counter(s: &Study) -> Vec<Violation> {
    let declared: BTreeSet<&str> = s.model.counters.iter().map(|c| c.id.as_str()).collect();
    s.model
        .transitions
        .iter()
        .filter(|t| {
            t.guard_counters
                .iter()
                .any(|c| !declared.contains(c.as_str()))
        })
        .map(|t| {
            violation(
                "undeclared-counter",
                &[&t.id],
                "a guard uses an undeclared counter",
            )
        })
        .collect()
}

/// A retry edge has no bound and no claim that says the bound is absent, or lacks its list of what changes.
pub fn unbounded_retry(s: &Study) -> Vec<Violation> {
    s.model
        .retry_edges()
        .into_iter()
        .filter(|t| match &t.retry {
            None => true,
            Some(r) => {
                (r.bound.is_none() && r.bound_absent_claims.is_empty()) || r.changes.is_none()
            }
        })
        .map(|t| {
            violation(
                "unbounded-retry",
                &[&t.id],
                "a retry edge has no bound, or no list of what changes",
            )
        })
        .collect()
}

/// A failure or retry edge appears in no run's observed transitions and has neither a `could-not-run` nor a
/// `no-observation` finding that names it.
pub fn unexercised_path(s: &Study) -> Vec<Violation> {
    s.model
        .failure_edges()
        .filter(|t| {
            !s.runs
                .iter()
                .any(|r| r.observed_transitions.contains(&t.id))
        })
        .filter(|t| {
            !s.findings.iter().any(|f| {
                matches!(
                    f.kind,
                    FindingKind::CouldNotRun | FindingKind::NoObservation
                ) && f.subject.contains(&t.id)
            })
        })
        .map(|t| {
            violation(
                "unexercised-path",
                &[&t.id],
                "no run observed the edge and no finding names it",
            )
        })
        .collect()
}

/// A claim carries `contradicts` and no `divergence` finding links the two claims.
pub fn divergence_unrecorded(s: &Study) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in s.claims {
        for other in &c.contradicts {
            let linked = s.findings.iter().any(|f| {
                f.kind == FindingKind::Divergence
                    && f.subject.contains(&c.id)
                    && f.subject.contains(other)
            });
            if !linked {
                out.push(violation(
                    "divergence-unrecorded",
                    &[&c.id, other],
                    "a contradiction has no divergence finding",
                ));
            }
        }
    }
    out
}

/// The unhandled-failures list, a retry's change list, or a touchpoint's no-answer rule is absent.
pub fn missing_limit_or_list(s: &Study) -> Vec<Violation> {
    let m = s.model;
    let mut out = Vec::new();
    if m.unhandled.is_none() {
        out.push(violation(
            "missing-limit-or-list",
            &["unhandled"],
            "the unhandled-failures list is absent",
        ));
    }
    for t in m.retry_edges() {
        if t.retry.as_ref().is_some_and(|r| r.changes.is_none()) {
            out.push(violation(
                "missing-limit-or-list",
                &[&t.id],
                "the retry-change list is absent",
            ));
        }
    }
    for p in &m.touchpoints {
        if p.no_answer.as_deref().is_none_or(blank) {
            out.push(violation(
                "missing-limit-or-list",
                &[&p.id],
                "the no-answer rule is absent",
            ));
        }
    }
    out
}

// ---- running them all --------------------------------------------------------------------------------------

/// True when a valid waiver clears this model-rule violation: a `waiver` finding that names the rule, covers
/// every offending id, carries a non-empty reason, and cites an existing user intervention record.
fn waived(s: &Study, v: &Violation) -> bool {
    s.findings.iter().any(|f| {
        f.kind == FindingKind::Waiver
            && f.waiver.as_ref().is_some_and(|w| {
                w.rule == v.rule
                    && !blank(&w.reason)
                    && w.intervention
                        .as_ref()
                        .is_some_and(|i| s.user_interventions.contains(i))
            })
            && v.ids.iter().all(|id| f.subject.contains(id))
    })
}

/// Runs every rule and returns a `study.validation/1` record. Evidence violations always stand; a model
/// violation is cleared only by a valid waiver.
pub fn validate(s: &Study) -> Validation {
    let results = RULES
        .iter()
        .map(|rule| {
            let (mut standing, mut cleared) = (BTreeSet::new(), BTreeSet::new());
            for v in (rule.check)(s) {
                let target = if rule.class == Class::Model && waived(s, &v) {
                    &mut cleared
                } else {
                    &mut standing
                };
                target.extend(v.ids);
            }
            RuleResult {
                rule: rule.id.to_string(),
                pass: standing.is_empty(),
                offending: standing.into_iter().collect(),
                waived: cleared.into_iter().collect(),
            }
        })
        .collect();
    Validation {
        schema: VALIDATION_SCHEMA.into(),
        study_id: s.pin.study_id.clone(),
        results,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{
        Actor, Concurrency, Counter, EvaluatedBy, MODEL_SCHEMA, Retry, State, StateKind,
        Touchpoint, Transition, Unhandled,
    };
    use crate::records::{
        CLAIM_SCHEMA, FINDING_SCHEMA, Mark, Meter, PIN_SCHEMA, RATIONALE_SCHEMA, RUN_SCHEMA,
        Severity, Stream, StreamStatus, Waiver,
    };
    use athena_data::Unit;
    use jiff::Timestamp;

    pub const PIN: &str = "1111111111111111111111111111111111111111";
    const OLD: &str = "2222222222222222222222222222222222222222";
    const OTHER: &str = "3333333333333333333333333333333333333333";
    const PATH: &str = "src/queue.example";

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn state(id: &str, kind: StateKind) -> State {
        State {
            id: id.into(),
            kind,
            claims: strings(&["c1"]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn transition(
        id: &str,
        from: &str,
        to: &str,
        trigger: &str,
        guard: &str,
        counters: &[&str],
        failure: bool,
        retry: Option<Retry>,
    ) -> Transition {
        Transition {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            trigger: trigger.into(),
            trigger_evaluated_by: EvaluatedBy::Code,
            trigger_actor: None,
            guard: Some(guard.into()),
            guard_evaluated_by: EvaluatedBy::Code,
            guard_counters: strings(counters),
            actor: Actor::Process,
            failure: failure.then(|| "timeout".to_string()),
            retry,
            claims: strings(&["c1"]),
        }
    }

    /// The synthetic "Example Runner" model of design §5.3.
    pub fn example_model() -> Model {
        let retry = Retry {
            bound: Some(3),
            bound_absent_claims: vec![],
            backoff: None,
            changes: Some(strings(&["nothing"])),
            claims: strings(&["c2"]),
        };
        Model {
            schema: MODEL_SCHEMA.into(),
            study_id: "s1".into(),
            states: vec![
                state("queued", StateKind::Initial),
                state("running", StateKind::Active),
                state("failed", StateKind::TerminalFailure),
                state("done", StateKind::TerminalSuccess),
            ],
            counters: vec![Counter {
                id: "attempts".into(),
                start: 0,
                changed_by: strings(&["t1"]),
                claims: strings(&["c1"]),
            }],
            transitions: vec![
                transition(
                    "t1",
                    "queued",
                    "running",
                    "worker-free",
                    "slot < limit",
                    &[],
                    false,
                    None,
                ),
                transition("t2", "running", "done", "exit-0", "none", &[], false, None),
                transition(
                    "t3",
                    "running",
                    "queued",
                    "timeout",
                    "attempts < 3",
                    &["attempts"],
                    true,
                    Some(retry),
                ),
                transition(
                    "t4",
                    "running",
                    "failed",
                    "timeout",
                    "attempts == 3",
                    &["attempts"],
                    true,
                    None,
                ),
            ],
            unhandled: Some(Unhandled {
                failures: vec![],
                claims: strings(&["c1"]),
            }),
            concurrency: Some(Concurrency {
                shared_resources: strings(&["target file"]),
                serialized_by: strings(&["one lock per file"]),
                limit: Some(4),
                on_conflict: "wait in queued".into(),
                claims: strings(&["c1"]),
            }),
            touchpoints: vec![Touchpoint {
                id: "h1".into(),
                at: "running".into(),
                no_answer: Some("none".into()),
                claims: strings(&["c1"]),
            }],
        }
    }

    fn claim(id: &str, mark: Mark, sources: Vec<Source>) -> Claim {
        Claim {
            schema: CLAIM_SCHEMA.into(),
            id: id.into(),
            text: "synthetic claim".into(),
            mark,
            sources,
            contradicts: vec![],
            mechanism: "work-handling".into(),
        }
    }

    fn card(id: &str, provenance: Option<Provenance>) -> Rationale {
        Rationale {
            schema: RATIONALE_SCHEMA.into(),
            id: id.into(),
            decision: "retries are bounded".into(),
            alternatives: strings(&["retry forever"]),
            tradeoff: "some work fails for good".into(),
            reversal_cost: "low".into(),
            provenance,
            authors_say: None,
            claims: strings(&["c2"]),
            sources: vec![],
        }
    }

    fn finding(id: &str, kind: FindingKind, subject: &[&str]) -> Finding {
        Finding {
            schema: FINDING_SCHEMA.into(),
            id: id.into(),
            kind,
            severity: Severity::Minor,
            subject: strings(subject),
            detail: "synthetic".into(),
            waiver: None,
        }
    }

    fn waiver(id: &str, rule: &str, subject: &[&str], intervention: Option<&str>) -> Finding {
        Finding {
            waiver: Some(Waiver {
                rule: rule.into(),
                reason: "the user accepts this gap".into(),
                intervention: intervention.map(String::from),
            }),
            ..finding(id, FindingKind::Waiver, subject)
        }
    }

    /// Owned study data; `study()` borrows it.
    pub struct Fixture {
        pub pin: Pin,
        pub claims: Vec<Claim>,
        pub runs: Vec<Run>,
        pub rationales: Vec<Rationale>,
        pub model: Model,
        pub findings: Vec<Finding>,
        pub interventions: BTreeSet<String>,
        pub voice: Vec<Provenance>,
        pub repo: FixtureRepo,
    }

    impl Fixture {
        /// A study that passes every rule.
        pub fn clean() -> Fixture {
            let mut repo = FixtureRepo::new();
            repo.add_file(
                PIN,
                PATH,
                "fn pick() {\n    // a task moves to running when a worker is free\n    run();\n}\n",
            );
            repo.add_reachable(PIN, OLD);
            let at = ts("2026-01-01T00:05:00Z");
            let run = Run {
                schema: RUN_SCHEMA.into(),
                id: "r1".into(),
                sha: PIN.into(),
                scenario_id: "sc1".into(),
                configuration: "default".into(),
                command: "example-runner --scenario sc1".into(),
                start: ts("2026-01-01T00:00:00Z"),
                end: ts("2026-01-01T00:10:00Z"),
                boundary: Some("throwaway user".into()),
                cap: Quantity::estimated(
                    Some(3.0),
                    Unit::Usd,
                    "cap set before the run",
                    None,
                    None,
                )
                .unwrap(),
                kill_switch: "process-group kill".into(),
                meters: vec![Meter::WallClockProcess, Meter::ModelSpend],
                cost: vec![Quantity::measured(1.0, Unit::Usd, "provider usage")],
                exit_state: "exited".into(),
                blocker: None,
                observed_transitions: strings(&["t1", "t2", "t3", "t4"]),
                streams: vec![Stream {
                    name: "timing".into(),
                    status: StreamStatus::Native,
                }],
            };
            let file = Source::file(PATH, 2, 2, PIN, "a worker is free");
            let mut stated = card("ra2", Some(Provenance::Stated));
            stated.authors_say = Some("we bound retries".into());
            stated.sources = vec![file.clone()];
            stated.claims = vec![];
            Fixture {
                pin: Pin {
                    schema: PIN_SCHEMA.into(),
                    study_id: "s1".into(),
                    system: "Example Runner".into(),
                    repository_url: "https://example.invalid/runner".into(),
                    sha: PIN.into(),
                    tag: None,
                    resolved_at: ts("2025-12-31T00:00:00Z"),
                    how_resolved: "default branch head".into(),
                },
                claims: vec![
                    claim("c1", Mark::Read, vec![file.clone(), Source::commit(OLD)]),
                    claim("c2", Mark::Both, vec![file, Source::run("r1", at)]),
                    claim("c3", Mark::Observed, vec![Source::run("r1", at)]),
                ],
                runs: vec![run],
                rationales: vec![card("ra1", Some(Provenance::Inferred)), stated],
                model: example_model(),
                findings: vec![],
                interventions: ["iv1".to_string()].into(),
                voice: vec![Provenance::Stated],
                repo,
            }
        }

        pub fn study(&self) -> Study<'_> {
            Study {
                pin: &self.pin,
                claims: &self.claims,
                runs: &self.runs,
                rationales: &self.rationales,
                model: &self.model,
                findings: &self.findings,
                user_interventions: &self.interventions,
                template_voice: &self.voice,
                git: &self.repo,
            }
        }
    }

    /// The ids a rule flags in a fixture.
    fn flagged(rule: &str, f: &Fixture) -> Vec<String> {
        let rule = RULES.iter().find(|r| r.id == rule).expect("a known rule");
        (rule.check)(&f.study())
            .into_iter()
            .flat_map(|v| v.ids)
            .collect()
    }

    /// A clean study passes the rule; the planted fault fails it with exactly these ids.
    fn assert_rule(rule: &str, plant: impl FnOnce(&mut Fixture), expected: &[&str]) {
        assert!(
            flagged(rule, &Fixture::clean()).is_empty(),
            "{rule}: clean case"
        );
        let mut f = Fixture::clean();
        plant(&mut f);
        assert_eq!(flagged(rule, &f), expected, "{rule}: planted fault");
    }

    #[test]
    fn the_clean_study_passes_every_rule() {
        let f = Fixture::clean();
        let v = validate(&f.study());
        assert!(
            v.passed(),
            "{:?}",
            v.results.iter().filter(|r| !r.pass).collect::<Vec<_>>()
        );
        assert_eq!(v.results.len(), 18);
        assert_eq!(
            RULES.iter().filter(|r| r.class == Class::Evidence).count(),
            8
        );
    }

    #[test]
    fn uncited_claim_is_flagged() {
        assert_rule("uncited-claim", |f| f.claims[0].sources.clear(), &["c1"]);
        assert_rule("uncited-claim", |f| f.claims[2].sources.clear(), &["c3"]);
        // A mark that does not match the sources.
        assert_rule(
            "uncited-claim",
            |f| f.claims[0].mark = Mark::Observed,
            &["c1"],
        );
        assert_rule(
            "uncited-claim",
            |f| drop(f.claims[1].sources.remove(0)),
            &["c2"],
        );
    }

    #[test]
    fn wrong_sha_is_flagged() {
        assert_rule(
            "wrong-sha",
            |f| f.claims[0].sources[0].sha = Some(OTHER.into()),
            &["c1"],
        );
        assert_rule("wrong-sha", |f| f.runs[0].sha = OTHER.into(), &["c2", "c3"]);
    }

    #[test]
    fn bad_citation_is_flagged() {
        assert_rule(
            "bad-citation",
            |f| f.claims[0].sources[0].end_line = Some(99),
            &["c1"],
        );
        assert_rule(
            "bad-citation",
            |f| f.claims[0].sources[0].path = Some("nope".into()),
            &["c1"],
        );
        assert_rule(
            "bad-citation",
            |f| f.claims[0].sources[1] = Source::commit(OTHER),
            &["c1"],
        );
        assert_rule(
            "bad-citation",
            |f| f.claims[2].sources[0].run_id = Some("r9".into()),
            &["c3"],
        );
        assert_rule(
            "bad-citation",
            |f| f.claims[2].sources[0].timestamp = Some(ts("2026-02-01T00:00:00Z")),
            &["c3"],
        );
    }

    #[test]
    fn quote_mismatch_is_flagged() {
        assert_rule(
            "quote-mismatch",
            |f| f.claims[0].sources[0].quote = Some("a worker is busy".into()),
            &["c1"],
        );
        assert_rule(
            "quote-mismatch",
            |f| f.claims[0].sources[0].quote = None,
            &["c1"],
        );
    }

    #[test]
    fn stated_without_source_is_flagged() {
        assert_rule(
            "stated-without-source",
            |f| f.rationales[1].sources[0].quote = None,
            &["ra2"],
        );
    }

    #[test]
    fn inferred_shown_as_stated_is_flagged() {
        assert_rule(
            "inferred-shown-as-stated",
            |f| f.rationales[0].authors_say = Some("the authors said so".into()),
            &["ra1"],
        );
        // The template can reach the field for an inferred card.
        assert_rule(
            "inferred-shown-as-stated",
            |f| f.voice.push(Provenance::Inferred),
            &["ra1"],
        );
    }

    #[test]
    fn incomplete_card_is_flagged() {
        assert_rule(
            "incomplete-card",
            |f| f.rationales[0].tradeoff = " ".into(),
            &["ra1"],
        );
        assert_rule(
            "incomplete-card",
            |f| f.rationales[0].alternatives.clear(),
            &["ra1"],
        );
        assert_rule(
            "incomplete-card",
            |f| f.rationales[0].provenance = None,
            &["ra1"],
        );
    }

    #[test]
    fn unsupported_number_is_flagged() {
        assert_rule(
            "unsupported-number",
            |f| {
                f.runs[0]
                    .cost
                    .push(Quantity::pending(Unit::Usd, "not known yet"))
            },
            &["r1"],
        );
    }

    #[test]
    fn uncited_element_is_flagged() {
        assert_rule(
            "uncited-element",
            |f| f.model.states[1].claims.clear(),
            &["running"],
        );
        assert_rule(
            "uncited-element",
            |f| {
                f.model.transitions[2]
                    .retry
                    .as_mut()
                    .unwrap()
                    .claims
                    .clear()
            },
            &["t3"],
        );
        assert_rule(
            "uncited-element",
            |f| f.model.unhandled.as_mut().unwrap().claims.clear(),
            &["unhandled"],
        );
    }

    #[test]
    fn unreachable_state_is_flagged() {
        assert_rule(
            "unreachable-state",
            |f| {
                f.model
                    .states
                    .push(state("orphan", StateKind::TerminalSuccess))
            },
            &["orphan"],
        );
    }

    #[test]
    fn dead_end_is_flagged() {
        assert_rule(
            "dead-end",
            |f| {
                f.model
                    .transitions
                    .retain(|t| t.id != "t2" && t.id != "t3" && t.id != "t4");
            },
            &["running"],
        );
    }

    #[test]
    fn dangling_target_is_flagged() {
        assert_rule(
            "dangling-target",
            |f| f.model.transitions[1].to = "nowhere".into(),
            &["t2"],
        );
    }

    #[test]
    fn missing_guard_is_flagged() {
        assert_rule(
            "missing-guard",
            |f| f.model.transitions[1].guard = None,
            &["t2"],
        );
    }

    #[test]
    fn undeclared_counter_is_flagged() {
        assert_rule(
            "undeclared-counter",
            |f| f.model.transitions[0].guard_counters = strings(&["slots"]),
            &["t1"],
        );
    }

    #[test]
    fn unbounded_retry_is_flagged() {
        assert_rule(
            "unbounded-retry",
            |f| f.model.transitions[2].retry.as_mut().unwrap().bound = None,
            &["t3"],
        );
        assert_rule(
            "unbounded-retry",
            |f| f.model.transitions[2].retry = None,
            &["t3"],
        );
        // An absent bound is allowed when a claim says so.
        let mut f = Fixture::clean();
        let retry = f.model.transitions[2].retry.as_mut().unwrap();
        retry.bound = None;
        retry.bound_absent_claims = strings(&["c1"]);
        assert!(flagged("unbounded-retry", &f).is_empty());
    }

    #[test]
    fn unexercised_path_is_flagged() {
        assert_rule(
            "unexercised-path",
            |f| f.runs[0].observed_transitions = strings(&["t1", "t2", "t4"]),
            &["t3"],
        );
        // A could-not-run finding that names the edge clears it.
        let mut f = Fixture::clean();
        f.runs[0].observed_transitions.retain(|t| t != "t3");
        f.findings
            .push(finding("f1", FindingKind::CouldNotRun, &["t3"]));
        assert!(flagged("unexercised-path", &f).is_empty());
        // A finding of another kind does not.
        f.findings[0].kind = FindingKind::ModelLimit;
        assert_eq!(flagged("unexercised-path", &f), ["t3"]);
    }

    #[test]
    fn divergence_unrecorded_is_flagged() {
        let plant = |f: &mut Fixture| f.claims[2].contradicts = strings(&["c1"]);
        assert_rule("divergence-unrecorded", plant, &["c3", "c1"]);
        let mut f = Fixture::clean();
        plant(&mut f);
        f.findings
            .push(finding("f1", FindingKind::Divergence, &["c1", "c3"]));
        assert!(flagged("divergence-unrecorded", &f).is_empty());
    }

    #[test]
    fn missing_limit_or_list_is_flagged() {
        assert_rule(
            "missing-limit-or-list",
            |f| f.model.unhandled = None,
            &["unhandled"],
        );
        assert_rule(
            "missing-limit-or-list",
            |f| f.model.transitions[2].retry.as_mut().unwrap().changes = None,
            &["t3"],
        );
        assert_rule(
            "missing-limit-or-list",
            |f| f.model.touchpoints[0].no_answer = None,
            &["h1"],
        );
    }

    fn with_unreachable_state() -> Fixture {
        let mut f = Fixture::clean();
        f.model
            .states
            .push(state("orphan", StateKind::TerminalSuccess));
        f
    }

    fn result<'a>(v: &'a Validation, rule: &str) -> &'a RuleResult {
        v.results.iter().find(|r| r.rule == rule).unwrap()
    }

    #[test]
    fn a_valid_waiver_clears_a_model_rule() {
        let mut f = with_unreachable_state();
        assert!(!validate(&f.study()).passed());
        f.findings
            .push(waiver("w1", "unreachable-state", &["orphan"], Some("iv1")));
        let v = validate(&f.study());
        assert!(v.passed());
        assert_eq!(result(&v, "unreachable-state").waived, ["orphan"]);
    }

    #[test]
    fn a_waiver_that_is_not_valid_clears_nothing() {
        let bad: Vec<(&str, Finding)> = vec![
            (
                "no intervention",
                waiver("w", "unreachable-state", &["orphan"], None),
            ),
            (
                "unknown intervention",
                waiver("w", "unreachable-state", &["orphan"], Some("agent-note")),
            ),
            (
                "wrong rule",
                waiver("w", "dead-end", &["orphan"], Some("iv1")),
            ),
            (
                "other ids",
                waiver("w", "unreachable-state", &["done"], Some("iv1")),
            ),
            ("no ids", waiver("w", "unreachable-state", &[], Some("iv1"))),
            (
                "not a waiver",
                Finding {
                    waiver: waiver("w", "unreachable-state", &["orphan"], Some("iv1")).waiver,
                    ..finding("w", FindingKind::ModelLimit, &["orphan"])
                },
            ),
        ];
        for (why, w) in bad {
            let mut f = with_unreachable_state();
            f.findings.push(w);
            assert!(!validate(&f.study()).passed(), "{why}");
        }
        let mut f = with_unreachable_state();
        let mut w = waiver("w", "unreachable-state", &["orphan"], Some("iv1"));
        w.waiver.as_mut().unwrap().reason = "  ".into();
        f.findings.push(w);
        assert!(!validate(&f.study()).passed(), "empty reason");
    }

    #[test]
    fn an_evidence_rule_cannot_be_waived() {
        let mut f = Fixture::clean();
        f.claims[0].sources.clear();
        f.findings
            .push(waiver("w1", "uncited-claim", &["c1"], Some("iv1")));
        let v = validate(&f.study());
        assert!(!v.passed());
        assert_eq!(result(&v, "uncited-claim").offending, ["c1"]);
        assert!(result(&v, "uncited-claim").waived.is_empty());
    }

    #[test]
    fn the_fixture_repository_answers_reachability() {
        let f = Fixture::clean();
        assert!(f.repo.reachable(OLD, PIN));
        assert!(f.repo.reachable(PIN, PIN));
        assert!(!f.repo.reachable(OTHER, PIN));
        assert!(f.repo.file_lines(PIN, "nope").is_none());
    }

    // ---- the firstmate work-handling study, checked against the real pinned tree ------------------------

    use serde::de::DeserializeOwned;
    use std::path::{Component, Path, PathBuf};

    const STUDY_SHA: &str = "6407052f4c521e39401ce2305dfa45d201249827";
    const STUDY_FILES: [&str; 6] = [
        "pin.json",
        "claims.json",
        "rationales.json",
        "model.json",
        "findings.json",
        "validation.json",
    ];

    /// The tree mounted at `pinned-source/`, answering git questions for the one commit it holds.
    #[derive(Debug)]
    struct MountedTree {
        sha: String,
        root: PathBuf,
    }

    impl GitLookup for MountedTree {
        fn file_lines(&self, sha: &str, path: &str) -> Option<Vec<String>> {
            let plain = Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
            if sha != self.sha || !plain {
                return None;
            }
            let text = std::fs::read_to_string(self.root.join(path)).ok()?;
            Some(text.lines().map(String::from).collect())
        }

        fn reachable(&self, commit: &str, from: &str) -> bool {
            commit == from && from == self.sha
        }
    }

    fn study_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../studies/firstmate-work-handling")
    }

    fn load<T: DeserializeOwned>(name: &str) -> T {
        let text = std::fs::read_to_string(study_dir().join(name)).expect("a study file");
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    /// The mounted tree, or `None` where `pinned-source/` is absent (a plain checkout has no mount).
    fn mounted_tree() -> Option<MountedTree> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pinned-source");
        root.is_dir().then(|| MountedTree {
            sha: STUDY_SHA.into(),
            root,
        })
    }

    /// The study's records, owned.
    #[derive(Clone)]
    struct Loaded {
        pin: Pin,
        claims: Vec<Claim>,
        rationales: Vec<Rationale>,
        model: Model,
        findings: Vec<Finding>,
    }

    impl Loaded {
        fn read() -> Loaded {
            Loaded {
                pin: load("pin.json"),
                claims: load("claims.json"),
                rationales: load("rationales.json"),
                model: load("model.json"),
                findings: load("findings.json"),
            }
        }

        fn violations(&self, tree: &MountedTree, rule: &str) -> Vec<String> {
            let none = BTreeSet::new();
            let study = Study {
                pin: &self.pin,
                claims: &self.claims,
                runs: &[],
                rationales: &self.rationales,
                model: &self.model,
                findings: &self.findings,
                user_interventions: &none,
                template_voice: &[Provenance::Stated],
                git: tree,
            };
            let rule = RULES.iter().find(|r| r.id == rule).expect("a known rule");
            (rule.check)(&study)
                .into_iter()
                .flat_map(|v| v.ids)
                .collect()
        }

        fn validate(&self, tree: &MountedTree) -> Validation {
            let none = BTreeSet::new();
            validate(&Study {
                pin: &self.pin,
                claims: &self.claims,
                runs: &[],
                rationales: &self.rationales,
                model: &self.model,
                findings: &self.findings,
                user_interventions: &none,
                template_voice: &[Provenance::Stated],
                git: tree,
            })
        }
    }

    #[test]
    fn the_firstmate_study_passes_every_rule_against_the_pinned_tree() {
        let Some(tree) = mounted_tree() else {
            eprintln!("pinned-source/ is absent; the study is not checked against a tree here");
            return;
        };
        let study = Loaded::read();
        let computed = study.validate(&tree);
        assert!(
            computed.passed(),
            "{:?}",
            computed
                .results
                .iter()
                .filter(|r| !r.pass)
                .collect::<Vec<_>>()
        );
        assert_eq!(computed.results.len(), RULES.len());
        // The recorded validation is what the rules answer now.
        let recorded: Validation = load("validation.json");
        assert_eq!(recorded, computed);
        // The citations were really checked: the study has file sources, all with quotes.
        let sources: Vec<&Source> = study
            .claims
            .iter()
            .flat_map(|c| &c.sources)
            .chain(study.rationales.iter().flat_map(|r| &r.sources))
            .collect();
        assert!(sources.len() > 20);
        assert!(sources.iter().all(|s| s.kind == SourceKind::File));
    }

    #[test]
    fn a_wrong_sha_or_a_changed_quote_fails_the_firstmate_study() {
        let Some(tree) = mounted_tree() else {
            return;
        };
        let study = Loaded::read();
        assert!(study.violations(&tree, "wrong-sha").is_empty());
        assert!(study.violations(&tree, "quote-mismatch").is_empty());

        let mut other_sha = study.clone();
        other_sha.claims[0].sources[0].sha = Some(OTHER.into());
        assert_eq!(other_sha.violations(&tree, "wrong-sha"), ["c1"]);

        let mut other_pin = study.clone();
        other_pin.pin.sha = OTHER.into();
        assert!(
            other_pin
                .violations(&tree, "wrong-sha")
                .contains(&"c1".to_string())
        );
        assert!(
            other_pin
                .violations(&tree, "bad-citation")
                .contains(&"c1".to_string())
        );

        let mut other_quote = study.clone();
        other_quote.claims[0].sources[0].quote = Some("text that is not in the file".into());
        assert_eq!(other_quote.violations(&tree, "quote-mismatch"), ["c1"]);

        let mut other_card_quote = study.clone();
        other_card_quote.rationales[0].sources[0].quote = Some("text that is not there".into());
        assert_eq!(
            other_card_quote.violations(&tree, "quote-mismatch"),
            ["ra1"]
        );

        let mut other_lines = study;
        other_lines.claims[0].sources[0].end_line = Some(999_999);
        assert_eq!(other_lines.violations(&tree, "bad-citation"), ["c1"]);
    }

    #[test]
    fn the_firstmate_study_carries_the_pin_and_tags_every_claim() {
        let study = Loaded::read();
        assert_eq!(study.pin.sha, STUDY_SHA);
        assert_eq!(study.pin.sha.len(), 40);
        for name in STUDY_FILES {
            let text = std::fs::read_to_string(study_dir().join(name)).unwrap();
            assert!(!text.contains("UNRESOLVED"), "{name}: a placeholder pin");
            assert!(!text.to_lowercase().contains("not examined"), "{name}");
        }
        for source in study
            .claims
            .iter()
            .flat_map(|c| &c.sources)
            .chain(study.rationales.iter().flat_map(|r| &r.sources))
        {
            assert_eq!(source.sha.as_deref(), Some(STUDY_SHA));
        }
        for claim in &study.claims {
            assert!(
                claim.text.starts_with("[mechanism] ") || claim.text.starts_with("[prose-only] "),
                "{}: enforcement is not tagged",
                claim.id
            );
        }
    }

    #[test]
    fn a_guard_of_none_is_never_said_to_be_evaluated_by_code() {
        let study = Loaded::read();
        let unguarded: Vec<&crate::model::Transition> = study
            .model
            .transitions
            .iter()
            .filter(|t| t.guard.as_deref() == Some("none"))
            .collect();
        assert!(!unguarded.is_empty());
        for t in unguarded {
            assert_ne!(t.guard_evaluated_by, EvaluatedBy::Code, "{}", t.id);
            assert!(t.guard_counters.is_empty(), "{}", t.id);
        }
    }
}
