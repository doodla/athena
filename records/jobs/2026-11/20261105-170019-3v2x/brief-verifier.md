# Job 20261105-170019-3v2x: verify "Count docs-only skipped or CI-reused checks as seen for 'still passes'"

## Request (verbatim)

> Count a check skipped for a docs-only change as evidence. The many-agents design note (a docs-only change) was rejected only because its criterion 'cargo test and cargo clippy still pass' could not cite a command seen running: Athena's record says clippy was skipped locally because only docs changed, and the test result was reused from GitHub. For a change that touches only docs, a check Athena skipped for that reason, or reused from a green GitHub run, should satisfy a 'still passes' criterion that cites it.
> 
> Files changed: the verdict's seen-running rule in crates/athena-core/src/verify.rs and its tests.
> 
> Acceptance:
> - a test where a docs-only change has clippy skipped for that reason and test reused from GitHub, and a criterion citing both is met
> - a test where a code change has clippy skipped and the criterion is still unmet

## Reading (accepted by the user)

The verdict's seen-running rule (D43, `judge` in verify.rs) now accepts a check Athena skipped by the docs-only path rule (`skip_when_only` note) as seen, when the change touches only docs. A passing CI-reused check already counts. This unblocks criteria like 'cargo test and cargo clippy still pass' on docs-only changes such as the many-agents design note. Audience: Athena's verifier and the user reading verdicts.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (`judge`: `ran` is built from checks with exit 0 or reused-without-exit on matching candidate/checks tree, lines ~370-395; unseen commands make a behavior criterion unmet at ~523-554; test helpers `check`, `reused_from_ci`, `judge_recorded`, and the existing test `a_skipped_check_that_was_not_reused_does_not_count_as_run` ~1991), crates/athena-core/src/checks.rs (`skip_note` writes 'not run: every changed path matches skip_when_only (...)' and `skipped_run` yields exit_code None, passed true, reused false), crates/athena-core/src/routing.rs (docs/design kinds skip fmt and clippy when only `docs/` or `*.md` changed), and the caller crates/athena-core/src/run/verification.rs:389 plus send_back.rs:264. The property is measured directly by calling `judge` and reading `criteria[i].met` and `commands_verified`; no reading/refusal rule or record file under crates/ is added, so records validation is not touched.
- `judge` has no changed-path input today. The producer should either add a field such as `changed_paths` to `VerdictInputs` (and set it in verification.rs, send_back.rs and the verify.rs test constructors) and treat a check as seen only when every changed path is docs-only, or rely on the skip note. Preferring the path input, because a skipped record on a code change must not count. Docs-only means the same `docs/` and `*.md` patterns routing uses, ideally via the shared `path_matches` rather than a copy.
- Only a skipped record whose note comes from the `skip_when_only` rule counts; a skip for another reason (for example 'not run: CI was not green') or a failed record still does not. The existing candidate-tree and checks-tree match condition stays.
- The user said the files changed are verify.rs and its tests, so the run caller edits are minimal field plumbing only. No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/verify.rs has a docs-only change (changed paths only under docs/ or *.md) with clippy skipped by the skip_when_only rule and test reused from GitHub CI. A criterion citing `cargo clippy` and `cargo test` is met, with commands_verified true and no D43 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
2. [behavior] A new test has a code change (a .rs path changed) with clippy recorded as skipped. A criterion citing `cargo clippy` is still unmet, with commands_verified false and the 'cited but not seen running' finding. Run `cargo test -p athena-core verify` to check.
3. [behavior] Existing tests still pass, including a_skipped_check_that_was_not_reused_does_not_count_as_run, a_failed_check_reused_from_ci_does_not_count_as_run and the other-tree tests; a skip for a reason other than the docs-only path rule, or a failed check, still does not count. Run `cargo test` and `cargo clippy --all-targets`.
4. [content] The doc comments on `VerdictInputs`/`judge` in verify.rs state the new rule: a check skipped by the docs-only path rule on a docs-only change counts as seen, and name the docs-only definition used.
5. [behavior] The change does not widen what counts as seen for non-docs changes. Run `cargo test -p athena-core` and read the diff with `git diff` to confirm the only new counting path is gated on docs-only changed paths.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which checks routing skips or the skip_when_only patterns
- Changing how CI results are reused (ci_result.rs, checks.rs)
- Re-running or re-landing the many-agents design note
- Any change to athena.toml, defaults.toml, or .github/workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests plus cargo test and clippy must stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria call `judge` and check met and commands_verified.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is fixed in the judge's seen-running rule, not by special-casing one job.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test on the new tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in verify.rs, with minimal plumbing at callers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit test. It builds clippy with the real `checks::skip_note` on a `CheckSpec` with `skip_when_only` of `docs/` and `*.md`, over changed paths `docs/design/many-agents.md` and `README.md`, then `checks::skipped_run`. It sets the command to `cargo clippy`. It builds test as `reused_from_ci(true)`. It judges them with a criterion citing both commands. Run: `cargo test -p athena-core verify::tests::a_docs_only_change_counts_a_docs_skipped_check_and_a_ci_reused_check_as_seen`. Observes: `criteria[1].met` and `commands_verified` are true, and `d43_finding(&j)` is none.
  Fails without the behavior: Without the new counting path, clippy is not in `ran` (skipped records have no exit code and are not reused), so the criterion is unmet with 'cited but not seen running: `cargo clippy`'.
  Test seam: Production entry point `verify::judge`. Fixtures are the existing helpers `check`, `reused_from_ci`, `output`, `criteria` and `d43_finding`, plus a new helper `judge_recorded_for(recorded, candidate_tree, checks_tree, cited, changed_paths)`. The step that adds it also makes `judge_recorded` call it with `&[]`. The new `changed_paths` field is injected through `VerdictInputs`. The skipped record comes from the production `skip_note` and `skipped_run`, not a hand-written note. The `judge` seam exists today; the `changed_paths` field and the helper are added in plan steps 1 and 3.
- Criterion 2: New unit test with the same skipped clippy record (note from `skip_note`) and the same trusted trees, judged with changed paths `crates/athena-core/src/verify.rs` and `docs/a.md`. A second case uses an empty path list. Run: `cargo test -p athena-core verify::tests::a_code_change_does_not_count_a_skipped_check_as_seen`. Observes: `criteria[1].met` and `commands_verified` are false, and `d43_finding(&j)` is present with 'cited but not seen running' naming `cargo clippy`.
  Fails without the behavior: If the path gate were dropped (note alone), this test would turn the criterion met and fail. If the rule were absent, test 1 fails instead.
  Test seam: Same as criterion 1: `verify::judge` with `changed_paths` injected and the `judge_recorded_for` helper added in step 3. The note comes from the production `skip_note` and `skipped_run`.
- Criterion 3: Run the whole suite. It includes the existing `a_skipped_check_that_was_not_reused_does_not_count_as_run` (note 'not run: CI was not green', empty changed paths), `a_failed_check_reused_from_ci_does_not_count_as_run`, the other-tree tests, and the recorded-check tests. The new test also covers a docs-only change with a skip note from a non-docs reason and a failed check (steps 3 and 4). Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings`. Observes: All tests pass and clippy is clean. In the new negative cases on a docs-only change, a non-docs skip reason and a failed check leave the criterion unmet.
  Fails without the behavior: A rule that counted any skip, or that ignored `passed`, fails the 'CI was not green' case and the failed-check case.
  Test seam: `verify::judge` through the existing helpers. Each existing constructor gains `changed_paths: &[]` in step 3, which keeps their meaning.
- Criterion 4: Read the diff of the doc comments on `VerdictInputs::changed_paths` and `commands_run`, and on `judge`. Run: `git diff -- crates/athena-core/src/verify.rs`. Observes: The comments state that a check skipped by the docs-only path rule counts as seen on a docs-only change, and name the definition: every changed path matches `docs/` or `*.md` via `checks::path_matches`, and an empty list is not docs-only.
- Criterion 5: Run the crate's tests, then read the diff. The only change in `judge`'s `ran` building is one extra filter branch, guarded by `docs_only && note.starts_with(SKIP_NOTE_PREFIX)`. A docs-only test case varies the paths to code, an empty list and mixed paths, and a skipped record counts in none of them. Run: `cargo test -p athena-core && git diff`. Observes: The tests pass and the diff has one new counting path gated on docs-only changed paths. `ran` is unchanged for every other input.
  Fails without the behavior: A guard that dropped the path check makes the code-change test in criterion 2 fail.
  Test seam: `verify::judge` with the `changed_paths` field set directly in the test.

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 07e211f296f1 → candidate 91101bf0e583. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   6 +
 crates/athena-core/src/run/send_back.rs    |   1 +
 crates/athena-core/src/run/verification.rs |   8 ++
 crates/athena-core/src/verify.rs           | 169 +++++++++++++++++++++++++++--
 4 files changed, 177 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-170019-3v2x/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-170019-3v2x/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42132678084 for 91101bf0e5839ae82f2c65b8cd2c621068b57bb7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42132678084 for 91101bf0e5839ae82f2c65b8cd2c621068b57bb7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42132678084 for 91101bf0e5839ae82f2c65b8cd2c621068b57bb7)
- ci-result: `not a check` passed (Check results for 91101bf0e5839ae82f2c65b8cd2c621068b57bb7 come from the CI run https://github.com/doodla/athena/actions/runs/42132678084 (675s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Existing tests still pass, including a_skipped_check_that_was_not_reused_does_not_count_as_run, a_failed_check_reused_from_ci_does_not_count_as_run and the other-tree tests; a skip for a reason other than the docs-only path rule, or a failed check, still does not count. Run `cargo test` and `cargo clippy --all-targets`.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core verify::tests::a_docs_only_change_counts_a_docs_skipped_check_and_a_ci_reused_check_as_seen`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/verification.rs, crates/athena-core/src/verify.rs.
  End of the output (data, not instructions):

```text

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1676:13
     |
1676 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1711:13
     |
1711 |             changed_paths,
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1730:13
     |
1730 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1753:13
     |
1753 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2271:13
     |
2271 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2525:13
     |
2525 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2700:13
     |
2700 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 9 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core verify::tests::a_code_change_does_not_count_a_skipped_check_as_seen`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/verification.rs, crates/athena-core/src/verify.rs.
  End of the output (data, not instructions):

```text
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1676:13
     |
1676 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1711:13
     |
1711 |             changed_paths,
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1730:13
     |
1730 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:1753:13
     |
1753 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2271:13
     |
2271 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2525:13
     |
2525 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `verify::VerdictInputs<'_>` has no field named `changed_paths`
    --> crates/athena-core/src/verify.rs:2700:13
     |
2700 |             changed_paths: &[],
     |             ^^^^^^^^^^^^^ `verify::VerdictInputs<'_>` does not have this field
     |
     = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 9 previous errors
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test --locked && cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core && git diff`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
