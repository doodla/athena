# Job 20261105-001940-nvyd: verify "Let the drain run proposed deferred requests whose reading passes checks"

## Request (verbatim)

> try again on 20261105-001814-tw8d

## Reading (accepted by the user)

Retry of job 20261105-001814-tw8d (dispatched, never verified, no findings, no candidate, so nothing to fix or keep): `athena drain` should run a request queued by `ask --later`, the Steward or Hestia whose latest reading is proposed and passes every reading check, in priority order. A changed-scope re-read after a settings change still waits for a person. Audience: the operator relying on unattended batches.

Assumptions:
- Job 20261105-001814-tw8d left no candidate and no findings, so its work is neither kept nor dropped. Its scope is reused: drain.rs, tests/drain.rs, steward.rs, CHANGELOG.md. Its out_of_scope items are carried over unchanged.
- Code survey: I read crates/athena-cli/src/drain.rs and crates/athena-core/src/steward.rs. In steward.rs, `queued_work` already lists Queued and Proposed jobs on DRAINED_CHANNELS (steward and cli) and marks proposed ones `read`. In drain.rs, `queue()` builds items from `approved_work`, `queued_work` and `interrupted_work`. `run_job` skips the second reading for a pre-read item. `run.needs_accepting` maps to outcome `needs accepting`, which is not a failure. `athena-core/src/run/mod.rs` `go` takes `AcceptedBy::Flag` from the drain, so a re-read with the same scope is accepted at once and a changed scope is not. The entry point is `drain()` in drain.rs. Selection and ordering run through the plan, then `pick_ahead`. No records or data files are added, so `check_record_file`, `AnyRecord` and the round-trip test are not involved. The measurement is the drain's own report lines in crates/athena-cli/tests/drain.rs.
- The listing code already appears to include proposed requests, so the producer must first find where a proposed, deferred, checks-passing request is actually skipped. Candidates are plan selection, the reading-check gate before the run, and priority ordering of read items. It then fixes that root cause, not the listing.
- This change does not add a refusal rule. It runs more items and refuses none, so no newly-refused rate applies.
- Changes to athena.toml, defaults.toml or .github/workflows are not needed. They stay untouched and are listed in out_of_scope.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs has a deferred request (channel from `ask --later`) in state proposed whose latest reading passes the reading checks. The drain runs it and reports it as passed. Check with `cargo test -p athena-cli --test drain`.
2. [behavior] A new test in crates/athena-cli/tests/drain.rs has a request re-read with a changed scope after a settings change. The drain does not run it, reports outcome `needs accepting`, counts no failure, and does not stop the batch. Check with `cargo test -p athena-cli --test drain`.
3. [behavior] A test shows a proposed deferred request whose latest reading fails a reading check is not run by the batch. Another test shows a mixed run of proposed and queued items goes in priority order, with the urgent one before the lower priority one. Check with `cargo test -p athena-cli --test drain`.
4. [content] `git diff` touches only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/steward.rs (only if selection lives there) and CHANGELOG.md. It shows no change to athena.toml, defaults.toml or .github/workflows, and no change to the reading checks themselves.
5. [content] The module docs in crates/athena-cli/src/drain.rs (the bullet 'A read request stays queued') state the new rule, and CHANGELOG.md has an entry for it.
6. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass for the workspace.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the reading checks themselves
- Changing `athena go` or `athena ask` behavior
- Changing the rule that a changed-scope re-read needs a person
- Running the 42 existing proposed requests
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests, and cargo test, clippy and fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the drain running or leaving the item, through tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer finds where proposed items are skipped and fixes it there.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain selection.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the CHANGELOG entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a1385620ff3a → candidate f77e74de31a6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  28 +++
 crates/athena-cli/src/drain.rs    |  20 +-
 crates/athena-cli/tests/drain.rs  | 390 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/steward.rs | 244 +++++++++++++++++++++++-
 4 files changed, 669 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41940620517 for f77e74de31a6ca6304496238c081d5964f131a7c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41940620517 for f77e74de31a6ca6304496238c081d5964f131a7c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41940620517 for f77e74de31a6ca6304496238c081d5964f131a7c)
- ci-result: `not a check` passed (Check results for f77e74de31a6ca6304496238c081d5964f131a7c come from the CI run https://github.com/doodla/athena/actions/runs/41940620517 (642s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes for the workspace, `cargo clippy` is clean, and `cargo fmt --check` passes. The module docs of drain.rs describing 'A read request stays queued' are updated to state the new rule.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
