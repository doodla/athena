# Job 20261114-041123-hvyh: Queue each reading's out_of_scope follow-ups as their own requests

## Request (verbatim)

> Requested by Sri (the owner). When a reading puts work out of scope as follow-up requests, queue each named follow-up as its own request, citing the original job, instead of leaving it as text in the reading record. Today nothing turns out_of_scope follow-ups into requests, so the work is lost: job 20261114-034034-fgwz read Sri's request as one Hestia change and listed the rest (queue order, holding the October backlog, review findings) as follow-ups that were never queued.

## Reading (accepted by the user)

Athena today records a reading's out_of_scope items as text only, so named follow-ups are lost (job 20261114-034034-fgwz). The change makes the run, once a job ends with a reading, file each follow-up item as its own queued steward-channel request that cites the original job, once and only once per item. Audience: Sri and the drain, who should see the follow-ups in the queue.

Assumptions:
- Code survey: read crates/athena-core/src/steward.rs (StewardRequest::new/from_change at ~526-640 builds a request text that must have a title, an Acceptance: bullet list and a Files: line; RequestQueue::apply at ~662 writes a Request record with channel REQUEST_CHANNEL and a fresh RecordId; Decision/decide at ~698), crates/athena-data/src/records.rs (Request {job,text,received_at,channel} at 434; Reading.out_of_scope field), crates/athena-core/src/reading.rs (out_of_scope used at ~835-912 and carried by carry/retry), crates/athena-core/src/manage.rs (NewFile writes at ~254, close path). The entry point to hook is where a job's outcome is closed and records are committed (run/ and manage.rs close); the producer must find the exact spot. The records allow-list already accepts athena.request/1, so no new record type or file is added; if one were, check_record_file, AnyRecord and the round-trip test would cover it. The property is measured directly: new Request records appear in the records written at close, not via a stand-in.
- Existing code: crates/athena-core/src/steward.rs (StewardRequest, RequestQueue, REQUEST_CHANNEL) and crates/athena-core/src/reading.rs (out_of_scope handling) are reused and extended; no second request writer is built, so queued follow-ups pass the same admission/plan parse as other steward requests (steward_requests_from_every_writer_pass_the_plan_parse).
- Term ladder: not used. 'Follow-up' means an out_of_scope item of a reading that names later work to do (for example 'propose these as follow-up requests', or fix X 'runs as its own job'), as opposed to a plain exclusion such as 'athena.toml and records/ are not touched'. Assumed rule: an item is a follow-up only when it names work to be done later; the producer decides a conservative, testable classifier (buy-first: no new crate), and plain exclusions are never queued.
- Assumption: follow-ups are queued for jobs that landed or were rejected alike (the fgwz case was landed), once per (job, item): a restart, retry or re-close must not duplicate them. Each request text cites the original job id and the item's words, and passes StewardRequest::new (title, acceptance bullet, Files: line; Files falls back from paths in the item to crates/).
- Assumption: the already-lost follow-ups of job 20261114-034034-fgwz (queue order, October backlog hold, review findings 1 and 3-13, plan critic memory) are not backfilled by code; the owner or steward can queue them with the new mechanism or by hand. Prior job 20261114-034034-fgwz: its landed candidate changed crates/athena-core/src/hestia.rs and CHANGELOG.md. hestia.rs is kept as landed and not touched here; its three verifier findings (recurrence judged by introspection time not base commit; the no-op `later.at = at(600)`; one over-long doc line) are unrelated to this request and are left to their own follow-up requests, not fixed here. CHANGELOG.md is a deliverable of this job, with a new entry.
- Environment: none reached. The change is unit and integration tested with fake agents and temp state; no CI workflow, push or records-branch read is needed. athena.toml, defaults.toml and workflows are not changed (no new config key, no consumers).
- This adds a request-creating mechanism, not a refusal rule: it refuses no reading. Queued follow-ups must not raise queued-request volume without bound, so a cap per job is assumed (the existing steward cap of twenty per run, `a_run_queues_at_most_twenty_items`, is a model).

## Acceptance criteria

1. [behavior] A test run by `cargo test` (athena-core) closes a job whose reading has out_of_scope items naming follow-up work and shows one new request per follow-up item is produced, each with channel REQUEST_CHANNEL, text citing the original job id, and text that passes StewardRequest::new / the plan parse; the test reproduces the fgwz case (items such as 'propose these as follow-up requests' and 'runs as its own job').
2. [behavior] A test run by `cargo test` shows plain exclusions in out_of_scope (for example 'athena.toml, defaults.toml, workflows, docs/decisions.md and records/') queue no request, and a job with an empty out_of_scope queues none and is not reported as an error.
3. [behavior] A test run by `cargo test` shows queuing is idempotent: closing, retrying or resuming the same job twice yields each follow-up request once, and the number queued per job is capped.
4. [behavior] A test run by `cargo test` shows a queued follow-up appears in the unread steward-channel queue (queued_work in crates/athena-core/src/steward.rs) so the drain picks it up, and that a follow-up whose item names no usable path still builds a valid request.
5. [content] The diff to crates/athena-core/src/steward.rs and the close/run path reuses StewardRequest and RequestQueue (Grep shows no second request-text writer) and adds no new record type, config key or crate; athena.toml, defaults.toml, .github/workflows, docs/decisions.md, records/ and crates/athena-core/src/hestia.rs are unchanged in `git diff`.
6. [content] CHANGELOG.md has a new entry saying that out_of_scope follow-ups are now queued as their own requests citing the original job, naming how plain exclusions are told apart.
7. [behavior] fmt, clippy and the whole test run pass: `cargo fmt --check`, `cargo clippy`, `cargo test` (or a passing Athena record on the same final tree); these supplement, not replace, the behavior tests above.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/followups.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Backfilling the follow-ups already lost from job 20261114-034034-fgwz, or fixing its jobs' other items (queue order, October backlog hold, closing 20261017-205552-y1bh and 20261030-060410-9e5h, plan critic memory, review findings 1 and 3-13).
- The three verifier findings on 20261114-034034-fgwz in crates/athena-core/src/hestia.rs (base-commit recurrence rule, no-op `later.at`, long doc line); hestia.rs stays as landed and unchanged.
- New refusal rules, changes to the reading prompts (the reader prompt is at its token limit), athena.toml, defaults.toml, workflows, docs/decisions.md and records/.
- Reprioritizing queued follow-ups ahead of or behind other work.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the follow-up requests actually appear in the queue, not that wiring exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New behavior tests plus fmt, clippy and test passing.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing StewardRequest and RequestQueue instead of building a second writer; no new crate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; backfill and other review findings are excluded.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-041123-hvyh/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-041123-hvyh/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test run by `cargo test` (athena-core) closes a job whose reading has out_of_scope items naming follow-up work and shows one new request per follow-up item is produced, each with channel REQUEST_CHANNEL, text citing the original job id, and text that passes StewardRequest::new / the plan parse; the test reproduces the fgwz case (items such as 'propose these as follow-up requests' and 'runs as its own job').
A test run by `cargo test` shows plain exclusions in out_of_scope (for example 'athena.toml, defaults.toml, workflows, docs/decisions.md and records/') queue no request, and a job with an empty out_of_scope queues none and is not reported as an error.
A test run by `cargo test` shows queuing is idempotent: closing, retrying or resuming the same job twice yields each follow-up request once, and the number queued per job is capped.
A test run by `cargo test` shows a queued follow-up appears in the unread steward-channel queue (queued_work in crates/athena-core/src/steward.rs) so the drain picks it up, and that a follow-up whose item names no usable path still builds a valid request.
The diff to crates/athena-core/src/steward.rs and the close/run path reuses StewardRequest and RequestQueue (Grep shows no second request-text writer) and adds no new record type, config key or crate; athena.toml, defaults.toml, .github/workflows, docs/decisions.md, records/ and crates/athena-core/src/hestia.rs are unchanged in `git diff`.
CHANGELOG.md has a new entry saying that out_of_scope follow-ups are now queued as their own requests citing the original job, naming how plain exclusions are told apart.
fmt, clippy and the whole test run pass: `cargo fmt --check`, `cargo clippy`, `cargo test` (or a passing Athena record on the same final tree); these supplement, not replace, the behavior tests above.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
