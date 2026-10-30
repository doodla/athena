//! The class-of-work trend (requirement OLY-9-10): for each class of work with enough landed jobs, whether the
//! minutes per landed job fell or rose from the older half of that class's jobs to the newer half, and, for a
//! class that rose, the earlier landed job after which it rose.
//!
//! The cost is the minutes of a landed job from [`crate::status::landed_jobs`] (invocation wall times, measured
//! check durations, landing and rebuild). Jobs are compared by median, so one slow job does not decide a class.
//! `athena status` prints the result as the `trend` line; it prints nothing when no class has
//! [`MIN_CLASS_JOBS`] landed jobs, so a thin history is not shown as a good result.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::status::{landed_jobs, median};
use crate::store::{JobState, Records};
use athena_data::records::OutcomeState;

/// Landed jobs a class needs before it is compared: two in the older half and two in the newer half.
pub const MIN_CLASS_JOBS: usize = 4;

/// One landed job's class and total minutes, in landing order across all classes.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassedJob {
    pub id: String,
    pub class: String,
    pub minutes: f64,
}

/// One class whose median minutes per landed job changed between its older and newer half.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassChange {
    pub class: String,
    /// Landed jobs of the class that were compared.
    pub jobs: usize,
    pub older_median: f64,
    pub newer_median: f64,
    /// For a class that got worse: the landed job (of any class) that landed just before the class's minutes
    /// rose above every older job of the class and stayed there. None for an improving class.
    pub after_job: Option<String>,
}

/// The classes that improved, the classes that got worse, and how many classes were compared.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassTrend {
    pub compared: usize,
    pub improving: Vec<ClassChange>,
    pub worse: Vec<ClassChange>,
}

/// Compares the classes in `jobs` (landed jobs, oldest first). None when no class has [`MIN_CLASS_JOBS`] jobs.
pub fn analyze(jobs: &[ClassedJob]) -> Option<ClassTrend> {
    let mut by_class: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (at, job) in jobs.iter().enumerate() {
        by_class.entry(job.class.as_str()).or_default().push(at);
    }
    let mut trend = ClassTrend {
        compared: 0,
        improving: Vec::new(),
        worse: Vec::new(),
    };
    for (class, positions) in by_class {
        if positions.len() < MIN_CLASS_JOBS {
            continue;
        }
        trend.compared += 1;
        let minutes: Vec<f64> = positions.iter().map(|&p| jobs[p].minutes).collect();
        let half = minutes.len() / 2;
        let older = median(&minutes[..half]);
        let newer = median(&minutes[half..]);
        let change = |after_job| ClassChange {
            class: class.to_string(),
            jobs: minutes.len(),
            older_median: older,
            newer_median: newer,
            after_job,
        };
        if newer < older {
            trend.improving.push(change(None));
        } else if newer > older {
            // The first job from which every later job of the class stays above the slowest older job.
            let ceiling = minutes[..half].iter().copied().fold(0.0, f64::max);
            let stays_from = (0..minutes.len())
                .rev()
                .take_while(|&i| minutes[i] > ceiling)
                .last()
                .filter(|&i| i > 0)
                .unwrap_or(half);
            let before = positions[stays_from].saturating_sub(1);
            trend.worse.push(change(Some(jobs[before].id.clone())));
        }
    }
    (trend.compared > 0).then_some(trend)
}

impl ClassTrend {
    /// Measures `records`. None when no class of work has [`MIN_CLASS_JOBS`] landed jobs.
    pub fn compute(records: &Records) -> Option<ClassTrend> {
        let classes: BTreeMap<String, String> = records
            .request_jobs()
            .into_iter()
            .filter(|j| matches!(j.state(), JobState::Closed(OutcomeState::Landed)))
            .filter_map(|j| Some((j.id().as_str().to_string(), j.class()?.to_string())))
            .collect();
        let jobs: Vec<ClassedJob> = landed_jobs(records)
            .into_iter()
            .filter_map(|landed| {
                Some(ClassedJob {
                    class: classes.get(&landed.id)?.clone(),
                    minutes: landed.stages.total(),
                    id: landed.id,
                })
            })
            .collect();
        analyze(&jobs)
    }

    /// The text `athena status` prints after the `trend` label: a header line, then one line per class.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "{} class(es) of work compared, median minutes per landed job in the older half against the newer half of each class's landed jobs",
            self.compared
        )];
        for c in &self.improving {
            out.push(format!(
                "improving: {} {:.1} to {:.1} min over {} jobs",
                c.class, c.older_median, c.newer_median, c.jobs
            ));
        }
        for c in &self.worse {
            let after = c.after_job.as_deref().unwrap_or("-");
            out.push(format!(
                "worse: {} {:.1} to {:.1} min over {} jobs; the rise followed landed job {after}",
                c.class, c.older_median, c.newer_median, c.jobs
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jobs(spec: &[(&str, &str, f64)]) -> Vec<ClassedJob> {
        spec.iter()
            .map(|(id, class, minutes)| ClassedJob {
                id: id.to_string(),
                class: class.to_string(),
                minutes: *minutes,
            })
            .collect()
    }

    #[test]
    fn a_class_whose_minutes_fall_is_improving() {
        let trend = analyze(&jobs(&[
            ("a", "docs", 20.0),
            ("b", "docs", 18.0),
            ("c", "docs", 10.0),
            ("d", "docs", 8.0),
        ]))
        .expect("one class compared");
        assert_eq!(trend.compared, 1);
        assert!(trend.worse.is_empty());
        assert_eq!(trend.improving.len(), 1);
        assert_eq!(trend.improving[0].class, "docs");
        assert_eq!(trend.improving[0].older_median, 19.0);
        assert_eq!(trend.improving[0].newer_median, 9.0);
    }

    #[test]
    fn a_class_whose_minutes_rise_names_the_job_after_which_they_rose() {
        // `x` is a landed job of another class between `b` and `c`; the class's cost rose from `c` on.
        let trend = analyze(&jobs(&[
            ("a", "code", 10.0),
            ("b", "code", 11.0),
            ("x", "docs", 5.0),
            ("c", "code", 20.0),
            ("d", "code", 22.0),
            ("e", "code", 25.0),
        ]))
        .expect("one class compared");
        assert!(trend.improving.is_empty());
        assert_eq!(trend.worse.len(), 1);
        assert_eq!(trend.worse[0].class, "code");
        assert_eq!(trend.worse[0].after_job.as_deref(), Some("x"));
        let text = trend.lines().join("\n");
        assert!(
            text.contains(
                "worse: code 10.5 to 22.0 min over 5 jobs; the rise followed landed job x"
            ),
            "{text}"
        );
    }

    #[test]
    fn a_class_with_too_few_jobs_is_not_reported() {
        let few = jobs(&[
            ("a", "docs", 20.0),
            ("b", "docs", 1.0),
            ("c", "docs", 1.0),
            ("d", "code", 1.0),
        ]);
        assert_eq!(analyze(&few), None);
        assert_eq!(analyze(&[]), None);
    }

    #[test]
    fn an_unchanged_class_is_compared_but_neither_improving_nor_worse() {
        let trend = analyze(&jobs(&[
            ("a", "docs", 5.0),
            ("b", "docs", 5.0),
            ("c", "docs", 5.0),
            ("d", "docs", 5.0),
        ]))
        .unwrap();
        assert_eq!(trend.compared, 1);
        assert!(trend.improving.is_empty() && trend.worse.is_empty());
    }
}
