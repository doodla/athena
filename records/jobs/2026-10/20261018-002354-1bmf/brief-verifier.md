# Job 20261018-002354-1bmf: verify "Add revise/stop paths and Athena's own plan checks to job planning"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 151f0e43433e → candidate 58e6b67eec38. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  16 +
 crates/athena-cli/tests/common/mod.rs |  45 ++-
 crates/athena-cli/tests/job_plan.rs   | 558 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/admission.rs   | 151 ++++++++-
 crates/athena-core/src/reading.rs     |  38 ++-
 crates/athena-core/src/run.rs         | 267 ++++++++++++----
 crates/athena-score/src/lib.rs        |  12 +-
 docs/design/job-planning.md           |   5 +
 8 files changed, 1029 insertions(+), 63 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (922 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
