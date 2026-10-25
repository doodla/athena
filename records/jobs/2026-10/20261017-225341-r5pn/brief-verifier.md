# Job 20261017-225341-r5pn: verify "Give the verifier the accepted plan's demonstrations and files"

## Request (verbatim)

> Build job 3b of docs/design/job-planning.md (revision 2, section 8): give the verifier the accepted plan's demonstrations and planned files, and prove the whole accept path end to end. Read the design's section 4 and review finding F5 in docs/design/reviews/job-planning-review-1.md. Job 3a threads one accepted planning round through dispatch and the producer.
> Plan: (1) When a job has an accepted plan, the verifier's brief uses verifier_brief's plan input from job 2: the critic-accepted demonstrations and the planned file list, never the approach, reasons or rejected options (F5); VerifierInputs::names records that input in verification.json. (2) Extend crates/athena-cli/tests/job_plan.rs (job 3a created it), turning planning on through Fixture::with_config, to check the accept path through the real lifecycle.
> Acceptance:
> - cargo test (crates/athena-cli/tests/job_plan.rs): with an accepting critic the roles run reader, planner, critic, producer, verifier in that order, and the job lands.
> - cargo test: the producer's prompt holds the planned files and each demonstration's command, and its plan section's sha256 equals the job-plan record's producer_section_sha256.
> - cargo test: the verifier's brief holds the demonstrations and the planned file list and not the plan's approach or decision reasons, and verification.json names the plan input.
> - cargo test: for a study job and with planning off, roles, briefs and records are exactly today's; every existing test passes.
> - cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: revise and stop paths, Athena's own checks on the plan, status display, and athena.toml.

## Reading (accepted by the user)

When a job has an accepted plan, run.rs builds the verifier brief with verifier_brief_with_plan from a VerifierPlan taken from the accepted JobPlan. That brief carries only demonstrations and planned files, and verification.json records the plan input through names_with_plan. job_plan.rs then proves the accept path end to end. Audience: the maintainer, who checks the tests and the verifier's brief.

Assumptions:
- Job 2 already provides verifier_brief_with_plan, VerifierPlan::from_plan and names_with_plan in athena-context. Job 3a already holds the accepted JobPlan in the run context. So the change is mostly wiring in run.rs plus tests.
- Fake planner and critic output (Scenario defaults) has non-empty demonstrations, files, approach and reasons. If it does not, the test scenario is extended to include them.
- The existing fake critic accepts by default. The test asserts the unique approach and reason strings are absent from the verifier brief.
- Goal: the verifier is anchored on the demonstrations and files the critic accepted, not on the producer-side reasoning. Audience: the maintainer reading job_plan.rs and verification.json.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test job_plan`: with an accepting critic the recorded roles run reader, planner, critic, producer, verifier in that order and the job lands (state Landed).
2. [behavior] Run `cargo test -p athena-cli --test job_plan`: the producer's prompt contains every planned file path and each demonstration's command. The sha256 of its plan section equals producer_section_sha256 in job-plan-1.json.
3. [behavior] Run `cargo test -p athena-cli --test job_plan`: the verifier's brief (seen_by_verifier) holds each demonstration and the planned file list. It holds none of the plan's approach text, decision reasons or rejected options (F5). The committed verification.json inputs include 'planned demonstrations and files'.
4. [behavior] Run `cargo test`: for a study job and with planning off, roles, briefs (including the byte-identical verifier brief) and verification.json inputs are exactly as before. Every existing test passes.
5. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean, and xtask's boundary test passes under `cargo test`.
6. [content] The run.rs verifier path calls verifier_brief_with_plan for both the stored copy (inline_diff false) and the live brief, and it records names_with_plan in verification.json. Both use the accepted plan's VerifierPlan::from_plan, and with no plan they behave as before.
7. [content] CHANGELOG.md has an entry for job 3b, covering the verifier receiving demonstrations and planned files and the end-to-end accept-path test.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- revise and stop paths
- Athena's own checks on the plan
- status display
- athena.toml changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New and existing tests pass; clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check roles, prompt and brief contents and the record, by running the integration tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The accept path is exercised through the real lifecycle.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused wiring change plus tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 4e2a909aad46 → candidate 91420e4b40ab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   9 +++
 crates/athena-cli/tests/job_plan.rs | 128 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/run.rs       |  13 ++--
 docs/design/job-planning.md         |   4 ++
 4 files changed, 149 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (907 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
