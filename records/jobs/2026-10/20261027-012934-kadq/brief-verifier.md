# Job 20261027-012934-kadq: verify "Take the seeded candidate when a seeded retry's producer adds nothing"

## Request (verbatim)

> Priority: urgent
> A retry seeded from the previous attempt's kept candidate fails as "the producer made no change" when the kept candidate already does the work and only evidence was missing. Job 20261027-011733-vrsk (G2 retry): the seed already moved the Transport items, the producer checked with git status, correctly made no new change, and run.rs (around line 1764) failed the job, so the complete candidate never reached checks and the verifier. Fix: when a retry is seeded from a kept candidate and the producer adds nothing, take the seeded candidate as the candidate and go on to admission, checks and the verifier as usual. A retry from the base with no change still fails as today.
> Acceptance:
> - A test seeds a retry from a kept candidate, has a fake producer make no change, and shows the seeded candidate reaches checks and the verifier and can land.
> - A test shows an unseeded run whose producer makes no change still fails as "the producer made no change".
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, CHANGELOG.md

## Reading (accepted by the user)

In run.rs (~line 1761) the idle-producer check fails a seeded retry whose work tree is unchanged since the seed, even though the seed already differs from the base. Fix: fail only when the change list against the base is empty; when seeded and unchanged but the change list is non-empty, keep the seeded candidate and continue to admission, checks and the verifier. Audience: the Athena operator whose retries lose complete candidates.

Assumptions:
- changes is the snapshot's change list against the base, so a seeded unchanged tree has a non-empty list; the fix is to fail only when changes.is_empty().
- A seeded retry whose seed equals the base (empty change list) still fails as today.
- Tests go in run.rs's existing test module using its fake producer; no changes to snapshot.rs are needed.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] A new test in run.rs seeds a retry from a kept candidate with a fake producer that makes no change; the run reaches checks and the verifier and lands. Run: cargo test -p athena-core.
2. [behavior] A new test shows an unseeded run whose fake producer makes no change still fails with the outcome 'the producer made no change'. Run: cargo test -p athena-core.
3. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
4. [content] CHANGELOG.md has an entry describing that a seeded retry whose producer adds nothing now goes on with the seeded candidate.
5. [content] The condition near run.rs line 1761 fails only when there is no change against the base; the comment explains the seeded case.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing Snapshot::unchanged_since_seed in athena-git
- Changing how retry seeds are chosen or stored
- Changing the checks or verifier logic

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests; fmt, clippy -D warnings and tests must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise the seeded no-change run reaching the verifier and landing, and the unseeded failure.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the idle-producer condition itself, not the one job.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the seeded candidate reaches the verifier.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in run.rs plus changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b0a5cf8c9f7c → candidate 941ad20301b3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     | 12 ++++++++++
 crates/athena-cli/tests/retry.rs | 50 ++++++++++++++++++++++++++++++++--------
 crates/athena-core/src/run.rs    |  9 +++++---
 docs/decisions.md                | 29 +++++++++++++++++++++++
 4 files changed, 88 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1397 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
