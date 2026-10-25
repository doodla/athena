# Job 20261025-041303-wbmn: verify "Judge PR-only commits after C0 by the pre-merge rule in check --history"

## Request (verbatim)

> Priority: high
> The cutover (C0 b4f6a8a, job 20261025-001232-vesh) broke the required 'the full history' check on every pull request: on a pull request's head, athena check --history hands every commit after C0 to the attribution rule, which requires a landing record naming each commit on main, so the pull request's own commits (not yet merged, so with no landing record) are all unattributed (PR #175: 50 violations, every one 'no landing record names it and no landing-intent matches its tree'). On a head that is not main, judge the commits after C0 that are not on main (the range from the merge base with main to the head) by the pre-merge rule instead (land.rs pr_commits: records-only, declared intervention, or a job landing whose verification passed), and keep the attribution rule for commits on main.
> Acceptance:
> - On the cutover fixture (test_support from 20261025-031609-k0tt), a pull-request branch after C0 holding records-only commits, an intervention and a job landing passes athena check --history, and a branch with an unverified governed change fails naming it.
> - athena check --history on main after C0 is unchanged.
> - The CI history job passes on a pull request built like this repository's (run on the PR's own head).
> Files: crates/athena-gate/src/history.rs, crates/athena-gate/src/attribution.rs, crates/athena-core/src/land.rs, .github/workflows/ci.yml, CHANGELOG.md

## Reading (accepted by the user)

On a head that is not main, `athena check --history` after C0 will judge commits in merge-base(main)..head by the pre-merge rule (records-only, declared intervention, or a verified job landing). It keeps the attribution rule for commits on main. The audience is the maintainer and CI, which currently fails the required 'the full history' check on every PR.

Assumptions:
- athena-gate does not depend on athena-core (gate Cargo.toml lists data, git, score), and land.rs classify_commit already uses gate's invariant::closing_jobs. So the classifier moves into the gate and land.rs pr_commits/classify_commit delegate to it, keeping their behavior and tests.
- 'main' means the remote main ref (origin/main or local main) available in the checkout; if the merge base cannot be found, the check falls back to today's behavior and adds a note.
- A head 'is main' when it is an ancestor of, or equal to, main's tip. Then the whole post-C0 range stays under attribution, so history on main is unchanged.
- The CI change is limited to what the PR-head run needs: fetch main and athena/records so the merge base and records resolve. The existing PR-event skip of the records fetch is revisited. The checkout token persistence is narrowed to the fetch step if feasible.
- Tests for the acceptance cases use the existing cutover fixture from crates/athena-core/src/test_support.rs (job 20261025-031609-k0tt), driven from crates/athena-cli/tests.

## Acceptance criteria

1. [behavior] A test on the cutover fixture creates a PR branch after C0 with a records-only commit, a declared-intervention commit and a verified job-landing commit, none named by a landing record. Run `cargo test -p athena-cli` (the new history test): `athena check --history` on that head reports no violations.
2. [behavior] A test adds a governed-content change with no verification or intervention to a PR branch after C0. Run `cargo test -p athena-cli` (the new history test): `athena check --history` fails and the violation names that commit's sha.
3. [behavior] A test shows `athena check --history` on the fixture's main after C0 gives the same report as before: commits on main still go through the attribution rule, and an unattributed commit on main still fails. Run `cargo test -p athena-gate -p athena-cli`.
4. [behavior] All existing history and attribution tests and the land.rs pr_commits tests pass unchanged. Run `cargo test --workspace`; `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
5. [content] The CI history job in .github/workflows/ci.yml has the refs it needs on a pull_request event (main, and athena/records where used) so the merge base resolves. The step comments say why. The job keeps the name 'the full history'.
6. [content] CHANGELOG.md has an entry that says a PR head's own commits after C0 are judged by the pre-merge rule and main's by attribution, and cites the PR #175 failure.
7. [content] The pre-merge classifier has one source of truth in the gate; land.rs pr_commits delegates to it, with no duplicated rule logic.

## Expected deliverables

- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-core/src/land.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-gate/src/legacy.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-cli/tests/history_pr_head.rs (create)
- crates/athena-cli/tests/cutover_fixture.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Ending commit-message trailers.
- Changing the attribution rule's treatment of commits on main.
- Changing branch protection or the other CI jobs (attribution, pr-head).
- Changing athena.toml or defaults.toml.
- Re-running or repairing the cutover import itself.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the PR-branch pass, the PR-branch failure and main unchanged; cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run athena check --history on fixture branches and check the pass/fail outcome and the named commit.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The range split is fixed in the history check itself, not by skipping the check in CI.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test on the fixture-based tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change: range split, classifier move, CI fetch tweak and changelog.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The behavior is enforced by the required history check and its tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust, plus the CI YAML.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test history_pr_head.rs builds the cutover fixture, branches from main (C0), adds a records-only commit, an intervention commit and a job-landing commit with a passing verification (no landing record names any), checks out the branch and runs the real binary via athena_check(&fx, "--history"). Run: `cargo test -p athena-cli --test history_pr_head`. Observes: exit success and '0 violation(s)' for the passing-branch test.
  Fails without the behavior: Without the range split, attribution flags every branch commit as 'no landing record names it', so the test fails with violations.
- Criterion 2: Same file: a branch with a commit that changes governed content (src/lib.rs) with no verification or intervention; run check --history. Run: `cargo test -p athena-cli --test history_pr_head`. Observes: the command fails and its output contains that commit's sha and the 'changes governed content' text.
  Fails without the behavior: If the pre-merge rule were not applied, the violation would be the attribution message (or none if the commit were skipped); the sha plus governed-content assertion fails.
- Criterion 3: The existing cutover_fixture test (history and attribution pass on main) plus a new test that adds an unattributed hand commit on main after C0 and asserts check --history fails naming it; a gate unit test in history.rs for the tip-is-ancestor-of-main path. Run: `cargo test -p athena-gate -p athena-cli`. Observes: main passes with 0 violations as before; an unattributed commit on main still fails, with the attribution wording.
  Fails without the behavior: If main's commits were routed to the pre-merge rule, the unattributed hand commit that is records-only or verified-looking would no longer be flagged, or the report would change.
- Criterion 4: Run the full suite and lints; land.rs pr_commits tests and the attribution/history tests are untouched. Run: `cargo test --workspace`. Observes: all pass, with no edits to existing assertions; then cargo clippy --workspace --all-targets and cargo fmt --check are clean.
  Fails without the behavior: A changed message or behavior in land.rs would fail the existing pr_commits tests.
- Criterion 5: Read the diff of ci.yml: the history job's fetch steps run on every event and bring main and athena/records; comments give the reason; name stays 'the full history'. CI itself is not runnable here; the merge-base logic it depends on is covered by the fixture tests. Run: `git diff -- .github/workflows/ci.yml`. Observes: no 'if: github.event_name != pull_request' on the fetch step, a fetch of main, and the job name unchanged.
- Criterion 6: Read the CHANGELOG diff. Run: `git diff -- CHANGELOG.md`. Observes: an entry stating PR-head commits after C0 are judged by the pre-merge rule, main's by attribution, citing PR #175 (50 violations).
- Criterion 7: Inspect that land.rs classify_commit has no rule logic of its own and calls the gate classifier; grep for the rule conditions. Run: `git diff -- crates/athena-core/src/land.rs crates/athena-gate/src/invariant.rs`. Observes: closing_jobs/added_interventions/fingerprint rule branches appear once, in the gate; land.rs only maps the result to PrCommit and error text.

Planned files:

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-core/src/land.rs (modify)
- .github/workflows/ci.yml (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/history_pr_head.rs (create)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/cutover_fixture.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/legacy.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## The change

Base 817c47943745 → candidate 7b399e204063. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml                   |  17 ++--
 CHANGELOG.md                               |  12 +++
 crates/athena-cli/tests/history_pr_head.rs | 155 +++++++++++++++++++++++++++++
 crates/athena-core/src/land.rs             |  55 ++--------
 crates/athena-gate/src/history.rs          |  95 +++++++++++++++++-
 crates/athena-gate/src/invariant.rs        |  83 +++++++++++++++
 6 files changed, 365 insertions(+), 52 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1187 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
