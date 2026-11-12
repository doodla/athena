//! Pushback: how Athena answers a request that evidence shows to be wrong, costly or in tension with
//! another, and how it proposes its next big task when a study is done (OLY-B-A1-03).
//!
//! Both functions are pure. [`push_back`] turns evidence into a [`Pushback`] that carries the evidence and
//! an alternative, and keeps the request in effect until the change is agreed: pushing back never refuses.
//! [`next_big_task`] turns a finished study and its candidate tasks into a proposal with reasons.

use serde::Serialize;

/// What the evidence says is the matter with a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Concern {
    Wrong,
    Costly,
    InTension,
}

impl Concern {
    fn phrase(self) -> &'static str {
        match self {
            Concern::Wrong => "looks wrong",
            Concern::Costly => "looks costly",
            Concern::InTension => "is in tension with another request",
        }
    }
}

/// One piece of evidence against a request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Evidence {
    pub concern: Concern,
    /// What was read or measured, such as a record id, a file or a count.
    pub finding: String,
}

/// What Athena does with the request after pushing back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Disposition {
    /// The request is met as written; the user has not agreed a change.
    MeetAsRequested,
    /// The user agreed the alternative; it replaces the request.
    MeetAlternative,
}

/// A pushback note.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pushback {
    pub request: String,
    pub evidence: Vec<Evidence>,
    pub alternative: String,
    pub disposition: Disposition,
}

impl Pushback {
    /// The text said to the user: the concerns with their evidence, the alternative, and what will be done.
    pub fn render(&self) -> String {
        let mut out = format!("Pushback on: {}\n", self.request);
        for e in &self.evidence {
            out.push_str(&format!("  {}: {}\n", e.concern.phrase(), e.finding));
        }
        out.push_str(&format!("Alternative: {}\n", self.alternative));
        out.push_str(match self.disposition {
            Disposition::MeetAsRequested => {
                "Until the change is agreed, the request is met as written.\n"
            }
            Disposition::MeetAlternative => "The change is agreed; the alternative is met.\n",
        });
        out
    }
}

/// Pushes back on `request`. With no evidence there is nothing to say and `None` is returned. Otherwise the
/// note carries all the evidence and the alternative, and the request stays in effect unless `agreed`.
pub fn push_back(
    request: &str,
    evidence: &[Evidence],
    alternative: &str,
    agreed: bool,
) -> Option<Pushback> {
    let evidence: Vec<Evidence> = evidence
        .iter()
        .filter(|e| !e.finding.trim().is_empty())
        .cloned()
        .collect();
    if evidence.is_empty() {
        return None;
    }
    Some(Pushback {
        request: request.to_string(),
        evidence,
        alternative: alternative.to_string(),
        disposition: if agreed {
            Disposition::MeetAlternative
        } else {
            Disposition::MeetAsRequested
        },
    })
}

/// A candidate for the next big task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub task: String,
    pub reasons: Vec<String>,
}

/// A proposal for the next big task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Proposal {
    pub task: String,
    pub reasons: Vec<String>,
}

/// When the study is done, proposes the candidate with the most reasons (the first on a tie). A study that
/// is not done, or no candidate with a reason, gives `None`: a proposal always comes with reasons.
pub fn next_big_task(study_done: bool, candidates: &[Candidate]) -> Option<Proposal> {
    if !study_done {
        return None;
    }
    let mut best: Option<&Candidate> = None;
    for c in candidates.iter().filter(|c| !c.reasons.is_empty()) {
        if best.is_none_or(|b| c.reasons.len() > b.reasons.len()) {
            best = Some(c);
        }
    }
    best.map(|c| Proposal {
        task: c.task.clone(),
        reasons: c.reasons.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn ev(concern: Concern, finding: &str) -> Evidence {
        Evidence {
            concern,
            finding: finding.into(),
        }
    }

    #[test]
    fn evidence_yields_a_pushback_with_the_evidence_and_an_alternative() {
        let e = [
            ev(
                Concern::Costly,
                "the last 3 jobs of this kind cost 4x the median",
            ),
            ev(Concern::InTension, "OLY-B-X-01 forbids the same file"),
        ];
        let p = push_back("rewrite everything", &e, "change one module first", false).unwrap();
        assert_eq!(p.evidence, e);
        assert_eq!(p.alternative, "change one module first");
        let text = p.render();
        assert!(text.contains("looks costly: the last 3 jobs"), "{text}");
        assert!(
            text.contains("in tension with another request: OLY-B-X-01"),
            "{text}"
        );
        assert!(
            text.contains("Alternative: change one module first"),
            "{text}"
        );
    }

    #[test]
    fn without_evidence_there_is_no_pushback() {
        assert!(push_back("r", &[], "a", false).is_none());
        assert!(push_back("r", &[ev(Concern::Wrong, "  ")], "a", false).is_none());
    }

    #[test]
    fn the_request_is_met_unless_the_change_is_agreed() {
        let e = [ev(Concern::Wrong, "the file it names does not exist")];
        let p = push_back("edit missing.rs", &e, "edit present.rs", false).unwrap();
        assert_eq!(p.disposition, Disposition::MeetAsRequested);
        assert!(p.render().contains("met as written"));
        let p = push_back("edit missing.rs", &e, "edit present.rs", true).unwrap();
        assert_eq!(p.disposition, Disposition::MeetAlternative);
        assert!(p.render().contains("alternative is met"));
    }

    #[test]
    fn a_finished_study_proposes_the_best_supported_task_with_reasons() {
        let c = [
            Candidate {
                task: "small".into(),
                reasons: vec!["one reason".into()],
            },
            Candidate {
                task: "big".into(),
                reasons: vec!["first".into(), "second".into()],
            },
            Candidate {
                task: "unreasoned".into(),
                reasons: vec![],
            },
        ];
        let p = next_big_task(true, &c).unwrap();
        assert_eq!(p.task, "big");
        assert_eq!(p.reasons, ["first", "second"]);
        assert!(next_big_task(false, &c).is_none());
        assert!(next_big_task(true, &c[2..]).is_none());
    }

    /// OLY-B-A1-03 must name this module as its mechanism, with owner and a measure naming this test.
    #[test]
    fn pushback_is_the_mechanism_of_oly_b_a1_03() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let spec: toml::Table =
            toml::from_str(&std::fs::read_to_string(root.join("docs/spec.toml")).unwrap()).unwrap();
        let req = spec["requirement"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"].as_str() == Some("OLY-B-A1-03"))
            .expect("OLY-B-A1-03 is in docs/spec.toml");
        let mechanism = req["mechanism"].as_str().unwrap();
        let path = "crates/athena-cli/src/pushback.rs";
        assert!(mechanism.contains(path), "mechanism: {mechanism}");
        assert!(root.join(path).is_file());
        assert_eq!(req["owner"].as_str(), Some("Athena"));
        let measure = req["measure"].as_str().unwrap();
        assert!(measure.ends_with("status: not shown"), "{measure}");
        assert!(measure.contains("pushback_is_the_mechanism_of_oly_b_a1_03"));
    }
}
