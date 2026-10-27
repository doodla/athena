# Job 20261027-033006-5ca6: verify "Rename run.rs to run/mod.rs with no behavior change"

## Request (verbatim)

> try again on 20261026-232423-hvyb

## Reading (accepted by the user)

Retry of structure pass R1: move crates/athena-core/src/run.rs to run/mod.rs unchanged, so the `run` module path stays the same. The only text edits are the two test-module include_str! paths, which need one more `../`, plus a CHANGELOG entry.

Assumptions:
- Retry of the rejected R1 job with the same reading as the earlier accepted one; no findings were recorded.
- run_pin_tests.rs is a sibling file referenced via #[path] or similar; check it still resolves after the move, since a path attribute in mod.rs resolves relative to run/ and may need an edit.
- Audience is the owner and verifier applying the section 5 proof.

## Acceptance criteria

1. [content] crates/athena-core/src/run.rs no longer exists and crates/athena-core/src/run/mod.rs holds its text; git diff -M shows a rename whose only changed lines are the two include_str! paths (athena.toml, defaults.toml) gaining one ../.
2. [content] No file outside crates/athena-core/src/run/ names a moved item by an old module path: a Grep the verifier runs for `run::` and `mod run` over crates/ shows the same paths resolving, and `pub mod run;` in lib.rs is unchanged.
3. [behavior] Run `cargo test --locked -p athena-core -- --list` on the change and on the parent commit (via git show/diff); the test names are identical in count and content.
4. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and no #[allow] is added.
5. [content] CHANGELOG.md has an entry saying run.rs became run/mod.rs with no behavior change.

## Expected deliverables

- crates/athena-core/src/run.rs (delete)
- crates/athena-core/src/run/mod.rs (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Splitting run/mod.rs into submodules
- Any behavior change
- Changes to lib.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy, and test criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: test list compared before and after
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: criteria rest on commands the verifier runs itself
- small: Prefer one focused change; size is a design choice, not a wish list. Here: pure rename plus two path edits and a changelog entry
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base dee5f8388b66 → candidate cd92f3abbb6a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  | 3 +++
 crates/athena-core/src/{run.rs => run/mod.rs} | 4 ++--
 2 files changed, 5 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1397 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and no #[allow] is added.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
