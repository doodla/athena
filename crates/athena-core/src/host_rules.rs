//! A read-only check of the host rules that `docs/design/records-history.md` §3 requires (D111). It is part 3
//! of job 1 in the build plan of §10.
//!
//! For `athena/records` and `main` it reads `repos/{owner}/{repo}/rules/branches/{branch}` with a plain GET
//! and reports three rules per branch: force-push forbidden, deletion forbidden, and up-to-date required.
//! Each is in force, not in force, or unknown. A failed call or a reply that is not a JSON list of rules
//! gives unknown for every rule on that branch, never in force or not in force.
//!
//! Only rulesets are seen. The view lists the rules that apply to a branch from rulesets; classic branch
//! protection is not visible through it, so "not in force" means "no ruleset sets it", not "unprotected".
//! The check changes no host setting.

use crate::gh::{Runner, get_args};
use serde_json::Value;

/// The branches §3 puts rules on.
pub const BRANCHES: [&str; 2] = ["athena/records", "main"];

/// The three rules the check reports, in report order.
pub const RULES: [&str; 3] = [
    "force-push forbidden",
    "deletion forbidden",
    "up-to-date required",
];

/// Whether a rule is in force on a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleState {
    InForce,
    NotInForce,
    /// The host could not be read, or its reply could not be understood.
    Unknown,
}

impl RuleState {
    fn text(self) -> &'static str {
        match self {
            RuleState::InForce => "in force",
            RuleState::NotInForce => "not in force",
            RuleState::Unknown => "unknown",
        }
    }
}

/// The three rule states of one branch, in [`RULES`] order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchRules {
    pub branch: String,
    pub states: [RuleState; 3],
}

/// Reads the rulesets view of both branches through `runner`. Every call is `gh api <path>` (a GET).
pub fn check(runner: &dyn Runner, owner: &str, repo: &str) -> Vec<BranchRules> {
    BRANCHES
        .iter()
        .map(|branch| {
            let path = format!("repos/{owner}/{repo}/rules/branches/{branch}");
            let states = match runner.gh(&get_args(&path)) {
                Ok(reply) => evaluate(&reply),
                Err(_) => [RuleState::Unknown; 3],
            };
            BranchRules {
                branch: (*branch).to_string(),
                states,
            }
        })
        .collect()
}

/// The rule states in one reply of the rulesets view. Anything but a JSON list gives unknown for all three.
pub fn evaluate(reply: &str) -> [RuleState; 3] {
    let Ok(Value::Array(rules)) = serde_json::from_str::<Value>(reply) else {
        return [RuleState::Unknown; 3];
    };
    let mut force_push = false;
    let mut deletion = false;
    let mut up_to_date = false;
    for rule in &rules {
        match rule.get("type").and_then(Value::as_str) {
            Some("non_fast_forward") => force_push = true,
            Some("deletion") => deletion = true,
            Some("required_status_checks") => {
                let strict = rule
                    .pointer("/parameters/strict_required_status_checks_policy")
                    .and_then(Value::as_bool);
                if strict == Some(true) {
                    up_to_date = true;
                }
            }
            _ => {}
        }
    }
    let state = |on| {
        if on {
            RuleState::InForce
        } else {
            RuleState::NotInForce
        }
    };
    [state(force_push), state(deletion), state(up_to_date)]
}

/// The plain-text report: one line per rule per branch, then the rulesets-only note.
pub fn render(results: &[BranchRules]) -> String {
    let mut out = String::new();
    for result in results {
        for (rule, state) in RULES.iter().zip(result.states) {
            out.push_str(&format!("{}: {}: {}\n", result.branch, rule, state.text()));
        }
    }
    out.push_str(
        "Only rulesets are seen; classic branch protection is not visible to this check, so set these rules as rulesets.\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gh::GhFailure;
    use anyhow::Result;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// Records every call and answers from a queue.
    struct Recording {
        calls: RefCell<Vec<Vec<String>>>,
        replies: RefCell<VecDeque<Result<String>>>,
    }

    impl Recording {
        fn new(replies: Vec<Result<String>>) -> Self {
            Recording {
                calls: RefCell::new(Vec::new()),
                replies: RefCell::new(replies.into()),
            }
        }
    }

    impl Runner for Recording {
        fn gh(&self, args: &[String]) -> Result<String> {
            self.calls.borrow_mut().push(args.to_vec());
            self.replies.borrow_mut().pop_front().expect("a reply")
        }
    }

    fn ok(text: &str) -> Result<String> {
        Ok(text.to_string())
    }

    fn failure(status: u16) -> Result<String> {
        Err(GhFailure {
            call: "api".into(),
            status_code: Some(status),
            text: format!("HTTP {status}"),
        }
        .into())
    }

    const RECORDS: &str =
        r#"[{"type":"non_fast_forward","ruleset_id":1},{"type":"deletion","ruleset_id":1}]"#;
    const MAIN: &str = r#"[{"type":"required_status_checks","parameters":{"strict_required_status_checks_policy":true,"required_status_checks":[]}}]"#;

    use RuleState::{InForce, NotInForce, Unknown};

    #[test]
    fn fixture_replies_give_in_force_states() {
        let runner = Recording::new(vec![ok(RECORDS), ok(MAIN)]);
        let got = check(&runner, "o", "r");
        assert_eq!(got[0].branch, "athena/records");
        assert_eq!(got[0].states, [InForce, InForce, NotInForce]);
        assert_eq!(got[1].branch, "main");
        assert_eq!(got[1].states, [NotInForce, NotInForce, InForce]);
    }

    #[test]
    fn empty_partial_and_non_strict_replies() {
        assert_eq!(evaluate("[]"), [NotInForce; 3]);
        assert_eq!(
            evaluate(r#"[{"type":"non_fast_forward"}]"#),
            [InForce, NotInForce, NotInForce]
        );
        let strict_false = r#"[{"type":"required_status_checks","parameters":{"strict_required_status_checks_policy":false}}]"#;
        assert_eq!(evaluate(strict_false)[2], NotInForce);
        let absent = r#"[{"type":"required_status_checks","parameters":{}}]"#;
        assert_eq!(evaluate(absent)[2], NotInForce);
        let no_parameters = r#"[{"type":"required_status_checks"},{"x":1}]"#;
        assert_eq!(evaluate(no_parameters), [NotInForce; 3]);
    }

    #[test]
    fn only_get_calls_to_the_rules_view_are_made() {
        let runner = Recording::new(vec![ok("[]"), ok("[]")]);
        check(&runner, "o", "r");
        let calls = runner.calls.borrow();
        assert_eq!(
            *calls,
            vec![
                vec![
                    "api".to_string(),
                    "repos/o/r/rules/branches/athena/records".into()
                ],
                vec!["api".to_string(), "repos/o/r/rules/branches/main".into()],
            ]
        );
        for arg in calls.iter().flatten() {
            assert!(!["--method", "-X", "-f", "-F", "--input"].contains(&arg.as_str()));
            assert!(!arg.contains("protection"));
        }
    }

    #[test]
    fn errors_and_bad_json_are_unknown_per_branch() {
        let runner = Recording::new(vec![failure(403), ok(MAIN)]);
        let got = check(&runner, "o", "r");
        assert_eq!(got[0].states, [Unknown; 3]);
        assert_eq!(got[1].states, [NotInForce, NotInForce, InForce]);

        let runner = Recording::new(vec![failure(503), ok("not json")]);
        let got = check(&runner, "o", "r");
        assert_eq!(got[0].states, [Unknown; 3]);
        assert_eq!(got[1].states, [Unknown; 3]);

        assert_eq!(evaluate(r#"{"message":"Not Found"}"#), [Unknown; 3]);
    }

    #[test]
    fn report_names_every_rule_and_the_rulesets_only_limit() {
        let runner = Recording::new(vec![ok(RECORDS), failure(403)]);
        let report = render(&check(&runner, "o", "r"));
        for line in [
            "athena/records: force-push forbidden: in force",
            "athena/records: deletion forbidden: in force",
            "athena/records: up-to-date required: not in force",
            "main: force-push forbidden: unknown",
            "main: deletion forbidden: unknown",
            "main: up-to-date required: unknown",
        ] {
            assert!(report.contains(line), "{line} missing in {report}");
        }
        assert!(report.contains("Only rulesets are seen"));
        assert!(report.contains("classic branch protection is not visible"));
    }
}
