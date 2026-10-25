# Job 20261025-150509-dmwc: verify "Raise producer_timeout_s to 3000 and record the decision"

## Request (verbatim)

> Priority: urgent
> Raise the producer deadline in athena.toml [budget] producer_timeout_s from 1800 to 3000 seconds. Six producer runs since 2026-10-24 hit the 1800 s deadline and lost all their work (jobs 20261019-013309-nvbb, 20261023-234149-jcd2, 20261024-140624-ewbs, 20261025-034731-2am0, 20261017-044405-4zcq, and the one before nvbb), several doing legitimate work whose targeted test builds take 5 to 10 minutes. Record the decision with its alternatives (splitting jobs, a faster suite, keeping work on timeout, each also queued) and the reversal.
> Acceptance:
> - athena.toml has producer_timeout_s = 3000 and parses (the repository config test passes).
> - docs/decisions.md has the decision entry; CHANGELOG.md has an entry.
> Files: athena.toml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Change athena.toml [budget] producer_timeout_s from 1800 to 3000, add a decision entry in docs/decisions.md (alternatives: splitting jobs, faster suite, keeping work on timeout; plus reversal), and add a CHANGELOG Unreleased entry. The audience is the Athena operator, who needs producers to stop losing work at the deadline.

Assumptions:
- The new decision is appended to docs/decisions.md as the next numbered entry after the last existing one (D15 is seen, later ones may exist), in the same heading format.
- The three alternatives are recorded as queued follow-ups, not built here.
- Reversal means the steps and the signal for setting the value back to 1800 (for example, if no timeouts occur in a stated number of jobs, or cost rises).
- The six job count comes from the request; only five job ids are named, so the entry names those five and says the sixth is unnamed.
- No code changes; verifier_timeout_s and other budget values are untouched. Other docs that may quote 1800 are checked and updated only if they state the current value.

## Acceptance criteria

1. [content] athena.toml [budget] has producer_timeout_s = 3000 and no other budget value changes.
2. [content] docs/decisions.md has a new entry naming the 1800 to 3000 change, the evidence (the named failed jobs), the alternatives (splitting jobs, a faster suite, keeping work on timeout, each marked queued), and how to reverse it.
3. [content] CHANGELOG.md has an entry under [Unreleased] describing the producer deadline raise from 1800 to 3000 seconds and why.
4. [behavior] Run `cargo test`: the repository config test that parses athena.toml passes with producer_timeout_s = 3000, and the whole suite passes; `cargo clippy` is clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Splitting jobs, speeding up the test suite, or preserving work on timeout (recorded as queued alternatives only)
- Changing verifier_timeout_s or other budget settings
- Code changes in crates/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision entry cites the named job ids and the 5 to 10 minute test build figure as the request's claim, labeled as reported.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The entry is written so an outside reader understands the context, options, and reversal.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Three files only: one config value, one decision entry, one changelog line.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test (including the config parse test) passes and clippy is clean.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that athena.toml parses with 3000 through the repo config test, not that the file was edited.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Inspect the diff of athena.toml. Run: `git diff -- athena.toml`. Observes: Exactly one changed line, producer_timeout_s = 1800 to 3000; reader, producer_usd, verifier_timeout_s = 2400 and check_timeout_s = 900 unchanged.
- Criterion 2: Read the new entry in the diff. Run: `git diff -- docs/decisions.md`. Observes: A new D124 entry naming 1800 to 3000, the five job ids with the sixth marked unnamed, the 5 to 10 minute build claim labeled as reported, three alternatives each marked queued, and reversal steps and signal.
- Criterion 3: Read the changelog diff. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under [Unreleased] saying the producer deadline rose from 1800 to 3000 s and why.
- Criterion 4: Run the suite; the config test parses athena.toml, and clippy runs with warnings denied. Run: `cargo test --locked`. Observes: All tests pass, including the repository config parse test with producer_timeout_s = 3000. Also run cargo clippy --all-targets --locked -- -D warnings with no warnings.
  Fails without the behavior: A malformed or mistyped producer_timeout_s (for example a quoted string) would make the config parse test fail.

Planned files:

- athena.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 8550c93643df → candidate d5af2ae0fe32. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md      |  4 ++++
 athena.toml       |  2 +-
 docs/decisions.md | 22 ++++++++++++++++++++++
 3 files changed, 27 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1230 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
