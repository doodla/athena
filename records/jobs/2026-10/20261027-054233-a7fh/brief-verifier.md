# Job 20261027-054233-a7fh: verify "Check at plan time that the design names the code it changes"

## Request (verbatim)

> Priority: normal
> 
> Add an early check, at plan or design time, that the design names the existing code it changes or reuses. A mismatch should then be caught before production starts, not at the final verdict. Check for overlap with the queued item reading-or-provenance-mismatch-caught-only-at-final-verdict.
> 
> Introspections that record the repeated cause: 20261015-205658-9fta, 20261016-163602-mx80, 20261017-062450-ft3t
> 
> Acceptance:
> - The change the request describes is made, with the root cause named
> - A test fails without the change and passes with it
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Add an early gate in the plan/reading step that refuses a design or plan entry which does not name the existing code (paths or symbols) it changes or reuses, so mismatches are caught before production rather than at the final verdict. Audience: the Athena maintainer. Overlap with the queued item reading-or-provenance-mismatch-caught-only-at-final-verdict is handled by implementing one shared early check and not duplicating it.

Assumptions:
- The plan gate lives in crates/athena-core (plan.rs PlanEntry::missing / reading.rs plan_entry); I did not read their bodies, so the producer must confirm the exact home.
- 'Names the existing code' means the design text cites at least one repository path that exists in the checkout, or states explicitly that it is wholly new code.
- The root cause is that the named-code requirement was only checked by the verifier at the final verdict, never when the plan or design was written.
- The queued item reading-or-provenance-mismatch-caught-only-at-final-verdict is the same root cause; the change should cover it where it can and the producer should record the overlap in the commit message and not close the other item.

## Acceptance criteria

1. [content] The commit message or code docs name the root cause: the design's code references were not checked until the final verdict, and say how the change overlaps with reading-or-provenance-mismatch-caught-only-at-final-verdict.
2. [behavior] A new test feeds a plan entry or design that names no existing code, or names a path that does not exist, and asserts it is refused or flagged before production. It fails on the pre-change code and passes with the change. Run: cargo test.
3. [behavior] A new test shows a design that names existing repository paths, or states that it is wholly new code, is accepted, so the gate is not a blanket refusal. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy (all targets, no warnings), and cargo test all pass.
5. [content] The refusal message tells the author what is missing, e.g. which design names no existing code, in plain wording a stranger can follow.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Closing or reworking the queued item reading-or-provenance-mismatch-caught-only-at-final-verdict beyond the shared check
- Changing prompt files or the verifier's final-verdict logic
- Editing records/ or the cited introspections

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust under crates/.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the early refusal happens, not that types exist.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Gate moved to plan time, where the cause arises, not patched at the verdict.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused gate, shared with the overlapping queued item.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code, not prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base e9760bf931e1 → candidate bac60967adc4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 +++
 crates/athena-core/src/plan.rs    | 148 ++++++++++++++++++++++++++++++++++++--
 crates/athena-core/src/reading.rs |  78 ++++++++++++++++++++
 3 files changed, 232 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1469 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy (all targets, no warnings), and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
