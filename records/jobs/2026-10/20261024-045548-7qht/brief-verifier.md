# Job 20261024-045548-7qht: verify "Count Athena-recorded passing same-tree checks as seen under D43"

## Request (verbatim)

> Since 88562f1, change verifiers cite Athena's recorded fmt, clippy and test results instead of rerunning them, but D43 (crates/athena-core/src/verify.rs, around line 335) still marks a behavior criterion unmet unless its evidence cites a command the verifier was seen running, so job 20261024-033933-ta60 was rejected on 'cargo test passes for the workspace' while Athena's own recorded test run on the same candidate passed. Count a command as run when Athena itself ran and recorded it on the same candidate tree and it passed (the checks shown under 'Checks Athena ran on this tree'), so citing it satisfies D43; a recorded check that failed, or one recorded for a different tree, still does not. Test both cases and update D43's documentation.

## Reading (accepted by the user)

Change `judge` in verify.rs so a behavior criterion's cited command counts as run if the verifier ran it, or Athena ran and recorded it on the same candidate tree and it passed. Failed or other-tree records still fail D43. Both cases are tested and D43 is documented as amended. The audience is the verifier and the maintainers who read D43.

Assumptions:
- The user's reply applies to this job: the workspace test/clippy/fmt criterion is a content criterion, judged by Athena's recorded checks; everything else from the earlier reading stays.
- D43 is amended through a new decision rather than rewriting D43's history.
- Recorded checks are matched to cited commands by command text, and same-tree means the recorded tree equals the candidate tree.

## Acceptance criteria

1. [behavior] A unit test shows a behavior criterion citing `cargo test` is met, with commands_verified true and no D43 finding, when the verifier ran nothing but a passing `cargo test` check is recorded for the same candidate tree. Run `cargo test -p athena-core verify`.
2. [behavior] A unit test shows the criterion is still unmet, with the D43 'cited but not seen running' finding, when the recorded `cargo test` check failed. Run `cargo test -p athena-core verify`.
3. [behavior] A unit test shows the criterion is still unmet when the passing recorded check is for a different tree than the candidate. Run `cargo test -p athena-core verify`.
4. [behavior] Existing D43/D63 tests still pass: a command never run by the verifier and not recorded is unseen, and a behavior criterion with no commands is still unmet. Run `cargo test -p athena-core verify`.
5. [content] Athena's own recorded fmt, clippy and test checks on the candidate tree passed ('Checks Athena ran on this tree'); this satisfies the workspace-wide test, clippy and fmt criterion, which is a content criterion rather than a behavior one.
6. [content] The module doc of crates/athena-core/src/verify.rs and the docs of `commands_run` and the new input field state the rule: a cited command is seen if the verifier ran it, or Athena ran and recorded it on the same candidate tree and it passed; failed or other-tree records do not count.
7. [content] docs/decisions.md has a new decision amending D43 with the rule, alternatives, tradeoff, and provenance citing job 20261024-033933-ta60 and commit 88562f1; D43's section points to it; docs/decision-links.toml links it to D43 as 'amended' with correct line numbers.
8. [content] crates/athena-core/src/run.rs passes the recorded checks and same-tree indication into `judge`, excluding nested-run 'not run' placeholders, so the live pipeline gets the new behavior.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the records-branch writer in athena-git or the two-threads test nit from job ta60.
- Changing how the verifier's own commands are observed.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the passing, failed and other-tree cases; workspace tests and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria are tests of the verdict that D43 produces.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion that job ta60 failed on is made checkable from a recorded check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision are written for outside readers.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision cites the job and commit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to judge, its caller and docs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the rule in D43 rather than the one rejected job.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in verify.rs. The verifier ran nothing, one recorded `test` check with command `cargo test` passed (exit 0), and `candidate_tree` equals `checks_tree`. The behavior criterion cites `cargo test`. Run: `cargo test -p athena-core verify`. Observes: The criterion is met, `commands_verified` is true, and no finding contains 'D43'.
  Fails without the behavior: Without the recorded-check path, `cargo test` is not in `ran`, so the criterion is unmet with the 'cited but not seen running' finding and the test fails.
- Criterion 2: Unit test: same setup, but the recorded check is `check(false)` (failed, exit 101). Run: `cargo test -p athena-core verify`. Observes: The criterion is unmet, `commands_verified` is false, and the finding evidence contains 'cited but not seen running: `cargo test`'.
  Fails without the behavior: If a failed record counted (the `passed` or `exit_code` guard missing), the criterion would be met and the test would fail.
- Criterion 3: Unit test: a passing record, but `checks_tree` differs from `candidate_tree`. A second case sets `checks_tree` to None. Run: `cargo test -p athena-core verify`. Observes: The criterion is unmet and the D43 finding is present in both cases.
  Fails without the behavior: If the tree comparison were absent, the record would count and the test would fail.
- Criterion 4: Run the existing verify tests unchanged apart from the two new `VerdictInputs` fields added to each constructor. They include the D63 test `cited_commands_match_whole_simple_commands_inside_what_ran` and the no-commands behavior criterion test. Add one case where the verifier ran nothing, no check is recorded, and the criterion is unmet. Run: `cargo test -p athena-core verify`. Observes: All pass. An unrun, unrecorded command is still unseen, and a behavior criterion with no commands is still unmet ('it cites no command').
  Fails without the behavior: If the amendment loosened the unseen test beyond recorded passing same-tree checks, these tests would fail.
- Criterion 5: Athena's own recorded fmt, clippy and test checks on the candidate tree, shown under 'Checks Athena ran on this tree'. The verifier cites those records, per the verifier prompt. Run: `cargo test --locked`. Observes: The recorded checks `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` all show passed.
- Criterion 6: Read the module doc and the docs on `commands_run`, `checks`, `candidate_tree` and `checks_tree` in verify.rs. A grep also checks that the rule is stated. Run: `git diff -- crates/athena-core/src/verify.rs`. Observes: The diff has the sentence: a cited command is seen if the verifier ran it, or Athena ran and recorded it on the same candidate tree and it passed; failed or other-tree records do not count.
- Criterion 7: Inspect the diff of docs/decisions.md and docs/decision-links.toml. Check that the new decision's 'Amends D43' line number equals the `line` value in the new [[link]], with older = D43, newer = D115, status = amended. Check that D43's section has the pointer line. Both checks come after the D43 edit shifts later lines, so the numbers of every existing link below D43 must also be updated. Run the backtest-related tests. Run: `cargo test --locked`. Observes: The diff shows D115 with its rule, alternatives, tradeoff, and provenance citing job 20261024-033933-ta60 and commit 88562f1. Decision-link and backtest tests pass, which shows the line numbers are correct.
- Criterion 8: Read the `judge` call in run.rs. It passes `candidate_tree: Some(c)` and `checks_tree: Some(c)` only when `self.nested.is_none()`, otherwise None. Placeholders have `exit_code None`, so `judge` ignores them. The existing nested-run lifecycle tests in run.rs and run_pin_tests.rs still pass. Run: `git diff -- crates/athena-core/src/run.rs`. Observes: The diff shows the new inputs wired as described. `cargo test -p athena-core` passes.

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## The change

Base 4aa5f0347964 → candidate 3ae690b3f658. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   5 ++
 crates/athena-core/src/run.rs    |   3 +
 crates/athena-core/src/verify.rs | 131 +++++++++++++++++++++++++++++++++++++--
 docs/decision-links.toml         |   6 ++
 docs/decisions.md                |  29 ++++++++-
 5 files changed, 169 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1025 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
