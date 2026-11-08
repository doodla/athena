# Job 20261026-144242-sjp3: Break up the 205-line status function in status.rs

## Request (verbatim)

> Structural pass: crates/athena-core/src/status.rs
> 
> Code health found crates/athena-core/src/status.rs over a limit: function status is 205 lines, over the limit of 200.
> 
> Measured: 916 production lines (test modules excluded); longest function status at 205 lines; 9 landings in the last 7 days; 14 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function status is 205 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/status.rs

## Reading (accepted by the user)

Refactor `Athena::status` (crates/athena-core/src/status.rs, line 2079, ~205 lines) into private helper methods or functions so every function is at most 200 lines. Behavior and output stay identical. The audience is code health, which measures the limit, and the maintainers who read the file.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. `status()` (lines 2079-2346) loads Records and the RecordSet, then builds job lines (jobs_by_state, recent, open, first_round), intervention counts, per-class forecast headlines, the pending quantities, the bootstrap note and the `next` action lines. It ends by assembling StatusView. The `trace` method follows it and is untouched.
- Natural seams in `status()`: (1) the job loop that produces jobs_by_state, recent, open and first_round; (2) the per-class forecast headlines; (3) the pending-quantity resolution join; (4) the bootstrap note; (5) the `next` line construction. Extract these into private free functions or methods that take `&Records` or `&[JobLine]`. Keep the order of `next` pushes exactly as it is.
- The file is 916 production lines, well under the 2000 line limit. Only the 200 line per-function limit is breached. Whole-file splitting into modules is not needed. The big test module stays where it is.
- No record or data files are added, so the records allow-list and round-trip tests are not affected. No rule is added or tightened, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows, lint config) are not needed. Do not add lint allows to silence the limit.

## Acceptance criteria

1. [content] In crates/athena-core/src/status.rs, no function, including the `status` method, is longer than 200 lines. A reader can check this by reading the file. `status` is now clearly shorter than 205 lines and delegates to named helpers.
2. [content] The diff touches only crates/athena-core/src/status.rs. The StatusView fields, public signatures and `next` line texts and order are unchanged, and no existing test is edited or deleted.
3. [behavior] `cargo test -p athena-core` passes with the existing status tests unchanged.
4. [behavior] `cargo clippy --all-targets` reports no warnings, and `cargo fmt --check` passes.
5. [content] The extracted helpers have doc comments written for an outside reader, and no lint suppression is added to hide the length.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Splitting status.rs into multiple modules
- Changing the `trace` method or any other function
- Changing the status output, next-step text or StatusView shape
- Changing code-health limits or athena.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file, one focused extraction.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Break up the long function along real responsibilities instead of trimming lines to land just under 200.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Existing tests are run to show behavior is unchanged.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New helpers get doc comments.
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
- 9 cases in 9 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs when a reading is accepted. For any request that names a size or line-count target, it measures the named files in the current snapshot and records the measured values in the reading. It refuses or flags the reading if the stated figure differs from the measurement. Requir...

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

- path: <state>/jobs/20261026-144242-sjp3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144242-sjp3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In crates/athena-core/src/status.rs, no function, including the `status` method, is longer than 200 lines. A reader can check this by reading the file. `status` is now clearly shorter than 205 lines and delegates to named helpers.
The diff touches only crates/athena-core/src/status.rs. The StatusView fields, public signatures and `next` line texts and order are unchanged, and no existing test is edited or deleted.
`cargo test -p athena-core` passes with the existing status tests unchanged.
`cargo clippy --all-targets` reports no warnings, and `cargo fmt --check` passes.
The extracted helpers have doc comments written for an outside reader, and no lint suppression is added to hide the length.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
