# Job 20261024-033935-c4w4: verify "Add read-only host check for records-history D111 rules"

## Request (verbatim)

> Add a read-only check, part 3 of job 1 of the records-history plan (docs/design/records-history.md section 3, decision D111): it asks the host whether force-pushes and deletion are forbidden on the branch athena/records and whether main requires branches to be up to date before merging, and reports each as in force or not, with tests against fixture replies. It changes no host setting.

## Reading (accepted by the user)

Add a new athena-core module (part 3 of job 1, D111, design §3) that uses the gh Runner to GET the rulesets view `repos/{owner}/{repo}/rules/branches/{branch}` for athena/records and main, and reports three rules as in force or not. Audience: Sri, who must set these host rules before the records cutover. Tests use fixture replies; nothing on the host is changed.

Assumptions:
- Implemented as a library function in athena-core using the existing gh::Runner trait; no new CLI command is wired unless trivial (not required).
- Repo owner/name is passed in by the caller, as with existing gh helpers.
- JSON parsing uses serde_json already in the workspace; no new crates.
- Rule types per the user: non_fast_forward, deletion, required_status_checks with parameters.strict_required_status_checks_policy true.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: tests feed fixture replies in the rules/branches shape (JSON list of rule objects) and assert: athena/records with a non_fast_forward rule => force-push forbidden in force; a deletion rule => deletion forbidden in force; main with a required_status_checks rule whose parameters.strict_required_status_checks_policy is true => up-to-date required in force.
2. [behavior] Tests assert an empty list `[]` reports all rules as not in force, and that a required_status_checks rule with strict_required_status_checks_policy false (or absent) reports up-to-date as not in force; a partial list (only non_fast_forward) reports force-push in force and deletion not in force.
3. [behavior] A recording fake Runner test asserts the check issues only GET calls to `repos/{owner}/{repo}/rules/branches/athena/records` and `.../rules/branches/main` (not the classic /protection endpoint), with no method override and no field/body arguments, so no host setting changes.
4. [behavior] A test where the runner returns an error (e.g. 403 or 5xx) or unparseable JSON asserts the affected rules are reported as unknown/error, not as 'not in force' and not as in force.
5. [content] The report names each of the three rules and its state in plain text, e.g. 'athena/records: force-push forbidden: in force', so Sri can see which host rule to set.
6. [content] Doc comments cite D111 and docs/design/records-history.md §3 and describe this as part 3 of job 1; the module is registered in athena-core lib.rs.
7. [behavior] Run `cargo clippy --all-targets`, `cargo fmt --check` and `cargo test`: all clean and passing for the workspace.

## Expected deliverables

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Changing any host setting, ruleset, or branch protection
- Wiring the check into a gate, CLI command, or cutover flow
- Other parts/jobs of the records-history plan

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based unit tests; cargo test and clippy clean.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses existing gh Runner and serde_json.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus its registration.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reported states from fixtures and GET-only calls.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite D111 and design §3 for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria runnable via cargo test/clippy/fmt.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests feed fixture JSON lists to a fake Runner and assert the reported states. Run: `cargo test -p athena-core host_rules`. Observes: athena/records with non_fast_forward and deletion rules reports both InForce. main with a required_status_checks rule whose strict policy is true reports up-to-date InForce.
  Fails without the behavior: If the parser ignored non_fast_forward, deletion or strict required_status_checks, the in-force assertions would fail.
- Criterion 2: Fixture tests for `[]`, for strict false, for strict absent, and for a list holding only non_fast_forward. Run: `cargo test -p athena-core host_rules`. Observes: `[]` gives all three NotInForce. Strict false or absent gives up-to-date NotInForce. The partial list gives force-push InForce and deletion NotInForce.
  Fails without the behavior: A parser that treats any required_status_checks rule as strict, or that defaults to in force, would fail these tests.
- Criterion 3: A recording fake Runner captures every args vector. Run: `cargo test -p athena-core host_rules`. Observes: Exactly two calls, equal to ["api","repos/o/r/rules/branches/athena/records"] and ["api","repos/o/r/rules/branches/main"]. No arg is --method, -f, -F, --input or protection.
  Fails without the behavior: If the check used a --method override, -f/-F fields, or the /protection endpoint, the exact-args assertion would fail.
- Criterion 4: Fake Runner returns a GhFailure with status 403, one with status 503, and a reply of `not json`. A plain object reply is also tested. Run: `cargo test -p athena-core host_rules`. Observes: The affected rules are Unknown with a reason. The other branch's rules are still reported when its call succeeds.
  Fails without the behavior: If an error were mapped to NotInForce or InForce, the Unknown assertions would fail.
- Criterion 5: Test the report's Display or render output. Run: `cargo test -p athena-core host_rules`. Observes: The text contains "athena/records: force-push forbidden: in force", "athena/records: deletion forbidden: not in force" and "main: up to date before merging required: unknown (...)", for example, one line per rule.
  Fails without the behavior: If the rendering omitted a rule name or its state, the string assertions would fail.
- Criterion 6: The module compiles only if it is registered in lib.rs. The doc comments are reviewed by reading `git diff`. Run: `cargo build`. Observes: `pub mod host_rules;` appears in lib.rs in alphabetical position, and the module docs cite D111, docs/design/records-history.md §3 and 'part 3 of job 1'.
- Criterion 7: Run the three workspace checks: `cargo fmt --all --check`, the clippy command above, and `cargo test --locked`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All three exit 0.
  Fails without the behavior: A lint, a format error or a failing test would fail the matching command.

Planned files:

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## The change

Base 73daedd4bb3e → candidate d6bb6225520b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   6 +
 crates/athena-core/src/gh.rs         |   3 +
 crates/athena-core/src/host_rules.rs | 302 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs        |   3 +
 docs/design/records-history.md       |   5 +-
 5 files changed, 318 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1041 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
