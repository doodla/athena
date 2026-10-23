# Job 20261023-181729-2sf0: verify "Make land --pr judge and guard the PR's current head"

## Request (verbatim)

> athena land --pr judges the checks of the job's own commit, not the pull request's current head. When a fix is pushed to the pull request (on 2026-10-18, PR #149 for job 20261018-145008-k7fq), land keeps re-running and reporting the old commit's failure even though the new head is green, so the only way to merge was around land. Make land judge the checks of the pull request's current head and guard the merge with that head, and never force-replace a newer head with the job's older commit. Until the records-history build (D111) lands, a squash of a job's landing plus any later commit is not a valid landing (that is how squash 02837e0 broke the history check), so land refuses when the pull request's current head is anything other than the job's landing commit, declared intervention or not, naming each extra commit and why. This repeats job 20261019-013306-zvmr, abandoned when the machine ran out of disk mid-run.

## Reading (accepted by the user)

Change `athena land --pr` so it judges checks of the pull request's current head and passes that head as the merge guard. It never force-replaces a newer head with the job's commit. It refuses, naming each extra commit and why, when the head is anything but the job's landing commit. Audience: the maintainer who landed PR #149 around land.

Assumptions:
- The land flow lives in crates/athena-core/src/land.rs with GitHub calls in gh.rs; I did not read them in depth.
- Refusal compares the PR head sha to the job's landing commit and lists the commits between them.
- The previous attempt's candidate touched gh.rs, main.rs, README.md and CHANGELOG.md, so those are likely again.

## Acceptance criteria

1. [behavior] A test (run with cargo test) shows that with a PR whose head differs from the job's commit and is green while the job's commit is red, land does not report the old commit's failure and judges the head's checks; and when the head equals the landing commit, land merges.
2. [behavior] A test (cargo test) shows the merge request is guarded with the PR's current head sha, and that no push or update replaces a newer head with the job's older commit.
3. [behavior] A test (cargo test) shows land refuses when the PR head is anything other than the job's landing commit, including when the extra commit is a declared intervention, and the message names each extra commit and why it is refused (squash with later commit invalid until D111 records-history build).
4. [content] README.md or CHANGELOG.md document the new land behavior and the D111 interim refusal, so a maintainer can find why land refused.
5. [behavior] cargo clippy and cargo fmt --check are clean; cargo test passes for the changed crates (the pre-existing reading_inputs_digest failure is noted if it persists).

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Building the records-history build (D111)
- Fixing the pre-existing reading_inputs_digest test failure
- Changing the squash-merge strategy beyond the guard

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each land behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check land's judging, guard and refusal by running tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix which commit land judges rather than working around it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to land.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in gh.rs using the existing Rig/FakeHost. FakeHost gives different Checks per commit (landing commit red, other head green) and records every commit passed to `checks`. FakeRemote reports a head different from the landing commit. A second case has the head equal to the landing commit and merge_via_rest on. Run: `cargo test -p athena-core land_pr_judges`. Observes: Differing head: the error text says the head's checks are green and does not mention a red or old-commit failure, and the `checks` log holds only the head sha, never the landing sha. Equal head: the report has `merge_requested` true and the host merge call is recorded.
  Fails without the behavior: The old code takes `link.head` as the job's commit (it force-replaces first), so the checks log would hold the landing sha and the red result would be reported. The old code would never refuse, so the differing-head test would fail.
- Criterion 2: Unit test in gh.rs. FakeHost.merge records its `head` argument. FakeRemote has no `replace`, and its `push` calls are recorded. Run once with the head equal to the landing commit, and once with a newer head. Run: `cargo test -p athena-core land_pr_guards`. Observes: The recorded merge head equals the remote's current head sha (distinct from any stored value). With a newer remote head there are no pushes after the first publish, no merge call, and `link.head` stored is the newer head, never the job's commit.
  Fails without the behavior: The old flow called replace/push to put the job's commit over the newer head and merged with the job's commit.
- Criterion 3: Unit test in gh.rs with a real temporary git repo and a bare remote through GitRemote. The job's landing L has later commits X and Y on the PR branch, where Y's message declares an Athena intervention. Land with merge_via_rest true. A second case uses a head older than L. Run: `cargo test -p athena-core land_pr_refuses`. Observes: `land_pr` returns Err. The message contains the short sha and subject of X and Y, the text 'records-history build (D111)', and says a declared intervention does not change it. No merge is requested and the remote branch is unchanged. For the older head, the message names the head itself.
  Fails without the behavior: Without the refusal, the old code would replace the head or merge it, and the test's Err assertion would fail.
- Criterion 4: Read the docs and search for the new text. Run: `git diff -- README.md CHANGELOG.md`. Observes: README has a land --pr paragraph saying it judges and guards the PR's current head, never replaces a newer head, and refuses any head other than the landing commit until D111, naming the extra commits. CHANGELOG has an entry under the unreleased section recording the change.
- Criterion 5: Run the project checks. Run: `cargo fmt --all --check`. Observes: No diff. Then `cargo clippy --all-targets --locked -- -D warnings` is clean, and `cargo test --locked` passes. The pre-existing reading_inputs_digest failure is noted if it persists.
  Fails without the behavior: A leftover use of the removed `replace`, an unused import, or a missed test impl of the changed Remote trait (such as crates/athena-cli/tests/external_calls.rs) would fail to compile or lint.

Planned files:

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)

## The change

Base 07ae878b9378 → candidate a3cee580c3fa. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   9 +
 README.md                                 |   8 +
 crates/athena-cli/tests/external_calls.rs |  14 +-
 crates/athena-core/src/gh.rs              | 472 +++++++++++++++++-------------
 4 files changed, 300 insertions(+), 203 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (966 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
