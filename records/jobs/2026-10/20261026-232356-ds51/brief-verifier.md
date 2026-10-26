# Job 20261026-232356-ds51: verify "Make gh.rs the gh/ directory (G1) and accept the structure-pass decision"

## Request (verbatim)

> Priority: high
> Structure pass job G1 (docs/design/structure-pass.md, section 4). Make gh.rs the directory gh/ (rename to gh/mod.rs), update the D87/D88 paths in introspect.rs per section 5.2 with a test that a landing touching a moved file still selects D87 and D88, and change the external_calls_guard WRAPPER constant to the gh/ directory prefix (approved). Also mark the structure-pass decision in docs/decisions.md accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26): module names and order approved, R0 approved, all jobs queued now with dependencies, the guard constant may change to the prefix. No behavior change. 
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/introspect.rs, crates/athena-core/tests/external_calls_guard.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Job G1 of the structure pass: rename gh.rs to gh/mod.rs with no code change. Point the D87/D88 `paths` in introspect.rs and the guard's WRAPPER constant at the `crates/athena-core/src/gh/` prefix. Mark D138 accepted in docs/decisions.md and add a CHANGELOG entry. The audience is the owner and the verifier, who check the section 5 proof. The reader found no matcher for `DecisionHeader.paths` in the code, which is the main uncertainty.

Assumptions:
- Goal: land the pure rename so later move jobs G2-G8 can start. Audience: the owner and the verifier, who check the section 5 proof.
- `paths` on DecisionHeader appears to be data only: Grep finds no code that matches it against a landing's files. The producer must re-check. If it is unmatched, the required test asserts that a moved file such as `crates/athena-core/src/gh/mod.rs` falls under a D87 and D88 `paths` entry by prefix, using a small prefix-match helper or inline check in the test. If a matcher does exist, the test drives it.
- The producer will edit the two `paths` entries in `headers()` to `crates/athena-core/src/gh/`; this touches production code but not behavior elsewhere.
- WRAPPER becomes the prefix `crates/athena-core/src/gh/` and the comparison changes from `==` to `starts_with`. The guard test's own check at line 65 (that the wrapper file is present) and its self-test list at line 85 must still work, so they need minimal edits, which are not import lines. The doc comment on line 1 should name gh/.
- The decisions.md entry is D138 (line 3570). Change its Status to accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26). Record that module names and order are approved, R0 is approved, all jobs are queued now with dependencies, and the guard constant may change to the prefix. Edit the heading's (proposed) tag and the Decision (proposed) label to match.
- The structure-pass.md pointer marking G1 done is optional (section 5 rule 7 allows it) but is not in the requested file list, so it is left out.
- The CHANGELOG entry goes under [Unreleased].

## Acceptance criteria

1. [behavior] Run `cargo test --locked`. It passes, and `cargo test --locked -p athena-core -- --list` shows the same test names as before, with only the new introspect test added, which the request requires. Report the counts.
2. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both are clean, and no `#[allow]` is added.
3. [content] `crates/athena-core/src/gh.rs` no longer exists and `crates/athena-core/src/gh/mod.rs` does. `git diff -M` shows it as a rename whose only changed lines, if any, are `use` lines, and moved text is identical.
4. [behavior] A new test in introspect.rs shows that a landing touching a moved file under `crates/athena-core/src/gh/` still selects D87 and D88 through their `paths`. Run `cargo test --locked -p athena-core introspect` to see it pass. It would fail if `paths` still named only the old `gh.rs`.
5. [content] D87 and D88 in `introspect.rs` list `crates/athena-core/src/gh/` and no longer list the exact path `crates/athena-core/src/gh.rs`.
6. [behavior] external_calls_guard.rs `WRAPPER` is the prefix `crates/athena-core/src/gh/`. Run `cargo test --locked -p athena-core --test external_calls_guard`. It passes on the real tree, still fails its self-test when a `Command::new("gh")` line is added outside `gh/`, and passes with the call in a file under `gh/`. Test text changes only where the constant is compared.
7. [content] The report lists the caller check for each moved name: Grep of `gh::` names such as GhClient, PrHost, land_pr, LandPr, PrReport, Poll, landed_commit, Runner and PagesHost over `crates/`, with file counts before and after (equal). It also lists the Grep of `gh.rs`, `include_str!` and `#[path]` hits, which find only prose, history or fixture text.
8. [content] docs/decisions.md D138 is marked accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26). It states that module names and order are approved, R0 is approved, all jobs are queued now with dependencies, and the guard constant may change to the prefix. No other decision is edited.
9. [content] CHANGELOG.md has an entry under [Unreleased] naming the `gh.rs` to `gh/mod.rs` rename, the D87/D88 path change, the guard prefix, and that behavior is unchanged.
10. [content] `git diff --stat` lists only the six requested paths (the rename counts as gh.rs and gh/mod.rs). No config, workflow or defaults file changes.

## Expected deliverables

- crates/athena-core/src/gh.rs (delete)
- crates/athena-core/src/gh/mod.rs (create)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/tests/external_calls_guard.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/structure-pass.md (modify)

## Out of scope

- Moving any code out of gh/mod.rs (G2-G8)
- Any run.rs change (R0-R9)
- Queueing the jobs or editing athena.toml, defaults.toml or workflows
- Any behavior change
- Editing existing tests beyond import lines and the guard constant

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and the new test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass, and a new test covers the D87/D88 selection.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the cargo test, clippy and fmt commands and the guard's self-tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check that the guard still fails on an outside `gh` call and that the D87/D88 selection holds, not just that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rename job. The moves stay in later jobs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision entry and the CHANGELOG are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The guard moves to a directory prefix so later splits do not break it again.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is unchanged behavior, shown by an equal test list, equal caller counts and passing tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 60f5dd0fa246 → candidate 83b0d4028a5c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |  9 +++++
 crates/athena-core/src/{gh.rs => gh/mod.rs}      |  0
 crates/athena-core/src/introspect.rs             | 45 ++++++++++++++++++++++--
 crates/athena-core/tests/external_calls_guard.rs | 17 ++++-----
 docs/decisions.md                                |  9 +++--
 5 files changed, 67 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1395 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
