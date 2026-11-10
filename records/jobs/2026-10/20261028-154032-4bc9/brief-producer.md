# Job 20261028-154032-4bc9: Ask root-cause requests for missing stage information before a refusal

## Request (verbatim)

> Priority: high
> The error review's root-cause requests default to adding a new rule that refuses work (for example 'add a reading-stage check'), which is how the word-matching rule of 2026-10-27 came about, and on 2026-10-28 such a request (20261028-082300-y4yj) was stopped twice in planning because its new rule would refuse past accepted readings. Most of these causes are a stage missing information, not a stage needing to be refused. Change the shared steward request builder's root-cause template in hestia.rs: (1) ask first for the information the failing stage lacked to be given to it (in its brief, the system facts or the reader's inputs); (2) allow a new refusal rule only when giving the information cannot prevent the failure, and then require the request to state the expected share of past accepted readings it would refuse, so the replay can hold it against that number.
> Acceptance:
> - A test shows a root-cause request built from a fixture introspection asks first for the missing information to be supplied to the stage, and mentions a refusal rule only with an expected refusal share.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/hestia.rs, CHANGELOG.md

## Reading (accepted by the user)

Change the body built by root_cause_request_at in hestia.rs so a root-cause request first asks that the information the failing stage lacked be given to it (its brief, the system facts or the reader's inputs), and allows a new refusal rule only when that cannot prevent the failure, stating the expected share of past accepted readings it would refuse. Adds a fixture test and a CHANGELOG.md entry. Job 20261028-082300-y4yj (the motivating request, stopped in planning) is not retried: its reading/briefs changes are dropped.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (root_cause_request, root_cause_request_at at ~464-521, which builds the body and acceptance with StewardRequest and checks the text with plain_boundary_text; existing tests near lines 3792 and 4020 build requests from fixture introspections via root_cause_request). The change edits text only, adds no record or data file, so the records allow-list (check_record_file/AnyRecord) is not touched. The property is measured directly by calling root_cause_request on a fixture cause and asserting on the returned text.
- Goal and audience: the Hestia steward's root-cause requests are read by the reader/planner/producer of the follow-up job; they should steer toward supplying missing information rather than adding refusal rules.
- Job 20261028-082300-y4yj failed in planning because its reading had no intended-refusal criterion and a new refusal rule would refuse past readings. Its finding does not apply here: this change adds no refusal rule in reading.rs, so no replay refusal is intended and no intended-refusal criterion is needed. Its candidate changed no files, so nothing is kept; the reading.rs, briefs.rs and run/mod.rs work it scoped is dropped and left to a follow-up request built with the new template.
- The wording must pass plain_boundary_text (admission word scan) and the builder's no-bullet-line rule; the producer rewords if the scan rejects a word.
- The template wording is mine to choose: information step first, then a refusal rule only if information cannot prevent the failure, requiring the line 'Expected share of past accepted readings it would refuse: N%'.
- athena.toml, defaults.toml and workflows need no change; existing tests asserting the request text are updated only where the old body text is pinned.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/hestia.rs builds a root-cause request with root_cause_request from a fixture introspection/cause and asserts the text asks first (earlier position in the text) for the missing information to be supplied to the failing stage (brief, system facts or reader's inputs). Check: cargo test -p athena-core hestia passes and the test is listed.
2. [behavior] The same test asserts the text mentions adding a refusal rule only as allowed when giving the information cannot prevent the failure, and requires the request to state the expected share of past accepted readings it would refuse; the asserted refusal-rule mention comes after the information ask. Check: cargo test -p athena-core hestia.
3. [content] The test fails against the old body text: the diff of crates/athena-core/src/hestia.rs shows the old body (find the cause where it originates...) extended in root_cause_request_at with the new paragraphs and no other change to the acceptance list semantics.
4. [content] Existing root-cause tests (e.g. the ones near the_root_cause_request_is_high_and_refuses_a_cause_that_reads_as_acceptance) still pass, and the built request still passes plain_boundary_text and the no-bullet-line rule. Check: cargo test -p athena-core.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has a new entry describing the root-cause template change (information first, refusal rule only with an expected refusal share). Check: git diff CHANGELOG.md.
7. [content] The file list of the change contains only crates/athena-core/src/hestia.rs and CHANGELOG.md. Check: git status.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding or changing any refusal rule in reading.rs, check_reading_acceptance or the replay (job 20261028-082300-y4yj's scope, dropped)
- Changes to crates/athena-context/src/briefs.rs and crates/athena-core/src/run/mod.rs
- Changing athena.toml, defaults.toml or workflows
- Changing planner, critic or reader prompts
- Changing gate-review or other steward request templates

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require a new test plus cargo test and clippy -D warnings.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the built request text, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only hestia.rs and CHANGELOG.md.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261028-154032-4bc9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-154032-4bc9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/hestia.rs builds a root-cause request with root_cause_request from a fixture introspection/cause and asserts the text asks first (earlier position in the text) for the missing information to be supplied to the failing stage (brief, system facts or reader's inputs). Check: cargo test -p athena-core hestia passes and the test is listed.
The same test asserts the text mentions adding a refusal rule only as allowed when giving the information cannot prevent the failure, and requires the request to state the expected share of past accepted readings it would refuse; the asserted refusal-rule mention comes after the information ask. Check: cargo test -p athena-core hestia.
The test fails against the old body text: the diff of crates/athena-core/src/hestia.rs shows the old body (find the cause where it originates...) extended in root_cause_request_at with the new paragraphs and no other change to the acceptance list semantics.
Existing root-cause tests (e.g. the ones near the_root_cause_request_is_high_and_refuses_a_cause_that_reads_as_acceptance) still pass, and the built request still passes plain_boundary_text and the no-bullet-line rule. Check: cargo test -p athena-core.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has a new entry describing the root-cause template change (information first, refusal rule only with an expected refusal share). Check: git diff CHANGELOG.md.
The file list of the change contains only crates/athena-core/src/hestia.rs and CHANGELOG.md. Check: git status.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
