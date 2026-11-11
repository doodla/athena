# Job 20261111-122637-aawj: verify "Make role-usage measures count all run sources, with a records test"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The retire rule was written against one data source, the per-job invocation list. Nothing required i
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: usage-measure-blind-to-roles-whose-runs-are-not-in-job-records (20261111-105030-a2bc)
> Cause: The retire rule was written against one data source, the per-job invocation list. Nothing required it to be run on the real records, or compared with known facts such as the steward, audience and auditor being active, before the job was jud...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261111-105017-5k6m (job 20261029-233051-jq6k); prevention it proposed: Any job that adds a usage, need or retire measure must replay it over the real records before it is judged. The replay must be checked against a list of roles known to be active. A role whose runs are not in the measured source must be show...
> Introspection 20261111-122618-q1ha (job 20261029-233058-fveq); prevention it proposed: Add a code check that every new status or measure stage is run over the real records and must give both outcomes: it must not be true for all items or false for all items, unless a stated reason is recorded. It must also be compared with kn...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Fixes the root cause of jobs 20261029-233051-jq6k and 20261029-233058-fveq: a usage/retire measure read only per-job invocations and never saw steward, audience and auditor runs kept in separate lists. Adds one Records accessor over every run source, builds the OLY-9-11 role-need measure on it, and tests it over real-shaped Records with known-active roles.

Assumptions:
- Goal/audience: Athena maintainers reading `athena status`; the status must never advise retiring a role that is actively running.
- Code survey: crates/athena-core/src/store.rs (Records struct lines ~239-251 keeps steward_runs, audience_runs, auditor_runs apart from jobs; split at ~421-456), crates/athena-core/src/status.rs (StatusView, STATUS_MEASURE_LABELS, improvement_cycle_line already present), crates/athena-data/src/records/introspection.rs (improvement_cycle already present with tests), crates/athena-cli/src/display.rs. No new record or data file is added, so check_record_file/AnyRecord is not touched.
- Existing code: crates/athena-core/src/store.rs Records and its steward_runs/audience_runs/auditor_runs fields, and crates/athena-core/src/status.rs StatusView, are extended, not replaced.
- Prior job 20261029-233051-jq6k (rejected): role_need.rs is absent from the repo, so its work is re-done here. Findings addressed: blocker (steward/audience/auditor invisible) fixed by counting every run source; minor missing compute test fixed by a Records-level test; minor unwindowed class history fixed by using only the newest N closed jobs per class; nit fixed by rendering through status() rather than the section formatter.
- Prior job 20261029-233058-fveq (rejected): its improvement_cycle code is already in the repo (introspection.rs, status.rs, display tests exist), so it is kept and not modified; its findings on that measure are out of scope here.
- Cause stated: a measure was written against one data source and never replayed over real records against a list of known-active roles. The information the stage lacked is the complete set of run sources, supplied by one accessor so a new measure cannot pick a partial source.
- No refusal rule is added: role-need candidates are advisory status lines only, so no refused-reading share applies (expected 0% of newly refused readings).

## Acceptance criteria

1. [content] crates/athena-core/src/store.rs has one Records method that yields invocations from jobs plus steward_runs, audience_runs and auditor_runs; crates/athena-core/src/role_need.rs uses it and not `invocations` alone (Grep).
2. [content] crates/athena-core/src/role_need.rs is declared in lib.rs, windows class history to the newest N closed jobs, and OLY-9-11 in docs/spec.toml names it as mechanism with a measure ending `status: role need`.
3. [behavior] A test in role_need.rs builds Records with more than eight closed jobs holding only reader/producer/verifier runs plus recent steward, audience and auditor runs, and asserts none of those three is a retire candidate; it fails when compute reads only job invocations. Run `cargo test -p athena-core role_need`.
4. [behavior] A test renders a StatusView through status() and asserts the `role need` line prints only when a candidate exists, and that mixed outcomes (Failed/Rejected/TimedOut counted failed; Declined/Closed excluded) are mapped correctly. Run `cargo test`.
5. [behavior] A test over Records asserts the measure yields both outcomes (some roles retired-or-not, some not) rather than all-true or all-false on a mixed fixture. Run `cargo test -p athena-core`.
6. [behavior] cargo fmt --check, cargo clippy and cargo test all pass.
7. [content] CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- Changes to improvement_cycle in introspection.rs and its status/display code (job 20261029-233058-fveq work kept as is)
- crates/athena-cli/tests/status_build_measurement.rs (unchanged)
- Changes to prompts, athena.toml or defaults.toml
- A generic refusal rule or gate on new measures

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that active roles are not flagged, not just that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test required.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1d5d683ed17b → candidate f6cf7b3c3181. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   9 +
 crates/athena-cli/src/display.rs            |   3 +
 crates/athena-cli/tests/status_role_need.rs |  56 +++++
 crates/athena-core/src/lib.rs               |   1 +
 crates/athena-core/src/role_need.rs         | 348 ++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs            |   6 +
 crates/athena-core/src/store.rs             |  14 ++
 docs/spec.toml                              |   4 +-
 xtask/shard-timings.txt                     |   1 +
 9 files changed, 440 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-122637-aawj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-122637-aawj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43040251168 for f6cf7b3c318192947961690a044dc3afc8c67735)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43040251168 for f6cf7b3c318192947961690a044dc3afc8c67735)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43040251168 for f6cf7b3c318192947961690a044dc3afc8c67735)
- ci-result: `not a check` passed (Check results for f6cf7b3c318192947961690a044dc3afc8c67735 come from the CI run https://github.com/doodla/athena/actions/runs/43040251168 (614s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
