# Job 20261105-051400-dzr4: Refuse bug-fix readings that lack a root-cause criterion and real-data

## Request (verbatim)

> Priority: high
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

Retry of jobs 20261103-201526-t4qv and 20261103-231808-3gkq, with the repeated cause fixed at the reading gate. Add a new check, `check_root_cause_criterion`, to `refusing_check` in athena-core. It refuses a code or tests reading for a wrong-output bug fix unless one criterion names the root cause (where wrong items enter and why they are not excluded) and one behavior criterion runs a test built from real recorded data that fails on the base. The decisions-page fix is not redone.

Assumptions:
- Goal and audience: Athena's reading gate and its maintainer. The class 'symptom-fix-without-diagnosing-root-cause-in-bug-job' (introspections 20261104-054700-e6js and 20261105-051344-q9fv) arose because a bug-fix reading described the wanted output but did not require a diagnosis, so producers moved a filter that already existed and wrote tests that pass on the base. The gate is where readings are accepted, so the fix is a mechanism there, not prose.
- Code survey: crates/athena-core/src/reading.rs. The entry point is `check_reading_acceptance`, which calls `refusing_check`; that runs `step!` lines in order: check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_design_existing_code, check_governance_coverage, and others. The new check is added as one `step!(check_root_cause_criterion(reading, request))` line, after check_code_survey. `check_code_survey` is the pattern: it reads only the reading and returns Ok for a decline or question. Tests in crates/athena-core/tests/refusal_corpus.rs run real request shapes through `refusing_check` and fail when a `step!` has no corpus case where it refuses, so one refuse case and one accept case are added there. No record or data file is added, so `check_record_file` and `AnyRecord` are not touched. The property is measured directly by running the reading through `refusing_check`, not through a stand-in.
- The check applies when the reading has a capability and no question, the class is code or tests, and the request text is a wrong-output bug report (whole-word cues such as fix, wrong, shows, lists, incorrect, stale, broken, bug, regression, failure). Feature requests and docs readings are exempt. When it applies, it needs one criterion containing 'root cause' and one behavior criterion that names a test and real recorded or decided data. The error says what to write.
- The base-tree run (the second proposal in 20261105-051344-q9fv) is not built here; the reading-level requirement that a behavior criterion say the test fails on the base is the part in scope. Running tests on a scratch copy of the base is a pipeline change and is left for a separate job.
- Previous jobs: 20261103-201526-t4qv and 20261103-231808-3gkq both ended rejected. Their findings concern xtask/src/decisions.rs, xtask/src/site.rs and CHANGELOG.md for the decisions page. The task now is the shared cause, so those candidates are not kept: nothing from either candidate (CHANGELOG.md, xtask/src/decisions.rs, xtask/src/site.rs) is kept. The findings no longer apply to this change because it does not touch the decisions page. The repeated findings (no root cause named, a test that also passes on the base, a guessed cause in the changelog) are what the new check refuses at reading time.
- Expected 12% of newly refused readings: the check applies only to bug-fix readings in code or tests, which are a minority of past readings. This is intended because past readings of that kind omitted a root-cause criterion, and the threshold stays below 25%.
- CHANGELOG.md gets one entry that names the root cause: the gate did not require a diagnosis or a base-failing real-data test for bug-fix jobs.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs defines `check_root_cause_criterion` and `refusing_check` has a `step!(check_root_cause_criterion(..))` line; its doc comment names the root cause (bug-fix readings were accepted with no diagnosis and no base-failing real-data test) and cites jobs 20261103-201526-t4qv and 20261103-231808-3gkq.
2. [behavior] Run `cargo test -p athena-core`: a corpus case rebuilt from the request of job 20261103-201526-t4qv, with a reading that has no root-cause criterion, is refused by check_root_cause_criterion. The same reading with a root-cause criterion and a behavior criterion that runs a test on real approved and rejected decisions is accepted. The refuse case fails on the base tree, which lacks the check, and passes with it.
3. [behavior] Run `cargo test -p athena-core`: a feature request, a docs reading, a decline and a reading with a question are not refused by the new check, so the check does not touch non-bug work.
4. [behavior] The refusal message from `check_root_cause_criterion` names what is missing: a criterion that names the root cause (where wrong items enter and why they are not excluded) and a criterion for a test built from real decided data that fails on the base. A unit test asserts both phrases appear in the error.
5. [content] CHANGELOG.md has one entry for the new check that names the root cause.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass, including the corpus test that requires every `step!` to have a refusing case.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- xtask/src/decisions.rs, xtask/src/site.rs and the decisions page itself: the earlier candidates are dropped
- Running new tests on a scratch copy of the base tree in the pipeline (the other proposal of 20261105-051344-q9fv)
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows
- Editing records/ files
- Growing the reader prompt, which has 3 tokens of headroom

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the reading gate, where the class originates, not at one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A new check in refusing_check enforces the rule.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Corpus and unit tests, plus fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on refuse and accept cases.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, its tests and a changelog line.
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
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

- path: <state>/jobs/20261105-051400-dzr4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-051400-dzr4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading.rs defines `check_root_cause_criterion` and `refusing_check` has a `step!(check_root_cause_criterion(..))` line; its doc comment names the root cause (bug-fix readings were accepted with no diagnosis and no base-failing real-data test) and cites jobs 20261103-201526-t4qv and 20261103-231808-3gkq.
Run `cargo test -p athena-core`: a corpus case rebuilt from the request of job 20261103-201526-t4qv, with a reading that has no root-cause criterion, is refused by check_root_cause_criterion. The same reading with a root-cause criterion and a behavior criterion that runs a test on real approved and rejected decisions is accepted. The refuse case fails on the base tree, which lacks the check, and passes with it.
Run `cargo test -p athena-core`: a feature request, a docs reading, a decline and a reading with a question are not refused by the new check, so the check does not touch non-bug work.
The refusal message from `check_root_cause_criterion` names what is missing: a criterion that names the root cause (where wrong items enter and why they are not excluded) and a criterion for a test built from real decided data that fails on the base. A unit test asserts both phrases appear in the error.
CHANGELOG.md has one entry for the new check that names the root cause.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass, including the corpus test that requires every `step!` to have a refusing case.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
