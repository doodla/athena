# Job 20261016-174508-k0w6: verify "Infer a request's rung from BRIEF.md and addenda, not a word list"

## Request (verbatim)

> The plan from #116 ranks 29 of 30 items 'implementation' and orders them oldest first, so the owner's 2026-10-16 priority (speed: CI routing s6tr, parallel jobs q0xy) lands last. A request's rung should come from what it serves in BRIEF.md and the owner's latest addenda, not a word list.

## Reading (accepted by the user)

Planning gives nearly every request the Implementation rung because `infer_rung` in reading.rs treats any touched path, `change` capability or word like "code" or "ci" as proof, so age alone orders the queue. The goal is for the rung to come from the BRIEF.md requirement or owner addendum a request serves, so the 2026-10-16 speed items (s6tr, q0xy) rank above older plain implementation work. The audience is the owner, who reads the plan order.

Assumptions:
- Fix site is `infer_rung` and `plan_entry` in crates/athena-core/src/reading.rs; plan.rs sorts by (rung, age) and stays as is.
- A request that serves a BRIEF.md addendum or requirement (for example the 2026-10-16 speed addendum) ranks at BriefRequirements, above Implementation. Roles still cannot change the top two rungs; this only affects ordering.
- Deterministic matching: the request text and reading summary are matched against BRIEF.md section headings and addenda text, passed in through the Lookup struct so plan_entry stays free of I/O. No model call is added.
- The IMPLEMENTATION_WORDS list stops being sufficient evidence on its own. A request that matches no brief section falls back to Implementation only when a path is touched or the reading chose `change`, else None, as today.
- The ordering rule in the plan.rs docs stays: rung first, then oldest first.
- BRIEF.md is protected and is not edited.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading.rs plans a request about CI routing or parallel jobs against the real 2026-10-16 speed addendum text and asserts its rung is above Implementation. Run `cargo test -p athena-core`.
2. [behavior] A unit test builds a plan from an old plain implementation request (a typo fix) and a newer speed-addendum request, and asserts the speed request is ordered first. Run `cargo test -p athena-core plan`.
3. [behavior] A request that matches no brief section and has only an IMPLEMENTATION_WORDS word, such as 'fix a bug', no longer gets a rung from the word alone. A test asserts this, and the existing test `a_rung_is_inferred_from_evidence_or_flagged_never_defaulted_to_implementation` is updated to match.
4. [behavior] `cargo test` passes and `cargo clippy --all-targets` is clean across the workspace.
5. [content] The doc comments on `IMPLEMENTATION_WORDS`, `infer_rung` and the plan.rs module header say the rung comes from what a request serves in BRIEF.md and its addenda. A short entry in docs/decisions.md and CHANGELOG.md records the call and the alternatives (a model-chosen rung, keeping the word list).
6. [content] BRIEF.md is unchanged.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Re-planning or reordering the 30 items of #116 (a later run of the fixed planner does that).
- Changing the ladder's rungs or which roles may change them.
- Editing BRIEF.md.
- Changing the CI routing or parallel jobs work itself (s6tr, q0xy).

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New and updated tests; cargo test and clippy are acceptance criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix replaces the word-list heuristic itself, not just the ordering of the two items.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the rung and order for the speed requests, not that code exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checkable with cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the decision entry are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to rung inference.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 90de63c2db36 → candidate f3db6312c71e. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |   8 ++
 crates/athena-cli/src/drain.rs    |  10 ++
 crates/athena-core/src/plan.rs    |   5 +-
 crates/athena-core/src/reading.rs | 194 +++++++++++++++++++++++++++++++++++---
 docs/decisions.md                 |  20 ++++
 docs/design/roles-and-ladder.md   |   5 +-
 6 files changed, 227 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (786 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
