# Job 20261017-221628-by6t: Thread one accepted planning round through dispatch behind [job_plan]

## Request (verbatim)

> Build job 3a of docs/design/job-planning.md (revision 2, section 8): thread one planning round through dispatch, the producer and the closing commit, behind [job_plan] enabled, so the plumbing is right before the full stage is built. Read the design's section 5 ("Plumbing") and review finding F1 and F6 in docs/design/reviews/job-planning-review-1.md. Jobs 1 and 2 added the records, roles, config, prompts, schemas and briefs.
> Plan: (1) In crates/athena-core/src/run.rs, when the job's config has [job_plan] enabled = true and the capability does not pin a repository, dispatch runs one planning round after the prediction commit and before produce_and_verify: a planner invocation and a critic invocation through agent_spec and invoke_and_record with the briefs from job 2, recorded as Role::Planner and Role::Critic. Accept only; revise and stop paths are job 4. (2) The round returns a Closing seeded with the two invocations and the job-plan-1 and critique-1 records; produce_and_verify continues from that Closing instead of a new one, numbering the producer's invocation after them (F1). (3) dispatch recomputes spent after planning, so the producer's budget check counts it (F1). (4) The producer's prompt is brief-producer.md plus render_plan_section of the accepted plan, and the send-back is built from that same augmented prompt (F6); the job-plan record stores producer_section_sha256. (5) close_dispatched writes the plan and critique records with the invocations. With planning off or absent, nothing changes.
> Acceptance:
> - cargo test with the fake driver and a fixture config with [job_plan] enabled = true: the invocations are 01 reader, 02 planner, 03 critic, 04 producer, 05 verifier, and the landing commit holds job-plan-1.json, critique-1.json and all five invocation records; check_history() is clean.
> - cargo test: the budget check before the producer sees the planning invocations' cost (a fixture cap set between "producer alone" and "planning plus producer" stops the job before the producer, naming the budget).
> - cargo test: a forced send-back's second producer prompt still contains the "Job plan (accepted, round 1)" section, and the job-plan record's producer_section_sha256 equals the sha256 of that section.
> - cargo test: with [job_plan] absent or enabled = false, and for a study job, the invocations and records are exactly today's; every existing test passes.
> - cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: revise and stop paths, Athena's own checks on the plan, the verifier's plan input (job 3b), status display, and athena.toml.

## Reading (accepted by the user)

Implements job 3a of the job-planning design: when [job_plan] enabled and the capability doesn't pin a repository, dispatch runs one planner and one critic invocation (accept-only) after the prediction commit, seeds a Closing with them, and produce_and_verify continues from it. Audience is the Athena maintainer and verifier, who check invocation numbering, budget, send-back prompt and landing-commit contents with the fake driver.

Assumptions:
- Test code lives in or alongside crates/athena-core/src/run.rs (existing fake-driver harness); no new crate dependencies.
- Planner output is treated as accepted when the critic returns an accept verdict; non-accept verdicts are not handled (job 4) and may return a clear error.
- Producer invocation number follows from the seeded Closing's count, per review finding F1.
- Fixture config is built inside tests, so athena.toml is not touched.

## Acceptance criteria

1. [behavior] Run cargo test: a fake-driver test with a fixture config having [job_plan] enabled = true yields invocations 01 reader, 02 planner, 03 critic, 04 producer, 05 verifier; the landing commit contains job-plan-1.json, critique-1.json and all five invocation records; check_history() is clean.
2. [behavior] Run cargo test: with a fixture cap between producer-alone cost and planning-plus-producer cost, the job stops before the producer is invoked and the failure names the budget, proving spent is recomputed after planning.
3. [behavior] Run cargo test: on a forced send-back the second producer prompt still contains the 'Job plan (accepted, round 1)' section, and job-plan-1.json's producer_section_sha256 equals the sha256 of the rendered section.
4. [behavior] Run cargo test: with [job_plan] absent, enabled = false, or for a study job (or a capability pinning a repository), the invocations and records are identical to current behavior; all existing tests pass.
5. [behavior] cargo clippy --all-targets -- -D warnings and cargo fmt --check pass, and xtask's boundary test passes under cargo test.
6. [content] Planner and critic are recorded through agent_spec and invoke_and_record as Role::Planner and Role::Critic using the job-2 briefs; the producer prompt is brief-producer.md plus render_plan_section, and the send-back is built from that same augmented prompt.
7. [content] CHANGELOG.md has an entry describing the planning round plumbing; athena.toml, defaults.toml and workflows are unchanged.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Revise and stop paths of planning (job 4)
- Athena's own checks on the plan
- Verifier's plan input (job 3b)
- Status display
- Changes to athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-driver tests plus clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check invocation order, budget stop, prompt content and hash rather than wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the accept path in run.rs; no revise/stop.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes F1 and F6 by seeding Closing and recomputing spent, not special-casing.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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
