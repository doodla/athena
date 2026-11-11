# Job 20261111-145417-cs81: verify "Retry 20261111-122637-aawj: role-need measure over all run sources"

## Request (verbatim)

> try again on 20261111-122637-aawj

## Reading (accepted by the user)

Retries job 20261111-122637-aawj (rejected): rebuild the OLY-9-11 role-need measure on one all-sources Records accessor, and address the verifier's findings. Specifically, never flag on-demand or out-of-job roles (steward, audience, auditor) for retirement, document the split-source trap where writers see it, and test with known-active roles.

Assumptions:
- Goal and audience: the Athena maintainer reading `athena status`, and future writers of usage/need measures, who must not be misled about which roles are active.
- Job 20261111-122637-aawj is addressed finding by finding. (1) Replay major: the measure advised retiring the auditor on real-shaped records. Fixed by a KNOWN_ACTIVE/on-demand list in role_need.rs (steward, audience, auditor) that are reported as 'not measured', never as retire candidates, and by a test replaying compute over Records with those roles. (2) Opt-in accessor major: doc comments on JobRecords::invocations and Records state that steward, audience and auditor runs are not in it; docs/system-facts.md gains a note naming the split and pointing to Records::all_invocations; a store test over Records::from_parts asserts every Invocation record loaded appears in all_invocations, so a future fourth split list fails. (3) Misread-request major: criteria now check the class (known-active roles unflagged, both outcomes, store coverage), not one instance. (4) Minor counts_as_work: Dropped, stale and abandoned jobs that never ran a model are excluded from the window and totals. (5) Minor extra files: crates/athena-cli/tests/status_role_need.rs and xtask/shard-timings.txt are kept as deliverables since they register the new test binary.
- Existing code: crates/athena-core/src/store.rs (Records, JobRecords, from_parts, steward_runs/audience_runs/auditor_runs), crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, docs/spec.toml OLY-9-11 (mechanism = none, measure 'status: not shown'); the criteria extend these, none is replaced.
- Code survey: read crates/athena-core/src/store.rs (JobRecords.invocations at line 29; Records steward_runs/audience_runs/auditor_runs lines 239-251; from_parts line 376 routes invocations to the separate lists at 421-427). `.invocations` is used in 24 files under crates/, so making it private is a large churn; this reading keeps it public but documented and covered by a coverage test, and the owner can ask for privacy as a follow-up. No new record or data file is added under crates/, so check_record_file/AnyRecord/round-trip are unaffected. The property is measured directly by calling RoleNeed::compute on Records, not via a stand-in.
- The previous candidate's role_need.rs, store.rs all_invocations, status.rs, display.rs, lib.rs, spec.toml and CHANGELOG changes are kept as the base where sound; the producer starts from the base and reapplies them with the fixes above.
- No refusal rule is added: the measure only advises on the status line and does not refuse work, so no refused-share figure applies.
- Governance files athena.toml, defaults.toml and workflows are unchanged and not needed for this outcome.

## Acceptance criteria

1. [content] crates/athena-core/src/store.rs has Records::all_invocations covering jobs' invocations plus steward_runs, audience_runs and auditor_runs, and doc comments on JobRecords::invocations and Records say those three roles' runs are not in the per-job list (Grep 'all_invocations' and the doc text).
2. [content] docs/system-facts.md names the steward, audience and auditor runs as kept outside job records and points usage measures to Records::all_invocations (Grep the file).
3. [content] crates/athena-core/src/role_need.rs reads usage only via all_invocations (Grep: no `.invocations` use there), is declared in lib.rs, treats steward/audience/auditor as known-active or on-demand ('not measured', never retire candidates), and excludes jobs that never ran a model (dropped, stale, abandoned, Declined, Closed) from the window and class totals.
4. [behavior] A store test over Records::from_parts asserts every Invocation record loaded appears in all_invocations, so a new split list would fail it. Run `cargo test -p athena-core store`.
5. [behavior] A role_need test builds Records with more than eight closed jobs holding only reader/producer/verifier runs and recent steward, audience and auditor runs, replays RoleNeed::compute, and asserts none of those three is a retire candidate while a role with no runs in any source is; it fails if compute reads only job invocations. Run `cargo test -p athena-core role_need`.
6. [behavior] A test over a mixed fixture asserts the measure yields both outcomes (some roles flagged, some not), not all-true or all-false; and a status() render test asserts the `role need` line prints only when a candidate exists, with Failed/Rejected/TimedOut counted failed. Run `cargo test`.
7. [content] docs/spec.toml OLY-9-11 sets mechanism to a real mechanism naming role_need and a measure ending `status: role need`; CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-cli/tests/status_role_need.rs (create)
- xtask/shard-timings.txt (modify)

## Out of scope

- Making JobRecords::invocations private across the 24 files that use it (follow-up)
- A standing gate or refusal rule on new measures
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- Changes to improvement_cycle in introspection.rs and its status/display code
- Changes to prompts, athena.toml or defaults.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria replay the measure on known-active roles and require both outcomes.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base dc8efdb8a97d → candidate 45cac31967fb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  10 +
 crates/athena-cli/src/display.rs            |   3 +
 crates/athena-cli/tests/status_role_need.rs | 183 +++++++++++
 crates/athena-context/src/briefs.rs         |   2 +-
 crates/athena-core/src/lib.rs               |   1 +
 crates/athena-core/src/role_need.rs         | 452 ++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs            |   7 +
 crates/athena-core/src/store.rs             |  78 +++++
 docs/spec.toml                              |   4 +-
 docs/system-facts.md                        |   5 +
 xtask/shard-timings.txt                     |   1 +
 11 files changed, 743 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-145417-cs81/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-145417-cs81/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43050238400 for 45cac31967fbd8f9c4a445cae680760a665cea5e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43050238400 for 45cac31967fbd8f9c4a445cae680760a665cea5e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43050238400 for 45cac31967fbd8f9c4a445cae680760a665cea5e)
- ci-result: `not a check` passed (Check results for 45cac31967fbd8f9c4a445cae680760a665cea5e come from the CI run https://github.com/doodla/athena/actions/runs/43050238400 (882s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
