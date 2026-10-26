# Job 20261026-091156-d0k9: Add code-health check, status table and design-review cadence

## Request (verbatim)

> try again on 20261026-064950-38jx

## Reading (accepted by the user)

Retry of 20261026-064950-38jx for Sri and the verifier. Add code_health.rs (per-file production lines, longest function, 7-day churn, failure citations, test run time), show it in athena status with thresholds, queue one structural-pass request per breaching file after each landing, and queue a self-contained design-review request every 10 landings. The earlier design was sound; it failed only on a disk-guard test, since fixed in 3947eb3.

Assumptions:
- Keep the earlier attempt's design; the rejection was environmental (low-disk guard), and tests now set the disk minimum to 0 per 3947eb3.
- Tests for the finished-job hook live in a code_health::hooks module or a test file; any extra test file is accepted.
- The first-review baseline should be derived from records where possible, not a state-directory file, so cadence survives state loss.
- The design-review request text is self-contained because no design-review role exists yet.
- Table header columns are aligned to their labels (the earlier nit).
- Audience is Sri and the verifier.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
2. [behavior] Run cargo test: tests show a breach of each threshold (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
3. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file citing the measured numbers; running again, or with an open structural-pass request for it, queues none.
4. [behavior] Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th; with no earlier review request the first review does not fire at once on existing history.
5. [behavior] Run cargo test: the queued design-review text names what to review (code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
6. [behavior] Run cargo test: a test of the Athena::land_pr path shows the code-health check runs after a landing and fails without the hook; the finished-job hook is likewise shown.
7. [behavior] Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time, with header columns aligned to labels.
8. [behavior] Run cargo clippy with -D warnings, cargo fmt --check and cargo test: all pass, and the evidence cites these commands as run.
9. [content] CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Expected deliverables

- crates/athena-core/src/code_health.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cli/tests/code_health_hooks.rs (create)

## Out of scope

- Building a design-review role or agent
- Changing the low-disk guard or the check/verify stage handling of environment holds
- Performing the structural passes or the design review themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy -D warnings and fmt --check to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check measures, breaches, queued requests and rendered output by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that breaches yield exactly one queued request and the cadence fires at the 10th landing.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: decisions.md records the call on measuring function length (syn vs hand-rolled).
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Evidence must cite the cargo commands that ran.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
Run cargo test: tests show a breach of each threshold (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
Run cargo test: on a breach exactly one structural-pass request is queued for that file citing the measured numbers; running again, or with an open structural-pass request for it, queues none.
Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th; with no earlier review request the first review does not fire at once on existing history.
Run cargo test: the queued design-review text names what to review (code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
Run cargo test: a test of the Athena::land_pr path shows the code-health check runs after a landing and fails without the hook; the finished-job hook is likewise shown.
Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time, with header columns aligned to labels.
Run cargo clippy with -D warnings, cargo fmt --check and cargo test: all pass, and the evidence cites these commands as run.
CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Report

End with the producer report as your structured output.
