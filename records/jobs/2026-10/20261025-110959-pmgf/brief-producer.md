# Job 20261025-110959-pmgf: Wait for the early CI run before pushing the landing commit

## Request (verbatim)

> Priority: urgent
> Cut A misses its cache by a race: the early pull request opens before Athena's checks (run.rs:1645), but its 'fmt, clippy, tests' CI run takes about 16 minutes while Athena's checks and verifier take about 14, so the landing commit is pushed and its CI run starts before the early run's pass is cached (PR #180: early run passed 10:46:49, landing run started 10:45:15 and ran the full 16 minutes). Before land pushes the landing commit, it waits for the early run's 'fmt, clippy, tests' on the candidate's commit to finish (with a bounded wait, for example 10 minutes, then it pushes anyway), so the landing run finds the cached pass. If the early run failed, land pushes as today and the full run decides.
> Acceptance:
> - A fake-host test shows land waiting for the candidate's check run to complete before pushing the landing commit, and pushing at once when it has already completed.
> - A test shows the bounded wait expiring pushes anyway, and a failed early run changes nothing about merging.
> - CHANGELOG.md records the next real landing's time, measured.
> Files: crates/athena-core/src/gh.rs, crates/athena-core/src/land.rs, CHANGELOG.md

## Reading (accepted by the user)

When land replaces the early-pushed head with the landing commit (gh.rs land_pr/replace_early), it first waits up to a bounded time (about 10 minutes) for the early commit's workflow run to complete, so the landing run hits the cached pass. If the wait expires or the early run failed, it pushes as today. Audience: Athena maintainers and anyone watching landing CI time.

Assumptions:
- The landing push the request describes is in gh.rs publish/land_pr via replace_early, not in land.rs, which holds planning and classification; land.rs is listed because the request names it, and likely stays unchanged.
- Completion is read from the existing PrHost::ci_runs (CiRunState status == completed) for the early commit. The request's 'fmt, clippy, tests' run is the commit's workflow run, so no new host call is needed; if per-run-name filtering is needed, gh.rs gets a small addition.
- Wait and sleep reuse the existing Poll sleep injection, with a new bounded-wait setting defaulting to 10 minutes.
- The criterion about the next real landing's time cannot be produced by the producer: it needs a landing that happens after this change. The CHANGELOG will say the measurement is pending, not carry a made-up number.

## Acceptance criteria

1. [behavior] A fake-host test (run with cargo test -p athena-core) shows land_pr, with the early head's CI run still in progress, polling ci_runs on the early commit and pushing the landing commit only after the run reports completed.
2. [behavior] A fake-host test shows that when the early run is already completed, the landing push happens at once with no sleep calls.
3. [behavior] A test shows that when the run never completes within the bounded wait, the landing commit is still pushed, and that the wait is bounded (default about 10 minutes, injectable so the test needs no real sleeping).
4. [behavior] A test shows that a failed (or timed-out) early run, and an early commit with no runs or a ci_runs error, change nothing about merging: land pushes and merges exactly as before and the full landing run decides.
5. [behavior] cargo test and cargo clippy --all-targets are clean, and cargo fmt --check passes; existing land_pr and publish tests still pass.
6. [content] The wait applies only when an early head exists and differs from the landing commit; the wait is logged or reported in a way a reader can see how long it waited and why it stopped.
7. [content] CHANGELOG.md has an entry for this change that says the next real landing's time is still to be measured. It must not state an invented number. The measured time is recorded only once a real landing has been observed, with its source (for example the PR number and the run timestamps).

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing CI workflows or the early-PR opening order at run.rs:1645
- Re-running or fixing failed early runs
- Changing merge method or the existing check-wait and re-run logic in wait_for_checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-host tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that push ordering and timing happen in fake-host tests, not that fields exist.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: CHANGELOG number must be measured and sourced, otherwise marked pending.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the landing path.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the race by waiting for the cache-producing run, not by shortening the CI.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
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

A fake-host test (run with cargo test -p athena-core) shows land_pr, with the early head's CI run still in progress, polling ci_runs on the early commit and pushing the landing commit only after the run reports completed.
A fake-host test shows that when the early run is already completed, the landing push happens at once with no sleep calls.
A test shows that when the run never completes within the bounded wait, the landing commit is still pushed, and that the wait is bounded (default about 10 minutes, injectable so the test needs no real sleeping).
A test shows that a failed (or timed-out) early run, and an early commit with no runs or a ci_runs error, change nothing about merging: land pushes and merges exactly as before and the full landing run decides.
cargo test and cargo clippy --all-targets are clean, and cargo fmt --check passes; existing land_pr and publish tests still pass.
The wait applies only when an early head exists and differs from the landing commit; the wait is logged or reported in a way a reader can see how long it waited and why it stopped.
CHANGELOG.md has an entry for this change that says the next real landing's time is still to be measured. It must not state an invented number. The measured time is recorded only once a real landing has been observed, with its source (for example the PR number and the run timestamps).

## Report

End with the producer report as your structured output.
