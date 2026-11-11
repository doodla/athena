# Job 20261111-164002-rgay: Make an empty producer diff carry a recorded reason

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Athena treats an empty producer diff as a bare failure. The producer isn't required to say whether t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: producer-makes-no-change-on-bounded-code-move-no-reason-recorded (20261027-012857-p9qn)
> Cause: Athena treats an empty producer diff as a bare failure. The producer isn't required to say whether the work was already done, blocked, or abandoned. Nothing checks before dispatch that the thing to be moved is still where the request says i...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261027-012842-7whw (job 20261027-011733-vrsk); prevention it proposed: Two code checks. (1) Before dispatch, resolve each named source symbol and file in the current snapshot (here the Transport row in gh/mod.rs and the target gh/transport.rs). If the source is absent or the target already holds it, mark the j...
> Introspection 20261111-163944-gnda (job 20261111-145400-dx3z); prevention it proposed: Require the producer to end every run with a structured outcome: changed, already done (with evidence), blocked (with the blocker), or gave up (with the reason). Have Athena also record policy denials and deadline or budget stops for the ru...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer is never asked to say why it changed nothing, and Athena ends an empty diff with the bare reason "the producer made no change" (candidate.rs), dropping the report it has. Fix by telling the producer in its brief to give a structured no-change outcome (already done with evidence, blocked, or gave up), carrying it in the report, and putting it in the failure reason and records. Retries of 20261027-011733-vrsk and 20261111-145400-dx3z are both of this class.

Assumptions:
- Prior jobs: 20261027-011733-vrsk (gh/transport.rs move; candidate changed no files, outcome 'the producer made no change') and 20261111-145400-dx3z (role-need retire rule; candidate changed no files). Neither candidate changed any file, so there is no work to keep. Neither job's deliverables (gh/transport.rs, gh/mod.rs, role_need.rs, store.rs, status.rs, display.rs, docs/system-facts.md, docs/spec.toml, xtask/shard-timings.txt) are redone here; they are out_of_scope. Each job's finding, 'the producer made no change', is addressed by making that outcome carry a reason: for dx3z the reason would be 'already done', with the evidence the producer reported.
- Goal and audience: the user and later introspections who read a failed job's reason and need to know whether work was already done, blocked or abandoned. They can see this in the failure reason and the kept report.
- Code survey: crates/athena-core/src/run/candidate.rs (candidate_stage: an empty change list ends in self.failure with the bare text; report_gaps only covers report problems); crates/athena-core/src/run/producer.rs (report check and send-back loop, ends with Flow::Go(Reported)); crates/athena-core/src/run/mod.rs (Reported to candidate_stage; ProducerReport built near line 2529); crates/athena-context/src/outputs.rs (ProducerOutput schema, serde(default) for optional fields); crates/athena-context/src/briefs.rs (producer_brief); crates/athena-data/src/records.rs (ProducerReport and Candidate, deny_unknown_fields, so any new field needs serde(default) and the AnyRecord round-trip tests must still pass; an existing record without the field must still parse). The property is measured directly: the failure reason string of a job whose fake producer changes nothing and reports a no-change outcome.
- Design: add an optional no-change outcome to the producer's structured output and report (kind: already done / blocked / gave up, plus detail or evidence). The producer brief asks for it whenever the diff is empty. candidate_stage includes it in the failure reason. This adds no refusal rule, so there is no refused-readings share.
- Existing code: extend crates/athena-context/src/outputs.rs ProducerOutput, crates/athena-data/src/records.rs ProducerReport, crates/athena-core/src/run/candidate.rs and crates/athena-core/src/run/producer.rs, and the existing retry test in crates/athena-cli/tests/retry.rs (which asserts 'the producer made no change').
- The pre-dispatch check of the named source symbols proposed by introspection 7whw is a refusal-style rule, so it is not built. The brief change is the information fix; the no-change outcome tells the next reader whether the source was already moved.
- Prompt headroom: the producer prompt has 157 tokens left, so the brief addition must be short; prompts stay unchanged beyond the Rust-built brief text.

## Acceptance criteria

1. [behavior] cargo test passes with a new test in crates/athena-core/src/run/tests.rs or crates/athena-cli/tests/retry.rs: a fake producer makes no change and reports 'already done' with evidence; the job's failure reason contains the kind and the evidence, not only 'the producer made no change'. The test fails on the base commit.
2. [behavior] Three more test cases for blocked, gave up, and no outcome given with an empty diff (the reason says no reason was reported, naming the missing field) pass, so the fix covers the class and not only the two jobs.
3. [content] git diff shows crates/athena-context/src/briefs.rs (producer brief) telling the producer to give the no-change outcome (already done with evidence, blocked, or gave up) when it leaves the tree unchanged; a briefs.rs unit test asserts this text is in producer_brief.
4. [content] git diff shows the new field in ProducerOutput (outputs.rs) and ProducerReport (records.rs) with serde(default), and cargo test shows existing records without the field still parse (the AnyRecord round-trip tests pass).
5. [content] The change adds no new rule that refuses or holds work: git diff shows no new admission, gate or check that fails a job, so no refused-readings share applies; an empty diff still fails the job but with the reason recorded.
6. [content] CHANGELOG.md has an entry naming the root cause: the producer is not asked for a no-change outcome and Athena discards it, so an empty diff was a bare failure.
7. [behavior] cargo fmt --check, cargo clippy and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/run/candidate.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Moving Transport items into gh/transport.rs (job 20261027-011733-vrsk work); no files kept because its candidate changed none
- The role-need retire rule fix (job 20261111-145400-dx3z work); no files kept because its candidate changed none
- A pre-dispatch check that refuses a job when the named source symbols are absent
- Changes to athena.toml, defaults.toml or prompt files
- Automatically retrying or reclassifying jobs whose producer reported 'already done'

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the failure reason a user reads, run through a test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the producer's brief and output schema, not in the two jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The first criterion runs a no-change job and checks its reason.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261111-164002-rgay/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-164002-rgay/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

cargo test passes with a new test in crates/athena-core/src/run/tests.rs or crates/athena-cli/tests/retry.rs: a fake producer makes no change and reports 'already done' with evidence; the job's failure reason contains the kind and the evidence, not only 'the producer made no change'. The test fails on the base commit.
Three more test cases for blocked, gave up, and no outcome given with an empty diff (the reason says no reason was reported, naming the missing field) pass, so the fix covers the class and not only the two jobs.
git diff shows crates/athena-context/src/briefs.rs (producer brief) telling the producer to give the no-change outcome (already done with evidence, blocked, or gave up) when it leaves the tree unchanged; a briefs.rs unit test asserts this text is in producer_brief.
git diff shows the new field in ProducerOutput (outputs.rs) and ProducerReport (records.rs) with serde(default), and cargo test shows existing records without the field still parse (the AnyRecord round-trip tests pass).
The change adds no new rule that refuses or holds work: git diff shows no new admission, gate or check that fails a job, so no refused-readings share applies; an empty diff still fails the job but with the reason recorded.
CHANGELOG.md has an entry naming the root cause: the producer is not asked for a no-change outcome and Athena discards it, so an empty diff was a bare failure.
cargo fmt --check, cargo clippy and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
