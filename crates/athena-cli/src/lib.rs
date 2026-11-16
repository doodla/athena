//! The `athena` binary's library half: wiring, displays, and hook entry points. `main.rs` parses the
//! command line and calls into these and the core.

pub mod backtest;
pub mod decisions;
pub mod display;
pub mod drain;
pub mod hooks;
pub mod pushback;
pub mod slot_exec;
pub mod strategy;
pub mod wiring;

use athena_core::hestia::Sweep;

/// `command` and every subcommand under it with each `{name}` of the help text replaced by its value from
/// `values`: the about and long about, the before and after help, and the help and long help of each argument.
/// A placeholder with no value in `values` stays as it is. A limit that help text states comes from the
/// constant that enforces it, never from a number typed into the text, so a change to the constant cannot
/// leave the help stale (decision D156, extended by D190).
pub fn fill_limits(command: clap::Command, values: &[(&str, String)]) -> clap::Command {
    let fill = |text: String| -> Option<String> {
        let filled = values.iter().fold(text.clone(), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        });
        (filled != text).then_some(filled)
    };
    let mut command = command;
    if let Some(text) = command.get_about().and_then(|t| fill(t.to_string())) {
        command = command.about(text);
    }
    if let Some(text) = command.get_long_about().and_then(|t| fill(t.to_string())) {
        command = command.long_about(text);
    }
    if let Some(text) = command.get_before_help().and_then(|t| fill(t.to_string())) {
        command = command.before_help(text);
    }
    if let Some(text) = command.get_after_help().and_then(|t| fill(t.to_string())) {
        command = command.after_help(text);
    }
    if let Some(text) = command
        .get_before_long_help()
        .and_then(|t| fill(t.to_string()))
    {
        command = command.before_long_help(text);
    }
    if let Some(text) = command
        .get_after_long_help()
        .and_then(|t| fill(t.to_string()))
    {
        command = command.after_long_help(text);
    }
    command
        .mut_args(|mut arg| {
            if let Some(text) = arg.get_help().and_then(|t| fill(t.to_string())) {
                arg = arg.help(text);
            }
            if let Some(text) = arg.get_long_help().and_then(|t| fill(t.to_string())) {
                arg = arg.long_help(text);
            }
            arg
        })
        .mut_subcommands(|sub| fill_limits(sub, values))
}

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
mod fill_limits_tests {
    use super::*;
    use clap::{Arg, Command};

    fn command() -> Command {
        Command::new("top")
            .about("Runs at most {widgets} widgets.")
            .long_about(
                "Runs at most {widgets} widgets.\n\nThe {unknown} stays; so does {ceiling}.",
            )
            .before_help("Before {widgets}.")
            .after_help("After {widgets} and {gap}.")
            .arg(Arg::new("n").long("n").help("From {low} to {high}."))
            .arg(Arg::new("m").long("m").long_help("Long {low}."))
            .subcommand(
                Command::new("sub")
                    .about("Below {gap}.")
                    .arg(Arg::new("x").long("x").help("Deep {widgets}.")),
            )
    }

    #[test]
    fn fill_limits_replaces_only_named_placeholders_limits() {
        for widgets in [8, 16, 64] {
            let values = [
                ("widgets", widgets.to_string()),
                ("gap", "2".to_string()),
                ("low", "1".to_string()),
                ("high", "5".to_string()),
            ];
            let mut filled = fill_limits(command(), &values);
            let top = filled.render_long_help().to_string();
            assert!(
                top.contains(&format!("Runs at most {widgets} widgets.")),
                "{top}"
            );
            assert!(top.contains(&format!("Before {widgets}.")), "{top}");
            assert!(top.contains(&format!("After {widgets} and 2.")), "{top}");
            assert!(top.contains("From 1 to 5."), "{top}");
            assert!(top.contains("Long 1."), "{top}");
            // A placeholder with no value is left, as the drain's `{ceiling}` is for its own fill.
            assert!(
                top.contains("The {unknown} stays; so does {ceiling}."),
                "{top}"
            );
            assert!(
                !top.contains("{widgets}") && !top.contains("{gap}"),
                "{top}"
            );
            let sub = filled
                .find_subcommand_mut("sub")
                .unwrap()
                .render_long_help()
                .to_string();
            assert!(sub.contains("Below 2."), "{sub}");
            assert!(sub.contains(&format!("Deep {widgets}.")), "{sub}");
        }
    }

    #[test]
    fn fill_limits_leaves_text_without_a_placeholder_as_it_is() {
        let plain = Command::new("top")
            .about("No limit here.")
            .arg(Arg::new("n").long("n").help("Nothing to fill."));
        let before = plain.clone().render_long_help().to_string();
        let after = fill_limits(plain, &[("widgets", "8".to_string())])
            .render_long_help()
            .to_string();
        assert_eq!(before, after);
    }
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
            duty_pass: None,
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
