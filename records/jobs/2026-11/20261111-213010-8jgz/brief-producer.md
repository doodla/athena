# Job 20261111-213010-8jgz: Match later_similar_task on class of work, not any later landed job

## Request (verbatim)

> try again on 20261029-233102-c04w

## Reading (accepted by the user)

Retry of job 20261029-233102-c04w (OLY-10-03 later-similar-task mechanism): redo the status check so "prevented" needs a later landed job of the same class of work, using the stored class_of_work, and prove it over real records. Fixes the verifier's major finding (any later job counted) and both minors.

Assumptions:
- Prior job 20261029-233102-c04w: its candidate files are kept as deliverables (spec.toml, introspection.rs, status.rs, display.rs, status_build_measurement.rs, CHANGELOG.md) and redone from base; the producer reworks later_similar_task. Findings addressed: (major) any later landed job counted, fixed by matching class_of_work; (minor) single later job and early return, fixed by requiring a minimum count of later same-class jobs and checking every accepted proposal; (minor) failed-ranks-before-rejected guess, fixed by ranking only on stages the records state and treating infrastructure failures and other categories as not shown/equal rank, with the rationale in the doc comment.
- Goal and audience: an owner running `athena status` must see a truthful OLY-10-03 line; a verifier judges it against real records. Unrelated later jobs must never produce 'shown'.
- Code survey: read crates/athena-data/src/records.rs (Job.class_of_work: Option<String> at line 607; AnyRecord at line 1089; check_record_file at line 1415), crates/athena-core/src/status.rs (landed_job_ids at line 382, which feeds the status measures) and the prior introspection.rs function. This job adds no record or data file and does not change a record schema, so check_record_file and AnyRecord need no change and records.rs is not a deliverable; existing record round-trip tests keep covering the schemas. Class comes from JobRecords.job in crates/athena-core/src/store.rs; the proposal's applies_to lists the classes. The measure is computed directly from those records.
- Mechanism owner stays Apollo; the measure ends with `status: not shown` or a label athena status prints.
- Defaults applied: outcome-check and no-empty-success. A job with no class, no records or no later job gives not shown, never shown.
- Environment: none; records are read through the existing store accessors, no push, CI or history read.

## Acceptance criteria

1. [behavior] Test in crates/athena-data/src/records/introspection.rs: an accepted proposal plus enough later landed jobs of the same class_of_work with no recurrence gives prevented. Run `cargo test -p athena-data introspection`.
2. [behavior] Test: a later landed job of a different class, or with no class, gives not shown; no later job gives not shown; a pending, rejected or pre-acceptance case gives not shown. Same run; the tests fail if the check ignores class.
3. [behavior] Test: a same-class recurrence at a stage the records show to be earlier gives detected earlier; same or later stage gives not shown; a recurrence on any accepted proposal blocks prevented. Same run.
4. [behavior] A test in crates/athena-core/src/status.rs runs the measure over fixture records holding both a same-class and a different-class later job and requires different answers, and the crates/athena-cli test asserts the athena status line equals the StatusView field. Run `cargo test -p athena-core status` and `cargo test -p athena-cli status`.
5. [content] The doc comment on later_similar_task in crates/athena-data/src/records/introspection.rs no longer says records carry no class for a landed job, and states the class match, the minimum count and the stage ranking.
6. [content] CHANGELOG.md has an entry that records the measure's output over the real records with one example of each outcome (prevented, not prevented by different class, no later job); Grep shows it.
7. [content] docs/spec.toml OLY-10-03 names the existing function path and the status line function as mechanism, keeps owner Apollo, and the measure ends in `status: not shown` or a printed athena status label; Grep shows it.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing OLY-10-01, OLY-10-02 or other requirements
- Changing the proposal, evaluation, introspection or job record schemas; crates/athena-data/src/records.rs is not changed
- Changing athena.toml, defaults.toml or CI workflows
- Writing real records or running a live demonstration job
- The generic standing code-check for all status measures proposed by the earlier introspection (separate job)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the line is true only for same-class later jobs, over fixtures and recorded real-record output.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: No records, no later job or no class gives not shown.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-233102-c04w raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-data/src/records/introspection.rs later_similar_task (the `landed.iter().any(|j| j > accepted)` branch and its doc comment)): 'Prevented' counts any later landed job as a 'similar task', although the records do carry each job's class of work. The doc comment says "Records carry no class for a landed job, so any later landed job stands for a later task." That is false. The Job record has `class_of_work: Option<String>` (crates/athena-data/src/records.rs line 607), and JobRecords holds `job: Option<Job>` (crates/athena-core/src/store.rs line 28). In the evidence snapshot, 1896 files under records/jobs carry a non-null class_of_work. ProposalRecord also has `applies_to` (classes of work), and the accepted reading named `class_of_work in applies_to` as the matching key. As written, once any proposal is accepted, the next unrelated job that lands (say a docs typo fix) makes `athena status` print `later similar task: shown (prevented)`. That is a false success on the one line meant to show OLY-10-03, against the outcome-check and no-empty-success defaults. A more correct approach: pass each landed job's id together with its class_of_work, and count only later landed jobs whose class is in the proposal's applies_to (or that match the failure class). If a job has no class, it should not count.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 18 cases in 18 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to fail on the known bad input, here the current duplicated te...
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

- path: <state>/jobs/20261111-213010-8jgz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-213010-8jgz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Test in crates/athena-data/src/records/introspection.rs: an accepted proposal plus enough later landed jobs of the same class_of_work with no recurrence gives prevented. Run `cargo test -p athena-data introspection`.
Test: a later landed job of a different class, or with no class, gives not shown; no later job gives not shown; a pending, rejected or pre-acceptance case gives not shown. Same run; the tests fail if the check ignores class.
Test: a same-class recurrence at a stage the records show to be earlier gives detected earlier; same or later stage gives not shown; a recurrence on any accepted proposal blocks prevented. Same run.
A test in crates/athena-core/src/status.rs runs the measure over fixture records holding both a same-class and a different-class later job and requires different answers, and the crates/athena-cli test asserts the athena status line equals the StatusView field. Run `cargo test -p athena-core status` and `cargo test -p athena-cli status`.
The doc comment on later_similar_task in crates/athena-data/src/records/introspection.rs no longer says records carry no class for a landed job, and states the class match, the minimum count and the stage ranking.
CHANGELOG.md has an entry that records the measure's output over the real records with one example of each outcome (prevented, not prevented by different class, no later job); Grep shows it.
docs/spec.toml OLY-10-03 names the existing function path and the status line function as mechanism, keeps owner Apollo, and the measure ends in `status: not shown` or a printed athena status label; Grep shows it.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
