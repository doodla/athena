# Job 20261018-002354-1bmf: Add revise/stop paths and Athena's own plan checks to job planning

## Request (verbatim)

> Build job 4 of docs/design/job-planning.md (revision 2, section 8): the revise and stop paths of job planning, and Athena's own checks on a plan. Read the design's sections 3 and 5 and review finding F11 in docs/design/reviews/job-planning-review-1.md. Jobs 3a and 3b built the accept path.
> Plan: (1) Rounds: up to max_rounds (default 3). A revise sends a revision brief that quotes the previous plan and every finding; reject, a reading that misread the request, or revise in the last round stops the job. (2) Athena's own checks run on each plan before the critic, at no model cost: every criterion has exactly one demonstration; every behavior demonstration has fails_without and a command the verifier's grant allows (athena_agent::policy::evaluate); files to modify exist and files to create do not; a governance path (admission::is_governance) must pass governance_change_is_listed, else a major finding; another path outside the deliverables is a minor finding. A major finding sends the plan back without a critic call. (3) A stopped job fails through failure() with the reason "planning stopped before any code was written:" and the findings; its verification_passed is unknown, not false (F11). (4) A retry's carry (earlier_attempt in crates/athena-core/src/reading.rs) quotes the last critique's findings (F11).
> Acceptance:
> - cargo test: after revise then accept, the second planner prompt quotes the first finding's text, and the job lands.
> - cargo test: when every round says revise, no producer or verifier runs; the outcome is failed with the "planning stopped" reason; three plans and three critiques are committed; the totals include all six planning invocations; verification_passed is unknown; check_history() is clean.
> - cargo test: a plan that edits an unlisted athena.toml, or cites a command outside the verifier's grant, gets an Athena critique with a major finding and no critic invocation.
> - cargo test: a retry of a job stopped in planning carries the last critique's findings into the new reading's brief.
> - cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: status display, recovery of a job killed during planning, and athena.toml.

## Reading (accepted by the user)

Implement job 4 of the job-planning design: a multi-round planner loop (revise with a quoting brief; stop on reject, misread, or last-round revise), free Athena checks before the critic, a planning-stopped failure with unknown verification, and retry carry of the last critique's findings. Audience: the maintainer verifying via cargo test and the design's F11.

Assumptions:
- Job 3a/3b accept path lives in crates/athena-core/src/run.rs and plan.rs; the new loop extends it rather than replacing it.
- max_rounds already exists or is added with default 3 in config; athena.toml itself is not edited.
- Unknown verification_passed is represented by an existing optional/None value in the outcome record.
- Tests go in crates/athena-cli/tests/job_plan.rs using the fake agent.

## Acceptance criteria

1. [behavior] Run cargo test: a test where the planner revises then accepts shows the second planner prompt contains the first finding's text, and the job lands.
2. [behavior] Run cargo test: a test where every round says revise asserts no producer or verifier invocation, outcome failed with reason starting 'planning stopped before any code was written:' plus the findings, three plans and three critiques committed, totals counting all six planning invocations, verification_passed unknown (not false), and check_history() clean.
3. [behavior] Run cargo test: a plan editing an unlisted athena.toml, or citing a command outside the verifier's grant (per athena_agent::policy::evaluate), yields an Athena critique with a major finding and zero critic invocations.
4. [behavior] Run cargo test: a retry of a job stopped in planning carries the last critique's findings into the new reading's brief via earlier_attempt.
5. [content] Athena checks cover: exactly one demonstration per criterion; behavior demos have fails_without and an allowed command; modify paths exist and create paths do not; governance paths use admission::is_governance and governance_change_is_listed (major otherwise); other unlisted paths are minor findings.
6. [content] Rounds are bounded by max_rounds (default 3); reject, a misread reading, or revise in the last round stops the job; the revision brief quotes the previous plan and every finding.
7. [behavior] Run cargo clippy --all-targets -- -D warnings, cargo fmt --check, and the xtask boundary test (cargo test -p xtask); all pass.
8. [content] CHANGELOG.md has an entry describing the revise/stop paths and Athena's plan checks; athena.toml is unchanged.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Status display of planning
- Recovery of a job killed during planning
- Changes to athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New cargo tests for each path; clippy and fmt required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests checking the behaviors, not just existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the planning loop and checks.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
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
