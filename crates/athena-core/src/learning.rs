//! Measured organizational learning (requirement OLY-9-09): each repeated failure class joined to the
//! improvement request that addresses it, and to the outcomes of the class of work that request belongs to
//! (the producer-setting specialization verdict and the minutes-per-landed-job trend).
//!
//! [`join`] is display-only. `athena status` prints its lines under the `repeated` label, and prints nothing
//! when no cause repeated, so an empty history is not shown as a good result.

use crate::class_trend::ClassTrend;
use crate::hestia::RepeatedCause;
use crate::status::{ClassSpecialization, SpecializationVerdict};

/// One line per failure class among `causes`: the class, the request that addresses it with its landed or open
/// state, and the outcome of that request's class of work. `work_class` gives the class of work of a job id.
/// A cause without a failure class is left out; a class with several causes uses the first request found.
pub fn join(
    causes: &[RepeatedCause],
    work_class: &dyn Fn(&str) -> Option<String>,
    specialization: &[ClassSpecialization],
    trend: Option<&ClassTrend>,
) -> Vec<String> {
    let mut classes: Vec<(&str, Option<&crate::hestia::AddressingRequest>)> = Vec::new();
    for cause in causes {
        let Some(class) = cause.class.as_deref() else {
            continue;
        };
        match classes.iter_mut().find(|(c, _)| *c == class) {
            Some((_, request)) => {
                if request.is_none() {
                    *request = cause.request.as_ref();
                }
            }
            None => classes.push((class, cause.request.as_ref())),
        }
    }
    classes
        .into_iter()
        .map(|(class, request)| {
            let Some(request) = request else {
                return format!("learning: {class} · no request yet · no outcome to measure");
            };
            let state = if request.landed { "landed" } else { "open" };
            let outcome = match work_class(&request.job) {
                Some(work) => outcome(&work, specialization, trend),
                None => "class of work unknown".to_string(),
            };
            format!(
                "learning: {class} · request {} ({state}) · {outcome}",
                request.job
            )
        })
        .collect()
}

fn outcome(
    work: &str,
    specialization: &[ClassSpecialization],
    trend: Option<&ClassTrend>,
) -> String {
    let role = match specialization.iter().find(|s| s.class == work) {
        Some(s) => match &s.verdict {
            SpecializationVerdict::Helps { model, effort } => {
                format!("specialization helps: {model} · {effort}")
            }
            SpecializationVerdict::NotEnoughJobs => "specialization: not enough jobs".to_string(),
            SpecializationVerdict::NoBetterSetting => {
                "specialization: no better setting".to_string()
            }
        },
        None => "specialization: no producer run".to_string(),
    };
    let trend = match trend {
        Some(t) if t.improving.iter().any(|c| c.class == work) => "trend: improving",
        Some(t) if t.worse.iter().any(|c| c.class == work) => "trend: worse",
        _ => "trend: not compared",
    };
    format!("class of work {work}: {role}; {trend}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_trend::ClassChange;
    use crate::hestia::AddressingRequest;

    fn cause(class: Option<&str>, request: Option<(&str, bool)>) -> RepeatedCause {
        RepeatedCause {
            key: "k".into(),
            class: class.map(String::from),
            cause: "c".into(),
            introspections: vec!["i1".into(), "i2".into()],
            jobs: vec!["a".into(), "b".into()],
            request: request.map(|(job, landed)| AddressingRequest {
                job: job.into(),
                state: if landed { "landed" } else { "queued" }.into(),
                landed,
            }),
        }
    }

    fn spec(class: &str) -> ClassSpecialization {
        ClassSpecialization {
            class: class.into(),
            baseline_model: "m".into(),
            baseline_effort: "low".into(),
            settings: Vec::new(),
            verdict: SpecializationVerdict::Helps {
                model: "big".into(),
                effort: "high".into(),
            },
        }
    }

    fn trend(class: &str) -> ClassTrend {
        ClassTrend {
            compared: 1,
            improving: vec![ClassChange {
                class: class.into(),
                jobs: 4,
                older_median: 9.0,
                newer_median: 5.0,
                after_job: None,
            }],
            worse: Vec::new(),
        }
    }

    #[test]
    fn learning_joins_a_class_to_its_landed_request_and_the_outcome() {
        let causes = [
            cause(Some("flaky tests (FC-1)"), Some(("job-9", true))),
            cause(Some("flaky tests (FC-1)"), None),
            cause(Some("slow gate (FC-2)"), Some(("job-8", false))),
            cause(None, None),
        ];
        let work = |job: &str| (job == "job-9").then(|| "code".to_string());
        let lines = join(&causes, &work, &[spec("code")], Some(&trend("code")));
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(
            lines[0],
            "learning: flaky tests (FC-1) · request job-9 (landed) · class of work code: specialization helps: big · high; trend: improving"
        );
        assert_eq!(
            lines[1],
            "learning: slow gate (FC-2) · request job-8 (open) · class of work unknown"
        );
    }

    #[test]
    fn learning_marks_a_class_with_no_request_and_is_empty_without_causes() {
        let lines = join(&[cause(Some("x (FC-3)"), None)], &|_| None, &[], None);
        assert_eq!(
            lines,
            ["learning: x (FC-3) · no request yet · no outcome to measure"]
        );
        assert!(join(&[], &|_| None, &[], None).is_empty());
    }
}
