# Job 20261031-082442-92qg: Record the person's focus and cap urgent/high priorities in the plan

## Request (verbatim)

> Priority: high
> Record the person's current focus, and cap how many items can be urgent or high
> 
> On 2026-10-31 the person running Athena re-tiered the queue by hand (13 urgent and 14 high items, so priority ordered nothing), after the owner said the focus is speed first. Athena should hold that focus and keep priorities meaningful itself. This is the first of two parts; a steward triage pass that uses it is a separate request.
> 
> 1. Add `athena focus "<text>"`, which appends a focus record (records/focus/) with the text and who set it; `athena focus` with no text prints the current one; `athena status` shows it. The latest record is the current focus.
> 2. Add caps in athena.toml (with the schema field in crates/athena-data/src/config.rs): at most 3 urgent and 10 high open items. When the effective priorities exceed a cap, `athena plan` treats the extra items as one tier lower, keeping the ones set most recently by a person first, then the rest in plan order, and prints one line per demoted item. Nothing is rewritten; a person's `athena priority` record still wins for its item.
> 
> Acceptance:
> - A test shows `athena focus "speed first"` writes a record that `athena focus` and `athena status` print, and a later focus replaces it.
> - A test with 5 urgent items shows the plan treats exactly 3 as urgent, keeps a person-set urgent item among them, and prints a line for each demoted item.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/main.rs, crates/athena-core/src/plan.rs, crates/athena-data/src/records.rs, crates/athena-data/src/config.rs, athena.toml, CHANGELOG.md

## Reading (accepted by the user)

Add an `athena focus` command backed by a new focus record (a directory that does not yet exist at the base commit and is created by this change), shown in `athena status`, plus config caps (3 urgent, 10 high) that make `athena plan` demote the excess items one tier in memory. Audience: the person running Athena, who needs priority to order work again.

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (PriorityRecord, PRIORITY_LEVELS, the AnyRecord enum with its validate, path and schema-name match arms, and the records/ allow-list error), crates/athena-core/src/plan.rs (Priority enum, effective_priorities, PlanEntry.priority, priority_violations) and crates/athena-cli/src/main.rs plus crates/athena-core/src/status.rs. A new athena.focus/1 variant must be added to AnyRecord with validate, path and schema arms so check_record_file and the AnyRecord round-trip test accept it; the producer must extend that round-trip test. The cap is measured directly on the order and tiers `athena plan` produces, not through a stand-in.
- The records/focus/ directory does not exist at the base commit; this change introduces it at runtime through the new record path (records/ is never edited by hand and no files are committed there). Focus record path pattern is assumed to be records/focus/<month>/<id>.json, like other month-sharded records, with fields text and set_by.
- Latest focus record means the greatest id (time-ordered). 'Set by' is taken from the same identity source other person-written records use, such as the priority record.
- Caps live in athena.toml under a new section, with a defaults-bearing field in crates/athena-data/src/config.rs (urgent_cap = 3, high_cap = 10). Demotion is computed in memory in plan.rs on effective priorities: urgent excess goes to high, then high excess (including those demoted) goes to normal. Kept order: items whose priority was set by a person record first (most recent first), then the rest in plan order. Items with a person's priority record are never demoted below their own record.
- Each demoted item prints one line in `athena plan` output; no request or priority record is rewritten.
- The steward triage pass that uses focus is a separate request and is not built here.

## Acceptance criteria

1. [behavior] Run `cargo test`: a CLI test runs `athena focus "speed first"`, then asserts a focus record file is written under the new focus records directory (created by the test's temp repo), that `athena focus` prints "speed first", and that `athena status` output contains it.
2. [behavior] Run `cargo test`: a test sets a second focus after the first and asserts `athena focus` and `athena status` print only the later text.
3. [behavior] Run `cargo test`: a plan test with 5 urgent items asserts exactly 3 are treated as urgent, a person-set urgent item (via a priority record) is among those 3, and the plan output has one demotion line for each of the 2 demoted items.
4. [behavior] Run `cargo test`: a test asserts the high cap of 10 demotes excess high items to normal, and that a person's priority record still wins for its own item; no request or priority record file is modified by planning.
5. [content] athena.toml sets the urgent cap to 3 and the high cap to 10, and crates/athena-data/src/config.rs declares the matching schema fields with those defaults; Grep shows both.
6. [content] crates/athena-data/src/records.rs adds the focus record to AnyRecord with validation (non-empty text and set_by), a path and a schema name, and the AnyRecord round-trip test covers it.
7. [content] CHANGELOG.md has an entry describing `athena focus`, the status line and the priority caps.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/focus.rs (create)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- The steward triage pass that uses the focus (a separate request)
- Re-tiering or rewriting existing queue items or priority records
- Editing anything under records/ by hand

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus the athena.toml setting.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for focus and the caps; cargo test and clippy -D warnings pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands that exercise focus, status and the plan.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the demotion lines and the exact count of urgent items, not just that fields exist.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The caps are enforced in the plan code, with config fields in the schema.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; the steward pass is left out.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 15 cases in 13 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make Athena own the build-folder lifecycle in one place that every run path calls, not only drain start. Before any retry or new build starts, and before the checks stage, Athena removes the build folders of jobs that are interrupted, abandoned, finished or superseded. A job's retry deletes its own ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a CLI test runs `athena focus "speed first"`, then asserts a focus record file is written under the new focus records directory (created by the test's temp repo), that `athena focus` prints "speed first", and that `athena status` output contains it.
Run `cargo test`: a test sets a second focus after the first and asserts `athena focus` and `athena status` print only the later text.
Run `cargo test`: a plan test with 5 urgent items asserts exactly 3 are treated as urgent, a person-set urgent item (via a priority record) is among those 3, and the plan output has one demotion line for each of the 2 demoted items.
Run `cargo test`: a test asserts the high cap of 10 demotes excess high items to normal, and that a person's priority record still wins for its own item; no request or priority record file is modified by planning.
athena.toml sets the urgent cap to 3 and the high cap to 10, and crates/athena-data/src/config.rs declares the matching schema fields with those defaults; Grep shows both.
crates/athena-data/src/records.rs adds the focus record to AnyRecord with validation (non-empty text and set_by), a path and a schema name, and the AnyRecord round-trip test covers it.
CHANGELOG.md has an entry describing `athena focus`, the status line and the priority caps.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
