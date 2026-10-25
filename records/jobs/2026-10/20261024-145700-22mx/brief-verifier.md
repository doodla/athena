# Job 20261024-145700-22mx: verify "Add required kind to LandingCommit with per-kind validation"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test serializes each kind's commit and checks JSON; diff inspection for deny_unknown_fields and three variants. Run: `cargo test -p athena-data history`. Observes: kind serializes as job_landing/declared_intervention/records_only; non-job commits have no job/verification keys; git diff shows deny_unknown_fields retained.
  Fails without the behavior: Without Option+skip_serializing_if the key-absence assertions fail; without snake_case rename the kind string assertions fail.
- Criterion 2: Unit tests mutate landing() commits per kind and call Landing::validate. Run: `cargo test -p athena-data history`. Observes: job_landing missing job or verification -> Err; declared_intervention/records_only with job or verification -> Err; valid combos -> Ok.
  Fails without the behavior: Without per-kind checks in Landing::validate the Err assertions fail.
- Criterion 3: Round-trip each kind via HistoryRecord::parse(to_pretty_json); mutate JSON for unknown kind, removed kind, extra field. Run: `cargo test -p athena-data history`. Observes: Each kind round-trips equal; the three malformed inputs return Err(Parse).
  Fails without the behavior: Without a required closed enum, a missing or unknown kind would parse and the Err assertions fail.
- Criterion 4: Run gh tests; the landing test compares commits() output with an expected LandingCommit of kind JobLanding with job and verification; git diff shows land.rs unchanged. Run: `cargo test -p athena-core gh`. Observes: Landing test passes; land.rs absent from the diff.
  Fails without the behavior: If commits() did not set JobLanding/job/verification the equality assertion at ~3787 fails (and the crate would not compile with the new required field).
- Criterion 5: Run clippy on both crates, then format check; any warning or unformatted code fails the commands. Run: `cargo clippy -p athena-data -p athena-core`. Observes: No warnings; then `cargo fmt --check` exits 0.
  Fails without the behavior: Unhandled Option job in intent(), unused variants/imports, or unformatted code would make clippy warn or fmt --check exit nonzero.

Planned files:

- crates/athena-data/src/history.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/manage.rs (modify)

## The change

Base c2069af81fc9 → candidate 1794db315df5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   4 +
 crates/athena-core/src/gh.rs      |  20 +++--
 crates/athena-data/src/history.rs | 178 ++++++++++++++++++++++++++++++++++++--
 3 files changed, 186 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1085 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
