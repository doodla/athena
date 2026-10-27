# Job 20261026-232423-hvyb: Move run.rs to run/mod.rs with no behavior change

## Request (verbatim)

> Priority: high
> Structure pass job R1 (docs/design/structure-pass.md, section 4). Make run.rs the directory run/ (rename to run/mod.rs). No behavior change. Depends on 20261026-232420-ctt9.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/run/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass R1: rename crates/athena-core/src/run.rs to run/mod.rs unchanged, so the `run` module path stays the same. The only text edits are the two test-module `include_str!` paths, which need one more `../`. The audience is the owner and verifier applying the section 5 proof.

Assumptions:
- Module path `run` is unchanged by the rename, so no re-exports are needed.
- The two include_str! paths in the test module (athena.toml, defaults.toml) are the only non-import edits and are necessary because the file is one directory deeper.
- Depends on the R0 test already being in run.rs; the move carries it along.
- Prose mentions of run.rs elsewhere (comments, fixture strings) are left alone unless they are live path references.

## Acceptance criteria

1. [content] crates/athena-core/src/run.rs no longer exists and crates/athena-core/src/run/mod.rs holds its text; `git diff -M` shows a rename whose only changed lines are the include_str! paths gaining one `../`.
2. [content] No file outside crates/athena-core/src/run/ names a moved item by an old module path: a Grep for `run::` and `mod run` over crates/ shows the same paths resolving, and `pub mod run;` in lib.rs is unchanged.
3. [behavior] Run `cargo test --locked -p athena-core -- --list` on the change and on the parent commit (git show / diff); the test names are identical in count and content.
4. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and no `#[allow]` is added.
5. [content] CHANGELOG.md has an entry saying run.rs became run/mod.rs with no behavior change.

## Expected deliverables

- crates/athena-core/src/run.rs (delete)
- crates/athena-core/src/run/mod.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Splitting run/mod.rs into submodules (later structure-pass jobs)
- Editing tests other than the two include_str! paths
- Changing athena.toml, defaults.toml or workflows
- Deduplicating the R0 test helpers

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check unchanged test list and passing builds, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Pure rename plus changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the test list comparison and grep itself.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test list comparison and grep show no behavior change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
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

16 more qualifying classes are left out.

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

crates/athena-core/src/run.rs no longer exists and crates/athena-core/src/run/mod.rs holds its text; `git diff -M` shows a rename whose only changed lines are the include_str! paths gaining one `../`.
No file outside crates/athena-core/src/run/ names a moved item by an old module path: a Grep for `run::` and `mod run` over crates/ shows the same paths resolving, and `pub mod run;` in lib.rs is unchanged.
Run `cargo test --locked -p athena-core -- --list` on the change and on the parent commit (git show / diff); the test names are identical in count and content.
`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and no `#[allow]` is added.
CHANGELOG.md has an entry saying run.rs became run/mod.rs with no behavior change.

## Report

End with the producer report as your structured output.
