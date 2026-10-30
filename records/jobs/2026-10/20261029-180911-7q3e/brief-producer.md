# Job 20261029-180911-7q3e: Add class-of-work trend to athena status for OLY-9-10

## Request (verbatim)

> Priority: urgent
> 
> Give requirement OLY-9-10 a mechanism
> 
> Requirement OLY-9-10 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 5 is accepted when comparable historical evidence shows repeated classes of work improving and identifies earlier changes that made things worse.
> Owner: Hestia
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-10
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-10 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Give OLY-9-10 a mechanism: a new class-trend measure in athena-core that compares older and newer landed jobs per class of work (minutes), reports classes improving, and names the earlier landed job after which the class's cost rose. It is printed by `athena status` as a `trend` line and recorded in docs/spec.toml with measure ending `status: trend`. Audience: Hestia/the owner judging Phase 5.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (Throughput::compute, landed_jobs, StageMinutes, median, STATUS_MEASURE_LABELS, StatusView, test every_measure_label_names_a_status_view_field_and_a_printed_line), crates/athena-cli/src/display.rs (status text assembly near the throughput line), crates/athena-core/src/store.rs (JobRecords.job.class_of_work, Records::request_jobs) and crates/athena-core/src/hestia.rs (measure label check). No new record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not affected; the measure is computed directly from existing job records, not via a stand-in.
- Design: new module crates/athena-core/src/class_trend.rs computes, per class_of_work (from the job record) with at least 4 landed jobs, the median total minutes of the older half against the newer half of landed_jobs(); a drop is 'improving', a rise is 'worse'. For a worsening class it names the landed job that begins the newer half's rise (the earliest job after which the class's minutes rose above the older median and stayed). StatusView gets a `class_trend` field, and STATUS_MEASURE_LABELS gets ("trend", "class_trend").
- The spec.toml measure is rewritten to end with `status: trend`, and mechanism points at class_trend.rs, status.rs and display.rs.
- Cost is measured as the existing minutes-per-landed-job (invocation, check and landing time), the comparable evidence available; token cost is out of scope.
- No refusal rule is added or tightened, so no replay refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-9-10 has mechanism naming crates/athena-core/src/class_trend.rs (a file that exists), keeps owner Hestia, and a measure ending with `status: trend`; STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains the `trend` label.
2. [behavior] Run `cargo test -p athena-core class_trend`: a test builds landed jobs of one class whose minutes fall from older to newer and asserts the class is reported improving; another builds a class whose minutes rise after a named earlier job and asserts that job id is reported as making it worse; a class with too few jobs is not reported. These tests fail if compute returns nothing.
3. [behavior] Run `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line`: passes with the new `trend` label, proving StatusView has class_trend and `athena status` prints a line starting `trend `.
4. [behavior] Run `cargo test -p athena-cli`: a status rendering test shows the `trend` line text with improving and worse classes; and the xtask/spec test over docs/spec.toml passes (`cargo test --workspace`).
5. [behavior] `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` all pass.
6. [content] The trend section is absent (no line) when no class has enough landed jobs, so an empty history is not shown as success.

## Expected deliverables

- crates/athena-core/src/class_trend.rs (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Token-cost trends or forecast-score trends
- Automatically reverting or proposing fixes for changes that made things worse
- Changing athena.toml, defaults.toml or workflows
- Other requirements without mechanisms

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the module; fmt, clippy, test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check improving/worse output.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Measure is enforced by status label test.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Absent output when history is too thin.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module plus wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 35 cases in 35 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a test that runs the real drain path through checks failing an...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

20 more qualifying classes are left out.

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

docs/spec.toml requirement OLY-9-10 has mechanism naming crates/athena-core/src/class_trend.rs (a file that exists), keeps owner Hestia, and a measure ending with `status: trend`; STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains the `trend` label.
Run `cargo test -p athena-core class_trend`: a test builds landed jobs of one class whose minutes fall from older to newer and asserts the class is reported improving; another builds a class whose minutes rise after a named earlier job and asserts that job id is reported as making it worse; a class with too few jobs is not reported. These tests fail if compute returns nothing.
Run `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line`: passes with the new `trend` label, proving StatusView has class_trend and `athena status` prints a line starting `trend `.
Run `cargo test -p athena-cli`: a status rendering test shows the `trend` line text with improving and worse classes; and the xtask/spec test over docs/spec.toml passes (`cargo test --workspace`).
`cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` all pass.
The trend section is absent (no line) when no class has enough landed jobs, so an empty history is not shown as success.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
