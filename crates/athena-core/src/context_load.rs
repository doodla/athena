//! Context compression (requirement OLY-B-A13-01): the tokens loaded per class of work, beside whether the
//! outcomes held.
//!
//! The cost of a class is its tokens loaded per landed job. The numerator is the tokens of every closed request
//! job of the class (landed, rejected, failed or timed out), so work that failed is charged to the jobs that
//! landed; the denominator is the class's landed jobs. Beside it the same class shows the share of its closed
//! jobs that landed and its recorded mistakes. Tokens loaded are `input + cache_read + cache_write` of each
//! invocation of the job (the reader, planner, critic, producer and verifier), each taken with
//! [`athena_data::Quantity::best_lower`]: the value, else the lower bound, else zero. A killed invocation that
//! recorded only a lower bound therefore counts as the least it loaded. Output tokens are what the model wrote,
//! not context loaded, and are left out.
//!
//! Not counted: steward, audience and auditor runs. Each has its own generated job id and is stored apart from
//! the request jobs, so no class of work owns it. A closed job with no recorded invocation did no model work and
//! is left out.
//!
//! The mistake count of a class is every recorded mistake of that class ([`crate::status::mistake_report`], the
//! source of the `mistakes` line), including mistakes with no job. The per-closed-job rate beside it makes
//! classes of different size comparable.
//!
//! `athena status` prints the result as the `context` line. A class with fewer than [`MIN_LANDED_JOBS`] landed
//! jobs prints nothing, and when no class qualifies there is no line, so a thin history is not shown as a good
//! result.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::status::mistake_report;
use crate::store::{JobState, Records};
use athena_data::records::OutcomeState;

/// Landed jobs a class needs before its figures are shown.
pub const MIN_LANDED_JOBS: usize = 3;

/// One closed request job: its class, whether it landed, and the tokens its invocations loaded.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassJob {
    pub id: String,
    pub class: String,
    pub landed: bool,
    pub tokens: f64,
}

/// The figures of one class of work.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassLoad {
    pub class: String,
    pub closed_jobs: usize,
    pub landed_jobs: usize,
    /// Tokens loaded by all the class's closed jobs.
    pub tokens: f64,
    pub tokens_per_landed_job: f64,
    /// Landed jobs as a share of closed jobs, 0 to 100.
    pub landed_percent: f64,
    /// Every recorded mistake of the class.
    pub mistakes: usize,
    pub mistakes_per_closed_job: f64,
}

/// The classes of work with enough landed jobs, by class name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContextLoad {
    pub classes: Vec<ClassLoad>,
}

/// Groups `jobs` by class. A class below [`MIN_LANDED_JOBS`] landed jobs is dropped; None when none is left.
pub fn analyze(jobs: &[ClassJob], mistakes: &BTreeMap<String, usize>) -> Option<ContextLoad> {
    let mut by_class: BTreeMap<&str, Vec<&ClassJob>> = BTreeMap::new();
    for job in jobs {
        by_class.entry(job.class.as_str()).or_default().push(job);
    }
    let classes: Vec<ClassLoad> = by_class
        .into_iter()
        .filter_map(|(class, group)| {
            let landed_jobs = group.iter().filter(|j| j.landed).count();
            if landed_jobs < MIN_LANDED_JOBS {
                return None;
            }
            let closed_jobs = group.len();
            let tokens: f64 = group.iter().map(|j| j.tokens).sum();
            let mistakes = mistakes.get(class).copied().unwrap_or(0);
            Some(ClassLoad {
                class: class.to_string(),
                closed_jobs,
                landed_jobs,
                tokens,
                tokens_per_landed_job: tokens / landed_jobs as f64,
                landed_percent: 100.0 * landed_jobs as f64 / closed_jobs as f64,
                mistakes,
                mistakes_per_closed_job: mistakes as f64 / closed_jobs as f64,
            })
        })
        .collect();
    (!classes.is_empty()).then_some(ContextLoad { classes })
}

impl ContextLoad {
    /// Measures `records`. None when no class of work has [`MIN_LANDED_JOBS`] landed jobs.
    pub fn compute(records: &Records) -> Option<ContextLoad> {
        let jobs: Vec<ClassJob> = records
            .request_jobs()
            .into_iter()
            .filter(|j| !j.invocations.is_empty())
            .filter_map(|j| {
                let JobState::Closed(state) = j.state() else {
                    return None;
                };
                if !matches!(
                    state,
                    OutcomeState::Landed
                        | OutcomeState::Rejected
                        | OutcomeState::Failed
                        | OutcomeState::TimedOut
                ) {
                    return None;
                }
                let tokens = j
                    .invocations
                    .iter()
                    .map(|i| {
                        i.tokens.input.best_lower()
                            + i.tokens.cache_read.best_lower()
                            + i.tokens.cache_write.best_lower()
                    })
                    .sum();
                Some(ClassJob {
                    id: j.id().as_str().to_string(),
                    class: j.class()?.to_string(),
                    landed: state == OutcomeState::Landed,
                    tokens,
                })
            })
            .collect();
        analyze(&jobs, &mistake_report(records).by_class)
    }

    /// The text `athena status` prints after the `context` label: a header line, then one line per class.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "{} class(es) of work with {MIN_LANDED_JOBS}+ landed jobs; tokens loaded (input, cache read, cache write) of recorded invocations per landed job, beside the share of closed jobs that landed",
            self.classes.len()
        )];
        for c in &self.classes {
            out.push(format!(
                "{}: {:.0} tokens per landed job · {} of {} closed jobs landed ({:.0}%) · {} mistake(s) recorded, {:.2} per closed job",
                c.class,
                c.tokens_per_landed_job,
                c.landed_jobs,
                c.closed_jobs,
                c.landed_percent,
                c.mistakes,
                c.mistakes_per_closed_job
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: &str, class: &str, landed: bool, tokens: f64) -> ClassJob {
        ClassJob {
            id: id.into(),
            class: class.into(),
            landed,
            tokens,
        }
    }

    fn two_classes() -> Vec<ClassJob> {
        vec![
            job("d1", "docs", true, 400.0),
            job("d2", "docs", true, 900.0),
            job("d3", "docs", true, 1700.0),
            job("d4", "docs", false, 600.0),
            job("c1", "code", true, 2000.0),
            job("c2", "code", true, 3000.0),
            job("c3", "code", true, 4000.0),
        ]
    }

    #[test]
    fn classes_are_grouped_with_exact_figures() {
        let mistakes = BTreeMap::from([("docs".to_string(), 2)]);
        let load = analyze(&two_classes(), &mistakes).expect("two classes");
        assert_eq!(load.classes.len(), 2);
        let code = &load.classes[0];
        assert_eq!(code.class, "code");
        assert_eq!(code.tokens_per_landed_job, 3000.0);
        assert_eq!(code.landed_percent, 100.0);
        assert_eq!((code.mistakes, code.mistakes_per_closed_job), (0, 0.0));
        let docs = &load.classes[1];
        assert_eq!(docs.class, "docs");
        assert_eq!((docs.closed_jobs, docs.landed_jobs), (4, 3));
        assert_eq!(docs.tokens, 3600.0);
        assert_eq!(docs.tokens_per_landed_job, 1200.0);
        assert_eq!(docs.landed_percent, 75.0);
        assert_eq!((docs.mistakes, docs.mistakes_per_closed_job), (2, 0.5));
    }

    #[test]
    fn a_class_below_the_minimum_is_dropped_and_none_remains_none() {
        let mut jobs = two_classes();
        jobs.push(job("g1", "config", true, 10.0));
        jobs.push(job("g2", "config", true, 10.0));
        jobs.push(job("g3", "config", false, 10.0));
        let load = analyze(&jobs, &BTreeMap::new()).unwrap();
        assert!(load.classes.iter().all(|c| c.class != "config"));
        let thin = [
            job("g1", "config", true, 10.0),
            job("g2", "config", true, 10.0),
            job("g3", "config", false, 10.0),
        ];
        assert_eq!(analyze(&thin, &BTreeMap::new()), None);
        assert_eq!(analyze(&[], &BTreeMap::new()), None);
    }

    #[test]
    fn the_lines_state_each_figure() {
        let mistakes = BTreeMap::from([("docs".to_string(), 2)]);
        let text = analyze(&two_classes(), &mistakes)
            .unwrap()
            .lines()
            .join("\n");
        assert!(
            text.contains(
                "docs: 1200 tokens per landed job · 3 of 4 closed jobs landed (75%) · 2 mistake(s) recorded, 0.50 per closed job"
            ),
            "{text}"
        );
        assert!(text.starts_with("2 class(es) of work"), "{text}");
    }
}
