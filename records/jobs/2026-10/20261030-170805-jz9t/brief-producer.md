# Job 20261030-170805-jz9t: Check reading's relied-on earlier-job files exist in the snapshot

## Request (verbatim)

> try again on 20261030-060751-5389

## Reading (accepted by the user)

Retry of job 20261030-060751-5389, which failed in planning round 3. Add a refusal in crates/athena-core/src/reading.rs, run from read_once via check_reading_acceptance, that refuses a reading relying on an earlier job's or path's unlanded files. Part 3 starts only from jobs, paths or file stems the reading's own text names; jobs the request merely cites as evidence stay unchecked. Audience: the planner, critic and verifier, who now find missing dependencies late.

Assumptions:
- Retry of job 20261030-060751-5389 (failed in planning, no candidate, no files changed). Its findings are addressed: (1) major, Part 3 over-refusing cited jobs: fixed by starting only from what the reading's own text names (job id, an unlanded added path, or its file stem); evidence-only citations stay unchecked as at reading.rs:510-512, and a test has a request citing a rejected job with unlanded files, the reading never mentions it, and the reading accepted. (2) minor, symbol-shape rule: CamelCase and UPPER_SNAKE words count only with a code marker (backtick span, `::` path or following `(`), so GitHub and env-var names are not candidates; guard tests for both. (3) nit, crate-relative paths like src/reading.rs: stated in docs/decisions.md as unchecked, with a test showing it. (4) minor, run_pin_tests.rs not a deliverable: now listed as a deliverable because the reader-level test through Athena::ask lives there.
- Self-application: the reading for this request, which cites six jobs but relies on none, is a test case and must be accepted.
- Code survey: entry point is read_once (crates/athena-core/src/reading.rs:372), which calls check_reading_acceptance (reading.rs:2678); the new check is added there beside check_boundary_and_repo_files (2600), reusing referenced_jobs (2335), unaccounted_files (2472) and names_path_token (2516). Path and symbol existence are measured directly against the snapshot file list and file contents, not via a stand-in. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. Reader-level tests go in crates/athena-core/src/run_pin_tests.rs through Athena::ask.
- Symbol matching beyond definitions found in .rs files and any change to dependency planning are not done here. The prior job's recorded refusal-rate estimate is kept: expected 8% of newly refused readings.

## Acceptance criteria

1. [behavior] Unit test in crates/athena-core/src/reading.rs: a reading that names an earlier job id (or one of its unlanded paths or file stems) as something it builds on is refused with a message naming the job and the missing files. The same reading is accepted when the item is a create deliverable, is in out_of_scope, or exists in the snapshot. Run: cargo test -p athena-core reading. The test fails without the change.
2. [behavior] A test uses at least two distinct missing dependencies (an unlanded path and a symbol or role) so the class is covered, not one job's case. Run: cargo test -p athena-core reading.
3. [behavior] A test where the request cites several rejected jobs with unlanded files, and the reading never names them, is accepted, using this request's own wording and reading as one case. Run: cargo test -p athena-core reading.
4. [behavior] A reader-level test in crates/athena-core/src/run_pin_tests.rs through Athena::ask fails if the check is not wired into read_once. Run: cargo test -p athena-core run_pin.
5. [behavior] Guard tests show that GitHub and an environment-variable name in a survey are not refused, and that a crate-relative path such as src/reading.rs is documented as unchecked with a test showing it. Run: cargo test -p athena-core reading.
6. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 8% of newly refused readings. A replay or test over stored past readings shows none that were previously accepted and rely on nothing from a cited job is newly refused. Run: cargo test -p athena-core.
7. [content] CHANGELOG.md and docs/decisions.md name the root cause (readings take a named earlier job's deliverable as existing without checking the snapshot), say the fix sits in the reading-acceptance check in crates/athena-core/src/reading.rs, and record the narrowings (evidence-only citations unchecked, crate-relative paths unchecked, symbol-marker rule).
8. [behavior] cargo fmt --check, cargo clippy and cargo test all pass. Run those from the repository root.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Retrying jobs 20261026-042558-5p65, 20261026-064950-38jx, 20261026-065228-02zf; their candidate files (crates/athena-core/src/code_health.rs, steward.rs, status.rs, gh.rs, display.rs, run.rs, introspect.rs, jev.rs, records.rs, briefs.rs) are dropped, not kept
- Changing planner or critic prompts
- Changing athena.toml or defaults.toml
- Symbol-shape matching beyond definitions found in .rs files, and any change to dependency planning

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check readings being refused or accepted, not that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix sits in the reading-acceptance check, covering the class.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo test commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision record and CHANGELOG are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

21 more qualifying classes are left out.

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

Unit test in crates/athena-core/src/reading.rs: a reading that names an earlier job id (or one of its unlanded paths or file stems) as something it builds on is refused with a message naming the job and the missing files. The same reading is accepted when the item is a create deliverable, is in out_of_scope, or exists in the snapshot. Run: cargo test -p athena-core reading. The test fails without the change.
A test uses at least two distinct missing dependencies (an unlanded path and a symbol or role) so the class is covered, not one job's case. Run: cargo test -p athena-core reading.
A test where the request cites several rejected jobs with unlanded files, and the reading never names them, is accepted, using this request's own wording and reading as one case. Run: cargo test -p athena-core reading.
A reader-level test in crates/athena-core/src/run_pin_tests.rs through Athena::ask fails if the check is not wired into read_once. Run: cargo test -p athena-core run_pin.
Guard tests show that GitHub and an environment-variable name in a survey are not refused, and that a crate-relative path such as src/reading.rs is documented as unchecked with a test showing it. Run: cargo test -p athena-core reading.
The change refuses more readings and the higher refusal rate is intended: expected 8% of newly refused readings. A replay or test over stored past readings shows none that were previously accepted and rely on nothing from a cited job is newly refused. Run: cargo test -p athena-core.
CHANGELOG.md and docs/decisions.md name the root cause (readings take a named earlier job's deliverable as existing without checking the snapshot), say the fix sits in the reading-acceptance check in crates/athena-core/src/reading.rs, and record the narrowings (evidence-only citations unchecked, crate-relative paths unchecked, symbol-marker rule).
cargo fmt --check, cargo clippy and cargo test all pass. Run those from the repository root.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
