//! The context-compression measure (requirement OLY-B-A13-01): the tokens loaded per landed job, by class of
//! work, beside the landed share and the mistakes of the same class.
//!
//! A job's loaded tokens are the sum, over its own invocations, of input, cache-read and cache-write tokens,
//! each taken as its best value or lower bound ([`athena_data::Quantity::best_lower`], as the job's spend is).
//! Output tokens are generated, not loaded, so they are left out. A class's figure is the mean over its landed
//! jobs, which is the class's total loaded tokens divided by its landed jobs; it is a point-in-time figure, and
//! a drop over time is read by comparing runs of `athena status`.
//!
//! The landed share is landed jobs over closed request jobs of the class. The mistakes are those whose `job`
//! is one of that class's closed jobs, so the mistakes and the share cover the same set of jobs. A mistake that
//! names no job, or a job outside the printed classes, is never folded into a class: it is counted in
//! [`ContextLoad::mistakes_left_out`].
//!
//! `athena status` prints the result as the `context` line. A class with fewer than [`MIN_CLASS_JOBS`] landed
//! jobs that have a token record gets no figure, and when no class has one nothing is printed.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::store::{JobRecords, JobState, Records};
use athena_data::records::{Invocation, OutcomeState};

/// Landed jobs with a token record a class needs before a figure is shown for it.
pub const MIN_CLASS_JOBS: usize = 3;

/// One class of work with enough landed jobs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassContext {
    pub class: String,
    /// Landed jobs of the class that have a token record; the mean is over these.
    pub landed_jobs: usize,
    /// Closed request jobs of the class, landed or not.
    pub closed_jobs: usize,
    /// All landed jobs of the class, with or without a token record.
    pub landed_all: usize,
    /// Mean input, cache-read and cache-write tokens per landed job.
    pub tokens_per_landed_job: f64,
    /// Landed jobs over closed jobs of the class, 0 to 1.
    pub landed_share: f64,
    /// Mistakes attached to the class's closed jobs.
    pub mistakes: usize,
}

/// The per-class figures and what was left out of them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContextLoad {
    pub classes: Vec<ClassContext>,
    /// Classes with closed jobs but too few landed jobs with a token record to show.
    pub thin_classes: usize,
    /// Landed jobs of the shown classes with no token value; they are not in any mean.
    pub landed_without_tokens: usize,
    /// Mistakes that are not counted in any shown class.
    pub mistakes_left_out: usize,
}

/// Input, cache-read and cache-write tokens of `invocations`, or None when none has a token value.
fn loaded_tokens(invocations: &[Invocation]) -> Option<f64> {
    let mut any = false;
    let mut sum = 0.0;
    for i in invocations {
        for q in [&i.tokens.input, &i.tokens.cache_read, &i.tokens.cache_write] {
            any |= q.value().or(q.lower()).is_some();
            sum += q.best_lower();
        }
    }
    any.then_some(sum)
}

impl ContextLoad {
    /// Measures `records`. None when no class has [`MIN_CLASS_JOBS`] landed jobs with a token record.
    pub fn compute(records: &Records) -> Option<ContextLoad> {
        let mut by_class: BTreeMap<&str, Vec<&JobRecords>> = BTreeMap::new();
        for job in records.request_jobs() {
            if matches!(job.state(), JobState::Closed(_))
                && let Some(class) = job.class()
            {
                by_class.entry(class).or_default().push(job);
            }
        }
        let all_mistakes = records.interventions.iter().filter(|i| i.is_mistake());
        let total_mistakes = all_mistakes.count();

        let mut out = ContextLoad {
            classes: Vec::new(),
            thin_classes: 0,
            landed_without_tokens: 0,
            mistakes_left_out: 0,
        };
        let mut counted = 0;
        for (class, jobs) in by_class {
            let landed: Vec<&&JobRecords> = jobs
                .iter()
                .filter(|j| j.state() == JobState::Closed(OutcomeState::Landed))
                .collect();
            let tokens: Vec<f64> = landed
                .iter()
                .filter_map(|j| loaded_tokens(&j.invocations))
                .collect();
            if tokens.len() < MIN_CLASS_JOBS {
                out.thin_classes += 1;
                continue;
            }
            let mistakes = records
                .interventions
                .iter()
                .filter(|i| i.is_mistake())
                .filter(|i| {
                    i.job
                        .as_ref()
                        .is_some_and(|id| jobs.iter().any(|j| j.id() == id))
                })
                .count();
            counted += mistakes;
            out.landed_without_tokens += landed.len() - tokens.len();
            out.classes.push(ClassContext {
                class: class.to_string(),
                landed_jobs: tokens.len(),
                closed_jobs: jobs.len(),
                landed_all: landed.len(),
                tokens_per_landed_job: tokens.iter().sum::<f64>() / tokens.len() as f64,
                landed_share: landed.len() as f64 / jobs.len() as f64,
                mistakes,
            });
        }
        out.mistakes_left_out = total_mistakes - counted;
        (!out.classes.is_empty()).then_some(out)
    }

    /// The text `athena status` prints after the `context` label: a header line, one line per class, then a
    /// note for each kind of job or mistake that is not in the figures, each only when there is one.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![
            "mean input, cache-read and cache-write tokens loaded per landed job, by class of work, with the landed share and the mistakes of the same closed jobs"
                .to_string(),
        ];
        for c in &self.classes {
            out.push(format!(
                "{}: {:.0} tokens per landed job over {} landed jobs · landed {} of {} closed ({:.0}%) · {} {} in those {} jobs",
                c.class,
                c.tokens_per_landed_job,
                c.landed_jobs,
                c.landed_all,
                c.closed_jobs,
                c.landed_share * 100.0,
                c.mistakes,
                if c.mistakes == 1 { "mistake" } else { "mistakes" },
                c.closed_jobs
            ));
        }
        if self.thin_classes > 0 {
            out.push(format!(
                "{} class(es) have fewer than {MIN_CLASS_JOBS} landed jobs with a token record: no figure",
                self.thin_classes
            ));
        }
        if self.landed_without_tokens > 0 {
            out.push(format!(
                "{} landed job(s) have no token record and are not in the means",
                self.landed_without_tokens
            ));
        }
        if self.mistakes_left_out > 0 {
            out.push(format!(
                "{} mistake(s) are not tied to a closed job of a class shown here",
                self.mistakes_left_out
            ));
        }
        out
    }
}
