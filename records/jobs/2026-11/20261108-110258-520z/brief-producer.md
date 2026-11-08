# Job 20261108-110258-520z: Refuse plans that leave caller and test impact to the producer

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planning stage is not required to find every existing caller and test that runs the code path it
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-leaves-existing-test-impact-open-fails-planning-rounds (20261025-070501-16qb)
> Cause: The planning stage is not required to find every existing caller and test that runs the code path it changes, or to settle each open question before it hands over the plan. The risk list and open questions can say "producer to check", and t...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261025-070448-xbdw (job 20261025-055626-er96); prevention it proposed: Change the plan template and the planning checks. (1) For every production path the plan changes, the planner must list the existing callers and tests that run it (a search of the tests for the entry point and for output assertions) and say...
> Introspection 20261108-110243-93rh (job 20261026-193430-avnq); prevention it proposed: Before dispatching a move or split job, run a check that searches the test sources for reads of the file or symbols being moved (for example include_str! or fs reads of display.rs) and lists them in the reading as files the job must update....
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: plan admission (`plan_findings` in admission.rs) checks open questions but not risks, so a plan can pass with a risk whose mitigation is "producer to check" and no list of existing callers/tests that run the changed path (seen in jobs 20261025-055626-er96 and 20261026-193430-avnq). Fix at that gate and in the planner brief: a risk that defers to the producer is a major finding, and the planner must list the callers and tests of each changed path in `dependents`.

Assumptions:
- Goal and audience: the planner and the plan gate, so the planning rounds stop ending 'plan still needed revising' because caller and test impact was left for the producer; the user and verifier judge by the new refusal and its test.
- Code survey: read crates/athena-core/src/admission.rs (plan_findings, open_questions_findings, dependents_findings, dependents_unnamed; entry point plan_findings, called from crates/athena-core/src/run/planning.rs), crates/athena-data/src/records/plan.rs (PlanRisk has risk and mitigation strings; JobPlan.risks), crates/athena-core/tests/refusal_corpus.rs (open_question_cases and plan_findings tests) and crates/athena-context/prompts/planner.md. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by calling plan_findings on a JobPlan whose risk mitigation defers to the producer.
- Cause as it originates: open_questions_findings already forces open questions to be User:/Host: only, but a risk's mitigation is unchecked, and its doc comment says the caller listing 'stays prose in the planner brief'. Job 20261025-055626-er96 failed in plan round 3 for exactly this (risk 'Existing go tests would call real gh' left to the producer; CLI lifecycle tests asserting no 'warning' were missed). Job 20261026-193430-avnq failed because a move broke athena-core's include_str! read of display.rs, which no one searched for.
- Previous result, job 20261025-055626-er96: its work (early push in gh.rs, run.rs, land.rs, pr.rs, test_support.rs) was never produced (planning stopped, no candidate). Its findings do not apply to this change; they are the evidence for the cause. The early-push feature itself is dropped here and stays a separate job.
- Previous result, job 20261026-193430-avnq: its candidate (display.rs split into display/status_view.rs, CHANGELOG entry) was rejected because status.rs's include_str! test read display.rs. That split is dropped here: display.rs and status_view.rs are not touched. Its CHANGELOG entry is not kept; this job adds its own entry.
- The new rule: a risk whose mitigation (or risk text) defers to the producer, with phrases such as 'producer to check', 'producer should check', 'producer will check', 'producer to verify', 'left to the producer', is a major finding at location 'risks' naming it, sent back to the planner without a critic call. A mitigation that states a found answer with file and line passes.
- The planner brief gains one short line (planner has 175 tokens of headroom; stay within it): for every production path the plan changes, search the tests and other crates for callers, include_str!/fs reads of moved files, and output assertions, list them in dependents, and never defer them to the producer. No other prompt file changes.
- Refusal rule: expected 5% of newly refused plans. The higher refusal rate is intended, because those plans spent critic rounds on impact the planner could find by searching.

## Acceptance criteria

1. [content] crates/athena-core/src/admission.rs has a risks rule called from plan_findings (next to open_questions_findings) whose doc comment names the root cause: planning was not required to settle caller and test impact and let risks defer it to the producer; the rule is a major finding at location 'risks'.
2. [behavior] A unit test in crates/athena-core/src/admission.rs builds a JobPlan with a risk whose mitigation is 'producer to check existing go tests' and asserts plan_findings returns a major Athena finding at 'risks' naming it; the same test fails on the base commit and passes with the change. Run: cargo test -p athena-core --lib admission.
3. [behavior] The test also shows a risk whose mitigation states a settled answer with file and line (for example 'lifecycle.rs:2627 asserts no warning; GhEarly skips quietly') and an empty risks list add no finding, so the rule is not blanket. Run: cargo test -p athena-core --lib admission.
4. [behavior] crates/athena-core/tests/refusal_corpus.rs maps each input form of the new rule (each deferring phrase, the settled answer, mixed case, deferral in the risk text rather than the mitigation) to a case with its expected outcome, and passes. Run: cargo test -p athena-core --test refusal_corpus.
5. [content] crates/athena-context/prompts/planner.md requires the planner to list, for each changed production path, the existing callers and tests that run it (including include_str!/fs reads of moved files and output assertions) in dependents and never leave them to the producer; Grep shows the line.
6. [content] The change expects 5% of newly refused plans; the higher refusal rate is intended because those plans cost critic rounds on impact the planner can find by searching.
7. [content] CHANGELOG.md has an entry naming the root cause and the new risks rule, and the display.rs/status_view.rs split of job 20261026-193430-avnq is not present in the diff.
8. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass, including the prompt token budget tests for the planner set.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- The early-push-to-PR-branch feature of job 20261025-055626-er96 (gh.rs, run.rs, land.rs, pr.rs, test_support.rs); changing ci.yml or the xtask fingerprint; merging the PR before verification passes; changing how land merges or waits for checks
- The display.rs split of job 20261026-193430-avnq: crates/athena-cli/src/display.rs and crates/athena-cli/src/display/status_view.rs are not changed or created; behavior or output format changes; splitting other files or raising limits; refactoring the status function; fixing status.rs's include_str! test
- Changing the critic prompt, the reader prompt, athena.toml or defaults.toml
- A mechanical repository-wide search for callers (the search stays the planner's; Athena checks only that risks do not defer it)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The rule and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A failing-without test plus refusal corpus cases; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in plan_findings, not left as prose only.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at plan admission and the planner brief where the cause originates, covering both jobs' failures.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings on a deferring plan and check the finding.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule, one brief line, tests and a changelog entry.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that plans deferring impact are refused before the critic, checked by the test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment names the cause for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

- path: <state>/jobs/20261108-110258-520z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-110258-520z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/admission.rs has a risks rule called from plan_findings (next to open_questions_findings) whose doc comment names the root cause: planning was not required to settle caller and test impact and let risks defer it to the producer; the rule is a major finding at location 'risks'.
A unit test in crates/athena-core/src/admission.rs builds a JobPlan with a risk whose mitigation is 'producer to check existing go tests' and asserts plan_findings returns a major Athena finding at 'risks' naming it; the same test fails on the base commit and passes with the change. Run: cargo test -p athena-core --lib admission.
The test also shows a risk whose mitigation states a settled answer with file and line (for example 'lifecycle.rs:2627 asserts no warning; GhEarly skips quietly') and an empty risks list add no finding, so the rule is not blanket. Run: cargo test -p athena-core --lib admission.
crates/athena-core/tests/refusal_corpus.rs maps each input form of the new rule (each deferring phrase, the settled answer, mixed case, deferral in the risk text rather than the mitigation) to a case with its expected outcome, and passes. Run: cargo test -p athena-core --test refusal_corpus.
crates/athena-context/prompts/planner.md requires the planner to list, for each changed production path, the existing callers and tests that run it (including include_str!/fs reads of moved files and output assertions) in dependents and never leave them to the producer; Grep shows the line.
The change expects 5% of newly refused plans; the higher refusal rate is intended because those plans cost critic rounds on impact the planner can find by searching.
CHANGELOG.md has an entry naming the root cause and the new risks rule, and the display.rs/status_view.rs split of job 20261026-193430-avnq is not present in the diff.
cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass, including the prompt token budget tests for the planner set.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
