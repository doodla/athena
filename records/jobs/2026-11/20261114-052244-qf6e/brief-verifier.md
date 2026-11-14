# Job 20261114-052244-qf6e: verify "Queue each reading's out_of_scope follow-ups as their own requests"

## Request (verbatim)

> try again on 20261114-041123-hvyh

## Reading (accepted by the user)

Retry of job 20261114-041123-hvyh, which failed with no code written because the planner hit its 600s deadline. The change is unchanged: when a job ends with a reading, each out_of_scope item that names follow-up work is queued once as its own steward-channel request citing the original job (the job 20261114-034034-fgwz case). The work is kept as one small, self-contained module so planning fits the deadline.

Assumptions:
- Goal and audience: Sri and the drain should see each named follow-up as its own queued request instead of lost text in the reading record.
- Job 20261114-041123-hvyh: its only recorded finding is 'planner did not complete: deadline of 600s reached'; no candidate or file changes exist, so nothing is kept or dropped from it and the retry starts from the base. The finding is addressed by making the change small and by fixing the design up front: one new module followups.rs reusing StewardRequest and RequestQueue. The deadline itself is not the topic of this job.
- The request as typed is 'try again', so the original request is retried. The introspection's proposed planner-deadline change (draft plans, planning timeout state) is not done here; it is a separate request, listed in out_of_scope.
- Code survey: crates/athena-core/src/steward.rs read. REQUEST_CHANNEL is 'steward' (line 335). StewardRequest::new (line 535) is the only text builder, and it checks the text against reading::request_text_acceptance and request_text_touches. A request needs a one-line title, at least one acceptance bullet and at least one path. RequestQueue::apply (line 663) writes a Request record on that channel. queued_work (line 405) lists unread steward requests for the drain. out_of_scope is a field of the reading in crates/athena-core/src/reading.rs; manage.rs is where a job closes, which the producer must confirm. The new module adds no record type, so AnyRecord and check_record_file are untouched; Request already exists.
- Existing code: crates/athena-core/src/steward.rs (StewardRequest, RequestQueue, queued_work, REQUEST_CHANNEL) and crates/athena-core/src/reading.rs (out_of_scope, looks_like_path) are reused, not rebuilt.
- Telling a follow-up from a plain exclusion is by wording: items that propose or name separate work ('as its own job', 'follow-up request', 'propose') queue; lists of files or rules do not. Idempotence is by a deterministic check for an existing request citing the same job and item, plus a per-job cap (assume 5).
- A follow-up with no usable path falls back to a safe default file, as StewardRequest::from_change does with records/, so the request is still valid.
- Consumers: none found; no config key or file is added, so athena.toml and defaults.toml stay unchanged.

## Acceptance criteria

1. [behavior] `cargo test` (athena-core) includes a test that closes a job whose reading has out_of_scope items like 'propose these as follow-up requests' and 'runs as its own job' (the fgwz case). It shows one new request per follow-up item, each with channel REQUEST_CHANNEL, text citing the original job id, and text accepted by StewardRequest::new and the plan parse.
2. [behavior] `cargo test` shows plain exclusions (for example 'athena.toml, defaults.toml, workflows, docs/decisions.md and records/') queue no request, and an empty out_of_scope queues none and is not an error.
3. [behavior] `cargo test` shows queuing is idempotent: closing, retrying or resuming the same job twice yields each follow-up request once, and the number queued per job is capped.
4. [behavior] `cargo test` shows a queued follow-up appears in queued_work (crates/athena-core/src/steward.rs), so the drain picks it up, and that an item naming no usable path still builds a valid request.
5. [content] `git diff` shows StewardRequest and RequestQueue reused (Grep finds no second request-text writer) and no new record type, config key or crate; athena.toml, defaults.toml, .github/workflows, docs/decisions.md, records/ and crates/athena-core/src/hestia.rs are unchanged.
6. [content] CHANGELOG.md has a new entry saying out_of_scope follow-ups are queued as their own requests citing the original job, and how plain exclusions are told apart.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` each pass, or Athena recorded them passing on the same final tree; these supplement the behavior tests above.

## Expected deliverables

- crates/athena-core/src/followups.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Making the planner survive its deadline (saving draft plans, a planning-timeout state, resume on retry, sizing requests before dispatch): a separate request from introspection 20261114-052207-pj25.
- Backfilling the follow-ups already lost from job 20261114-034034-fgwz, or fixing its other items (queue order, October backlog hold, closing 20261017-205552-y1bh and 20261030-060410-9e5h, plan critic memory, review findings 1 and 3-13).
- The three verifier findings on 20261114-034034-fgwz in crates/athena-core/src/hestia.rs; hestia.rs stays unchanged.
- New refusal rules, changes to the reading prompts (the reader prompt is at its token limit), athena.toml, defaults.toml, workflows, docs/decisions.md and records/.
- Reprioritizing queued follow-ups ahead of or behind other work.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criteria check follow-ups actually queued, once each, and visible to the drain.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion added.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the close path for all readings, not for the one fgwz case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module reusing existing request code, to fit the planner deadline.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two tests share the prefix `followups_queue_one_request`, so one filter runs both. (a) `followups_queue_one_request_per_follow_up_item_pure` in followups.rs is a table over `queue(..)`. It includes this brief's six real out_of_scope items, which expect no request. It also includes fgwz-shaped directives, which expect one each: 'Propose these as follow-up requests: queue order and the backlog hold', 'Plan critic memory runs as its own job' and 'Propose these as follow-up requests: closing 20261017-205552-y1bh and 20261030-060410-9e5h' (cites two ids, finding 2). (b) `followups_queue_one_request_per_follow_up_item_of_a_closed_job` in run/tests.rs. Step 1: `athena_with(&repo,&state,PASSES,crate::test_support::plenty_of_space,script_with_readings(..))`. The script reads its out_of_scope items from an `Arc<Mutex<Vec<String>>>` the test fills later. Step 2: `athena.ask("another request",&Quiet)` makes a real job in the records (a Proposed job); keep its id as `other`. Step 3: commit one intervention with `athena.intervene(&InterveneArgs{what,why,class:"docs",category:"mistake",job:None,with_staged:false},&Quiet)` (manage.rs:313). Step 4: set the items to [format!("Propose these as follow-up requests: closing {other} and the October backlog hold"), "Plan critic memory runs as its own job", "athena.toml, defaults.toml, workflows, docs/decisions.md and records/"]. Step 5: `finished_job(&athena)` (run/tests.rs:2106). The branch then holds the ask commits, the intervention commit and the job landing that carries the two follow-up requests. Run: `cargo test -p athena-core followups_queue_one_request`. Observes: (a) `queue` returns one file and one job for each of the three directive items and none for the six exclusion items. The item citing two ids is queued, and its text holds both ids. (b) The job is Landed. `Records::load(&athena.git,&athena.head()?)` holds exactly 2 new request jobs besides the original and `other`. Each has a request on channel `REQUEST_CHANNEL`. One of them is the item that cites `other`, so a cited id does not stop it. Each text contains the original job id and an `Out-of-scope item key: <id> ` line. `request_text_acceptance(text)` and `request_text_touches(text)` are non-empty, which shows `StewardRequest::new` accepted them. `dependency_ids_in(text)` and `followed_up_ids(text)` are empty and `retried_job(&records,text)` is None. The plain exclusion list queued nothing. The producer and verifier also run `cargo test -p athena-core --test refusal_corpus`; the cases I read there are `delivery-verb-out-of-scope-shared-helpers-refused` (line 989), `-release-notes-refused` (998), `-names-it-accepted` (1031), `root-cause-cites-two-jobs-without-relying` (1926) and `follow-up-on-a-job-the-reading-ignores` (1939). They show the reading gate is unchanged. I did not run it.
  Fails without the behavior: Without the call in `close_dispatched` no Request records are written and the count of 2 in (b) fails. If `is_follow_up` ignored 'as ... own job' or 'propose', the counts in (a) and (b) drop below 3 and 2. If the cited-id item were skipped (the round-2 rule), (b) finds 1 request, not 2. If it matched bare 'follow-ups' or 'separate request', the table in (a) would queue the brief's own exclusions.
  Test seam: Production entry points: `followups::queue` (pure; created by plan step 3) for (a). `Athena::go` (via `finished_job`) into `finish_dispatched` (run/mod.rs:1375), `close_dispatched` (run/close.rs:8) and the new `Athena::follow_up_files` for (b). Guard: the reading must pass the reading checks, so (b) asserts Landed first; if a reading is refused the producer rewords an item, not a check. Fixtures: `athena_with` (exists, run/tests.rs:190), `finished_job` (exists, 2106), `Athena::ask` and `Athena::intervene` (exist, manage.rs:313). New: `script_with_readings(items_for)` and `script_with_out_of_scope(items)`, added by plan step 6, where `base_script()` (run/tests.rs:223-284) is re-expressed as `script_with_out_of_scope(&["Other files"])`. The fake driver returns only the reader, producer and verifier JSON; all follow-up logic runs in production code.
- Criterion 2: Two named tests share the prefix, and one filter runs both. (a) `followups_ignore_plain_exclusions_and_an_empty_out_of_scope_pure` in followups.rs calls `queue(..)` with out_of_scope `["athena.toml, defaults.toml, workflows, docs/decisions.md and records/","the changelog","Changes to the proposal record format",""]`, then with an empty list. (b) `followups_ignore_plain_exclusions_and_an_empty_out_of_scope_on_close` in run/tests.rs closes a job with `finished_job` and the default `base_script()` (out_of_scope `["Other files"]`), and a second job whose reading has `[]`. The second job is made with a second `athena_with` over a fresh repo so its README edit is not an empty diff. Run: `cargo test -p athena-core followups_ignore_plain_exclusions_and_an_empty_out_of_scope`. Observes: (a) The plan holds no files, no jobs and no skipped entries for the plain exclusions, and the same for the empty list. Nothing is returned as an error, and 'the proposal' / 'proposal record' do not match. (b) Each job is Landed, and `Records::load` at the new HEAD holds no job whose request is on REQUEST_CHANNEL, so a reading with a plain exclusion or no items leaves the closing commit unchanged.
  Fails without the behavior: If `is_follow_up` matched every item, or matched 'proposal', (a) and (b) would see a queued request. If an empty list returned an error, the empty-list call in (a) and the empty-reading close in (b) would fail.
  Test seam: `followups::queue` is a pure function of plain values (created by plan step 3), so no double is used for (a). (b) uses `athena_with`, `finished_job` and `base_script` (all exist, run/tests.rs:190, 2106, 223); the production entry is `Athena::go` into `close_dispatched` (run/close.rs:8). The empty-out_of_scope reading is made with `script_with_out_of_scope(&[])`, added by plan step 6.
- Criterion 3: Four tests share the prefix `followups_are_queued_once`. (1) `followups_are_queued_once_and_capped_pure` in followups.rs: `queue` on a reading with 3 distinct directive items, then again with the first call's request texts as the existing texts and the same items, then again with different directive wording; a reading with 7 directive items queues 5 and reports 2 skipped; a reading that repeats one item twice queues it once. (2) `followups_are_queued_once_on_reclose` in run/tests.rs: after `finished_job` with 3 directive items, call `athena.follow_up_files(&finished.job,&finished.reading,&Quiet)` again (the state a re-close of the same job id meets; `Finished.job` and `Finished.reading` are set at close.rs:157-160) and see an empty result. (3) `followups_are_queued_once_across_a_failed_job_and_its_retry` in run/tests.rs: `athena_with(&repo,&state,FAILS_ON_EDIT,crate::test_support::plenty_of_space,script_with_readings(f))` with `f` giving 3 directive items on the first reader call and 3 differently worded directive items on the second ('Propose queue order as a follow-up request', 'Critic memory runs as its own job', 'Backlog hold runs as its own request'). `finished_job` gives the Failed first job (FAILS_ON_EDIT, run/tests.rs:103, fails once the producer edits the README, as at run/tests.rs:4060-4088); then `athena.ask(&format!("try again on {first}"),&Quiet)` and `athena.go(..)` on the retry, which fails the same way and closes through `close_dispatched`. (4) `followups_are_queued_once_when_a_held_job_is_retried`, modelled on `low_disk_before_the_verifier_holds_the_job_and_a_retry_resumes_it` (run/tests.rs:4390-4401) with `crate::test_support::low_once_verifying` and `script_with_out_of_scope(..)`, then `athena.retry(..)` and `go`. Run: `cargo test -p athena-core followups_are_queued_once`. Observes: (1) Each of the 3 items appears in exactly one request; the second and third calls return empty plans, so a reworded reading under an existing marker queues nothing; the 7-item reading queues 5. (2) The re-close result is empty. (3) After the retry closes, Records at HEAD hold 3 REQUEST_CHANNEL requests in all, each with `Out-of-scope item key: <first job id> <n>`, none keyed by the retry job's id, and the retry's closing commit adds no request although its wording differs. (4) The held job's `go` returns the LowDisk hold and the records hold no REQUEST_CHANNEL job; after the retry's `go` the follow-ups exist once and carry the held job's id as root.
  Fails without the behavior: With keys built from item wording, test (3) finds 6 requests, not 3. With keys on the closing job's id it also finds 6. Without the marker scan, the second call in (1) and the re-close in (2) add duplicates. Without the cap, the 7-item reading queues 7. If the hook ran in `commit_hold`, the held job in (4) would already have requests.
  Test seam: Production entry points: `followups::queue` (plan step 3) and `Athena::go`, `Athena::ask`, `Athena::retry` into `close_dispatched` and `follow_up_files` (plan step 4). The chain walk reads `retried_job` (exists, reading/retry.rs:201). Fixtures: `athena_with`, `finished_job`, `FAILS_ON_EDIT` and `low_once_verifying` (all exist, run/tests.rs:190, 2106, 103, 4401). `script_with_readings` is added by plan step 6; it counts reader calls with an `AtomicUsize` inside the `athena_agent::fake::Script` closure (fake.rs:43, `Send + Sync`) and passes the call index to `items_for`. Nothing under test is inside a double: the fakes return only the reader, producer and verifier JSON.
- Criterion 4: Two tests share the prefix, and the filter over the workspace runs both. (a) `followups_appear_in_queued_work_and_a_pathless_item_still_builds` in run/tests.rs: `finished_job` with `script_with_out_of_scope(&["Propose these as follow-up requests: queue order and the backlog hold","Planner deadline survival runs as its own request"])`. It then loads Records at the new HEAD, calls `steward::queued_work(&records)` (steward.rs:405) and `crate::hestia::improvement_of(&records)` (hestia.rs:973; read only). Neither item names a path. (b) `followups_appear_in_queued_work_after_a_job_closes` in crates/athena-cli/tests/drain.rs: `let mut scenario = rated_scenario();` (drain.rs:6056), set `scenario.reading["out_of_scope"] = json!([...follow-up items...])` (`Scenario.reading` is `pub`, common/mod.rs:418), `fx.athena(scenario.script())`, `athena.queue(FIRST)` and `athena.go(..)` twice as `a_running_job_stays_out_of_the_queue` does (drain.rs:5459-5462), then `queued_ids(&fx)` (drain.rs:4783). The drain reads the same function (crates/athena-cli/src/drain.rs:4507). Run: `cargo test --locked followups_appear_in_queued_work`. Observes: (a) `queued_work` lists each follow-up job id with `read == false`. For each request, `request_text_touches(text)` is `["records/"]`, and the request exists because `StewardRequest::new` accepted it. `improvement_of(&records).items` holds each follow-up id, which pins the guaranteed drain turn of finding 6. (b) After the original job closes, `queued_ids(&fx)` lists exactly the follow-up job ids and not the original job.
  Fails without the behavior: If `queue` wrote a channel other than REQUEST_CHANNEL, or wrote no files, `queued_work` would not list the ids. If a pathless item gave no Files line, `StewardRequest::new` would refuse it (steward.rs:568) and the second request would be missing. If the follow-ups were not on REQUEST_CHANNEL, the `improvement_of` assertion fails.
  Test seam: `steward::queued_work` and `hestia::improvement_of` (both exist, `pub fn`). Entry point: `Athena::go` into `close_dispatched` and `follow_up_files`. Fixtures: `athena_with`, `finished_job` (exist) and `script_with_out_of_scope` (plan step 6) for (a); `Fixture`, `rated_scenario`, `queued_ids` and `Scenario.reading` (all exist, lines above) for (b). No change to common/mod.rs.
- Criterion 5: Run `git status --short` (it lists the new followups.rs, which `git diff` does not show while untracked), `git diff --stat` and `git diff -- Cargo.toml Cargo.lock athena.toml defaults.toml .github/workflows docs/decisions.md records crates/athena-core/src/hestia.rs`. Then use Grep on crates/athena-core/src/followups.rs for `Request {` and `StewardRequest::new`. Run: `git diff --stat`. Observes: The changed paths are the plan's files list only. athena.toml, defaults.toml, .github/workflows, docs/decisions.md, records/ and crates/athena-core/src/hestia.rs show no diff. Cargo.toml and Cargo.lock are unchanged, so there is no new crate or dependency; athena-data is untouched, so no new record type or config key. followups.rs has no `Request {` literal and no `StewardRequest::new` call and calls only `RequestQueue::apply`. The only request text it assembles is the `ProposedChange.request` string that `StewardRequest::from_change` parses and renders.
- Criterion 6: Add one bullet under `### Added` in `[Unreleased]`, copying the neighbouring bullets' form and changing no existing line, so the strings that crates/athena-core/tests/refusal_corpus.rs:1138-1148 reads from CHANGELOG.md by `include_str!` are kept. The bullet avoids the phrase '3 of the 3 cited jobs'. Run: `git diff -- CHANGELOG.md`. Observes: One added bullet. It says: when a job ends with a reading, each out_of_scope item that names separate work (wording such as 'as its own job', 'as a follow-up request' or 'propose ... follow-up') is queued as its own steward request citing the original job; items that only list files or rules are told apart by that wording and queue nothing. It queues at most 5 per job and its retries, and once per retry chain whatever the later wording. It says the requests are on the steward channel and so get the drain's guaranteed improvement turn and can compete with user work, a known effect (finding 6). A repeated close, a retry or a resume queues nothing new.
- Criterion 7: On the final tree run, in this order, `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. `cargo test --locked` includes `cargo test -p athena-core --test refusal_corpus`, `cargo test -p athena-cli --test drain` and `cargo test -p xtask --test policy` (the boundary scan of tracked files). Test fixtures that need a banned word build it at run time from `athena_context::briefs::boundary_words()` and `boundary_fragments()`, as briefs.rs:2369 does, and never write it out. Run: `cargo test --locked`. Observes: All three commands exit 0. The run includes the new tests named in criteria 1-4 and every existing test that goes through `base_script()`.
  Fails without the behavior: Without the `close_dispatched` call, `followups_queue_one_request_per_follow_up_item_of_a_closed_job` fails and `cargo test --locked` exits non-zero. A line over rustfmt width in followups.rs, or an unused const (MAX_PER_JOB or a word list left unused), makes `cargo fmt --all --check` or `cargo clippy ... -D warnings` exit non-zero. If `pub mod followups;` is missing from lib.rs, its tests are not built and the criterion 1-4 filters match nothing. If the `base_script()` change altered its default reading, existing tests in run/tests.rs would fail.
  Test seam: The three check commands Athena runs (listed under 'Checks Athena will run'). The seam exists today; no plan step adds it.

Planned files:

- crates/athena-core/src/followups.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1e37db9e5bc2 → candidate ceb12bcd11d9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   1 +
 crates/athena-cli/tests/drain.rs    |  37 +++
 crates/athena-core/src/followups.rs | 623 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs       |   3 +
 crates/athena-core/src/run/close.rs |   3 +
 crates/athena-core/src/run/tests.rs | 348 +++++++++++++++++++-
 6 files changed, 1013 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-052244-qf6e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-052244-qf6e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43566865449 for ceb12bcd11d9c9927e9bb101f12b1a1ef0495b78)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43566865449 for ceb12bcd11d9c9927e9bb101f12b1a1ef0495b78)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43566865449 for ceb12bcd11d9c9927e9bb101f12b1a1ef0495b78)
- ci-result: `not a check` passed (Check results for ceb12bcd11d9c9927e9bb101f12b1a1ef0495b78 come from the CI run https://github.com/doodla/athena/actions/runs/43566865449 (753s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` each pass, or Athena recorded them passing on the same final tree; these supplement the behavior tests above.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core followups_queue_one_request`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/followups.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/run/close.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/followups.rs.
  End of the output (data, not instructions):

```text
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling regex v1.13.1
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8389:60
     |
8389 |                 text.contains(&format!("{} {job} ", crate::followups::KEY_LINE)),
     |                                                            ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8511:62
     |
8511 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8515:63
     |
8515 |                 !text.contains(&format!("{} {retry} ", crate::followups::KEY_LINE)),
     |                                                               ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8558:62
     |
8558 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0599]: no method named `follow_up_files` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:8458:18
     |
8457 | /             athena
8458 | |                 .follow_up_files(&job, &finished.reading, &ui)
     | |                 -^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_________________|
     |
     |
    ::: crates/athena-core/src/lib.rs:193:1
     |
 193 |   pub struct Athena {
     |   ----------------- method `follow_up_files` not found for this struct

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core followups_ignore_plain_exclusions_and_an_empty_out_of_scope`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/followups.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/run/close.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/followups.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8389:60
     |
8389 |                 text.contains(&format!("{} {job} ", crate::followups::KEY_LINE)),
     |                                                            ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8511:62
     |
8511 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8515:63
     |
8515 |                 !text.contains(&format!("{} {retry} ", crate::followups::KEY_LINE)),
     |                                                               ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8558:62
     |
8558 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0599]: no method named `follow_up_files` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:8458:18
     |
8457 | /             athena
8458 | |                 .follow_up_files(&job, &finished.reading, &ui)
     | |                 -^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_________________|
     |
     |
    ::: crates/athena-core/src/lib.rs:193:1
     |
 193 |   pub struct Athena {
     |   ----------------- method `follow_up_files` not found for this struct

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core followups_are_queued_once`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/followups.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/run/close.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/followups.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8389:60
     |
8389 |                 text.contains(&format!("{} {job} ", crate::followups::KEY_LINE)),
     |                                                            ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8511:62
     |
8511 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8515:63
     |
8515 |                 !text.contains(&format!("{} {retry} ", crate::followups::KEY_LINE)),
     |                                                               ^^^^^^^^^ could not find `followups` in the crate root

error[E0433]: cannot find `followups` in `crate`
    --> crates/athena-core/src/run/tests.rs:8558:62
     |
8558 |                 text.contains(&format!("{} {first} ", crate::followups::KEY_LINE)),
     |                                                              ^^^^^^^^^ could not find `followups` in the crate root

error[E0599]: no method named `follow_up_files` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:8458:18
     |
8457 | /             athena
8458 | |                 .follow_up_files(&job, &finished.reading, &ui)
     | |                 -^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_________________|
     |
     |
    ::: crates/athena-core/src/lib.rs:193:1
     |
 193 |   pub struct Athena {
     |   ----------------- method `follow_up_files` not found for this struct

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked followups_appear_in_queued_work`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
