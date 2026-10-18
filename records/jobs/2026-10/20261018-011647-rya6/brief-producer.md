# Job 20261018-011647-rya6: Show job planning in athena status; fix recovery of jobs killed in

## Request (verbatim)

> Build job 5a of docs/design/job-planning.md (revision 2, section 8): show job planning in athena status, and recover a job killed during planning correctly. Jobs 1 to 4 built the stage behind [job_plan].
> Plan: (1) In crates/athena-core/src/status.rs and crates/athena-cli/src/display.rs, a planned job shows "plan accepted r<n>" or "plan stopped r<n>", the summary shows first-round accept and stop counts, and `athena status <job>` lists each round with its verdict and findings; `status <job> --json` includes the plans and critiques. Jobs without a plan show nothing new. (2) In crates/athena-core/src/manage.rs, recover_interrupted must not invent a "missing producer" invocation for a job killed during planning.
> Acceptance:
> - cargo test: a planned job's status line shows its plan state and round; an unplanned job's status output is byte-identical to today's.
> - cargo test: `status <job> --json` for a planned job includes every round's plan and critique, with verdicts and findings.
> - cargo test: closing a job whose process died during planning records the planning invocations it has and adds no producer invocation.
> - cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: athena.toml and turning planning on.

## Reading (accepted by the user)

Job 5a of the job-planning design. Planned jobs get a plan state and round in the status line, first-round accept and stop counts in the summary, per-round verdicts and findings in `status <job>`, and plans and critiques in `--json`. `recover_interrupted` stops adding a "missing producer" invocation when the job died in planning. The audience is a maintainer running `athena status` and `athena close`.

Assumptions:
- Planner and Critic roles and plan/critique records already exist from jobs 1 to 4; this job only reads them.
- Recovery fix: skip the missing-producer record when any Planner or Critic invocation exists and no producer ran (a job that died while planning).
- Unplanned output stays byte-identical because new fields are skipped when empty.

## Acceptance criteria

1. [behavior] Run `cargo test`. A test shows a planned job's status line contains "plan accepted r<n>" or "plan stopped r<n>" with the correct round, and the summary shows first-round accept and stop counts.
2. [behavior] Run `cargo test`. A test compares an unplanned job's status output with its pre-change text and finds it byte-identical, with no new lines or columns.
3. [behavior] Run `cargo test`. A test shows `athena status <job>` for a planned job lists each round with its verdict and findings.
4. [behavior] Run `cargo test`. A test shows `status <job> --json` for a multi-round planned job includes every round's plan and critique, with verdicts and findings, and that an unplanned job's JSON gains no new keys.
5. [behavior] Run `cargo test`. A test closes a job whose process died during planning (planner or critic records or usage partials exist, no producer). The planning invocations are recorded and no Missing producer invocation is added. A job with no trace of any invocation still gets the existing missing-producer behavior.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and xtask's boundary test (`cargo test -p xtask`). All pass.
7. [content] CHANGELOG.md has an entry describing planning in status and the recovery fix.
8. [content] athena.toml is unchanged and planning is not turned on.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- athena.toml
- turning planning on
- job 5b and later jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests; clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered output and recorded invocations.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only status, display and manage changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands.
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

## Report

End with the producer report as your structured output.
