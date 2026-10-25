# Job 20261024-145700-22mx: Add required kind to LandingCommit with per-kind validation

## Request (verbatim)

> Part 1 of the fix for job 20261024-140624-ewbs (which timed out as one job): in crates/athena-data/src/history.rs, give LandingCommit a required kind, a snake_case enum with job_landing, declared_intervention and records_only. A job_landing entry must carry its job and verification; the other two must not claim a job. Keep deny_unknown_fields. Update the validation and round-trip tests (each kind round-trips; an unknown kind, a missing kind, an unknown field, and a job_landing entry without job or verification are rejected), and update the one place in crates/athena-core/src/gh.rs that builds a LandingCommit so it sets job_landing, leaving pr_commits for part 2. Run only the athena-data and athena-core tests you need while working.

## Reading (accepted by the user)

Add a required snake_case `kind` enum (job_landing, declared_intervention, records_only) to LandingCommit in history.rs, make job/verification optional and enforced by validation per kind, update tests, and set job_landing in the one gh.rs builder. pr_commits stays for part 2.

Assumptions:
- Goal: let Landing records classify commits so part 2 can land PRs with records-only and intervention commits; audience is the athena maintainer and verifier.
- job and verification must become Option<..> on LandingCommit with skip_serializing_if, since non-job kinds must not claim a job; the validation lives in Landing::validate.
- Non-job kinds also must not carry a verification (reading 'must not claim a job' as covering both fields).
- Other users of Landing commits (manage.rs, display.rs) only pass them through and need no change; the existing gh.rs test constructor at ~3788 and history.rs test at ~411 need the new field.

## Acceptance criteria

1. [content] LandingCommit in history.rs has a required `kind` field of a snake_case enum with exactly job_landing, declared_intervention, records_only; deny_unknown_fields is kept; job and verification become optional (omitted when absent).
2. [behavior] Run `cargo test -p athena-data`: Landing::validate rejects a job_landing entry lacking job or verification, and rejects declared_intervention/records_only entries that carry a job or verification.
3. [behavior] Run `cargo test -p athena-data`: each kind round-trips through JSON; parsing rejects an unknown kind, a missing kind, and an unknown field.
4. [behavior] Run `cargo test -p athena-core gh`: the gh.rs commits() builder sets kind job_landing with job and verification, and existing landing tests (around line 3788) pass; pr_commits in land.rs is unchanged.
5. [behavior] `cargo clippy -p athena-data -p athena-core` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-data/src/history.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/manage.rs (modify)

## Out of scope

- Changing pr_commits in land.rs or content-based classification (part 2)
- Changes to athena-gate or the CLI display

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Update and add tests; run athena-data and athena-core tests, clippy clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only history.rs and the one gh.rs builder; pr_commits left for part 2.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rejection and round-trip behavior via cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
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

LandingCommit in history.rs has a required `kind` field of a snake_case enum with exactly job_landing, declared_intervention, records_only; deny_unknown_fields is kept; job and verification become optional (omitted when absent).
Run `cargo test -p athena-data`: Landing::validate rejects a job_landing entry lacking job or verification, and rejects declared_intervention/records_only entries that carry a job or verification.
Run `cargo test -p athena-data`: each kind round-trips through JSON; parsing rejects an unknown kind, a missing kind, and an unknown field.
Run `cargo test -p athena-core gh`: the gh.rs commits() builder sets kind job_landing with job and verification, and existing landing tests (around line 3788) pass; pr_commits in land.rs is unchanged.
`cargo clippy -p athena-data -p athena-core` is clean and `cargo fmt --check` passes.

## Report

End with the producer report as your structured output.
