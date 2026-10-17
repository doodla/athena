# Job 20261017-205545-ygfd: verify "Add job-planning records, roles and config (job 1, no behavior change)"

## Request (verbatim)

> Build job 1 of docs/design/job-planning.md (revision 2, section 8): the records, roles and config for job planning, with no change in behavior. Sri, 2026-10-17: "The producer verifier can be a planner critic loop." Read the design and its review, docs/design/reviews/job-planning-review-1.md (findings F10, F15, F16 apply to this job). Where the code disagrees with the design, follow the code and say so in the changelog.
> Plan: (1) In crates/athena-data/src/records.rs add the record types athena.job-plan/1 (at <job>/job-plan-<round>.json) and athena.critique/1 (at <job>/critique-<round>.json) with the fields in the design's sections 3 to 5, Role::Planner and Role::Critic for invocation records, and FindingSource::Critic (the critique reuses Finding). A critique whose final verdict is accept while it holds a blocker or major finding is invalid; round 0 is invalid. (2) In crates/athena-data/src/config.rs add Config.job_plan: Option<JobPlanConfig> with deny_unknown_fields and skip_serializing_if = "Option::is_none": enabled (default false), max_rounds, planner_usd, critic_usd and the two timeouts; reset it to None in reading_inputs, so every existing reading's config digest is unchanged. (3) In crates/athena-core/src/store.rs give job_of arms for the new record types. (4) Give the new roles arms in every exhaustive match: crates/athena-agent/src/fake.rs (a deterministic planner that covers every criterion in its brief, and a critic that accepts), crates/athena-core/src/run_pin_tests.rs, crates/athena-cli/tests/retry.rs and crates/athena-cli/tests/common/mod.rs. (5) Document the records in docs/records.md. Do not edit athena.toml or defaults.toml: the schema lands first.
> Acceptance:
> - Run `cargo test --locked`: every test passes, and a fake-driver `go` still runs only reader, producer and verifier.
> - cargo test: round-2 records pass check_record_file at job-plan-2.json and critique-2.json and are refused as misplaced at the round-1 paths; a round-0 record and an accepting critique with a major finding are refused by parse.
> - cargo test: reading_inputs() for the repository's athena.toml matches a sha256 digest recorded in the test from the code before this change, and adding [job_plan] with enabled = true leaves it unchanged; max_rounds = 0, a zero cap and an unknown key are each refused with a message naming the field.
> - cargo test: Records::load files committed plans and critiques under their job, and a dispatched job with a plan but no outcome stays dispatched.
> - cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean; xtask's boundary test passes; CHANGELOG.md has an entry.
> Out of scope: calling a planner or critic, briefs and prompts, status display, and athena.toml.

## Reading (accepted by the user)

Add the job-planning schema (job-plan and critique records, Planner/Critic roles, FindingSource::Critic, optional job_plan config excluded from reading_inputs, store job_of arms, fake-driver arms, docs) with no behavior change. Audience is maintainers and the verifier, who check by cargo tests that digests are unchanged and invalid records are refused.

Assumptions:
- Goal: land the job-planning schema (records, roles, config) with no behavior change, for maintainers and the verifier running tests.
- The design and review files are read by the producer; where code disagrees, code wins and the changelog says so.
- Digest test pins a hard-coded sha256 computed from pre-change code.
- Sri's quote about planner/critic loop is context, not extra work.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: all tests pass, and a fake-driver `go` still invokes only reader, producer and verifier roles.
2. [behavior] Run `cargo test`: round-2 job-plan and critique records pass check_record_file at job-plan-2.json / critique-2.json and are refused as misplaced at round-1 paths; a round-0 record and an accepting critique holding a blocker or major finding are refused by parse.
3. [behavior] Run `cargo test`: reading_inputs() for the repo's athena.toml matches a sha256 digest recorded in the test (computed from the pre-change code); adding [job_plan] enabled = true leaves it unchanged; max_rounds = 0, a zero cap, and an unknown key are each refused with a message naming the field.
4. [behavior] Run `cargo test`: Records::load files committed plans and critiques under their job, and a dispatched job with a plan but no outcome stays dispatched.
5. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: clean; xtask's boundary test passes.
6. [content] records.rs defines athena.job-plan/1, athena.critique/1, Role::Planner, Role::Critic, FindingSource::Critic with design sections 3-5 fields; config.rs has Config.job_plan: Option<JobPlanConfig> (deny_unknown_fields, skip_serializing_if none, enabled default false) reset to None in reading_inputs; store.rs job_of handles new records; all exhaustive matches (fake.rs, run_pin_tests.rs, tests/retry.rs, tests/common/mod.rs) cover the new roles, with a fake planner covering every brief criterion and an accepting critic.
7. [content] docs/records.md documents both records so an outside reader can see paths, fields and validity rules; CHANGELOG.md has an entry that notes any place the code disagreed with the design (findings F10, F15, F16 addressed).
8. [content] athena.toml and defaults.toml are unchanged.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Calling a planner or critic
- Briefs and prompts
- Status display
- Editing athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for records, config and store; clippy and fmt clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check refusals and unchanged digest, not mere existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Schema only; no planner calls, prompts, or status.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base fe0007e46cfa → candidate cbf7415d2976. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  14 ++
 crates/athena-agent/src/fake.rs         |  63 +++++++
 crates/athena-cli/tests/common/mod.rs   |   3 +
 crates/athena-cli/tests/lifecycle.rs    |  23 +++
 crates/athena-cli/tests/retry.rs        |   7 +-
 crates/athena-core/src/run_pin_tests.rs |   3 +
 crates/athena-core/src/store.rs         | 158 ++++++++++++++++-
 crates/athena-data/src/config.rs        | 126 +++++++++++++-
 crates/athena-data/src/records.rs       | 289 ++++++++++++++++++++++++++++++++
 docs/design/job-planning.md             |   3 +-
 docs/records.md                         |   8 +-
 11 files changed, 688 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (890 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
