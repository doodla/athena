# Job 20261102-001719-1fva: Stop D87 firing on merge 405s caused by pending checks

## Request (verbatim)

> D87's hard signal reads every 405/422 merge refusal as 'merge commits not allowed', but the November 405s came from checks still pending
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261101-222510-9gap (key d87-merge-refusal-signal-fires-on-pending-check-405), queued at 2026-11-01T22:25:10.504641491Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> path crates/athena-core/src/watcher.rs
> ```
> //! The watcher: decisions put under review by exact matching (`docs/design/decision-sets.md` section 4,
> //! "Watching" and "Firing"; build order step 4).
> //!
> //! The watcher reads stored fields only and decides by code, with no model. This slice has one signal source,
> //! `host-error`, and one decision with a typed signal, D87. A host-error signal names an operation, the HTTP
> //! statuses that falsify the decision's assumption, the phase and a threshold in distinct jobs. A hard signal
> //! (a refusal by the host) fires on the first job; the threshold of a hard signal cannot be raised.
> //!
> //! * [`load`] checks a signal's values against the record vocabulary and refuses one outside it, naming the
> //!   valid values. The signals in this module pass through it when they are read, so a wrong value is an error
> //!   and not a signal that can never fire.
> //! * [`evaluate`] returns the firings. It reads `athena.host-error/1` records and the decision-state records
> //!   already written. It never reads an introspection, so the introspection of a firing cannot cause another
> //!   firing. A host error that an earlier decision-state record already cites is not evaluated twice, and a
> //!   decision that is already under review does not fire again.
> //! * [`Athena::watch_landing`] is the landing trigger: it commits the refusals of a landing run as records, runs
> //!   [`evaluate`] over everything committed and appends one `athena.decision-state/1` record per firing.
> //!
> //! Clearing a review is not built; only the user clears it, in a later change.
> 
> use crate::Athena;
> use crate::Ui;
> use crate::commits::{NewFile, message};
> use crate::gh::HostErrorLog;
> use crate::store::Records;
> use anyhow::Result;
> use athena_data::RecordId;
> use athena_data::records::{
>     AnyRecord, DecisionState, DecisionStateKind, DefaultExpectations, HostError, HostOperation,
>     Intervention, MessageClass, SignalStrength, WorkPhase,
> };
> use serde::Serialize;
> use std::collections::BTreeSet;
> 
> /// A host-error signal as it is written, before its values are checked.
> #[derive(Debug, Clone, Copy)]
> pub struct RawSignal {
>     pub decision: &'static str,
>     pub name: &'static str,
>     pub strength: SignalStrength,
>     pub operation: &'static str,
>     pub status_codes: &'static [i64],
>     pub phase: &'static str,
>     pub threshold_jobs: usize,
> }
> 
> /// D87's hard signal: the host refusing a merge with 405 or 422 falsifies "the repository allows merge commits".
> pub const D87_SIGNAL: RawSignal = RawSignal {
>     decision: "D87",
>     name: "host-error: operation merge, status 405 or 422, phase landing",
>     strength: SignalStrength::Hard,
>     operation: "merge",
>     status_codes: &[405, 422],
>     phase: "landing",
>     threshold_jobs: 1,
> };
> 
> /// A host-error signal whose values are in the vocabulary.
> #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
> pub struct HostErrorSignal {
>     pub decision: String,
>     pub name: String,
>     pub strength: SignalStrength,
>     pub operation: HostOperation,
>     pub status_codes: Vec<u16>,
>     pub phase: WorkPhase,
>     pub threshold_jobs: usize,
> }
> 
> const PHASES: [(&str, WorkPhase); 4] = [
>     ("reading", WorkPhase::Reading),
>     ("producing", WorkPhase::Producing),
>     ("verifying", WorkPhase::Verifying),
>     ("landing", WorkPhase::Landing),
> ];
> 
> /// Checks a signal's values against the vocabulary of the host-error record. The error names the valid values.
> pub fn load(raw: &RawSignal) -> Result<HostErrorSignal, String> {
>     let operation = HostOperation::parse(raw.operation).ok_or_else(|| {
>         format!(
>             "signal {:?}: operation {:?} is not one of: {}",
>             raw.name,
>             raw.operation,
>             HostOperation::ALL.map(HostOperation::as_str).join(", ")
>         )
>     })?;
>     let phase = PHASES
>         .iter()
>         .find(|(name, _)| *name == raw.phase)
>         .map(|(_, phase)| *phase)
>         .ok_or_else(|| {
>             format!(
>                 "signal {:?}: phase {:?} is not one of: {}",
>                 raw.name,
>                 raw.phase,
>                 PHASES.map(|(name, _)| name).join(", ")
>             )
>         })?;
>     if raw.status_codes.is_empty() {
>         return Err(format!("signal {:?}: no status_code is named", raw.name));
>     }
>     let mut status_codes = Vec::new();
>     for code in raw.status_codes {
>         match u16::try_from(*code) {
>             Ok(code) if (100..=599).contains(&code) => status_codes.push(code),
>             _ => {
>                 return Err(format!(
>                     "signal {:?}: status_code {code} is not an HTTP status (100 to 599)",
>                     raw.name
>                 ));
>             }
>         }
>     }
>     let least = match raw.strength {
>         SignalStrength::Hard => 1,
>         SignalStrength::Soft => 2,
>     };
>     if raw.strength == SignalStrength::Hard && raw.threshold_jobs != 1 {
>         return Err(format!(
>             "signal {:?}: a hard signal fires on the first job; its threshold is 1, not {}",
>             raw.name, raw.threshold_jobs
>         ));
>     }
>     if raw.threshold_jobs < least {
>         return Err(format!(
>             "signal {:?}: a soft signal needs at least {least} jobs, not {}",
>             raw.name, raw.threshold_jobs
>         ));
>     }
>     Ok(HostErrorSignal {
>         decision: raw.decision.to_string(),
>         name: raw.name.to_string(),
>         strength: raw.strength,
>         operation,
>         status_codes,
>         phase,
>         threshold_jobs: raw.threshold_jobs,
>     })
> }
> 
> /// Every typed host-error signal of the build: D87's, as of this slice.
> pub fn host_error_signals() -> Result<Vec<HostErrorSignal>, String> {
>     [D87_SIGNAL].iter().map(load).collect()
> }
> 
> impl HostErrorSignal {
>     /// Exact matching on stored fields: the operation, the phase and one of the statuses are all equal. A
>     /// refusal with no status never matches.
>     pub fn matches(&self, error: &HostError) -> bool {
>         error.operation == self.operation
>             && error.phase == self.phase
>             && error
>                 .sta
> [clipped: 22847 more characters]
> ```
> 
> ## Evidence
> 
> ### 1. path crates/athena-core/src/watcher.rs
> ```
> //! The watcher: decisions put under review by exact matching (`docs/design/decision-sets.md` section 4,
> //! "Watching" and "Firing"; build order step 4).
> //!
> //! The watcher reads stored fields only and decides by code, with no model. This slice has one signal source,
> //! `host-error`, and one decision with a typed signal, D87. A host-error signal names an operation, the HTTP
> //! statuses that falsify the decision's assumption, the phase and a threshold in distinct jobs. A hard signal
> //! (a refusal by the host) fires on the first job; the threshold of a hard signal cannot be raised.
> //!
> //! * [`load`] checks a signal's values against the record vocabulary and refuses one outside it, naming the
> //!   valid values. The signals in this module pass through it when they are read, so a wrong value is an error
> //!   and not a signal that can never fire.
> //! * [`evaluate`] returns the firings. It reads `athena.host-error/1` records and the decision-state records
> //!   already written. It never reads an introspection, so the introspection of a firing cannot cause another
> //!   firing. A host error that an earlier decision-state record already cites is not evaluated twice, and a
> //!   decision that is already under review does not fire again.
> //! * [`Athena::watch_landing`] is the landing trigger: it commits the refusals of a landing run as records, runs
> //!   [`evaluate`] over everything committed and appends one `athena.decision-state/1` record per firing.
> //!
> //! Clearing a review is not built; only the user clears it, in a later change.
> 
> use crate::Athena;
> use crate::Ui;
> use crate::commits::{NewFile, message};
> use crate::gh::HostErrorLog;
> use crate::store::Records;
> use anyhow::Result;
> use athena_data::RecordId;
> use athena_data::records::{
>     AnyRecord, DecisionState, DecisionStateKind, DefaultExpectations, HostError, HostOperation,
>     Intervention, MessageClass, SignalStrength, WorkPhase,
> };
> use serde::Serialize;
> use std::collections::BTreeSet;
> 
> /// A host-error signal as it is written, before its values are checked.
> #[derive(Debug, Clone, Copy)]
> pub struct RawSignal {
>     pub decision: &'static str,
>     pub name: &'static str,
>     pub strength: SignalStrength,
>     pub operation: &'static str,
>     pub status_codes: &'static [i64],
>     pub phase: &'static str,
>     pub threshold_jobs: usize,
> }
> 
> /// D87's hard signal: the host refusing a merge with 405 or 422 falsifies "the repository allows merge commits".
> pub const D87_SIGNAL: RawSignal = RawSignal {
>     decision: "D87",
>     name: "host-error: operation merge, status 405 or 422, phase landing",
>     strength: SignalStrength::Hard,
>     operation: "merge",
>     status_codes: &[405, 422],
>     phase: "landing",
>     threshold_jobs: 1,
> };
> 
> /// A host-error signal whose values are in the vocabulary.
> #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
> pub struct HostErrorSignal {
>     pub decision: String,
>     pub name: String,
>     pub strength: SignalStrength,
>     pub operation: HostOperation,
>     pub status_codes: Vec<u16>,
>     pub phase: WorkPhase,
>     pub threshold_jobs: usize,
> }
> 
> const PHASES: [(&str, WorkPhase); 4] = [
>     ("reading", WorkPhase::Reading),
>     ("producing", WorkPhase::Producing),
>     ("verifying", WorkPhase::Verifying),
>     ("landing", WorkPhase::Landing),
> ];
> 
> /// Checks a signal's values against the vocabulary of the host-error record. The error names the valid values.
> pub fn load(raw: &RawSignal) -> Result<HostErrorSignal, String> {
>     let operation = HostOperation::parse(raw.operation).ok_or_else(|| {
>         format!(
>             "signal {:?}: operation {:?} is not one of: {}",
>             raw.name,
>             raw.operation,
>             HostOperation::ALL.map(HostOperation::as_str).join(", ")
>         )
>     })?;
>     let phase = PHASES
>         .iter()
>         .find(|(name, _)| *name == raw.phase)
>         .map(|(_, phase)| *phase)
>         .ok_or_else(|| {
>             format!(
>                 "signal {:?}: phase {:?} is not one of: {}",
>                 raw.name,
>                 raw.phase,
>                 PHASES.map(|(name, _)| name).join(", ")
>             )
>         })?;
>     if raw.status_codes.is_empty() {
>         return Err(format!("signal {:?}: no status_code is named", raw.name));
>     }
>     let mut status_codes = Vec::new();
>     for code in raw.status_codes {
>         match u16::try_from(*code) {
>             Ok(code) if (100..=599).contains(&code) => status_codes.push(code),
>             _ => {
>                 return Err(format!(
>                     "signal {:?}: status_code {code} is not an HTTP status (100 to 599)",
>                     raw.name
>                 ));
>             }
>         }
>     }
>     let least = match raw.strength {
>         SignalStrength::Hard => 1,
>         SignalStrength::Soft => 2,
>     };
>     if raw.strength == SignalStrength::Hard && raw.threshold_jobs != 1 {
>         return Err(format!(
>             "signal {:?}: a hard signal fires on the first job; its threshold is 1, not {}",
>             raw.name, raw.threshold_jobs
>         ));
>     }
>     if raw.threshold_jobs < least {
>         return Err(format!(
>             "signal {:?}: a soft signal needs at least {least} jobs, not {}",
>             raw.name, raw.threshold_jobs
>         ));
>     }
>     Ok(HostErrorSignal {
>         decision: raw.decision.to_string(),
>         name: raw.name.to_string(),
>         strength: raw.strength,
>         operation,
>         status_codes,
>         phase,
>         threshold_jobs: raw.threshold_jobs,
>     })
> }
> 
> /// Every typed host-error signal of the build: D87's, as of this slice.
> pub fn host_error_signals() -> Result<Vec<HostErrorSignal>, String> {
>     [D87_SIGNAL].iter().map(load).collect()
> }
> 
> impl HostErrorSignal {
>     /// Exact matching on stored fields: the operation, the phase and one of the statuses are all equal. A
>     /// refusal with no status never matches.
>     pub fn matches(&self, error: &HostError) -> bool {
>         error.operation == self.operation
>             && error.phase == self.phase
>             && error
>                 .sta
> [clipped: 22847 more characters]
> ```
> 
> ### 2. path records/decision-states/2026-10/20261031-085139-1y06.json
> ```
> {
>   "schema": "athena.decision-state/1",
>   "id": "20261031-085139-1y06",
>   "at": "2026-10-31T08:51:39.441990125Z",
>   "decision": "D87",
>   "state": "under-review",
>   "signal": "host-error: operation merge, status 405 or 422, phase landing",
>   "firing_records": [
>     "20261031-085135-nwn4"
>   ],
>   "decision_job": null
> }
> ```
> 
> ### 3. path records/host-errors/2026-11/20261101-174051-3t6g.json
> ```
> {
>   "schema": "athena.host-error/1",
>   "id": "20261101-174051-3t6g",
>   "at": "2026-11-01T17:40:51.113469832Z",
>   "status_code": 405,
>   "operation": "merge",
>   "job": "20261101-164247-5mrg",
>   "phase": "landing",
>   "message_class": "method-not-allowed"
> }
> ```
> 
> ### 4. path records/host-errors/2026-11/20261101-184717-b93x.json
> ```
> {
>   "schema": "athena.host-error/1",
>   "id": "20261101-184717-b93x",
>   "at": "2026-11-01T18:47:17.78262572Z",
>   "status_code": 405,
>   "operation": "merge",
>   "job": "20261031-031749-am9j",
>   "phase": "landing",
>   "message_class": "method-not-allowed"
> }
> ```
> 
> ### 5. path records/host-errors/2026-11/20261101-184811-fjgf.json
> ```
> {
>   "schema": "athena.host-error/1",
>   "id": "20261101-184811-fjgf",
>   "at": "2026-11-01T18:48:11.490682146Z",
>   "status_code": 405,
>   "operation": "merge",
>   "job": "20261031-031749-am9j",
>   "phase": "landing",
>   "message_class": "method-not-allowed"
> }
> ```
> 
> ### 6. path records/interventions/2026-11/20261101-185305-wpkf.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261101-185305-wpkf",
>   "at": "2026-11-01T18:53:05.762790577Z",
>   "what": "Mistake: my request to judge pull requests by the required check let the drain merge while another check that the host also waits on was still running, so GitHub refused the merge and the batch stopped",
>   "why": "I assumed the one recorded check name was the whole merge rule; I landed the job with athena land --pr and queued a fix to wait for pending checks",
>   "class_of_work": "design",
>   "category": "mistake",
>   "job": "20261031-031749-am9j",
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 7. job 20261031-031749-am9j
> Request:
> ```
> try again on 20261031-030119-tw3v. Second half of async drain A6: rebase each verified candidate onto the latest main before it lands.
> Depends on 20261031-031744-c5yx
> Priority: urgent
> 
> The first half (20261031-031744-c5yx) adds the landing queue. This half adds the rebase the queue head runs before landing. Returning a conflicting candidate to its producer, and the thrash stop of async-drain.md section 6, are not in this job.
> 
> Design, from the round-3 plan of tw3v and its critic's findings, with the open ruling made:
> - A new `Steps::rebase` (default: clean) runs for the queue head before `Steps::land`. In production it is `Athena::rebase_for_landing`, a thin wrapper over a free function `rebase_job_branch` in crates/athena-core/src/gh/landing.rs that takes git, the state dir and the remote as arguments, so tests run it against a bare origin.
> - `plan_rebase` is stack-aware: landing commits are stacked on one local branch, so it replays the commits between the fork and the job's landing commit one at a time onto origin's main.
> - A conflict in the job's own landing commit holds that job back: not landed, main untouched, no retry, no drain stop. The next queued candidate still lands.
> - Ruling on the critic's major finding: an earlier commit on the stack that conflicts is carried as a records-only commit. Its records/ paths are kept and its code paths are dropped, so no job's records leave the branch. A test shows the held-back job's records still load from the new tip.
> - Earlier unlanded commits that replay cleanly still ride along in the next job's pull request, as they do today. Say this in CHANGELOG.md and async-drain.md; do not claim the re-check rule covers stacked content.
> - Size: the wrapper test and the second bare-origin test may be reported as read-only checks if the run is long.
> 
> Acceptance:
> - Core tests of plan_rebase: a clean move, already current, a conflict in the job's own commit, an earlier conflicting commit carried as records only (its job record still loads), and a clean earlier commit carried.
> - A bare-origin test: two jobs where main moved after the second was verified; the second lands rebased.
> - A drain test: a conflicting head is held back and the next candidate lands.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, docs/design/async-drain.md, CHANGELOG.md
> ```
> Read as: Add stack-aware plan_rebase and rebase_job_branch for landing (code). Job 20261031-030119-tw3v (A6 second half) failed in planning, so I cut it to one deliverable: core rebase logic. This adds `plan_rebase` and `rebase_job_branch` in athena-core's gh/landing.rs, replaying the stacked landing commits onto origin's main. A conflicting earlier commit is carried as records-only. The drain wiring is split off as a follow-up. It depends on 20261031-031744-c5yx, which landed the queue.
> Outcome: landed. verified and landed
> 
> 
> ### 8. job 20261101-164247-5mrg
> Request:
> ```
> try again on 20261101-135902-9bxs
> The design passed review; only one test fixture was missed. In crates/athena-cli/tests/external_calls.rs the fake host still returns a check run with no name, so land_pr_with_a_one_off_500_on_each_call_is_retried_and_lands stays Open. Name that check run 'fmt, clippy, tests' as the gh/tests.rs fixtures now are, and search all test fakes for other unnamed check runs.
> - crates/athena-cli/tests/external_calls.rs may change, as may crates/athena-core/src/gh/mod.rs and crates/athena-gate/src/landing_tests.rs.
> - Return the optional-check notes from the same read as the verdict, not a second API call.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion; the checks stage covers it.
> Priority: urgent
> ```
> Read as: Retry 9bxs: judge PRs by required check; held-job records pass head (code). Retry of job 20261101-135902-9bxs (rejected: external_calls.rs fake host returned an unnamed check run, so the new classifier read it as Pending). Redo both fixes for the drain maintainer: judge a PR only by the required check "fmt, clippy, tests" (optional red checks become notes, returned from the same read as the verdict), let a held job's records pass the PR head check, and name the fake's check run in external_calls.rs and any other test fake.
> Outcome: landed. verified and landed

## Reading (accepted by the user)

D87's hard signal (watcher.rs) treats every landing merge 405/422 as "merge commits not allowed", but the November 405s (jobs 20261101-164247-5mrg, 20261031-031749-am9j) came from checks still pending. Make only refusals that are not explained by pending checks count toward D87, so the watcher stops opening false reviews. Goal: the maintainer's decision-state records reflect real falsification only.

Assumptions:
- Code survey: read crates/athena-core/src/watcher.rs (load, evaluate, HostErrorSignal::matches, watch_landing) and grepped crates/athena-data/src/records.rs: MessageClass::MethodNotAllowed is derived from status 405 only (line ~1075), so the stored record cannot tell pending checks from a disallowed merge method. I assume the fix lives in the code that logs host errors during landing (crates/athena-core/src/gh/landing.rs, already knows check state after job 20261101-164247-5mrg) and/or watcher.rs, and needs no new record field. If a new MessageClass variant were needed, records.rs would be touched and must be accepted by `check_record_file` (records.rs:2932) and `AnyRecord`, with a round-trip test; I judge this unlikely, so it is not a firm deliverable.
- Job 20261101-164247-5mrg (landed; findings: nit on landing_tests.rs comment naming candidate2.json, minor on hard-coded required check name): both are unrelated to this change and not re-fixed; its 11 changed files are kept as landed and not modified except gh/landing.rs.
- Job 20261031-031749-am9j (landed; findings about later-stacked records-only commits, empty-replay reporting, and a linear-history check wording): unrelated to D87 and left as is; its files (gh/landing.rs, gh/mod.rs, async-drain.md, CHANGELOG.md) are kept, only landing.rs and CHANGELOG.md may be edited here.
- Job 20261031-030119-tw3v failed in planning and changed no files; its drain.rs/tests/drain.rs wiring remains out of scope.
- Existing records/ are never edited; the already-open D87 review is cleared only by the user.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core watcher`: a new test shows a merge 405 recorded while required checks were pending does not make `evaluate` fire D87, and a 405/422 with checks complete still fires D87 on the first job.
2. [behavior] Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is not logged as a D87-matching host error (or is logged distinguishably), while a refusal with green checks is logged as before.
3. [content] Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment and name state that pending-check refusals are excluded; the signal name string stays unchanged so existing decision-state records (e.g. 20261031-085139-1y06) still match.
4. [content] CHANGELOG.md has an entry naming the pending-check 405 false positive and stating existing records/ are untouched.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Editing any file under records/ or clearing the open D87 review
- Reading branch protection to find the required check name
- Changes to athena.toml, defaults.toml or .github/workflows
- Drain wiring in crates/athena-cli/src/drain.rs
- Adding new decisions' signals other than D87

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New watcher and landing tests; cargo test and clippy pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the signal's input so pending-check 405s are not read as a disallowed merge method, not just suppress the one review.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing D87 does and does not fire.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG explain the exclusion.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No records or personal data committed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

Run `cargo test -p athena-core watcher`: a new test shows a merge 405 recorded while required checks were pending does not make `evaluate` fire D87, and a 405/422 with checks complete still fires D87 on the first job.
Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is not logged as a D87-matching host error (or is logged distinguishably), while a refusal with green checks is logged as before.
Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment and name state that pending-check refusals are excluded; the signal name string stays unchanged so existing decision-state records (e.g. 20261031-085139-1y06) still match.
CHANGELOG.md has an entry naming the pending-check 405 false positive and stating existing records/ are untouched.
Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
