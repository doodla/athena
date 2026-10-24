# Job 20261024-082859-skh5: verify "Add read-only host-rules check for athena/records and main (D111, part"

## Request (verbatim)

> try again on 20261024-033935-c4w4

## Reading (accepted by the user)

Retry of job 20261024-033935-c4w4: add an athena-core module that GETs the rulesets view rules/branches/{branch} for athena/records and main and reports force-push forbidden, deletion forbidden and up-to-date required as in force, not in force, or unknown. Audience: Sri, who must set these host rules before the records cutover. The retry also fixes the earlier findings: the docs state the check sees rulesets only, the CHANGELOG cites §3 and §10, and the gate evidence cites the commands that ran.

Assumptions:
- The typed request means redo the original check job; the introspection's proposed pre-handover evidence gate is a separate pipeline change and is not built here.
- The user's correction stands: use GET repos/{owner}/{repo}/rules/branches/{branch}, not the classic /protection endpoint (403 here).
- Rules: non_fast_forward => force-push forbidden; deletion => deletion forbidden; required_status_checks with parameters.strict_required_status_checks_policy true => up-to-date required.
- Runner errors or unparseable JSON give an 'unknown' state, never 'in force' or 'not in force'.
- The check sees rulesets only; classic branch protection is not visible through this view.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: fixture replies in the rules/branches shape (JSON list of rule objects) give athena/records with non_fast_forward => force-push forbidden in force; deletion rule => deletion forbidden in force; main with required_status_checks whose strict_required_status_checks_policy is true => up-to-date required in force.
2. [behavior] Run `cargo test -p athena-core`: tests assert `[]` reports all rules not in force; strict_required_status_checks_policy false or absent reports up-to-date not in force; a partial list (only non_fast_forward) reports force-push in force and deletion not in force.
3. [behavior] Run `cargo test -p athena-core`: a recording fake Runner test asserts only GET calls to `repos/{owner}/{repo}/rules/branches/athena/records` and `.../rules/branches/main`, with no method override and no field/body arguments, and no /protection call.
4. [behavior] Run `cargo test -p athena-core`: a runner error (403 or 5xx) or unparseable JSON makes the affected rules report unknown, not in force and not not-in-force.
5. [content] The plain-text report names each of the three rules per branch and its state, e.g. 'athena/records: force-push forbidden: in force', so Sri can see which host rule to set; the report or module doc states that only rulesets are seen, not classic branch protection.
6. [content] Module doc comments cite D111 and docs/design/records-history.md §3 and call this part 3 of job 1; the module is registered in crates/athena-core/src/lib.rs; the CHANGELOG [Unreleased] entry cites both §3 and §10.
7. [behavior] Run `cargo clippy --all-targets`, `cargo fmt --check` and `cargo test` from the repo root: all exit 0 for the workspace. The producer's evidence for this criterion cites each command as run with its exit status and output tail.

## Expected deliverables

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-core/src/gh.rs (modify)

## Out of scope

- Changing any host setting or ruleset
- The pre-handover evidence gate in the job pipeline
- Wiring the check into a CLI command or the cutover flow
- Reading classic branch protection

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Module and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based unit tests; cargo test and clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Gate commands run and cited with output.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reported states from fixture replies and the GET-only calls, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus registration and docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite D111 and the design section.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The doc now says rulesets only and the evidence cites commands, fixing the findings.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests with a fake Runner that returns canned rules/branches JSON: a list with non_fast_forward and deletion for athena/records, and one with required_status_checks (strict true) for main. Run: `cargo test -p athena-core host_rules`. Observes: Check results give force-push forbidden and deletion forbidden InForce on athena/records, and up-to-date required InForce on main.
  Fails without the behavior: If the mapping from non_fast_forward, deletion and strict required_status_checks to in-force states were missing, the asserts on the fixture replies would fail.
- Criterion 2: Tests for `[]`, strict false, strict absent, and a list with only non_fast_forward. Run: `cargo test -p athena-core host_rules`. Observes: `[]` gives all three NotInForce. Strict false or absent gives up-to-date NotInForce. The partial list gives force-push InForce and deletion NotInForce.
  Fails without the behavior: If absent rules defaulted to in force, or the strict flag was ignored, these asserts would fail.
- Criterion 3: A recording fake Runner stores every args vector. Run: `cargo test -p athena-core host_rules`. Observes: Exactly two calls: ["api","repos/o/r/rules/branches/athena/records"] and ["api","repos/o/r/rules/branches/main"]. No arg is --method, -f, -F or --input, and none contains "protection".
  Fails without the behavior: If the code used --method, -f, --input or the /protection path, the recorded-args assertion would fail.
- Criterion 4: A fake Runner returns Err(GhFailure with status 403), Err(GhFailure with status 503), and Ok("not json"). One test also covers a failure on one branch only. Run: `cargo test -p athena-core host_rules`. Observes: All three rules on the affected branch are Unknown. The other branch still gets its own result.
  Fails without the behavior: If errors or bad JSON were mapped to NotInForce (or InForce), the Unknown asserts would fail.
- Criterion 5: A test renders the report and checks its text. Run: `cargo test -p athena-core host_rules`. Observes: The report contains lines such as 'athena/records: force-push forbidden: in force', one line for each of the 3 rules on each of the 2 branches (including 'unknown' lines), and a sentence that only rulesets are seen and classic branch protection is not.
  Fails without the behavior: If the report omitted a rule or the rulesets-only note, the string asserts would fail.
- Criterion 6: Read the //! doc in host_rules.rs, `pub mod host_rules;` in lib.rs, and the CHANGELOG [Unreleased] ### Added entry. The test run shows the module compiled in. Run: `cargo test -p athena-core host_rules`. Observes: The doc cites D111 and docs/design/records-history.md §3 and says 'part 3 of job 1'. The CHANGELOG entry cites §3 and §10. lib.rs lists the module, and the host_rules tests run under athena-core.
  Fails without the behavior: If the module were not registered, the tests above would not be compiled or run, so this command would show no host_rules tests.
- Criterion 7: The producer runs `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` from the repo root, and quotes each command with its exit status and last output lines. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All three exit 0 and the evidence cites each command as run.
  Fails without the behavior: A lint, format or test regression anywhere in the workspace would give a non-zero exit.

Planned files:

- crates/athena-core/src/host_rules.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## The change

Base 2005cddeaef4 → candidate 293b7926228c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   9 ++
 crates/athena-core/src/gh.rs         |   6 +
 crates/athena-core/src/host_rules.rs | 244 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs        |   1 +
 docs/design/records-history.md       |   5 +-
 5 files changed, 264 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1038 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
