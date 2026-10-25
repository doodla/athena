# Job 20261015-012203-y8dx: Give job introspections the unmet criteria, findings and failed checks

## Request (verbatim)

> Six introspections said the stored fields do not name the unmet criterion or findings, but each job's verification.json does; one misfiled a failed test check as an unmet criterion
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261015-003903-khz6 (key introspection-input-omits-unmet-criterion-and-findings), queued at 2026-10-15T00:39:03.457454177Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> path crates/athena-core/src/introspect.rs
> ```
> //! Introspection (`docs/design/defaults-loop.md` sections 3.1 to 3.4; build order step 1).
> //!
> //! This module holds two things that share one set of definitions:
> //!
> //! 1. **Shared reading code.** The reader of `records/` ([`read_records`]), the typed decision headers and
> //!    their signals, the evidence-after-decision rule ([`after_start`]) and the citation validation
> //!    ([`validate_citations`]). The read-only `athena backtest` and the introspection step both use these; the
> //!    backtest re-exports them and keeps no copy. The `athena.introspection/1` schema is defined once, in
> //!    `athena_data::records` ([`IntrospectionRecord`], [`INTROSPECTION_SCHEMA`]); [`Introspection`] is the view
> //!    of its code-written fields that the backtest builds.
> //! 2. **The introspection step.** After each of these events, Athena records one introspection with no request
> //!    from the user: a job that ends `rejected` or `failed`, a correction of a reading (or an answer to its
> //!    question), and any other intervention ([`Athena::introspect_job`], [`Athena::introspect_intervention`]).
> //!    Host errors and landings are later builds. When Athena has a Jev (`Athena::jev`; the fake in tests), it
> //!    is asked the failure-class and is-repeat questions in one call, in shadow ([`ask_jev`]): the answers
> //!    are stored in the record's `jev` field so they can be scored later, and never change the class, the
> //!    status or any outcome. A failed Jev call is stored as an error and blocks nothing.
> //!
> //! The reading code (section 3.3) is a function of Athena itself. It reads only the stored fields it lists:
> //! `what`, `why`, `class_of_work`, `category`, job id and date of the event; and for earlier cases the record
> //! id, job id, date, the one-line `what`, the class and the `cause`. It never reads transcripts, diffs,
> //! standard-error text or files of other jobs. It narrows earlier cases by `class_of_work` and `category`,
> //! caps them at [`CASE_CAP`], and passes only those fields to the model. On return, any id that was not on a
> //! list passed in is dropped and logged, not repaired.
> //!
> //! A failed introspection (model error, invalid answer, a cause that restates the event) is recorded as
> //! failed, never blocks the job that caused it, and is not itself introspected: only the events above start
> //! one.
> 
> use crate::capability::JobConfig;
> use crate::commits::{NewFile, message};
> use crate::jev::{Answer as JevAnswer, Evaluate, Question};
> use crate::store::Records as GitRecords;
> use crate::watcher::{self, D87_SIGNAL, HostErrorSignal};
> use crate::{Athena, Ui};
> use anyhow::{Context, Result};
> use athena_agent::env::{EnvSpec, Side};
> use athena_agent::{AgentInvocation, NoSink};
> use athena_data::records::{
>     AnyRecord, ClassStatus, FailureClass, HostError, INTROSPECTION_PROSE_FIELDS,
>     INTROSPECTION_SCHEMA, InterventionCategory, IntrospectionEvidence, IntrospectionJev,
>     IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, JevClassAnswer, OutcomeState,
>     PreventionKind, ProposalRecord, Role,
> };
> use athena_data::{JobId, RecordId};
> use jiff::Timestamp;
> use serde::de::DeserializeOwned;
> use serde::{Deserialize, Serialize};
> use serde_json::json;
> use std::collections::{BTreeMap, BTreeSet};
> use std::path::{Path, PathBuf};
> use std::time::Duration;
> 
> /// The most earlier cases passed to the model (an estimate; `defaults-loop.md` section 3.3).
> pub const CASE_CAP: usize = 5;
> 
> // ---------------------------------------------------------------------------------------------------------
> // The typed header (decision-sets.md section 2), for D87 and D88 only.
> // ---------------------------------------------------------------------------------------------------------
> 
> #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
> #[serde(rename_all = "kebab-case")]
> pub enum Kind {
>     Requirement,
>     Expectation,
>     Design,
>     Process,
> }
> 
> #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
> #[serde(rename_all = "kebab-case")]
> pub enum Status {
>     Active,
>     Proposed,
>     Superseded,
> }
> 
> #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
> #[serde(rename_all = "kebab-case")]
> pub enum Enforcement {
>     CodeCheck,
>     Brief,
>     ProseOnly,
> }
> 
> #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
> #[serde(rename_all = "kebab-case")]
> pub enum Phase {
>     Reading,
>     Producing,
>     Verifying,
>     Landing,
> }
> 
> /// An exact match on a stored intervention: the category and class of work are equal, and the job it names
> /// landed. Nothing here is free text.
> #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
> pub struct InterventionMatch {
>     pub category: InterventionCategory,
>     pub class_of_work: &'static str,
>     pub on_landed_job: bool,
> }
> 
> /// A watched condition. It is run when it has an exact match: on an intervention (`exact`) or on a host-error
> /// record (`host_error`, the same signal the landing watcher evaluates). A signal with neither cannot be decided
> /// from the records this reader reads (history checks have no record yet), so it is listed and not run.
> #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
> pub struct Signal {
>     pub name: &'static str,
>     pub threshold_jobs: usize,
>     pub exact: Option<InterventionMatch>,
>     pub host_error: Option<HostErrorSignal>,
> }
> 
> #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
> pub struct DecisionHeader {
>     pub id: &'static str,
>     pub kind: Kind,
>     pub classes: Vec<&'static str>,
>     pub paths: Vec<&'static str>,
>     pub phases: Vec<Phase>,
>     pub status: Status,
>     pub superseded_by: Option<&'static str>,
>     pub amended_by: Vec<&'static str>,
>     pub enforcement: Enforcement,
>     pub assumptions: Vec<&'static str>,
>     pub signals: Vec<Signal>,
> }
> 
> /// The typed headers known: D87 and D88, as `decision-sets.md` section 4 gives them. D87 stays `active` with
> /// `amended_by = D88`, not `superseded`: it is superseded for its landing shape only, and its two rerun fixes
> /// stand (review 1, section 3). Its enforcement is `brief` (finding F4).
> [clipped: 51700 more characters]
> ```
> 
> ## Evidence
> 
> ### 1. path docs/design/defaults-loop.md
> ```
> # The defaults-learning loop: introspection on every correction, and repeats become proposed, versioned defaults
> 
> **Status:** proposed (D90), revised after review 1 (`docs/design/reviews/defaults-loop-review-1.md`), extended
> with introspection as a first-class step (`BRIEF.md`, addendum "2026-10-13: Introspection on every correction"),
> and revised again after the joint review of this document with `docs/design/decision-sets.md`
> (`docs/design/reviews/decision-sets-review-1.md`, accept with changes; its section 6 changes are applied in both
> documents). Awaiting the confirming look at that revision. This document builds nothing. It changes no Rust, no tests and no `defaults.toml`. A later job reviews it;
> nothing is built before that review (the `designed` default). Paths are relative to the repository root. The
> reader is someone who knows what `defaults.toml` is (D26) and what an intervention record is
> (`docs/records.md`) but has not seen the jobs that motivated this.
> 
> Claims are marked **(read)** when taken from a file named beside them, **(design)** when they are proposals,
> **(reported)** when they come from the job request, the user's probes or the review and were not checked here,
> and **(estimate)** for numbers that nobody measured. Jobs are cited by job id and pull requests by number; no
> person is named.
> 
> ## 1. The problem
> 
> `defaults.toml` is Athena's model of what its user expects without saying so. It has 12 entries today
> **(read: `defaults.toml`, counted)**. D26 says "eleven"; that text was written before `root-cause` was added and
> is corrected in the same change as this revision **(read: `docs/decisions.md` D26)**. The seed entries each cite
> `BRIEF.md`. Nothing yet turns what the user repeatedly has to say back into an entry. The roadmap names the gap
> in RM-4, "feed scored mispredictions back into `defaults.toml`" **(read: `docs/design/roadmap.md`)**. RM-3,
> "Close the intervention loop", is the nearby item: it is about a trend per class of work, an audit step and a
> link to an improvement job, not about defaults **(read: same file)**. This design supplies the part of RM-3 that
> turns a correction into a diagnosis (§3) and the part of RM-4 that turns repeated diagnoses into defaults.
> 
> When the user corrects a reading, answers a question, or fixes something by hand, Athena records an
> intervention (D27) **(read)**. The record carries `what`, `why`, `class_of_work`, a `category` (one of eight,
> such as `corrected-output` or `overrode-decision`), the job, and a source
> **(read: `crates/athena-data/src/records.rs`, `Intervention`, `InterventionCategory`)**. A correction record
> carries the intervention recorded for it **(read: `docs/records.md`)**. Today a person has to notice that the
> same thing happened again and then write a defaults change, and each retry after a failure starts from the user
> rewriting the request. The change request asks to "close the defaults loop: use Jev to match interventions and
> corrections against the defaults model, so repeats become new, versioned defaults" **(reported)**, and the
> brief's addendum asks that every correction be followed by introspection by Athena itself **(read: `BRIEF.md`)**.
> 
> ## 2. Evidence: four repeats, and what they do and do not prove
> 
> **Access caveat.** The `records/` tree is not part of the snapshot this document was written from. The
> records behind the repeats could not be opened, so their record ids are not cited, and the counts come from the
> job request **(reported)**. Where the repository itself shows the repeat, that is cited **(read)**.
> 
> | # | Repeat | Count | What it is |
> |---|--------|-------|------------|
> | R1 | A change reading's scope criteria excluded `CHANGELOG.md`, so the verifier saw a changed file outside scope | x4 **(reported)** | **Closed in code by D89.** Change readings now always list `CHANGELOG.md`, and the verifier brief says the deliverables list is the one statement of scope **(read: D89)**. R1 cannot recur as it did. It is a test case for matching (a correction that should land on "none" or at most `small`), not proof that the loop is needed. |
> | R2 | The reader's always-loaded set went over its token budget and blocked the job | x4 **(reported)** | **A counter-example, not a defaults repeat.** The reader budget is 1,930 tokens, and an over-budget static set fails a **test**; it is not a check at job time **(read: D25; `crates/athena-context/src/lib.rs`, `check_budget`, called only from tests and `crates/athena-cli/tests/budgets.rs`; `docs/enforcement.md` row M18)**. What actually blocked the four jobs is reported and was not verified here. D89 records one instance **(read)**. The fault is caught by a test, so a default such as "keep prompts under budget" would restate a rule the tests enforce and cost reader tokens. R2 is labelled "none" in the labeled set (§10.2). Its other lesson, that four retries each patched the last finding, belongs to introspection (§3, case C2). |
> | R3 | One job bundled design, review and build | x1 visible **(reported for the rest)** | The nearest written rules are `small` and `designed` **(read: `defaults.toml`)**. No instance is visible here: **unverified**. Test case for "close to `small`, not covered by it" (an amend, §7.2). |
> | R4 | A new config field and its first use landed in one commit | not visible **(reported)** | Matches no entry. No instance is visible here: **unverified**. Test case for a new id. |
> 
> **The case for the loop is unproved.** Of the four, one is closed in code (R1), one is not a defaults repeat
> (R2), and two have no visible instance (R3, R4). The review makes the same point **(read: review 1, F2)**.
> 
> **Gate before any build.** The building job's first task is to read `records/` and list the actual record ids
> for each repeat. If fewer than **N distinct jobs** (N as in §7.2) show a repeat that a default would cover, the
> loop is not built, and this design is closed with that finding. The step that needs no such evidence is
> introspection (§3): it has value when a si
> [clipped: 50582 more characters]
> ```
> 
> ### 2. path records/introspections/2026-10/20261014-172320-k3k0.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-172320-k3k0",
>   "at": "2026-10-14T17:23:20.471665651Z",
>   "trigger": "rejection",
>   "job": "20261014-170031-2r9b",
>   "intervention": null,
>   "class_of_work": "prompts",
>   "category": "rejected",
>   "what_happened": "The job \"Make role prompts carry a request's real goal through to the verdict\" (class of work: prompts) ended rejected. The verifier returned a fail verdict: 1 of 9 acceptance criteria was unmet and there was at least one blocker or major finding. The stored fields do not say which criterion failed or what the finding was.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": true,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "Nothing in the stored event shows the unmet criterion being checked before the final verdict. The likely gap is that criteria are verified only at the end, so a miss costs the whole job instead of triggering a repair pass. The goal of the job (prompts that preserve the request's real goal) is also hard to test, and a prompt change is easy to call done without evidence for each criterion. The event also lacks the failing criterion and finding, so the exact cause cannot be pinned down.",
>   "prevention_kind": "design-change",
>   "prevention": "Have the implementer check each acceptance criterion against concrete evidence before submitting, and give the job a bounded repair loop: when the verifier reports an unmet criterion or a blocker or major finding, the job returns to the implementer with that finding instead of ending rejected. Also record the failing criterion and finding text in the rejection event so cases can be grouped by real cause.",
>   "proposed_request": "Redo the work from job 20261014-170031-2r9b: make the role prompts carry a request's real goal through to the verdict. First read that job's verifier report and identify the 1 unmet criterion of 9 and the blocker or major finding. Fix that gap without regressing the 8 criteria that passed. Before finishing, give evidence for each of the 9 criteria, for example a sample request whose stated goal can be traced through each role prompt to the final verdict. Done means the verifier passes all 9 criteria with no blocker or major findings.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0275638,
>     "input_tokens": 4,
>     "output_tokens": 1764
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 3. path records/introspections/2026-10/20261014-174804-mgn1.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-174804-mgn1",
>   "at": "2026-10-14T17:48:04.172979495Z",
>   "trigger": "rejection",
>   "job": "20261014-172342-rqgb",
>   "intervention": null,
>   "class_of_work": "prompts",
>   "category": "rejected",
>   "what_happened": "The job \"Redo role-prompt goal-preservation change; update every 1,800 mention\" (class of work: prompts) ended rejected. It was a redo of the earlier rejected prompts job (20261014-170031-2r9b). The verifier returned fail: 1 of 10 acceptance criteria was unmet and there were 2 blocker or major findings. The stored fields do not name the criterion or the findings. The \"every 1,800 mention\" wording suggests a sweep for all occurrences of a value, and a missed occurrence is a likely gap.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": false,
>   "earlier_cases": [
>     "20261014-172320-k3k0"
>   ],
>   "decisions_matched": [],
>   "cause": "Criteria are checked only by the final verifier, so a miss rejects the whole job. There is no self-check against each criterion before submission. The redo also seems to have repeated the first attempt's gap, so the first rejection's finding probably did not carry into the second attempt as a checked item. An \"update every mention\" requirement has no enumerated search to prove completeness. The goal-preservation criterion for prompts is hard to test, so work can be called done without evidence. The stored event omits the failing criterion and findings, so the cause cannot be tied to one gap.",
>   "prevention_kind": "design-change",
>   "prevention": "Add a pre-verdict step where the worker maps each acceptance criterion to concrete evidence and runs a repair pass for any criterion without it. For \"update every mention\" criteria, require a repo-wide search with zero remaining stale hits, and the result must be recorded. A redo must start from the prior rejection's unmet criterion and findings, and each one must be shown fixed. The rejection event must also store the failing criterion and findings.",
>   "proposed_request": "Redo the role-prompt change so each role prompt carries the request's real goal through to the verdict. Before finishing, do three things. (1) Search the whole repo for every mention of the old 1,800 value and update all of them. Show that a repeat search finds no stale mentions. (2) Fetch the unmet criterion and the blocker/major findings from the rejected jobs 20261014-170031-2r9b and 20261014-172342-rqgb, and fix each one. (3) List every acceptance criterion with the evidence that it is met, and fix any criterion without evidence before submitting.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0171756,
>     "input_tokens": 2,
>     "output_tokens": 949
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 4. path records/introspections/2026-10/20261014-181337-4ns7.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-181337-4ns7",
>   "at": "2026-10-14T18:13:37.990862237Z",
>   "trigger": "rejection",
>   "job": "20261014-174937-hrzn",
>   "intervention": null,
>   "class_of_work": "prompts",
>   "category": "rejected",
>   "what_happened": "The job \"Redo role-prompt goal-preservation change; fix every stale 1,800 mention\" (class of work: prompts) ended rejected. It is the second redo in a chain: the same work was rejected in 20261014-170031-2r9b and again in 20261014-172342-rqgb. The verifier returned fail: 1 of 11 acceptance criteria was unmet, with 1 blocker or major finding. The stored fields do not name the criterion or the finding. The count of criteria grew from 9 to 10 to 11 across attempts, and each attempt still failed one.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": false,
>   "earlier_cases": [
>     "20261014-174804-mgn1",
>     "20261014-172320-k3k0"
>   ],
>   "decisions_matched": [],
>   "cause": "Criteria are judged only by the final verifier, and a redo does not turn the previous rejection's finding into a checked item with evidence. So the same kind of gap can survive three attempts. \"Fix every mention\" has no enumerated search whose result can be compared against the criterion. The goal-preservation criterion for prompts has no defined test, so work is called done without evidence. The rejection record also does not carry the failing criterion and finding, which leaves the next attempt without the specific gap to fix.",
>   "prevention_kind": "design-change",
>   "prevention": "Add a pre-submission self-check step: before a job goes to the verifier, the worker lists each acceptance criterion with the evidence that it is met (for sweep criteria, the search command and its remaining-hit result). For a redo, the earlier verifier findings must be copied in as mandatory criteria and checked first. Make the rejection event store the failing criterion text and the finding, so the next attempt and later introspections can see them. A miss then triggers a repair pass inside the job instead of rejecting the whole job.",
>   "proposed_request": "Add a pre-verification self-check to Athena jobs. Before a job is submitted to the verifier, the worker must produce a table of every acceptance criterion with concrete evidence that it is met. For \"every mention\" or \"all occurrences\" criteria, the evidence must be a repo-wide search with its query and a zero-remaining-hits result. For redo jobs, the failing criteria and findings from every earlier rejection in the chain must be added as required criteria and checked first. If any row lacks evidence, the worker fixes it before submission. Also make rejected-job events record the unmet criterion text and the blocker/major findings. Then check this works on the role-prompt goal-preservation change: no stale \"1,800\" mention remains in any role prompt, and each prompt keeps the request's real goal through to the verdict. Done when the verifier passes all criteria, and a test job with a deliberately missed mention is caught by the self-check before it reaches the verifier.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0205536,
>     "input_tokens": 2,
>     "output_tokens": 1092
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 5. path records/introspections/2026-10/20261014-165951-resy.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-165951-resy",
>   "at": "2026-10-14T16:59:51.32050158Z",
>   "trigger": "rejection",
>   "job": "20261014-163844-c2kt",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked to gate shipping of an explainer on a fresh model reader's newcomer rating. It ended rejected: the verifier gave a fail verdict, with 1 of 8 acceptance criteria unmet and 2 blocker or major findings. The stored fields don't say which criterion failed or what the findings were.",
>   "failure_class": "20261014-163954-85d6",
>   "new_class": false,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "The request put a subjective quality bar (\"newcomer rating\" from a fresh model reader) in the outcome without fixing how to check it: which reader prompt, what rating scale, what pass threshold, and what counts as a blocker or major finding. Admission accepted the reading without those. The verifier then applied its own judgment to a bar the build never had in concrete form, so the build could not aim at it. The result was a rejection at the end instead of a settled check at the start. This is a loose fit with the registry class: the stored fields don't show whether Athena asked the user, only that the acceptance method was left open. A different cause is possible, such as the explainer simply being poor.",
>   "prevention_kind": "design-change",
>   "prevention": "When a reading's acceptance criterion depends on a subjective or model-judged rating, admission should require the reading to state the check as a rubric before the build starts. It should name the reader persona and prompt, the rating dimensions and scale, the pass threshold, and the severity definitions. The builder should run that same check on its own draft and iterate until it passes, before handing off to the verifier. A reading without these should be sent back to the user for the missing parts, or filled from a default rubric and shown for approval.",
>   "proposed_request": "Revise the explainer so a fresh model reader with no prior context rates it as clear to a newcomer. Use this check. A fresh reader is given only the explainer and asked to rate it 1-5 on four dimensions: can they say what it is, why it matters, how to start, and what the key terms mean. The reader must also list any blocker or major confusion. Pass means every dimension is at least 4 and there are no blocker or major findings. Run the check on your own draft, fix the findings, and repeat until it passes. Then report the final ratings and findings. Only after it passes, mark the explainer ready to ship. If the check cannot run, say so rather than shipping.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.015075599999999998,
>     "input_tokens": 2,
>     "output_tokens": 935
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 6. path records/introspections/2026-10/20261014-213402-g1b6.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-213402-g1b6",
>   "at": "2026-10-14T21:34:02.046860064Z",
>   "trigger": "rejection",
>   "job": "20261014-192610-xgke",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked to make the steward an agent that reads Athena broadly and queues work. It ended rejected. The verifier's verdict was fail, with 1 of 7 acceptance criteria unmet and 1 blocker or major finding. The stored fields don't say which criterion failed or what the finding was.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": false,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "Nothing in the job checked each acceptance criterion before the final verifier ran. The build finished without an interim check against the criteria, so a gap in one criterion surfaced only at the end, when rejection was the only option. The request's outcome (\"reads broadly\", \"queues work\") was also open-ended. Without observable checks for it, the build could not tell when it was done. This is a partial inference, because the stored fields don't name the unmet criterion.",
>   "prevention_kind": "design-change",
>   "prevention": "Before a job can be declared complete, require a self-check step. It runs every accepted criterion as a concrete check, such as a run showing the steward reads the named sources and writes items to the queue. It must record pass or fail per criterion. Any unmet criterion sends the job back to build, so the final verifier confirms rather than discovers. At admission, each criterion in the reading must state how it will be observed.",
>   "proposed_request": "Make the steward an agent that reads Athena broadly (jobs, events, introspections, class registry and decisions) and queues follow-up work. Done means: (1) a run of the steward visibly reads each of those sources; (2) it queues at least one well-formed work request when it finds a real gap, and queues none when it finds nothing; (3) queued items go through the normal admission path and don't duplicate items already queued; (4) a test or recorded demo run shows each of these behaviors happening, not just the code being present. Before finishing, check each acceptance criterion against that run and fix any that fail.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.021186,
>     "input_tokens": 2,
>     "output_tokens": 817
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [
>     "20261014-165951-resy"
>   ],
>   "from_jev": false
> }
> ```
> 
> ### 7. path records/introspections/2026-10/20261014-222026-jm1z.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261014-222026-jm1z",
>   "at": "2026-10-14T22:20:26.615408991Z",
>   "trigger": "rejection",
>   "job": "20261014-221551-09t4",
>   "intervention": null,
>   "class_of_work": "docs",
>   "category": "rejected",
>   "what_happened": "A docs job to make the firstmate explainer page followable by newcomers finished its work, but the final verdict was fail because a check test failed. The job ended rejected, and the stored fields don't say which check failed or why.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": false,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "The \"followable by newcomers\" outcome was subjective and probably never became a concrete, runnable check before the work began. The failing check was found only at final verification, so nothing made the job run or confirm it earlier. The stored record also doesn't say which check failed, so the cause can't be confirmed. This is an inference from the thin record.",
>   "prevention_kind": "design-change",
>   "prevention": "At admission, turn each outcome into acceptance criteria that name a runnable check. For docs, that means link and build checks plus a defined newcomer-readability check. Require the job to run those checks before it ends its work phase and fix any failure then. Record the failing check's name and output in the rejection so later introspection can classify the cause.",
>   "proposed_request": "Make the firstmate explainer page followable by a newcomer who knows nothing about the project. Before finishing, run the repo's existing docs checks and tests on the page and fix every failure. Acceptance: the page defines each term on first use, walks through a basic example from start to finish, and has working links. All repo checks and tests pass. If a check fails, report its name and output.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0137936,
>     "input_tokens": 2,
>     "output_tokens": 648
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 8. path records/jobs/2026-10/20261014-170031-2r9b/verification.json
> ```
> {
>   "schema": "athena.verification/1",
>   "job": "20261014-170031-2r9b",
>   "candidate_commit": "4d2104e3c7ee672ecf3833d868da6dc0b0783779",
>   "verified_fingerprint": "sha256:a57a1a65bc93c877eb83ba77f6bad508156c8926b5bc8415ea966e2111d05b32",
>   "tool_config_from_base": [],
>   "checks": [
>     {
>       "name": "fmt",
>       "command": [
>         "cargo",
>         "fmt",
>         "--all",
>         "--check"
>       ],
>       "exit_code": 0,
>       "passed": true,
>       "tests_ran": null,
>       "duration": {
>         "value": 0.80585587,
>         "unit": "seconds",
>         "provenance": "measured",
>         "basis": "athena monotonic clock"
>       },
>       "output_tail": "",
>       "note": null
>     },
>     {
>       "name": "clippy",
>       "command": [
>         "cargo",
>         "clippy",
>         "--all-targets",
>         "--locked",
>         "--",
>         "-D",
>         "warnings"
>       ],
>       "exit_code": 0,
>       "passed": true,
>       "tests_ran": null,
>       "duration": {
>         "value": 21.435787366,
>         "unit": "seconds",
>         "provenance": "measured",
>         "basis": "athena monotonic clock"
>       },
>       "output_tail": "   Compiling proc-macro2 v1.0.107\n   Compiling unicode-ident v1.0.26\n   Compiling quote v1.0.47\n   Compiling serde_core v1.0.229\n    Checking cfg-if v1.0.5\n   Compiling version_check v0.9.5\n   Compiling libc v0.2.190\n   Compiling generic-array v0.14.7\n    Checking typenum v1.20.1\n   Compiling syn v3.0.7\n    Checking memchr v2.8.3\n   Compiling getrandom v0.3.4\n    Checking crypto-common v0.1.7\n    Checking block-buffer v0.10.4\n   Compiling thiserror v2.0.21\n   Compiling zmij v1.0.23\n    Checking digest v0.10.7\n   Compiling serde v1.0.229\n   Compiling serde_json v1.0.151\n    Checking winnow v1.0.4\n    Checking cpufeatures v0.2.17\n    Checking sha2 v0.10.9\n    Checking toml_parser v1.1.5+spec-1.1.0\n    Checking toml_datetime v0.7.5+spec-1.1.0\n    Checking itoa v1.0.18\n    Checking winnow v0.7.15\n    Checking toml_writer v1.1.3+spec-1.1.0\n    Checking serde_spanned v1.1.2\n    Checking jiff-core v0.1.1\n   Compiling rustix v1.1.5\n    Checking jiff v0.2.38\n    Checking bitflags v2.13.2\n    Checking linux-raw-sys v0.12.1\n    Checking toml v0.9.12+spec-1.1.0\n   Compiling thiserror-impl v2.0.21\n   Compiling serde_derive v1.0.229\n    Checking once_cell v1.21.4\n    Checking fastrand v2.5.0\n   Compiling ref-cast v1.0.27\n    Checking tempfile v3.27.0\n   Compiling ref-cast-impl v1.0.27\n   Compiling serde_derive_internals v0.30.0\n    Checking aho-corasick v1.1.5\n    Checking regex-syntax v0.8.11\n   Compiling anyhow v1.0.104\n   Compiling schemars_derive v1.2.2\n    Checking dyn-clone v1.0.20\n    Checking athena-git v0.1.0 (<job>/verify/crates/athena-git)\n    Checking equivalent v1.0.2\n    Checking hashbrown v0.17.1\n    Checking regex-automata v0.4.18\n    Checking indexmap v2.14.2\n    Checking utf8parse v0.2.2\n    Checking athena-data v0.1.0 (<job>/verify/crates/athena-data)\n    Checking schemars v1.2.2\n    Checking toml_edit v0.23.10+spec-1.0.0\n    Checking anstyle-parse v1.0.0\n    Checking anstyle-query v1.1.5\n    Checking colorchoice v1.0.5\n    Checking is_terminal_polyfill v1.70.2\n    Checking anstyle v1.0.14\n   Compiling heck v0.5.0\n    Checking regex v1.13.1\n    Checking anstream v1.0.0\n    Checking strsim v0.11.1\n    Checking clap_lex v1.1.1\n   Compiling clap_derive v4.6.7\n    Checking clap_builder v4.6.7\n   Compiling crc32fast v1.5.2\n    Checking adler2 v2.0.1\n    Checking simd-adler32 v0.3.10\n    Checking miniz_oxide v0.9.1\n    Checking xattr v1.6.1\n    Checking filetime v0.2.29\n    Checking tar v0.4.46\n    Checking flate2 v1.1.10\n    Checking clap v4.6.7\n    Checking athena-context v0.1.0 (<job>/verify/crates/athena-context)\n    Checking athena-score v0.1.0 (<job>/verify/crates/athena-score)\n    Checking athena-agent v0.1.0 (<job>/verify/crates/athena-agent)\n    Checking athena-gate v0.1.0 (<job>/verify/crates/athena-gate)\n    Checking athena-core v0.1.0 (<job>/verify/crates/athena-core)\n    Checking xtask v0.1.0 (<job>/verify/xtask)\n    Checking athena-cap-study v0.1.0 (<job>/verify/crates/athena-cap-study)\n    Checking athena-cap-change v0.1.0 (<job>/verify/crates/athena-cap-change)\n    Checking athena-cli v0.1.0 (<job>/verify/crates/athena-cli)\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.15s\n",
>       "note": null
>     },
>     {
>       "name": "test",
>       "command": [
>         "cargo",
>         "test",
>         "--locked"
>       ],
>       "exit_code": 0,
>       "passed": true,
>       "tests_ran": 547,
>       "duration": {
>         "value": 248.279345438,
>         "unit": "seconds",
>         "provenance": "measured",
>         "basis": "athena monotonic clock"
>       },
>       "output_tail": "… in 0.66s\n\n     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_score-45e65d08d8fd5961)\n\nrunning 10 tests\ntest tests::brier_values ... ok\ntest tests::omitting_a_changed_path_never_helps ... ok\ntest tests::padding_deliverables_never_lowers_the_score ... ok\ntest tests::brier_is_strictly_proper_on_a_grid ... ok\ntest tests::vague_forecasts_score_worse_than_sharp_calibrated_ones ... ok\ntest tests::skill_is_withheld_below_the_minimum_job_count ... ok\ntest tests::wis_rewards_tight_correct_intervals_and_punishes_misses ... ok\ntest tests::wis_median_term_is_proper_for_the_median ... ok\ntest tests::zero_cost_is_floored_not_undefined ... ok\ntest tests::reference_uses_priors_without_history_and_moves_with_it ... ok\n\ntest result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n     Running unittests src/main.rs (<job>/target/verify/debug/deps/xtask-fa076530dc5838eb)\n\nrunning 12 tests\ntest site::tests::a_citation_of_an_unknown_claim_is_an_error ... ok\ntest site::tests::missing_study_data_is_an_error_naming_the_file ... ok\ntest site::tests::the_intro_and_glossary_come_before_the_states ... ok\ntest site::tests::text_is_html_escaped ... ok\ntest s
> [clipped: 12181 more characters]
> ```
> 
> ### 9. job 20261014-221551-09t4
> Request:
> ```
> Someone who's never seen firstmate can't follow the explainer page at https://doodla.github.io/athena/. Make it useful to them.
> ```
> Read as: Make the firstmate explainer page followable by newcomers (docs). The page already has a "Start here" intro and glossary (intro.json, xtask/src/site.rs), but the sections after it still show raw ids (t1, c12), "decided by agent/code" fields and terse state tags that a newcomer can't read. I'll make those sections self-explanatory with cited plain-language lead-ins and card summaries, so a reader who has never seen firstmate can follow the page top to bottom.
> Outcome: rejected. the final verdict is fail: check test failed
> 
> 
> ### 10. job 20261014-170031-2r9b
> Request:
> ```
> Rework the role prompts so a request's real goal survives to the verdict; job 20261014-154909-f4bt shows how it is lost. Its reader turned 'useful to someone who has never seen firstmate' into presence checks (a section exists, a glossary exists), never looked at the page, and put the real gaps out of scope; the producer is told to change only what the criteria need; the verifier saw that the page never says what firstmate is but filed it minor because the criteria passed, and request_reading_mismatch is not used in any verdict. Change it so: the reader names the audience and writes at least one criterion as an outcome that audience can check, never puts the request's own goal out of scope, and reads the current output before planning a change to it; the producer treats criteria as a floor and serves the request; a request-reading mismatch from the verifier counts as major in code; the verifier judges user-facing output as its audience would. The reader's fixed prompt is at 1,798 of 1,800 tokens; raise that budget as far as these rules need, and record why.
> ```
> Read as: Make role prompts carry a request's real goal through to the verdict (prompts). Edit the reader, producer and verifier prompts so the reader names the audience, writes an outcome criterion and reads current output; the producer treats criteria as a floor; and the verifier judges output as its audience would. In judge(), a non-empty request_reading_mismatch becomes a Major finding that blocks a pass. Raise the reader budget just enough for the new text and record why.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 of 9 criteria unmet; 1 blocker or major finding(s)
> 
> 
> ### 11. job 20261014-163844-c2kt
> Request:
> ```
> Athena's verifier passed the firstmate explainer twice, but an independent reader rates it 2 of 5 for a newcomer both times. Athena should be able to tell when an explainer it ships isn't useful, before it ships.
> ```
> Read as: Gate explainer shipping on a fresh model reader's newcomer rating (code). Before an explainer ships, a fresh model reader with no part in building it reads the rendered page as a newcomer and rates it 1–5. A rating below 4 blocks shipping. Cheap mechanical checks on the intro and glossary sit alongside the rating.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 of 8 criteria unmet; 2 blocker or major finding(s)

## Reading (accepted by the user)

When a rejected or failed job is introspected, build the event's `what` from the job's verification.json (unmet criteria, blocker/major findings, failed checks with their names) instead of only the title and outcome reason. The model then stops saying the stored fields omit them and no longer files a failed test check as an unmet criterion. The audience is the steward and the person reading introspections to group rejections by real cause.

Assumptions:
- The root cause is that event_of_job builds `what` from only the title and outcome.reason, so the model never sees verification.json; the fix is to read it there (JobRecords already has a `verification: Option<Verification>` field).
- Surfacing failed checks separately addresses the misfiling of job 20261014-221551-09t4's test failure as an unmet criterion.
- The module doc's 'reads only the stored fields it lists' rule is extended to name these verification fields; transcripts and diffs stay excluded.
- The schema athena.introspection/1 needs no change since the extra information lives in the existing `what` input.

## Acceptance criteria

1. [content] event_of_job in crates/athena-core/src/introspect.rs reads the job's stored Verification and puts each unmet criterion's text and each blocker or major finding (severity, summary, location) into the Event's `what`; the field list in the module docs (and defaults-loop.md section 3.3 if it lists fields) names these additions.
2. [content] A failed check (CheckRun with passed=false) is rendered in `what` under a separate, explicitly labelled 'failed checks' heading with its name and a bounded output_tail, never in the unmet-criteria list; the system prompt tells the model that criteria and checks are different things.
3. [content] Added text is bounded (counts and character caps), redacted like the rest of the prompt, and still excludes transcripts, diffs and other jobs' files; a job with no verification.json (e.g. failed before verification) still produces the old `what`.
4. [behavior] New unit tests in introspect.rs cover: a rejection with one unmet criterion and a major finding (both appear in the prompt), a rejection whose only failure is a failed test check (appears as failed check, no unmet criterion), and a job without verification. Run `cargo test -p athena-core introspect`; `cargo test` passes, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all --check` are clean.
5. [behavior] Outcome check for the steward: with the fake model, the user prompt for a rejected job built from a verification.json like that of job 20261014-170031-2r9b contains the text of the unmet criterion 8 and the major finding location, so a reader of the resulting introspection need not say 'the stored fields do not name them'.
6. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)
- docs/design/decision-sets.md (modify)

## Out of scope

- Rewriting or re-running the six existing introspection records
- Adding verification data to the rejection outcome.json itself
- Host-error and landing introspections
- Changing Jev shadow behaviour or the introspection schema

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the input the model is given, not the six records or the wording of one prompt.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the three job shapes; test, clippy, fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A behavior criterion exercises the prompt built from a real-shaped rejection.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in introspect.rs plus changelog.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The sources are enforced in code, with tests, not by prompt prose alone.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments written for an outside reader.
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
