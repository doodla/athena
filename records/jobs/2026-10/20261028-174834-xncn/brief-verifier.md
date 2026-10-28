# Job 20261028-174834-xncn: verify "Add reading size gate comparing forecast wall time to deadlines"

## Request (verbatim)

> try again on 20261027-054148-g63s

## Reading (accepted by the user)

Retry of job 20261027-054148-g63s (dispatched, never verified, no findings recorded, so nothing to fix but the original work remains undone): add a deterministic size gate in build_reading (crates/athena-core/src/reading.rs) that refuses a reading whose own forecast wall_time_s.p50 exceeds config.budget.producer_timeout_s or verifier_timeout_s, telling the reader to pick one deliverable and name the rest as follow-up requests. Audience: Athena maintainers and the reader loop, which receives a clear retry message.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: build_reading (line ~499) parses ReadingOutput, validates class, capability, producer model, then builds the Forecast and calls reader.validate(); check_reading_acceptance (line ~2231) runs later with no config access. The gate goes in build_reading right after forecast validation, using jc.config.budget.producer_timeout_s and verifier_timeout_s (defined in crates/athena-data/src/config.rs lines ~232-233). Records validation is unaffected: no record or data file is added, so check_record_file and AnyRecord are untouched. The property is measured directly: the reading's own wall_time_s.p50 against the configured timeouts.
- Job 20261027-054148-g63s's earlier reading used the name limits.producer_timeout_s; the actual config path is budget.producer_timeout_s, corrected here. The gate is a threshold on existing config values; no new config keys are added, so athena.toml needs no change (named out of scope).
- Deliverables crates/athena-context/src/outputs.rs (reader output guidance wording, if the retry message needs mirroring) and crates/athena-data/src/config.rs are kept in the allowed scope from the earlier job but are unlikely to change; the work is mostly reading.rs and CHANGELOG.md.
- Root cause named in the change: reading validation never compares a reading's scope with the producer and verifier deadlines, so multi-item sources yield readings no single run can finish.

## Acceptance criteria

1. [content] The change description and a doc comment on the gate in crates/athena-core/src/reading.rs name the root cause: reading validation never compared reading scope with the producer/verifier deadline; the fix sits in build_reading.
2. [behavior] A new unit test in athena-core builds a reading whose forecast wall_time_s.p50 exceeds budget.producer_timeout_s and asserts build_reading returns an error whose message tells the reader to pick one deliverable and name the rest as follow-up requests. Run cargo test -p athena-core.
3. [behavior] A companion test shows a reading with p50 within both deadlines is still accepted, and a test with p50 above only verifier_timeout_s is refused; the oversize tests fail if the gate is removed and use a generic oversize forecast, not one job's data. Run cargo test -p athena-core.
4. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings (only readings forecasting a p50 wall time beyond a configured timeout).  Existing tests and the default athena.toml readings still pass: cargo test --workspace.
5. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass.
6. [content] CHANGELOG.md records the new reading size gate.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Changing athena.toml limits or adding new config keys
- Retry-scope tracking across jobs or re-ask handling
- Splitting documents into sections automatically
- Changing producer or verifier timeouts

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for the gate; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed in reading validation where the missing comparison originates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check a refusal and an acceptance happening, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The size rule is enforced by code in build_reading, not prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG entry written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by reading the doc comment on `check_reading_size` in reading.rs and the new CHANGELOG entry (the change description). Grep, then Read the doc comment. Run: `grep -n "never compared\|reading scope" crates/athena-core/src/reading.rs CHANGELOG.md`. Observes: The doc comment and the CHANGELOG entry both state the root cause: reading validation never compared reading scope with the producer and verifier deadline. They also state that the fix sits in `build_reading`.
- Criterion 2: Unit test in the reading.rs `mod tests`. Entry point: private `Athena::build_reading` (same module tree, so callable). Preconditions: class "docs", capability "study" in the capabilities slice, producer model "claude-haiku-5-5" and a README.md deliverable, all from the existing fake reader JSON. The JSON is a generic valid reading whose forecast has wall_time_s p10 100, p50 5000, p90 9000. Seam: `crate::test_support::make_athena(&Git::at(tmp), tmp_state, vec![], script)` already exists, and the script is never invoked. `JobConfig` is built directly (its fields are pub) from `Config::parse(include_str!("../../../athena.toml"))` and `DefaultsModel::parse(include_str!("../../../defaults.toml"))`. The test sets `jc.config.budget.producer_timeout_s = 3000` and `verifier_timeout_s = 10_000`, so only the producer deadline is exceeded. `Records::default()` and `Redactor::new(&athena_gate::builtin_config())` are inputs. Run: `cargo test -p athena-core --lib a_reading_forecasting_more_than_the_producer_timeout_is_refused`. Observes: `build_reading` returns Err. Its text contains "pick one deliverable", "follow-up request" and "producer timeout".
  Fails without the behavior: Without the gate, `build_reading` returns Ok and the test's `unwrap_err()` panics. With the gate removed, the test also fails on the missing message text.
- Criterion 3: Two more unit tests in the same module, using the same helper and entry point `build_reading`. The oversize forecasts are generic (p50 2700 for the verifier-only case). The verifier-only test sets producer_timeout_s 3000 and verifier_timeout_s 2400, so p50 2700 is above only the verifier timeout. The within-deadline test uses p50 30 and a boundary case p50 exactly 2400, which is accepted. A third short check (optional, in the accepted test): the same oversize forecast with capability null and a decline_reason is accepted, because declines are not gated. Run: `cargo test -p athena-core --lib a_reading_forecasting_more_than_only_the_verifier_timeout_is_refused a_reading_within_both_timeouts_is_accepted`. Observes: The verifier-only case is Err whose message names the verifier timeout and has the "pick one deliverable" advice. The p50 30 and p50 == 2400 cases return Ok with `forecasts.reader.wall_time_s.p50()` equal to the input.
  Fails without the behavior: Without the gate, the verifier-only test finds Ok instead of Err and fails. The accepted test passes in both worlds, so it guards against an over-eager gate: a gate that refuses everything fails it. Together they pin the threshold. Run each name as a separate `cargo test` filter or use `cargo test -p athena-core --lib reading::tests::a_reading_`.
- Criterion 4: Run the whole workspace suite. Existing integration tests in crates/athena-core (run/mod.rs, run_pin_tests.rs, scope_guard.rs) and crates/athena-cli/tests use fake readers with wall_time_s p50 5 or 30 under the default athena.toml timeouts 3000 and 2400. The 3% figure is an estimate, not measured, so the CHANGELOG entry labels it as such: only readings forecasting a p50 beyond a timeout are newly refused. Run: `cargo test --workspace`. Observes: All existing tests pass, so default readings are still accepted. The only new refusals are readings whose own p50 exceeds a configured timeout.
  Fails without the behavior: If the gate wrongly refused the shipped fake readings (p50 30 or 5 against timeouts 3000 and 2400), the existing run and lifecycle tests that go through `ask` and `go` would fail. Without the gate, the new oversize tests fail.
- Criterion 5: Run the three checks Athena runs. The criterion also names `cargo clippy --workspace --all-targets`, which the verifier may run as well. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: Exit code 0 for each command.
  Fails without the behavior: A formatting, lint or test regression would fail the corresponding command.
- Criterion 6: Read CHANGELOG.md under `## [Unreleased]` / `### Added` for the new entry. It is written for an outside reader, names the root cause and the new refusal message, and says that the refusal rate rises as intended (estimate about 3% of readings, unmeasured). Run: `grep -n -i "size gate" CHANGELOG.md`. Observes: One new bullet, as the first or latest Added item, describing the reading size gate.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base a3bbddd0da41 → candidate f155e3501393. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   9 +++
 crates/athena-core/src/reading.rs | 139 ++++++++++++++++++++++++++++++++++++++
 2 files changed, 148 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1591 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
