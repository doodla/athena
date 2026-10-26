//! The `athena` binary's library half: wiring, displays, and hook entry points. `main.rs` parses the
//! command line and calls into these and the core.

pub mod backtest;
pub mod decisions;
pub mod display;
pub mod drain;
pub mod hooks;
pub mod strategy;
pub mod wiring;

use athena_core::hestia::Sweep;

/// An age in whole seconds as a short phrase: `new`, `45s`, `20m`, `3h`, `2d`.
fn age_phrase(seconds: u64) -> String {
    match seconds {
        0 => "new".into(),
        1..=59 => format!("{seconds}s"),
        60..=3599 => format!("{}m", seconds / 60),
        3600..=86_399 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86_400),
    }
}

/// What `athena hestia last` and `athena hestia sweep` print: the sweep's time, every check with how many it
/// looked at and how many findings it reported (a check that failed to run is marked), then each finding with
/// its check, subject, age, state and evidence. `None` prints that no sweep is recorded, so an empty history
/// is never mistaken for a clean one.
pub fn hestia_report(sweep: Option<&Sweep>) -> String {
    let Some(sweep) = sweep else {
        return "no sweep recorded; run `athena hestia sweep` to make one\n".into();
    };
    let mut out = format!("Hestia sweep {} at {}\n", sweep.id, sweep.at);
    if sweep.checks.is_empty() {
        out.push_str("no checks registered; nothing was looked at\n");
    } else {
        out.push_str("Checks:\n");
        for c in &sweep.checks {
            let mark = if c.errored {
                " (check failed to run)"
            } else {
                ""
            };
            out.push_str(&format!(
                "  {}: {} finding(s), {} examined{mark}\n",
                c.check, c.count, c.examined
            ));
        }
    }
    if sweep.findings.is_empty() {
        out.push_str("Findings: none\n");
    } else {
        out.push_str("Findings:\n");
        for f in &sweep.findings {
            out.push_str(&format!(
                "  [{}] {} (age {}, {}): {}\n",
                f.check,
                f.subject,
                age_phrase(f.age_seconds),
                f.state.as_str(),
                f.evidence
            ));
        }
    }
    out
}

#[cfg(test)]
mod hestia_tests {
    use super::*;
    use athena_core::hestia::{Finding, FindingState};
    use athena_data::RecordId;
    use athena_data::records::HestiaCheckResult;

    fn sweep() -> Sweep {
        Sweep {
            id: RecordId::parse("20261026-090000-5abc").unwrap(),
            at: "2026-10-26T09:00:00Z".parse().unwrap(),
            checks: vec![
                HestiaCheckResult {
                    check: "branches".into(),
                    examined: 7,
                    count: 0,
                    errored: false,
                },
                HestiaCheckResult {
                    check: "binary-age".into(),
                    examined: 0,
                    count: 1,
                    errored: true,
                },
            ],
            findings: vec![Finding {
                check: "binary-age".into(),
                subject: "binary-age".into(),
                evidence: "the check failed to run: no installed binary".into(),
                age_seconds: 7200,
                state: FindingState::Open,
            }],
        }
    }

    #[test]
    fn with_no_sweep_recorded_it_says_so() {
        let text = hestia_report(None);
        assert!(text.contains("no sweep recorded"), "{text}");
    }

    #[test]
    fn a_sweep_lists_each_check_with_its_count_and_each_finding_in_full() {
        let text = hestia_report(Some(&sweep()));
        assert!(
            text.contains("branches: 0 finding(s), 7 examined\n"),
            "{text}"
        );
        assert!(
            text.contains("binary-age: 1 finding(s), 0 examined (check failed to run)"),
            "{text}"
        );
        assert!(
            text.contains(
                "[binary-age] binary-age (age 2h, open): the check failed to run: no installed binary"
            ),
            "{text}"
        );
    }

    #[test]
    fn a_sweep_with_no_findings_or_no_checks_says_so_plainly() {
        let mut clean = sweep();
        clean.checks.truncate(1);
        clean.findings.clear();
        let text = hestia_report(Some(&clean));
        assert!(text.contains("Findings: none"), "{text}");
        clean.checks.clear();
        let text = hestia_report(Some(&clean));
        assert!(text.contains("no checks registered"), "{text}");
    }
}
