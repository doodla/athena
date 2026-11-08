# Job 20261108-041939-fqaz: verify "Run cargo fmt on the producer's work tree before checks and verifier"

## Request (verbatim)

> try again on 20261108-023649-67f6

## Reading (accepted by the user)

Follow-up to rejected job 20261108-023649-67f6: its skill work is dropped and not retried here. This job does the fix its introspection (20261108-041919-1x5m) proposed: after the producer stage, Athena runs `cargo fmt` on the work tree and then `cargo fmt --check`, so a formatting-only defect no longer rejects a job. Audience: Sri and the verifier judging whether jobs are rejected for work rather than layout.

Assumptions:
- Code survey: read crates/athena-core/src/run/producer.rs (producer_snapshot creates the work tree; producer_loop runs the producer, then send-backs for criteria, scope, mirrors and decisions), crates/athena-core/src/run/mod.rs (produce_and_verify at about line 1737; checks run through routing::run_routed_with_record and SnapExec near line 2021, on the candidate), crates/athena-core/src/checks.rs (run_check) and crates/athena-agent/src/policy.rs (denies cargo fmt without --check to agent roles). The format step belongs at the end of producer_loop, before the change list is taken. No record or data file under crates/ is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly: the fmt check on the formatted tree.
- Job 20261108-023649-67f6 addressed: its skill work (skills/diagnose-failure.md, introspect.rs and briefs.rs changes, its introspection test) is dropped, not kept. Its findings are not fixed here. The fmt finding is fixed at the root by this mechanism. The skill-path and PromptAsset-versioning findings do not apply because the skill is not re-added. A separate request can redo the skill.
- Where a candidate is tree-seeded from an earlier rejected one, the formatter runs on it too, since the work tree is formatted after each producer stage.
- Unfixable fmt errors (a parse error) are left in the tree; the normal fmt check fails and reports them.
- The base and any pre-existing unformatted files can be reformatted by `cargo fmt`; accepted because the repository is expected to be fmt-clean at base.
- Governance: athena.toml, defaults.toml, workflows, rust-toolchain.toml and rustfmt config are not changed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core builds a work tree whose Rust file is mis-formatted (e.g. a long constant on one line), runs the new post-producer format step, and asserts the file is rewritten and `cargo fmt --check` then passes. Run `cargo test -p athena-core` and it passes.
2. [behavior] A new test shows a fmt failure the formatter cannot fix (e.g. a file that does not parse) still yields a failed fmt result whose output text appears in the failed-check output. Run `cargo test -p athena-core`.
3. [content] Grep of crates/athena-core/src/run shows the format step is called after the producer loop and before the checks and verifier run, so the check and the verifier see the formatted tree.
4. [behavior] A test asserts the recorded candidate diff (change list) includes the formatter's changes, not only the producer's own edits. Run `cargo test -p athena-core`.
5. [content] The format step runs only for a work tree with a Cargo.toml and is Athena code, not an agent command, so the policy in crates/athena-agent/src/policy.rs that denies `cargo fmt` without --check to agent roles is unchanged.
6. [content] CHANGELOG.md has an Unreleased entry stating the outcome: a job whose only defect is rustfmt layout reaches the verifier formatted.
7. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean on the workspace.
8. [content] Job 20261108-023649-67f6's candidate files are not present: crates/athena-context/skills/diagnose-failure.md does not exist, and crates/athena-core/src/introspect.rs and crates/athena-cli/tests/introspection.rs are unchanged. No secrets or personal data are added.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/format.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Job 20261108-023649-67f6 files, dropped: crates/athena-context/skills/diagnose-failure.md, crates/athena-core/src/introspect.rs, crates/athena-cli/tests/introspection.rs, and its CHANGELOG entry
- Adding the diagnose-failure skill or any other skill
- Changing athena.toml, defaults.toml, workflows or rustfmt/toolchain configuration
- Auto-fixing clippy or test failures
- Letting agent roles run cargo fmt without --check
- Relaxing the fmt gate

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the format step; cargo test, clippy and fmt --check are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Formatting is fixed by a mechanism in the pipeline, not by reformatting one file.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code step enforces it, not a prompt rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check that a mis-formatted tree is reformatted and that an unfixable error still fails.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused pipeline step.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `a_misformatted_rust_file_is_rewritten_and_cargo_fmt_check_then_passes` in run/tests.rs. `temp_crate` writes Cargo.toml, the pinned rust-toolchain.toml and a src/lib.rs with a long `pub const` array on one line and `fn f( )->u32{1}`. The CheckContext takes its env from the process (PATH, HOME, RUSTUP_HOME, CARGO_HOME, RUSTUP_TOOLCHAIN when set), `Registry::new(tempdir)` and `Redactor::new(&athena_gate::builtin_config())`. Run `run_check` with `cargo fmt --all --check` and assert it fails; call `format_work_tree(dir, &[entry('M',"src/lib.rs")], &ctx)`; assert `Done`, that the text changed to rustfmt layout, and that the `--check` through `run_check` now passes. Run: `cargo test -p athena-core`. Observes: The file bytes change to the formatted layout, the outcome is Done, and the fmt check goes from failed to passed. The whole athena-core suite passes.
  Fails without the behavior: Without the formatter run the file stays mis-formatted, so the content, Done and post-`--check` assertions fail.
  Test seam: Production entry point `run::format::format_work_tree` (added by plan step 1), called with a real `CheckContext` over a tempdir and real `cargo fmt` resolved by the written rust-toolchain.toml. Guards: Cargo.toml present and a non-deleted `.rs` entry. No double. `checks::run_check` exists (checks.rs:513). The test and the `temp_crate`, `entry` and context helpers are added by step 5.
- Criterion 2: Unit test `an_unfixable_fmt_error_is_reported_and_the_fmt_check_fails_with_the_same_text`. The temp crate's src/lib.rs is `pub fn broken( {`. `format_work_tree` with `entry('M',"src/lib.rs")` must return `Unfixed{output, ..}`. Take the first line of `output` that starts with `error` (the `--check` run's own capture) and assert it is non-empty. Then `run_check` with the configured spec `cargo fmt --all --check` on the same dir must give `!passed` and its `output_tail` must contain that exact line. The broken file must be byte-identical afterwards. Run: `cargo test -p athena-core -- an_unfixable_fmt_error_is_reported_and_the_fmt_check_fails_with_the_same_text`. Observes: A failed fmt result whose rustfmt error text appears in the failed check's output tail; the tree is unchanged and the gate is not relaxed.
  Fails without the behavior: If the step swallowed the failure and returned Done, the Unfixed assertion fails. If it rewrote or deleted the broken file, the byte-identity assertion fails. If the check's output tail did not carry the line, the contains assertion fails.
  Test seam: `format_work_tree` (step 1) plus `checks::run_check` (exists), real cargo and rustfmt in a tempdir with the pinned toolchain file. No double. Test added by step 5.
- Criterion 3: Three parts, each its own run. (a) Content: Grep crates/athena-core/src/run for `format_work_tree` and `change_list`. In producer.rs the call sits in each loop pass after the `raw` report parse (producer.rs:325-335) and before the first `snapshot.change_list()` (line 339), `loop_replay` (line 370) and the `break`; this is the reading's 'end of producer_loop, before the change list is taken', and the module docs of format.rs and the producer_loop doc comment quote that sentence and say the call is the last producer-stage act of every pass, with the loop exit, candidate, checks and verifier after it. mod.rs adds only `mod format;`; `producer_loop` returns inside `produce_and_verify` before `candidate_stage` and `check_and_verify`. (b) Behavior, formatted path: end-to-end test `a_job_whose_only_defect_is_rustfmt_layout_passes_a_cargo_fmt_check_and_the_candidate_holds_the_formatted_file` uses the real `cargo fmt --all --check` as the capability check on the verifier's snapshot; the base commits Cargo.toml, the pinned rust-toolchain.toml and a clean src/lib.rs. (c) Behavior, stop path through the pipeline: end-to-end test `a_job_stopped_during_the_producer_turn_closes_at_the_format_step_and_runs_no_check_or_verifier`. The base commits Cargo.toml and a clean src/lib.rs. A script wrapper over `counting(counts, None)` makes the producer turn edit src/lib.rs (non-deleted `.rs`, so the step is not Skipped) and also `create_dir_all` + write the STOP file at `StateDir::new(state.path().join("repo")).registry().stop_file()` (the path `athena_with` uses, tests.rs:194-196). After `go` assert: state Failed; reason contains `the format step stopped`; the `Lines` capture holds a line starting `failed     the format step stopped`; producer count 1, verifier count 0; the stored job record has no candidate and no recorded check results. Unit test `a_stopped_format_step_returns_stopped_and_leaves_the_file_untouched` covers `format_work_tree` with the STOP file under `Registry::new(tempdir)`. Part (a) uses the Grep tool. Run: `cargo test -p athena-core -- a_job_whose_only_defect_is_rustfmt_layout_passes_a_cargo_fmt_check_and_the_candidate_holds_the_formatted_file a_job_stopped_during_the_producer_turn_closes_at_the_format_step_and_runs_no_check_or_verifier a_stopped_format_step_returns_stopped_and_leaves_the_file_untouched`. Observes: (b) The job reaches Landed and the real fmt check passed, so the check and verifier saw the formatted tree. (c) The job closes Failed with the format-step reason, the verifier never runs, no candidate is recorded. The unit case returns Stopped with the file byte-identical.
  Fails without the behavior: With the `format_work_tree` call removed from producer_loop the candidate holds the mis-formatted file, the fmt check fails and the job is Rejected, not Landed. With the Stopped arm removed or not returning `Flow::Stop`, test (c) fails: the loop would go on to loop_replay, the checks and the verifier (verifier count 1, a candidate recorded, or a state other than Failed).
  Test seam: Production entry point `Athena::go` on the fake driver, in `produce_and_verify_order_tests`. `athena_with(repo, state, checks, plenty_of_space, script)` (tests.rs:187) wires `FakeAgentDriver::new(script).with_registry(state.registry())`; the driver checks the switch only before the script runs (fake.rs:92-97), so the STOP file written during the producer turn is first seen by the format step. `base_script()` (tests.rs:219), `counting`/`Counts` (tests.rs:2117-2184), `commit_file` (tests.rs:341), `Lines` (tests.rs:167) and `Registry::stop_file()` (procs.rs:364) exist. Steps 5-7 add the `FMT_CHECK` const, the script wrappers and the tests. Real cargo fmt runs in (b) only. The double (the script) only writes files; the formatter and the Stopped arm are production code.
- Criterion 4: The same end-to-end formatted-path test, after `go`: load `Records::load(&athena.git, &athena.head())`, take `jobs[id].candidate`, assert `candidate.changes` lists README.md and src/lib.rs and nothing else. The producer's src/lib.rs is the base function plus a new mis-formatted `pub fn two( )->u32{2}`, so the formatted file still differs from the base. Assert `athena.git.show_file(candidate_commit, "src/lib.rs")` equals `pub fn one() -> u32 {\n    1\n}\n\npub fn two() -> u32 {\n    2\n}\n` and differs from the producer's text. Assert the `Lines` capture holds a line starting `format`. The pure test `paths_added_by_format_names_only_new_paths` checks the added-paths helper. If `cargo fmt` writes a Cargo.lock the base commits one. Run: `cargo test -p athena-core -- a_job_whose_only_defect_is_rustfmt_layout_passes_a_cargo_fmt_check_and_the_candidate_holds_the_formatted_file paths_added_by_format_names_only_new_paths`. Observes: The recorded candidate commit holds the formatter's bytes, so the change list and diff are of the formatted tree.
  Fails without the behavior: If the formatter ran on a copy, or after the change list was taken, `show_file` returns the producer's mis-formatted text and the equality assertion fails.
  Test seam: `Athena::go` with the fake driver, as in criterion 3. `build_candidate` (candidate.rs:34) reads `snapshot.change_list()` after the loop (exists). The test reads the stored candidate record and `Git::show_file` (used near tests.rs 4718-4726; exists). Additions: the script edits and the pure helper (steps 1, 5 and 6).
- Criterion 5: Three separate runs. (1) Unit test `a_work_tree_without_cargo_toml_or_a_changed_rust_file_is_not_formatted` writes a mis-formatted src/lib.rs with no Cargo.toml and calls `format_work_tree` with `entry('M',"src/lib.rs")`; then adds Cargo.toml and calls it with only `entry('M',"README.md")`, and with only `entry('D',"src/lib.rs")`. All three return `Skipped(..)` and leave the file byte-identical. (2) `git diff --stat HEAD~1` in the verifier's snapshot (base then candidate): the stat must not list crates/athena-agent/src/policy.rs, and must list the five deliverables. (3) The unchanged policy tests (near policy.rs 1231-1235) pass. The Grep tool shows format.rs spawns only through `run_check` from host code with no agent grant. Run: `cargo test -p athena-core -- a_work_tree_without_cargo_toml_or_a_changed_rust_file_is_not_formatted
git diff --stat HEAD~1
cargo test -p athena-agent`. Observes: Three Skipped outcomes with unchanged bytes; a candidate diff stat that does not contain policy.rs; the agent policy tests for `cargo fmt` green.
  Fails without the behavior: If the Cargo.toml or the non-deleted-`.rs` guard were missing, the mis-formatted file would be rewritten and the byte-identity assertions fail. If policy.rs were edited, `git diff --stat HEAD~1` would list it.
  Test seam: `format_work_tree` (step 1), real tempdir, no double. Test added by step 5. The policy tests exist. `git diff --stat HEAD~1` compares candidate with base in the verifier's two-commit snapshot (verification.rs:444-469).
- Criterion 6: Read the CHANGELOG diff against the base commit. Under `## [Unreleased]` one bullet states that a job whose only defect is rustfmt layout now reaches the verifier formatted, because Athena runs `cargo fmt` then `cargo fmt --check` on the producer's work tree after each producer pass. It names the root cause and says an unfixable fmt error still fails the fmt check. The diff must touch only the Unreleased section. Run: `git diff HEAD~1 -- CHANGELOG.md`. Observes: The new bullet sits in the Unreleased section and the diff touches only that section. Without the change this command prints an empty diff.
  Fails without the behavior: With no CHANGELOG edit the diff is empty and no Unreleased bullet is shown.
- Criterion 7: The producer formats its own new code with `cargo fmt --all` (step 8) before finishing. The verifier then runs the three workspace commands separately, each a whole simple command; the formatting check is the non-writing `--check` form, so no verifier command rewrites a file. Existing fake-job tests have no Cargo.toml, so the step returns Skipped and prints nothing; the slot test at tests.rs:4185 edits README.md only, so it is Skipped too. Run: `cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --all --check`. Observes: All three exit 0. The new tests pass; clippy reports no warnings; `cargo fmt --all --check` reports no diff.
  Fails without the behavior: A failing new test, a clippy warning or an unformatted new file makes the matching command exit non-zero.
  Test seam: The workspace's own checks; no double. Real cargo fmt in the new tests resolves the pinned toolchain from the written rust-toolchain.toml.
- Criterion 8: Run `git diff --stat HEAD~1` in the verifier's snapshot: it must list exactly the five deliverables (producer.rs, mod.rs, format.rs, tests.rs, CHANGELOG.md) and no introspect.rs, introspection.rs, policy.rs, or any path under crates/athena-context/. Run `ls crates/athena-context`: the listing has no `skills` entry (the directory is absent at base and expected to stay absent, so diagnose-failure.md does not exist). The Grep tool checks the added text for secrets and personal data. Bare `git status` is not used. Run: `git diff --stat HEAD~1
ls crates/athena-context`. Observes: The stat names only the five deliverables; the crates/athena-context listing shows no skills directory; introspect.rs and introspection.rs are not in the stat.
  Fails without the behavior: If the dropped job's files were present, the stat would list diagnose-failure.md, introspect.rs or introspection.rs, and `ls` would show `skills`.
  Test seam: `git diff --stat HEAD~1` works on the verifier's base-then-candidate snapshot (verification.rs:444-469; verifier.md:3). No double.

Planned files:

- crates/athena-core/src/run/format.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7cdae23ab37a → candidate cfeefa731a1d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  13 ++
 crates/athena-core/src/run/format.rs   | 133 +++++++++++++
 crates/athena-core/src/run/mod.rs      |   1 +
 crates/athena-core/src/run/producer.rs |  74 +++++++
 crates/athena-core/src/run/tests.rs    | 353 +++++++++++++++++++++++++++++++++
 5 files changed, 574 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-041939-fqaz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-041939-fqaz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42450514248 for cfeefa731a1d989878d6ae3f6110aad4fd1c80e1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42450514248 for cfeefa731a1d989878d6ae3f6110aad4fd1c80e1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42450514248 for cfeefa731a1d989878d6ae3f6110aad4fd1c80e1)
- ci-result: `not a check` passed (Check results for cfeefa731a1d989878d6ae3f6110aad4fd1c80e1 come from the CI run https://github.com/doodla/athena/actions/runs/42450514248 (649s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean on the workspace.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/format.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:463:20
    |
463 |         let done = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                    ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:483:22
    |
483 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:513:22
    |
513 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:526:17
    |
526 |                 format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:543:22
    |
543 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:562:13
    |
562 |             format::paths_added_by_format(&before, &after),
    |             ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:565:17
    |
565 |         assert!(format::paths_added_by_format(&before, &before).is_empty());
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -- an_unfixable_fmt_error_is_reported_and_the_fmt_check_fails_with_the_same_text`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/format.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:463:20
    |
463 |         let done = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                    ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:483:22
    |
483 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:513:22
    |
513 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:526:17
    |
526 |                 format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:543:22
    |
543 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:562:13
    |
562 |             format::paths_added_by_format(&before, &after),
    |             ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:565:17
    |
565 |         assert!(format::paths_added_by_format(&before, &before).is_empty());
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -- a_job_whose_only_defect_is_rustfmt_layout_passes_a_cargo_fmt_check_and_the_candidate_holds_the_formatted_file a_job_stopped_during_the_producer_turn_closes_at_the_format_step_and_runs_no_check_or_verifier a_stopped_format_step_returns_stopped_and_leaves_the_file_untouched`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/format.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:463:20
    |
463 |         let done = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                    ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:483:22
    |
483 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:513:22
    |
513 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:526:17
    |
526 |                 format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:543:22
    |
543 |         let result = format::format_work_tree(dir.path(), &changes, &unit.ctx(dir.path())).unwrap();
    |                      ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:562:13
    |
562 |             format::paths_added_by_format(&before, &after),
    |             ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `format` in this scope
   --> crates/athena-core/src/run/tests.rs:565:17
    |
565 |         assert!(format::paths_added_by_format(&before, &before).is_empty());
    |                 ^^^^^^ use of unresolved module or unlinked crate `format`
    |
    = help: if you wanted to use a crate named `format`, use `cargo add format` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core -- a_job_whose_only_defect_is_rustfmt_layout_passes_a_cargo_fmt_check_and_the_candidate_holds_the_formatted_file paths_added_by_format_names_only_new_paths`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core -- a_work_tree_without_cargo_toml_or_a_changed_rust_file_is_not_formatted git diff --stat HEAD~1 cargo test -p athena-agent`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff HEAD~1 -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked cargo clippy --all-targets --locked -- -D warnings cargo fmt --all --check`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`git diff --stat HEAD~1 ls crates/athena-context`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
