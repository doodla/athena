# Job 20261114-052244-qf6e: Queue each reading's out_of_scope follow-ups as their own requests

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

- path: <state>/jobs/20261114-052244-qf6e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-052244-qf6e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

`cargo test` (athena-core) includes a test that closes a job whose reading has out_of_scope items like 'propose these as follow-up requests' and 'runs as its own job' (the fgwz case). It shows one new request per follow-up item, each with channel REQUEST_CHANNEL, text citing the original job id, and text accepted by StewardRequest::new and the plan parse.
`cargo test` shows plain exclusions (for example 'athena.toml, defaults.toml, workflows, docs/decisions.md and records/') queue no request, and an empty out_of_scope queues none and is not an error.
`cargo test` shows queuing is idempotent: closing, retrying or resuming the same job twice yields each follow-up request once, and the number queued per job is capped.
`cargo test` shows a queued follow-up appears in queued_work (crates/athena-core/src/steward.rs), so the drain picks it up, and that an item naming no usable path still builds a valid request.
`git diff` shows StewardRequest and RequestQueue reused (Grep finds no second request-text writer) and no new record type, config key or crate; athena.toml, defaults.toml, .github/workflows, docs/decisions.md, records/ and crates/athena-core/src/hestia.rs are unchanged.
CHANGELOG.md has a new entry saying out_of_scope follow-ups are queued as their own requests citing the original job, and how plain exclusions are told apart.
`cargo fmt --check`, `cargo clippy` and `cargo test` each pass, or Athena recorded them passing on the same final tree; these supplement the behavior tests above.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
