# Job 20261027-011638-5xv8: Record crate placement facts and add a test-placement reading rule

## Request (verbatim)

> Add the crate dependency facts that matter for test placement to docs/system-facts.md (which crates have library targets, which may depend on which, and where xtask code must be tested), and add a reading-brief rule to place each test in a crate that can reach the target. Merge failure classes 20261026-155850-0ksw and 20261026-170231-4fcf into one. Acceptance: system-facts drift test covers the new facts.
> 
> Acceptance:
> - The change is made as described: Reading or plan puts a test in a crate that cannot import the code under test (e.g. xtask has no library target); found late, fixed by hand; registry holds two classes for one failure
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/system-facts.md

## Reading (accepted by the user)

Add a crate-dependency fact (library targets, allowed dependency directions, where xtask code is tested) to docs/system-facts.md, add a ninth reading-brief rule that a test must sit in a crate that can reach the code under test, and extend the system-facts drift test to cover the new facts. The failure-class merge lives in records/, which no job changes, so it is left out. The goal is to stop readings and plans from putting tests where they cannot import the target; the audience is the planner, critic and reader, plus the verifier.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (READING_RULES, an array of 8; SYSTEM_FACTS include_str; the test the_system_facts_state_every_fact_with_a_source_and_stay_short with its 5,000-byte cap and marker list), crates/athena-cli/tests/budgets.rs (check_system_facts and system_facts_drift_is_rejected), crates/athena-core/src/reading/checks.rs and surveys.rs (check_code_survey order). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Resolution and probing are untouched.
- Goal: the planner, critic and reader stop placing tests in a crate that cannot import the code under test, such as xtask, which has no library target. The facts file is read by the planner and critic; the new rule is read by the reader.
- The facts must come from the Cargo.toml files. Only athena-cli declares a [[bin]] (named athena); xtask is a binary-only package with a name 'xtask' and its tests sit in xtask/tests/policy.rs plus unit tests inside xtask/src/main.rs. The producer must read each crates/*/Cargo.toml [dependencies] to state the allowed dependency directions rather than guess.
- The new facts bullet follows the file's form: a bold title and a 'Source' naming Cargo.toml paths. The file stays under 5,000 bytes. The new fact stays free of the banned-word scan list and of email-like text.
- The drift test extension is in crates/athena-cli/tests/budgets.rs: a check that reads the crate facts lines (a fixed line form such as '- crate <name>: lib|bin-only; depends on <list>') and compares them with cargo metadata or the Cargo.toml files, rejecting a doctored line, a missing crate and an unknown crate. This mirrors the budget-line check.
- The reading-brief rule is added to READING_RULES in briefs.rs as a ninth line, with its array length and any test or count that asserts 8 updated. The reader prompt is at 1997 of 2000 tokens, and the rules render in the reader brief rather than the always-loaded set, so the producer must run the budgets test to confirm no set exceeds its budget. If the rule is enforced by a check, it may be marked prose-only; this reading assumes prose-only and says so in the rule's text.
- Failure classes 20261026-155850-0ksw and 20261026-170231-4fcf live in the records registry. records/ is absent from the snapshot and never changes, so the merge is outside this job. Neither id appears in any tracked file (searched with Grep).
- Docs/system-facts.md is embedded by include_str, so the budget lines stay accurate; budgets are not changed. CHANGELOG.md gets one Unreleased line.

## Acceptance criteria

1. [content] docs/system-facts.md has a new bullet with a 'Source' naming Cargo.toml paths. It states which crates have library targets, that athena-cli is the only crate with a binary and that xtask has no library target, which crates may depend on which, and that xtask code is tested in xtask/tests/ or in unit tests inside xtask/src/main.rs. A reader can check each claim against the Cargo.toml files.
2. [content] READING_RULES in crates/athena-context/src/briefs.rs has a new line saying to place each test in a crate that can reach the code under test (for example, no integration test for a binary-only crate like xtask), and the array length and every test that counts the rules are updated. A Grep of briefs.rs shows the line.
3. [behavior] Run cargo test: crates/athena-cli/tests/budgets.rs has a drift test that covers the new crate facts lines. It fails when a facts line is doctored, missing or names an unknown crate, and it passes on the real file. The existing system_facts_budgets_match_computed and system_facts_drift_is_rejected tests still pass.
4. [behavior] Run cargo test: the_system_facts_state_every_fact_with_a_source_and_stay_short passes with the new bullet (markers added for it, file under 5,000 bytes), and the reader prompt-budget tests pass.
5. [behavior] Run cargo fmt --check, cargo clippy and cargo test: all pass with no unrelated behavior changes.
6. [content] The diff changes nothing under records/ and no governance files (athena.toml, defaults.toml, workflows); the failure-class merge is not attempted.

## Expected deliverables

- docs/system-facts.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Merging failure classes 20261026-155850-0ksw and 20261026-170231-4fcf: they live in the records registry (records/), which no job changes; the user must do it with the records tooling.
- A mechanical check that refuses a misplaced test; the new rule is prose-only.
- Changing budgets, athena.toml, defaults.toml, or any workflow.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The drift test is Rust in the existing budgets.rs test file.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New facts lines are covered by the drift test; cargo test and clippy must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new facts name Cargo.toml sources.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs fmt --check, clippy and test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Facts are enforced by the drift test; the reading rule is marked prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; the records merge is excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule targets the whole class of unreachable test placement, not the one xtask case.
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

- path: <state>/jobs/20261027-011638-5xv8/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-011638-5xv8/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/system-facts.md has a new bullet with a 'Source' naming Cargo.toml paths. It states which crates have library targets, that athena-cli is the only crate with a binary and that xtask has no library target, which crates may depend on which, and that xtask code is tested in xtask/tests/ or in unit tests inside xtask/src/main.rs. A reader can check each claim against the Cargo.toml files.
READING_RULES in crates/athena-context/src/briefs.rs has a new line saying to place each test in a crate that can reach the code under test (for example, no integration test for a binary-only crate like xtask), and the array length and every test that counts the rules are updated. A Grep of briefs.rs shows the line.
Run cargo test: crates/athena-cli/tests/budgets.rs has a drift test that covers the new crate facts lines. It fails when a facts line is doctored, missing or names an unknown crate, and it passes on the real file. The existing system_facts_budgets_match_computed and system_facts_drift_is_rejected tests still pass.
Run cargo test: the_system_facts_state_every_fact_with_a_source_and_stay_short passes with the new bullet (markers added for it, file under 5,000 bytes), and the reader prompt-budget tests pass.
Run cargo fmt --check, cargo clippy and cargo test: all pass with no unrelated behavior changes.
The diff changes nothing under records/ and no governance files (athena.toml, defaults.toml, workflows); the failure-class merge is not attempted.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
