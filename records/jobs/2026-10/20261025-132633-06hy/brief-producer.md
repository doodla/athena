# Job 20261025-132633-06hy: Wait only for the 'fmt, clippy, tests' check run before landing push

## Request (verbatim)

> Priority: urgent
> The wait before the landing push (gh.rs around line 2048) stops at the first failed check run on the early candidate, but on a candidate 'the full history' and 'the pull request head' always fail (a candidate is not a landing), so it never waits for the job the cache serves: on PR for job 20261025-033731-5hac it gave up after 140 s with 'a run failed' while 'fmt, clippy, tests' passed at 13:10, and the landing took 18 minutes again. Wait only for the check run named 'fmt, clippy, tests' on the candidate; ignore the other jobs' conclusions there.
> Acceptance:
> - A fake-host test where the candidate's history job fails and 'fmt, clippy, tests' is still running shows land waiting for 'fmt, clippy, tests' to complete before pushing.
> - A test where 'fmt, clippy, tests' itself failed pushes at once.
> Files: crates/athena-core/src/gh.rs, CHANGELOG.md

## Reading (accepted by the user)

The pre-push wait in gh.rs (wait_for_early_run) ends at any failed workflow run on the early candidate. Because the candidate's history and PR-head jobs always fail, it never waits for the cached job. Read the candidate's check run named 'fmt, clippy, tests' only: wait until it completes, push at once if it failed, and ignore the other jobs' conclusions. The audience is the maintainer running land, who wants it to stop taking 18 minutes.

Assumptions:
- Workflow-run data (ci_runs) cannot name a job, so a PrHost method reads the commit's check runs (the check-runs endpoint already used by checks) and picks the one named 'fmt, clippy, tests'; all PrHost implementations (real, test_support fakes, gh.rs test hosts) are updated.
- A missing named check run is treated like no runs: push at once; timeout and unreadable replies behave as before.
- The 20-second step and the bound are unchanged.

## Acceptance criteria

1. [behavior] A new fake-host test in gh.rs: the candidate has a failed history job and 'fmt, clippy, tests' in progress then completed; land polls (sleeps) until 'fmt, clippy, tests' completes and only then replaces the early head. Run cargo test -p athena-core and see it pass.
2. [behavior] A new test: 'fmt, clippy, tests' on the candidate is already failed, with the other jobs in any state; land pushes at once with no sleeping and the note says that job failed. Run cargo test -p athena-core.
3. [behavior] Existing early-run tests (waits before pushing, pushes at once when completed, pushes anyway on timeout, unreadable or no runs) still pass, and cargo clippy and cargo fmt --check are clean.
4. [content] wait_for_early_run no longer ends on any failed workflow run; it decides only from the check run named 'fmt, clippy, tests' (a constant), found through the host seam, and its doc comment says so and why (a candidate is not a landing).
5. [content] CHANGELOG.md has an entry describing the changed wait.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing the CI workflow or job names
- Changing wait_for_checks after the landing push
- Changing the timeout length

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise wait versus push-at-once through the fake host.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the wait's selection of runs, not the timeout.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in gh.rs and the changelog.
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

A new fake-host test in gh.rs: the candidate has a failed history job and 'fmt, clippy, tests' in progress then completed; land polls (sleeps) until 'fmt, clippy, tests' completes and only then replaces the early head. Run cargo test -p athena-core and see it pass.
A new test: 'fmt, clippy, tests' on the candidate is already failed, with the other jobs in any state; land pushes at once with no sleeping and the note says that job failed. Run cargo test -p athena-core.
Existing early-run tests (waits before pushing, pushes at once when completed, pushes anyway on timeout, unreadable or no runs) still pass, and cargo clippy and cargo fmt --check are clean.
wait_for_early_run no longer ends on any failed workflow run; it decides only from the check run named 'fmt, clippy, tests' (a constant), found through the host seam, and its doc comment says so and why (a candidate is not a landing).
CHANGELOG.md has an entry describing the changed wait.

## Report

End with the producer report as your structured output.
