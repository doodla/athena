# Job 20261111-222407-mz22: Brief producers to find and run tests that read the output they change

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing made the producer find the existing tests that read the output it was changing before it fin
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: new-status-output-line-breaks-existing-test-caught-only-at-final-verdict (20261111-111854-e6vm)
> Cause: Nothing made the producer find the existing tests that read the output it was changing before it finished. The new section was printed unconditionally, and no step ran the full athena-cli test suite, or the tests that read status or plan ou...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261111-111841-aan3 (job 20261111-105048-n7bm); prevention it proposed: Before the producer reports done, require it to run the full athena-cli test suite whenever it changes shared output code such as display.rs. The check should record the failing test name and assertion in the kept evidence. For any change t...
> Introspection 20261111-222347-g374 (job 20261111-213010-8jgz); prevention it proposed: Before finishing, the producer should be required to run the tests that read the output or behavior it changes, and then the full test suite, in the same shard layout CI uses. It should cite the commands it ran as evidence for the behavior ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer brief never says to find existing tests that read the output being changed (status, plan, display text) and run them before reporting, so breakage surfaced only at the final verdict (jobs 20261111-105048-n7bm, 20261111-213010-8jgz). Fix by adding a section to the producer brief in crates/athena-context/src/briefs.rs, with a test that fails without it. Retried jobs' candidates are dropped, not kept.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (producer_brief_with_evidence, checkable_evidence_section, replay_while_working_section, criteria_check_section); the change is a new brief section like those, no record files added so check_record_file/AnyRecord are unaffected.
- Goal: producers, and the verifiers judging them, get early notice of broken existing tests; the fix is information in the brief, not a refusal rule.
- The brief cannot know changed files at dispatch time, so the section instructs the producer to search for tests itself.
- Only the producer role is targeted; no prompt files, athena.toml or defaults.toml change.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs.rs adds a producer-brief section, emitted by producer_brief_with_evidence, that tells the producer: before reporting, Grep tests/ and #[cfg(test)] modules for text of any output it changes (status, plan, display), run those tests with cargo test -p <crate> --test <name>, then the full crate's tests, and cite the commands and any failing test name in its evidence.
2. [behavior] A new unit test in briefs.rs asserts the producer brief contains this section for any reading; it fails on the base (run `cargo test -p athena-context briefs`) and passes with the change.
3. [content] The section is given to the producer as information in the brief; no new refusal or gate rule is added, so the expected share of newly refused past readings is 0%.
4. [content] Jobs 20261111-105048-n7bm and 20261111-213010-8jgz: their candidates (role_need.rs, status.rs, display.rs, introspection.rs, docs/spec.toml, status_build_measurement.rs) are dropped, not kept; their findings are about those features, not this cause. The drain flake in job 8jgz is out of scope.
5. [behavior] cargo fmt --check, cargo clippy and cargo test --locked pass.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- role_need.rs retire-candidate feature and later_similar_task feature from the earlier jobs
- Fixing the drain parallel test flake
- Adding a refusal or gate rule on producer reports
- Changes to athena.toml, defaults.toml, CI workflows or prompt files
- CI shard layout changes

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion tests the brief content by running the unit test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
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

- path: <state>/jobs/20261111-222407-mz22/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-222407-mz22/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs.rs adds a producer-brief section, emitted by producer_brief_with_evidence, that tells the producer: before reporting, Grep tests/ and #[cfg(test)] modules for text of any output it changes (status, plan, display), run those tests with cargo test -p <crate> --test <name>, then the full crate's tests, and cite the commands and any failing test name in its evidence.
A new unit test in briefs.rs asserts the producer brief contains this section for any reading; it fails on the base (run `cargo test -p athena-context briefs`) and passes with the change.
The section is given to the producer as information in the brief; no new refusal or gate rule is added, so the expected share of newly refused past readings is 0%.
Jobs 20261111-105048-n7bm and 20261111-213010-8jgz: their candidates (role_need.rs, status.rs, display.rs, introspection.rs, docs/spec.toml, status_build_measurement.rs) are dropped, not kept; their findings are about those features, not this cause. The drain flake in job 8jgz is out of scope.
cargo fmt --check, cargo clippy and cargo test --locked pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
