# Job 20261027-055331-4w2w: Rebuild and re-exec Athena after drain lands changes to its own code

## Request (verbatim)

> Priority: urgent
> Let athena drain run unattended across its own landings. Today, when drain lands a change to Athena's own code or config schema, the running binary stays at the old build; the next landing or config read can fail (2026-10-26: 'unknown field hard_producer_effort' stopped the landing of 20261025-034237-2rep until ./setup.sh was rerun by hand), and nothing warns (the stale-build request of 2026-10-17 never landed). Fix: after drain lands a change touching crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh, it rebuilds and installs Athena from the landed tree (the same steps as ./setup.sh) and continues the run with the new binary (re-exec with the remaining budget and job limit); if the rebuild fails it stops with the stop reason 'rebuild failed' and the build output. athena status and every agent-running command (go, drain, audit, audience) print one line when the installed binary's build commit is older than HEAD for those paths.
> Acceptance:
> - A test drains two fake items where the first landing changes a crates/ file, and shows the rebuild step runs once and the second item runs under the new build commit.
> - A test shows a failed rebuild stops the drain with 'rebuild failed'.
> - A test shows status prints the stale-build line when a crates/ file changed after the build commit, and none when only docs/ or records/ changed.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

After drain lands a change touching crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh, it runs the setup.sh build/install steps, then re-execs with the remaining budget and job limit; a failed rebuild stops with 'rebuild failed' plus build output. status and agent-running commands (go, drain, audit, audience) print one stale-build line when the installed build commit is older than HEAD for those paths. Audience: the operator running drain unattended.

Assumptions:
- The build commit is already embedded in the binary (or is added at build time by the same steps as setup.sh); the producer finds how setup.sh records it.
- Rebuild and install are behind an injectable step so tests use a fake instead of running cargo.
- Re-exec carries remaining budget and job limit as arguments to the same drain command.
- The stale check compares the build commit with HEAD using git diff limited to crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh.

## Acceptance criteria

1. [behavior] Run cargo test: a test drains two fake items where the first landing changes a crates/ file; it shows the rebuild step ran exactly once and the second item ran under the new build commit.
2. [behavior] Run cargo test: a test shows a failed rebuild stops the drain with stop reason 'rebuild failed' and includes the build output.
3. [behavior] Run cargo test: a test shows status prints the stale-build line when a crates/ file changed after the build commit, and prints none when only docs/ or records/ changed.
4. [content] go, drain, audit and audience print the same one-line stale-build warning when the build commit is older than HEAD for the watched paths.
5. [content] The re-exec passes the remaining budget and job limit.
6. [behavior] cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
7. [content] CHANGELOG.md has an entry describing the self-rebuild and stale-build warning.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Changing setup.sh itself
- Rebuilding after landings that touch only docs/ or records/
- Changing config schema or the planner model settings

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for rebuild, failure and status line; fmt, clippy -D warnings and cargo test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check that the rebuild ran once and the second item ran under the new build commit, not only that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the drain continuing unattended across a landing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the named files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All Rust.
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

Run cargo test: a test drains two fake items where the first landing changes a crates/ file; it shows the rebuild step ran exactly once and the second item ran under the new build commit.
Run cargo test: a test shows a failed rebuild stops the drain with stop reason 'rebuild failed' and includes the build output.
Run cargo test: a test shows status prints the stale-build line when a crates/ file changed after the build commit, and prints none when only docs/ or records/ changed.
go, drain, audit and audience print the same one-line stale-build warning when the build commit is older than HEAD for the watched paths.
The re-exec passes the remaining budget and job limit.
cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
CHANGELOG.md has an entry describing the self-rebuild and stale-build warning.

## Report

End with the producer report as your structured output.
