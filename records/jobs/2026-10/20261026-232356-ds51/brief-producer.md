# Job 20261026-232356-ds51: Make gh.rs the gh/ directory (G1) and accept the structure-pass decision

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 29 cases in 29 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-handover check for reader-move jobs. It would enumerate the readers by grep and require, for each, a named test that calls the reader's real entry point. It would also run a mutation step: revert each reader to the in-tree path and confirm its test fails. The producer would have to pass th...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

15 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked`. It passes, and `cargo test --locked -p athena-core -- --list` shows the same test names as before, with only the new introspect test added, which the request requires. Report the counts.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both are clean, and no `#[allow]` is added.
`crates/athena-core/src/gh.rs` no longer exists and `crates/athena-core/src/gh/mod.rs` does. `git diff -M` shows it as a rename whose only changed lines, if any, are `use` lines, and moved text is identical.
A new test in introspect.rs shows that a landing touching a moved file under `crates/athena-core/src/gh/` still selects D87 and D88 through their `paths`. Run `cargo test --locked -p athena-core introspect` to see it pass. It would fail if `paths` still named only the old `gh.rs`.
D87 and D88 in `introspect.rs` list `crates/athena-core/src/gh/` and no longer list the exact path `crates/athena-core/src/gh.rs`.
external_calls_guard.rs `WRAPPER` is the prefix `crates/athena-core/src/gh/`. Run `cargo test --locked -p athena-core --test external_calls_guard`. It passes on the real tree, still fails its self-test when a `Command::new("gh")` line is added outside `gh/`, and passes with the call in a file under `gh/`. Test text changes only where the constant is compared.
The report lists the caller check for each moved name: Grep of `gh::` names such as GhClient, PrHost, land_pr, LandPr, PrReport, Poll, landed_commit, Runner and PagesHost over `crates/`, with file counts before and after (equal). It also lists the Grep of `gh.rs`, `include_str!` and `#[path]` hits, which find only prose, history or fixture text.
docs/decisions.md D138 is marked accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26). It states that module names and order are approved, R0 is approved, all jobs are queued now with dependencies, and the guard constant may change to the prefix. No other decision is edited.
CHANGELOG.md has an entry under [Unreleased] naming the `gh.rs` to `gh/mod.rs` rename, the D87/D88 path change, the guard prefix, and that behavior is unchanged.
`git diff --stat` lists only the six requested paths (the rename counts as gh.rs and gh/mod.rs). No config, workflow or defaults file changes.

## Report

End with the producer report as your structured output.
