//! `athena status` prints the `role need` line only when a role is a retire candidate. No model is called.

mod common;

use athena_core::role_need::{ClassOutcomes, RoleNeed, RoleUse};
use common::*;

fn need(candidates: &[&str]) -> RoleNeed {
    let roles = ["reader", "steward", "planner", "critic"]
        .iter()
        .map(|r| RoleUse {
            role: r.to_string(),
            runs: usize::from(!candidates.contains(r)),
            retire_candidate: candidates.contains(r),
        })
        .collect();
    RoleNeed {
        window_jobs: 8,
        roles,
        classes: vec![ClassOutcomes {
            class: "code".into(),
            closed: 8,
            failed: 3,
        }],
    }
}

#[test]
fn status_prints_the_role_need_line_only_with_a_candidate() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let mut view = athena.status().unwrap();
    let find = |text: &str| {
        text.lines()
            .find(|l| l.starts_with("role need"))
            .map(str::to_string)
    };

    // No records: no candidate, so no line.
    assert_eq!(view.role_need, None);
    assert_eq!(find(&athena_cli::display::status(&view)), None);

    // A measure with no candidate prints nothing.
    view.role_need = Some(need(&[]));
    assert_eq!(find(&athena_cli::display::status(&view)), None);

    // A candidate prints the line, naming it and the class outcomes, and not the active roles.
    view.role_need = Some(need(&["planner", "critic"]));
    let line = find(&athena_cli::display::status(&view)).expect("a role need line");
    assert!(
        line.contains("retire candidates: planner, critic"),
        "{line}"
    );
    assert!(!line.contains("steward"), "{line}");
    assert!(line.contains("code 3/8 failed"), "{line}");
}
