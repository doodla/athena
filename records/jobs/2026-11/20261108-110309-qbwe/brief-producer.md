# Job 20261108-110309-qbwe: Move status rendering out of display.rs and repoint source-reading test

## Request (verbatim)

> try again on 20261026-193430-avnq

## Reading (accepted by the user)

Retry of job 20261026-193430-avnq: move the status-rendering group from crates/athena-cli/src/display.rs into display/status_view.rs (re-exported, bodies verbatim) to get under the 1500-line/5-landing limit, and also repoint the athena-core test that reads display.rs's source text, which broke last time. Audience: the code-health gate and maintainers; no behavior change.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (test mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line at ~1402 uses include_str!("../../athena-cli/src/display.rs") and searches for printed label strings like "code health ") and crates/athena-cli/src/display.rs (imports at top, tests module at ~1944). That include_str! is the only source-text reader of display.rs found in crates/ by Grep.
- Job 20261026-193430-avnq: its candidate (CHANGELOG.md entry, display.rs trimmed to ~1030 lines, new display/status_view.rs) is kept, redone from base. Its blocker (athena-core test reading display.rs text broke) is fixed by adding include_str!("../../athena-cli/src/display/status_view.rs") to the searched sources in status.rs. Its major finding (reading narrowed tests to athena-cli) is fixed by requiring the full workspace test. The nit (misplaced repeated-causes doc comment) is carried verbatim and left alone as a pure move.
- display.rs becomes a directory-module root with a child file display/status_view.rs declared by `mod status_view;` and `pub use status_view::{status, snapshot_status};`; lib.rs probably needs no change (listed with low probability).
- The test keeps checking the same labels; only its source set widens, so it is not weakened.
- Moved bodies are verbatim; helpers shared with other display functions stay in display.rs, with visibility widened to pub(super) only where needed.

## Acceptance criteria

1. [content] crates/athena-cli/src/display.rs has fewer than 1500 production lines (test module excluded) and the status-rendering functions (plan_suffix, status, snapshot_status, code_health_section, introspection and their section helpers) are defined in the new crates/athena-cli/src/display/status_view.rs, checkable with Grep/Read.
2. [content] display.rs re-exports status and snapshot_status so no caller outside display changes; moved function bodies are verbatim (git diff shows only moves plus visibility/import edits).
3. [content] crates/athena-core/src/status.rs test every_measure_label_names_a_status_view_field_and_a_printed_line additionally searches include_str!("../../athena-cli/src/display/status_view.rs"); its label list and assertions are unchanged, and no test is deleted or weakened.
4. [behavior] Run `cargo test --workspace`: all tests pass, including athena-core status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line and the athena-cli display tests.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --workspace --all-targets`: both clean.
6. [content] CHANGELOG.md has a Changed entry for the split; no governance files (athena.toml, defaults.toml, Cargo configs) change.

## Expected deliverables

- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/display/status_view.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/lib.rs (modify)

## Out of scope

- Behavior or output format changes
- Splitting other files or raising limits
- Refactoring the status function itself
- Fixing the misplaced doc comment on trend_section/repeated_causes_section
- Changes to athena.toml, defaults.toml or other governance files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module split.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full workspace test and clippy required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Finds and repoints the source-text-reading test, not just display.rs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run workspace tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is gate-passing size with all tests green.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261026-193430-avnq raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (crates/athena-core/src/status.rs:1402-1421 (mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line)): The move breaks an existing athena-core test that reads display.rs's source text, so the full test suite fails. The test calls include_str!("../../athena-cli/src/display.rs") and checks that each status label (for example "code health ") is printed in that source. Those printing lines moved to display/status_view.rs. Running `cargo test -p athena-core --lib every_measure_label_names_a_status_view_field_and_a_printed_line` panicked: "`athena status` prints no line starting \"code health\"". Athena's recorded `cargo test --locked` for this candidate also failed on this test. The request requires "Existing tests keep passing" and "fmt, clippy and test pass". Fix: add include_str!("../../athena-cli/src/display/status_view.rs") to the sources the test searches. That file is not on the deliverables list, so the plan needs to add it.
> major: The reading does not match the request, so passing criteria do not show the request was met The request says "Existing tests keep passing" and "fmt, clippy and test pass" for the whole workspace. The reading narrowed the test criterion to `cargo test -p athena-cli` display tests. It did not notice that athena-core's status.rs test reads display.rs's source with include_str!, so moving the status lines out of display.rs breaks that test. Because of this, the deliverables list also leaves out crates/athena-core/src/status.rs, the file the fix needs.

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

- path: <state>/jobs/20261108-110309-qbwe/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-110309-qbwe/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-cli/src/display.rs has fewer than 1500 production lines (test module excluded) and the status-rendering functions (plan_suffix, status, snapshot_status, code_health_section, introspection and their section helpers) are defined in the new crates/athena-cli/src/display/status_view.rs, checkable with Grep/Read.
display.rs re-exports status and snapshot_status so no caller outside display changes; moved function bodies are verbatim (git diff shows only moves plus visibility/import edits).
crates/athena-core/src/status.rs test every_measure_label_names_a_status_view_field_and_a_printed_line additionally searches include_str!("../../athena-cli/src/display/status_view.rs"); its label list and assertions are unchanged, and no test is deleted or weakened.
Run `cargo test --workspace`: all tests pass, including athena-core status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line and the athena-cli display tests.
Run `cargo fmt --check` and `cargo clippy --workspace --all-targets`: both clean.
CHANGELOG.md has a Changed entry for the split; no governance files (athena.toml, defaults.toml, Cargo configs) change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
