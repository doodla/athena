# Job 20261108-100324-9bx9: Split status.rs into modules and shorten status() under limits

## Request (verbatim)

> try again on 20261026-144242-sjp3

## Reading (accepted by the user)

Retry of job 20261026-144242-sjp3 (rejected: the earlier reading trusted a stale 916-line figure; at base status.rs has about 2439 production lines before the test modules at line 2440, so it breaches the 2000 file limit as well as the 200-line `status` limit). Goal: make status.rs pass code health on both limits, for code health and maintainers. Move the throughput/waste code and the planning/keep-rule code into submodules under crates/athena-core/src/status/, re-exported so paths and tests are unchanged, and extract helpers from `status`.

Assumptions:
- Prior job 20261026-144242-sjp3 findings addressed: (1) major, file still over 2000 production lines: fixed by splitting into submodules, moving the old 'splitting is out of scope' item into scope; (2) major, reading mismatch with request: this reading adds a criterion for the file limit and requires measured before/after counts; (3) minor, status only 15 lines under the limit and the job loop not extracted: status must end at 150 lines or fewer, with the job loop moved into a helper; (4) nit, no tests for helpers: existing tests keep covering them via Athena::status, and the producer may add tests but need not. Its candidate changes to status.rs and CHANGELOG.md were discarded and are redone from base; both files stay deliverables.
- Measured with Grep over the base snapshot: status.rs has `mod mistake_tests` at line 2440, so about 2439 production lines (the verifier's 2478 figure counts differently, so re-measure with code health's method). Throughput/waste/LandedJob/Bars/BuildWait code spans roughly lines 575-1416 and planning/keep-rule/later-harm/producer-settings spans roughly lines 120-490. Moving both leaves about 1200-1300 production lines.
- Code survey: read crates/athena-core/src/status.rs (items listed by Grep: Athena::status at line 2079, trace at 2349, tests from 2440 using the file's items) and found no existing status/ directory. lib.rs declares `mod status`; submodules go in crates/athena-core/src/status/ and are re-exported with `pub use` from status.rs so existing `crate::status::X` paths, the mistake_tests and build_measurement_tests, and other crates compile unchanged. No new record or data file is added, so the records validation allow-list, check_record_file and AnyRecord are not touched. The line count is measured directly with code_health::measure_source on the file.
- Dependencies: the new modules use `use super::*` or explicit imports; private items used across modules become pub(super) or pub(crate). Visibility changes must not widen the public API of the crate beyond the existing re-exports.
- Tests in status.rs may need imports adjusted; that is allowed, but no test body, assertion or name is deleted or weakened.

## Acceptance criteria

1. [content] crates/athena-core/src/status.rs has fewer than 1500 production lines (total lines minus #[cfg(test)] module ranges, counted as code_health::measure_source does), down from about 2439 at base. A reader can count it from the file, and the before and after figures appear in the producer's final report and the CHANGELOG entry.
2. [content] No function in status.rs or the new files under crates/athena-core/src/status/ is longer than 200 lines; `Athena::status` is 150 lines or fewer and delegates the job loop and next-line construction to named private helpers.
3. [content] New modules live under crates/athena-core/src/status/, split by responsibility (throughput/waste, planning cost and keep rule), each under 2000 lines and each item re-exported from status.rs so every existing `crate::status::` path still resolves. Public signatures, StatusView fields, and next-line texts and order are unchanged.
4. [content] No existing test is edited in substance or deleted, no #[allow] or other lint suppression is added, and every new module and extracted helper has a doc comment written for an outside reader.
5. [content] CHANGELOG.md gains one Changed entry saying status.rs was split into modules, with the before and after production line counts.
6. [behavior] Run `cargo test` from the repository root: it passes with the existing status tests, including mistake_tests and build_measurement_tests, unchanged.
7. [behavior] Run `cargo clippy --all-targets` (no warnings) and `cargo fmt --check` (clean).

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/status/throughput.rs (create)
- crates/athena-core/src/status/planning.rs (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/lib.rs (modify)

## Out of scope

- Changing the status output, next-step text, StatusView shape or any public signature
- Changing the `trace` method or other behavior
- Changing code-health limits, athena.toml or other governance files
- Adding a reading-acceptance code check (the introspection's prevention proposal, already tried in earlier cases)
- Editing or deleting existing tests

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One refactor with no behavior change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria measure the file against the limit, not the activity of splitting.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Line counts and the tests are the checks.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the file-size breach, not only the long function.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments on new modules and helpers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261026-144242-sjp3 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/status.rs): status.rs is still over the 2000-production-line file limit after the change, so the file is not under its limits as the request asks. The candidate file is 4557 lines. Its #[cfg(test)] modules span lines 2465-2681, 2683-2816 and 2818-4545, which is 2079 lines, leaving about 2478 production lines (code_health::measure_source counts total lines minus cfg(test) module ranges). That is over MAX_FILE_LINES = 2000; the base was already about 2451. The request's 916-line figure comes from a code-health run on 2026-10-26 (records/jobs/2026-10/20261026-144242-sjp3/request.json in the evidence snapshot) and is out of date. Code health suppresses a new structural pass only while this one is open, so it will flag the file again right after landing. Better fix: split status.rs by responsibility (throughput/waste, planning cost/keep rule, the StatusView types) into modules, or re-scope this job to include that split.
> major: The reading does not match the request, so passing criteria do not show the request was met The request asks to "Restructure the file so it is under its limits". The reading assumed status.rs is "916 production lines, well under the 2000 line limit. Only the 200 line per-function limit is breached", and put splitting the file into modules out of scope. That figure comes from an out-of-date measurement (2026-10-26). The file at base has about 2451 production lines (4557 total minus 2079 lines of cfg(test) modules), so it is over the 2000-line file limit too. The reading guessed where the file could have been measured, and after this change the file is still over its limits.

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

- path: <state>/jobs/20261108-100324-9bx9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-100324-9bx9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/status.rs has fewer than 1500 production lines (total lines minus #[cfg(test)] module ranges, counted as code_health::measure_source does), down from about 2439 at base. A reader can count it from the file, and the before and after figures appear in the producer's final report and the CHANGELOG entry.
No function in status.rs or the new files under crates/athena-core/src/status/ is longer than 200 lines; `Athena::status` is 150 lines or fewer and delegates the job loop and next-line construction to named private helpers.
New modules live under crates/athena-core/src/status/, split by responsibility (throughput/waste, planning cost and keep rule), each under 2000 lines and each item re-exported from status.rs so every existing `crate::status::` path still resolves. Public signatures, StatusView fields, and next-line texts and order are unchanged.
No existing test is edited in substance or deleted, no #[allow] or other lint suppression is added, and every new module and extracted helper has a doc comment written for an outside reader.
CHANGELOG.md gains one Changed entry saying status.rs was split into modules, with the before and after production line counts.
Run `cargo test` from the repository root: it passes with the existing status tests, including mistake_tests and build_measurement_tests, unchanged.
Run `cargo clippy --all-targets` (no warnings) and `cargo fmt --check` (clean).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
