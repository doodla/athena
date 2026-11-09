# Job 20261109-161505-bp5e: Refuse a change at admission when it omits the changelog deliverable

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The changelog default is a rule the producer must remember, and no code compares the diff's file lis
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: changelog-default-deliverable-not-produced-caught-only-at-final-verdict (20261108-185500-x0jg)
> Cause: The changelog default is a rule the producer must remember, and no code compares the diff's file list with the expected deliverables before the verdict. For doc-only design jobs the changelog feels optional, so it is easy to skip. The readi...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261108-185451-t573 (job 20261108-183842-fsaz); prevention it proposed: Add a code check that runs before the verifier. When the changelog default applies to a job, it fails fast if the diff does not touch CHANGELOG.md. It should report the missing file to the producer, which gets a repair round. The reading sh...
> Introspection 20261109-161451-cn7n (job 20261028-000831-rkv5); prevention it proposed: Add a code check before handover and verification. When the changelog default applies, the check fails the producer's run if the diff has no CHANGELOG.md change, and it names the missing file. Readings should list CHANGELOG.md as a delivera...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the `changelog` default is added to every change reading's deliverables by `ensure_changelog` (crates/athena-core/src/reading.rs), but nothing in code compares the candidate's changed files with that deliverable before the verifier, so jobs 20261108-183842-fsaz and 20261028-000831-rkv5 were failed by the final verdict only. Fix: add an admission rule in crates/athena-core/src/admission.rs that raises a violation naming CHANGELOG.md when the reading lists it, the changelog default applies, and the candidate's changes do not touch it.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (ensure_changelog adds CHANGELOG.md modify p=0.9 and the `changelog` default for capability change), crates/athena-core/src/admission.rs (admit / admit_without_secrets take AdmissionInput with reading and changes, and return PolicyViolation entries), and crates/athena-core/src/run/mod.rs (~line 773-814: admission violations are summarised and fail the candidate before check_and_verify). No record or data file is added under crates/, so check_record_file/AnyRecord are not affected. The property is measured directly by comparing input.changes paths with the reading's deliverables.
- Job 20261108-183842-fsaz: its docs/design files (per-job-landing-branches.md, continuous-landing.md, squash-chain-landing.md) and its major finding (CHANGELOG.md not modified) are addressed by the class fix, not by redoing that design; that candidate's three docs files are excluded in out_of_scope as not kept here. Its minor findings (status.rs path, attribution rule) concern the design note and are dropped with it.
- Job 20261028-000831-rkv5: its candidate's changes to studies/firstmate-work-handling/intro.json and xtask/src/site.rs are not kept or redone here (out_of_scope); its major finding (no CHANGELOG.md entry) is the class this change prevents; its minor/nit findings in xtask tests are dropped as unrelated to the cause.
- Assumption: the rule applies only to change-capability readings whose deliverables include CHANGELOG.md with action modify and whose defaults_applied has id `changelog`; a violation is reported to the producer through the existing admission-violation path, which stops the candidate before the verifier. If a repair round for the producer does not already exist for admission violations, only the existing behavior is used.
- Governance: no change to athena.toml, defaults.toml or workflows is needed; the rule is code.
- The rule refuses more candidates: expected 5% of newly refused readings (candidates that skipped the changelog today fail later at the verifier, so the higher refusal rate is intended).

## Acceptance criteria

1. [content] crates/athena-core/src/admission.rs contains a rule that emits a PolicyViolation naming CHANGELOG.md when the reading's deliverables list it, defaults_applied has id `changelog`, and no change in input.changes has path CHANGELOG.md; its doc comment names the root cause (the default was a rule the producer must remember, with no code comparing the diff to the deliverables).
2. [behavior] Run `cargo test -p athena-core admission`: new tests show a candidate that changes only docs/ files with a change reading carrying the changelog default yields a violation naming CHANGELOG.md; the same candidate plus a CHANGELOG.md change yields none; a non-change reading or a reading without the default yields none. The first test fails when the new rule is removed.
3. [behavior] Run `cargo test -p athena-core` : a test with a docs-only design reading built through ensure_changelog (the path both failed jobs took) is refused by admit before verification, showing the check is for the class and not one job.
4. [behavior] Existing tests that run admit or the run pipeline with a change reading still pass (cargo test) after fixtures are updated where they omitted CHANGELOG.md.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass.
6. [content] CHANGELOG.md has an entry for this change naming the cause and the new admission rule.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Redoing the design docs of job 20261108-183842-fsaz (docs/design/per-job-landing-branches.md, continuous-landing.md, squash-chain-landing.md): dropped, not kept
- Redoing the glossary work of job 20261028-000831-rkv5 (studies/firstmate-work-handling/intro.json, xtask/src/site.rs): dropped, not kept
- Changes to athena.toml, defaults.toml, workflows or prompts
- Editing records/ or owner files
- Changing how ensure_changelog adds the deliverable

## User defaults that apply

- changelog: (not in the defaults model) Here: The change records itself in CHANGELOG.md.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New admission tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed at admission, where candidate files are first compared with deliverables, not per job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by an admission rule in code, not prose.

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

- path: <state>/jobs/20261109-161505-bp5e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-161505-bp5e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/admission.rs contains a rule that emits a PolicyViolation naming CHANGELOG.md when the reading's deliverables list it, defaults_applied has id `changelog`, and no change in input.changes has path CHANGELOG.md; its doc comment names the root cause (the default was a rule the producer must remember, with no code comparing the diff to the deliverables).
Run `cargo test -p athena-core admission`: new tests show a candidate that changes only docs/ files with a change reading carrying the changelog default yields a violation naming CHANGELOG.md; the same candidate plus a CHANGELOG.md change yields none; a non-change reading or a reading without the default yields none. The first test fails when the new rule is removed.
Run `cargo test -p athena-core` : a test with a docs-only design reading built through ensure_changelog (the path both failed jobs took) is refused by admit before verification, showing the check is for the class and not one job.
Existing tests that run admit or the run pipeline with a change reading still pass (cargo test) after fixtures are updated where they omitted CHANGELOG.md.
cargo fmt --check, cargo clippy --all-targets and cargo test pass.
CHANGELOG.md has an entry for this change naming the cause and the new admission rule.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
