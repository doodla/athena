# Job 20261013-095538-59x4: verify "Reorder roadmap so acceleration work comes first, study alongside"

## Request (verbatim)

> Reorder the roadmap: Sri's 2026-10-13 addendum makes acceleration Athena's mandate, so speed work (sccache's 0 hits, parallel jobs incl. shared CHANGELOG/decisions collisions, native PR landing, timings) goes first, with the firstmate study running alongside.

## Reading (accepted by the user)

Edit docs/design/roadmap.md so a new first acceleration step (sccache hit rate, parallel jobs with shared CHANGELOG/decisions collisions, native PR landing, timings fed back) precedes the others, citing the BRIEF.md 2026-10-13 "Athena owns acceleration" addendum. The firstmate study step (RM-6) is marked as running alongside it. Docs only, no code.

Assumptions:
- The 'acceleration' addendum is BRIEF.md 'Athena owns acceleration' dated 2026-10-13 (lines 494-496); BRIEF.md itself is not edited.
- The sccache '0 hits' figure comes from the user, not from anything measured in the repository; the roadmap will cite it as reported by the user.
- Existing RM-n identifiers stay stable where possible to avoid breaking references; a new step is added at the top and RM-8 and AD-11 rows move into it.
- Nothing is implemented by this change; the roadmap remains a proposal, as it states.

## Acceptance criteria

1. [content] docs/design/roadmap.md opens its ordered steps with an acceleration step that cites the BRIEF.md 2026-10-13 'Athena owns acceleration' addendum (BRIEF.md:494-496).
2. [content] The acceleration step covers: sccache showing 0 cache hits (diagnose and fix), parallel jobs including collisions on shared CHANGELOG.md and docs/decisions.md, native pull-request landing, and timings fed back into the system; each with a 'done when' test.
3. [content] The firstmate study step (formerly RM-6) is stated to run alongside the acceleration step, not after it, and the 'How the order was chosen' paragraph is rewritten to explain the new order.
4. [content] Every partial or uncovered row still appears in exactly one step; rows moved from RM-8 (A4-3, A5-3, A5-4, A5-5, SD-6, AD-4, AD-5, AD-9) and AD-11 are not duplicated, and the stated step count and row total are correct.
5. [content] Any references to step numbers or 'first three steps' in strategy-view.md, requirements-map.md and README.md are consistent with the new order; CHANGELOG.md has an entry for the reorder.
6. [behavior] Run `git diff --stat` and `git status` to confirm only documentation files changed; run `cargo test` to confirm nothing else broke.

## Expected deliverables

- docs/design/roadmap.md (modify)
- CHANGELOG.md (modify)
- docs/design/strategy-view.md (modify)
- docs/decisions.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Implementing any acceleration work (sccache fix, parallel jobs, PR landing, timing feedback).
- Editing BRIEF.md.
- Running or changing the firstmate study.
- Changing athena.toml or CI configuration.

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The reordered roadmap explains the reason for the order to an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new step cites BRIEF.md line numbers and labels the 0-hit sccache figure as user-reported, not measured.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only reorder; no implementation of the speed work.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the step content from roadmap.md RM-8, AD-11 and decisions D79-D81.

## The change

Base ba9c3f93ce42 → candidate 250817e32baa. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                    |  6 ++++
 docs/decisions.md               | 14 ++++++++++
 docs/design/requirements-map.md |  4 ++-
 docs/design/roadmap.md          | 62 ++++++++++++++++++++++++++++++-----------
 docs/design/strategy-view.md    |  6 ++--
 5 files changed, 72 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (327 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
