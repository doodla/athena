# Job 20261026-150803-c7d4: Resume planning-capped jobs from the kept plan; report planning cost per

## Request (verbatim)

> Priority: urgent
> A plan that stops at the three-round cap with only a small fix left throws away the whole planning spend, and the retry starts over from a fresh reading and plan. Today this happened three times (20261026-091156-d0k9, 20261025-001236-brde, 20261026-145955-s7e2): each round-3 verdict was "revise" with one major finding the critic itself called small, plus minors. Make a planning stop resumable from its last plan: when planning stops at the cap, keep the round-3 plan and the critic's findings with the job, and have `athena retry` of a planning-stopped job start the planner from that plan plus the findings (one revision round, then the critic) instead of a fresh plan. Measure the effect: report in athena status the planning rounds and planning cost spent per landed job, before and after, from the job records.
> Acceptance:
> - A test stops a job at the planning cap with a fixture critic; `athena retry` of it gives the planner the kept plan and the critic findings, and the first critic round of the retry sees the revised plan.
> - A test shows a retry of a job that stopped for any other reason still starts with a fresh plan.
> - athena status shows planning rounds and planning cost per landed job, computed from records.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, CHANGELOG.md

## Reading (accepted by the user)

When planning stops at the three-round cap, keep the round-3 plan and the critic's findings with the job. `athena retry` of such a job seeds the planner with them (one revision round, then the critic) instead of a fresh plan. Retries of jobs stopped for any other reason still plan fresh. `athena status` reports planning rounds and planning cost per landed job from records. Audience: the Athena operator.

Assumptions:
- Goal: stop wasting planning spend when a plan hits the round cap with only small findings left. Audience: the Athena operator.
- 'Before and after' in status is read as a split of landed jobs by whether the planning-resume change applied. The split is a cutover point derived from records, and the values are labeled as measured.
- Kept plan and findings may need a new typed record in athena-data; if existing records already hold the plan and critique, no new variant is added.
- If the retried plan's base or request changed, resume still applies, since only planning-stopped jobs qualify.
- The default retry behavior for non-planning stops is unchanged.

## Acceptance criteria

1. [behavior] A test stops a job at the planning cap with a fixture critic. `athena retry` of it gives the planner the kept round-3 plan and the critic's findings. The first critic round of the retry sees the revised plan, and the retry does only one revision round before the critic. Run: cargo test --locked (named test).
2. [behavior] A test shows a retry of a job that stopped for any other reason (e.g. timeout, verification failure) still starts the planner with a fresh plan and receives no kept plan. Run: cargo test --locked.
3. [behavior] athena status prints planning rounds and planning cost per landed job, computed from job records. A test with fixture records asserts the printed numbers, including a before/after split. Run: cargo test --locked.
4. [content] The kept plan and findings are stored with the job as records, with no secrets or personal data.
5. [behavior] cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked all exit 0.
6. [content] CHANGELOG.md has an entry describing resumable planning stops and the new status measure.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing the planning round cap
- Changing how the critic grades findings
- Retry behavior for non-planning stops
- Backfilling records for old jobs beyond computing the measure from existing records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for resume, fresh retry, and status; fmt, clippy -D warnings and test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the planner input and the critic's first-round input, not just that fields exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Status measure reports rounds and cost per landed job from records.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the listed files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Kept plans contain no secrets.
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

A test stops a job at the planning cap with a fixture critic. `athena retry` of it gives the planner the kept round-3 plan and the critic's findings. The first critic round of the retry sees the revised plan, and the retry does only one revision round before the critic. Run: cargo test --locked (named test).
A test shows a retry of a job that stopped for any other reason (e.g. timeout, verification failure) still starts the planner with a fresh plan and receives no kept plan. Run: cargo test --locked.
athena status prints planning rounds and planning cost per landed job, computed from job records. A test with fixture records asserts the printed numbers, including a before/after split. Run: cargo test --locked.
The kept plan and findings are stored with the job as records, with no secrets or personal data.
cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked all exit 0.
CHANGELOG.md has an entry describing resumable planning stops and the new status measure.

## Report

End with the producer report as your structured output.
