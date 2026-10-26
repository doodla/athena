# Job 20261026-104801-0pwx: Add code-health check, status table, and 10-landing design review

## Request (verbatim)

> try again on 20261026-100656-h9wq

## Reading (accepted by the user)

Retry of job 20261026-100656-h9wq, keeping its accepted design. Add code_health.rs (syn-measured production lines and longest function, 7-day churn, failure citations, test run time), show it in athena status, queue one deduplicated structural-pass request per breach after each landing, and queue a self-contained high-priority design review every 10 landings, anchored by a typed baseline record. Tests must not hit the disk guard (set the minimum to 0, as fixed in 3947eb3).

Assumptions:
- Audience: Sri (user) and the verifier; goal is that code health is watched automatically.
- The prior design is kept as accepted; no design-review role exists, so the queued request carries its full content.
- Disk-guard fix 3947eb3 is present in the base; nested-run tests set the minimum to 0.
- Adding syn to athena-core is the buy-over-build choice for function length.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
2. [behavior] Run cargo test: tests show each threshold breach (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
3. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; a second run, or an open structural-pass request for the file, queues none.
4. [behavior] Run cargo test: a baseline record (new AnyRecord variant in crates/athena-data/src/records.rs, with a schema round-trip test) is written, and a test reloads Records from git into a fresh state directory and shows the baseline survives. The first landing sets the baseline and fires no review.
5. [behavior] Run cargo test: the design-review request is queued through Athena::queue at high priority on the 10th landing after the baseline, and not on the 9th.
6. [behavior] Run cargo test: the queued design-review text names what to review (code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
7. [behavior] Run cargo test: tests show the check runs after a landing in both Athena::land_pr (after resync_onto_merge and the Pages watch) and note_finished in steward.rs, and each test fails without its hook. Tests set the disk-guard minimum to 0.
8. [behavior] Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time, with aligned headers.
9. [behavior] Run cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked: all three exit 0.
10. [content] CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length (syn), the baseline record, and the 10-landing cadence.

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
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cli/tests/code_health_hooks.rs (create)

## Out of scope

- Building a design-review role or running the structural pass itself
- Changing athena.toml or defaults.toml
- Backfilling the cadence from existing landing history

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests; fmt, clippy, test run.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use syn and record the call in docs/decisions.md.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise queuing, hooks, and status output.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Status output verified by rendered-output test.
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
Run cargo test: tests show each threshold breach (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; a second run, or an open structural-pass request for the file, queues none.
Run cargo test: a baseline record (new AnyRecord variant in crates/athena-data/src/records.rs, with a schema round-trip test) is written, and a test reloads Records from git into a fresh state directory and shows the baseline survives. The first landing sets the baseline and fires no review.
Run cargo test: the design-review request is queued through Athena::queue at high priority on the 10th landing after the baseline, and not on the 9th.
Run cargo test: the queued design-review text names what to review (code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
Run cargo test: tests show the check runs after a landing in both Athena::land_pr (after resync_onto_merge and the Pages watch) and note_finished in steward.rs, and each test fails without its hook. Tests set the disk-guard minimum to 0.
Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time, with aligned headers.
Run cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked: all three exit 0.
CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length (syn), the baseline record, and the 10-landing cadence.

## Report

End with the producer report as your structured output.
