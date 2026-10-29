# Job 20261029-142844-ynv3: verify "Measure build cache and disk (A1); accept D145"

## Request (verbatim)

> Priority: urgent
> Async drain build job A1 (docs/design/async-drain.md, decision D145). Measure the build cache and disk (design section 12, job 1): find the cause of D84's 0 sccache hits by running sccache --show-stats inside a job's build, measure hits across two per-job worktrees with different target directories, and measure T (one cold target/) and S (worktree source size). Record the numbers in docs/decisions.md as measurements; no athena.toml change. Also mark decision D145 accepted, decided by the brain under Sri's delegation (2026-10-29). With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/disk.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Docs job for audience Sri and the verifier: run sccache --show-stats in a build, measure hits across two worktrees with different target dirs, measure T and S, record the numbers in docs/decisions.md, mark D145 accepted by the brain under Sri's delegation (2026-10-29), add a CHANGELOG entry. No drain behavior changes; max_in_flight = 1 is today's drain.

Assumptions:
- Goal: give the async drain design measured numbers for build cache and disk; audience is Sri and the verifier of later build jobs.
- Code survey: read crates/athena-core/src/disk.rs (df_free_bytes, sweep_idle_targets, hold functions); it is context for disk measurement and is not expected to change, since the request is measurement and recording only and adds no drain behavior.
- If the producer cannot run sccache (not installed or unavailable), it records that fact and the command output as the measurement instead of inventing numbers; unmeasured values are labeled estimates.
- The D145 heading in docs/decisions.md (currently '(proposed)') becomes accepted, with the decider and the date 2026-10-29; docs/design/async-drain.md status line may be updated to match.
- No athena.toml change and no governance file change; there are no new behaviors, so the existing drain tests are untouched and must pass unchanged.

## Acceptance criteria

1. [content] docs/decisions.md contains a measurements entry for A1 that states the sccache --show-stats output or the reason it could not run, the cause found for D84's 0 hits (or an explicit statement that it was not found), the hit counts across two worktrees with different target directories, and numeric values for T (one cold target/) and S (worktree source size) with units and the commands used.
2. [content] The D145 heading in docs/decisions.md no longer says proposed and states accepted, decided by the brain under Sri's delegation, 2026-10-29.
3. [content] CHANGELOG.md has a new entry naming the A1 measurements and D145 acceptance.
4. [content] git diff shows no change to athena.toml, defaults.toml or any file under .github/workflows/.
5. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass, including the existing drain tests unchanged (max_in_flight = 1 behaves as today).

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/disk.rs (modify)

## Out of scope

- Any athena.toml change or re-enabling sccache
- Implementing read-ahead, per-job worktrees or max_in_flight
- Changing drain behavior or existing drain tests

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Measurements in decisions.md cite the commands run and label estimates.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs fmt, clippy and tests and reads the diff.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the recorded numbers and cause, not the activity.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs and changelog only.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No paths with personal data or secrets are recorded.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f195831dd083 → candidate c861fd535fac. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md               |  4 ++++
 docs/decisions.md          | 27 +++++++++++++++++++++++++--
 docs/design/async-drain.md |  2 +-
 3 files changed, 30 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1665 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "docs/decisions.md contains a measurements entry for A1 that states the sccache --show-stats output or the reason it could not run, the cause found for D84's 0 hits (or an explicit statement that it was not found), the hit counts across two worktrees with different target directories, and numeric values for T (one cold target/) and S (worktree source size) with units and the commands used.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass, including the existing drain tests unchanged (max_in_flight = 1 behaves as today).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
