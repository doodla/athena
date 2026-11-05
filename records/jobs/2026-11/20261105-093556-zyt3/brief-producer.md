# Job 20261105-093556-zyt3: Require a root-cause diagnosis and a base-failing test in bug-fix

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request and reading described the wanted page content but did not require a diagnosis: why decid
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: symptom-fix-without-diagnosing-root-cause-in-bug-job (20261104-054711-kb90)
> Cause: The request and reading described the wanted page content but did not require a diagnosis: why decided proposals reach the published page, from which data source and state. Without a criterion for the root cause and a test that uses real de...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261104-054700-e6js (job 20261103-201526-t4qv); prevention it proposed: For any job that fixes wrong output, the reading must include a criterion that names the root cause: where the wrong items enter and why they are not excluded. It must also require a test built from real decided (approved and rejected) prop...
> Introspection 20261105-051344-q9fv (job 20261103-231808-3gkq); prevention it proposed: Two changes together. First, for bug-fix jobs, Athena runs the new tests on the base tree on a scratch copy before the producer edits code, and records the result. If a test passes on the base, the job must say the bug is not reproduced and...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Jobs 20261103-201526-t4qv and 20261103-231808-3gkq both moved a filter without diagnosing why decided proposals reached the page, because the reading never required a diagnosis. Fix it at the reading gate: a new check refuses a bug-fix code/tests reading that has no root-cause criterion and no criterion for a test that fails on the base tree, plus reader guidance. Audience: Athena's reader and verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: refusing_check (step! lines in order: check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_design_existing_code, check_governance_coverage, ...) is the entry point and the new check is added as one step! line after check_code_survey; it reads the request, class_of_work, criteria and title only, never the repository. crates/athena-core/tests/refusal_corpus.rs fails when a step! has no corpus case where that check refuses, so a corpus case is required. crates/athena-context/src/briefs.rs holds CODE_SURVEY_GUIDE, the model for the new reader guide. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by calling check_reading_acceptance / refusing_check on real reading shapes (the 20261103-201526-t4qv reading) and asserting the check name that refuses.
- Goal: stop the class 'symptom fix without diagnosis' seen in jobs 20261103-201526-t4qv and 20261103-231808-3gkq (introspections 20261104-054700-e6js and 20261105-051344-q9fv). Fix at the origin: the reading gate that accepted readings with no root-cause criterion and a test that passes on the base. Both jobs were rejected; their candidate edits to xtask/src/decisions.rs, xtask/src/site.rs and CHANGELOG.md are not part of this job and stay dropped, and their findings (decided proposals still shown, test passing on base) are addressed by the gate requirement, not by re-fixing the decisions page.
- A bug-fix request is detected from the request text by whole-word matches such as fix, bug, wrong, broken, regression, fails, 'already', 'still shows'; the check applies only when the reading has a capability, no question, class code or tests, and at least one behavior criterion.
- The check requires two criteria: one containing 'root cause' that says where the wrong output enters and why it is not excluded, and one behavior criterion saying a named test fails on the base tree (without the change) and passes with it, using real recorded data shapes. Running the new tests on a scratch base copy (the first introspection's second proposal) is out of this job; only the reading gate and reader guidance change.
- Expected 8% of newly refused readings; the refusal rate rises intentionally because bug-fix readings now need a root-cause criterion. Replay stays under the 10% hold.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new unit test in crates/athena-core/src/reading.rs builds the reading for a request like 'The page lists proposals that were already decided; fix it' with criteria that only describe the wanted page content, and check_root_cause_diagnosis refuses it, naming the missing root-cause criterion and the missing base-failing test criterion; the same reading with both criteria is accepted. The refusal test fails on the base tree because the function does not exist and the reading is accepted.
2. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs shaped like the 20261103-201526-t4qv reading is refused by check_root_cause_diagnosis, and the corpus coverage test passes with the new step! line in refusing_check.
3. [behavior] Run `cargo test -p athena-core`: a non-bug request (add a feature), a docs reading, a decline and a reading with a question are not refused by the new check.
4. [content] crates/athena-core/src/reading.rs doc comment on the new check names the root cause (reading did not require a diagnosis of where wrong items enter, nor a test that fails on the base), cites jobs 20261103-201526-t4qv and 20261103-231808-3gkq, and states 'expected 8% of newly refused readings'.
5. [content] crates/athena-context/src/briefs.rs gains a reader guide constant, composed beside CODE_SURVEY_GUIDE, telling the reader to add a root-cause criterion and a base-failing test criterion for bug-fix jobs; its brief test asserts the section contains it. The prompt files under the token budget are not edited.
6. [content] CHANGELOG.md has a Fixed entry that names the root cause and the new gate.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- xtask/src/decisions.rs, xtask/src/site.rs and the decisions page fix: candidates of jobs 20261103-201526-t4qv and 20261103-231808-3gkq are dropped, not continued
- Running new tests on a scratch base copy in the pipeline (drain/producer changes)
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows
- Editing records/ or prompt files with no token headroom (reader, verifier prompts)
- Changing approve-work or reject-work behavior or record schemas

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core and crates/athena-context.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and a corpus case; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the reading gate where the class originates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a refusal check, not prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on real reading shapes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the gate refuses the symptom-only reading.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one guide, tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments name cause and jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-093556-zyt3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-093556-zyt3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a new unit test in crates/athena-core/src/reading.rs builds the reading for a request like 'The page lists proposals that were already decided; fix it' with criteria that only describe the wanted page content, and check_root_cause_diagnosis refuses it, naming the missing root-cause criterion and the missing base-failing test criterion; the same reading with both criteria is accepted. The refusal test fails on the base tree because the function does not exist and the reading is accepted.
Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs shaped like the 20261103-201526-t4qv reading is refused by check_root_cause_diagnosis, and the corpus coverage test passes with the new step! line in refusing_check.
Run `cargo test -p athena-core`: a non-bug request (add a feature), a docs reading, a decline and a reading with a question are not refused by the new check.
crates/athena-core/src/reading.rs doc comment on the new check names the root cause (reading did not require a diagnosis of where wrong items enter, nor a test that fails on the base), cites jobs 20261103-201526-t4qv and 20261103-231808-3gkq, and states 'expected 8% of newly refused readings'.
crates/athena-context/src/briefs.rs gains a reader guide constant, composed beside CODE_SURVEY_GUIDE, telling the reader to add a root-cause criterion and a base-failing test criterion for bug-fix jobs; its brief test asserts the section contains it. The prompt files under the token budget are not edited.
CHANGELOG.md has a Fixed entry that names the root cause and the new gate.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
