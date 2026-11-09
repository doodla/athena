# Job 20261109-004033-y1jj: Split briefs.rs into submodules to get under code-health limits

## Request (verbatim)

> try again on 20261027-114907-99s2

## Reading (accepted by the user)

Retry of job 20261027-114907-99s2: measure briefs.rs in this snapshot (about 2544 production lines, reader_brief about 219 lines, not the quoted 1524), then split it into several submodules (boundary, reader, planner/critic, plan rendering) and break up reader_brief so briefs.rs is under 1500 lines and no function exceeds 200. Audience: code health (`athena status`). No behavior change.

Assumptions:
- Retry of 20261027-114907-99s2, which was rejected because briefs.rs stayed at 2287 production lines with a 217-line function. Findings addressed: (1) blocker, file still over limit: this retry moves about 1100+ lines across several submodules, not one 257-line slice. (2) major, plan sized from the quoted 1524 rather than measurement: the producer must measure first (production lines before the first #[cfg(test)], longest function, e.g. via `cargo run -q -p athena-cli -- status`) and size the split to the measurement. (3) major, reading mismatch: the earlier boundary-only reading is replaced. (4) nit, tab indentation in moved BOUNDARY_SCAN_INSTRUCTION: moves must be byte-for-byte, no tabs introduced. The candidate's work (boundary.rs, briefs.rs edits, CHANGELOG.md) is kept as the starting design (boundary move) but redone from base and extended.
- Code survey: read crates/athena-context/src/briefs.rs (about 2544 production lines; #[cfg(test)] mod tests at line 2546). Groups by Grep of top-level items: boundary scan and plain rewrite lines 815-1100 (boundary_words, boundary_fragments, banned_words_section, BOUNDARY_SCAN_INSTRUCTION, MIRROR_CHECK_INSTRUCTION, BoundaryHit, boundary_scan, plain_boundary_text); reader types and brief lines 2007-2540 (EarlierCase, RetryCause, EarlierAttempt, prior_result_section, carried_work, ReaderInputs, READING_RULES, reader_brief at 2315-2533, about 219 lines); planner and critic lines 1820-2006; plan and findings rendering lines 1589-1750; verifier lines 1105-1590. crates/athena-context/src/lib.rs and outputs.rs are the only other files in the crate. Property measured directly with athena_core::code_health through `athena status`.
- Measured at this snapshot by Grep, an estimate pending the producer's own measurement: about 2544 production lines; reader_brief about 219 lines. A target of under 1500 needs about 1050 or more lines moved; moving the reader group (about 530), boundary group (about 285) and planner/critic (about 190) gives about 1000, plus plan rendering (about 160) reaches about 1370.
- Public paths stay unchanged: each new submodule is declared in briefs.rs and its public items are re-exported with `pub use`; private helpers used by briefs.rs or its tests become pub(super). The existing tests module stays in briefs.rs, with import adjustments only. Tests or xtask policy scans that read briefs.rs as text (for example the xtask policy test) must be checked and fixed if they break; a text-reading test for the moved code may be repointed to the new file, not weakened.
- reader_brief is broken into private helper functions in the reader submodule (for example per-section helpers), keeping output byte-identical.
- CHANGELOG.md gets one entry because the repo convention records changes there. No athena.toml or limits change.

## Acceptance criteria

1. [behavior] Run `cargo run -q -p athena-cli -- status` on the candidate: crates/athena-context/src/briefs.rs shows under 1500 production lines and is not listed over any limit; no function in briefs.rs or the new submodules exceeds 200 lines (reader_brief included).
2. [content] New files under crates/athena-context/src/briefs/ (at least boundary.rs and a reader submodule) exist; briefs.rs declares each with `mod` and `pub use`s the public items, so `athena_context::briefs::` paths including boundary_words, boundary_fragments, boundary_scan, plain_boundary_text, BoundaryHit, reader_brief, ReaderInputs, planner_brief and critic_brief still resolve for other crates.
3. [content] The diff only moves code, apart from visibility changes and splitting reader_brief into helpers: Grep for a tab character in the new files finds none (string continuation lines keep the base's indentation), and no string literal or brief text changes.
4. [behavior] Run `cargo test --workspace`: all existing tests pass, including the briefs tests and any test that reads source files as text; none is deleted or weakened in the diff (test module changes limited to imports).
5. [behavior] Run `cargo clippy --workspace --all-targets` with no warnings, and `cargo fmt --check` passes.
6. [content] CHANGELOG.md has an entry describing the briefs.rs split.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/boundary.rs (create)
- crates/athena-context/src/briefs/reader.rs (create)
- crates/athena-context/src/briefs/planner.rs (create)
- crates/athena-context/src/briefs/plan.rs (create)
- CHANGELOG.md (modify)
- xtask/tests/policy.rs (modify)
- crates/athena-context/src/lib.rs (modify)

## Out of scope

- Any behavior change to brief text or boundary scanning
- Splitting other files or changing athena.toml limits
- Changes to lib.rs unless a module declaration proves necessary
- Adding the proposed Athena code check for size-reduction jobs (a separate change)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code stays Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused structural split with no behavior change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Sized to the measured file, not the quoted figure.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the code-health measurement itself.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New modules get clear doc comments.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-114907-99s2 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (crates/athena-context/src/briefs.rs): briefs.rs is still over its code-health limits after the change: 2287 production lines and a 217-line function Running `cargo run -q -p athena-cli -- status` on the candidate tree shows: crates/athena-context/src/briefs.rs 2287 lines, longest fn 217, over on file length and function length. The request asks to restructure the file 'so it is under its limits'. The base had 2544 production lines (test module at line 2545), so the move cut only about 257 lines.
> major: The plan was sized from the request's quoted figure instead of measuring the file The reading's code survey records the test module at line 2545, which implies about 2544 production lines, yet it planned around 1524. Measuring with athena_core::code_health::measure_source, which `athena status` shows, would have shown that one 270-line move cannot reach 1500. A correct approach would split briefs.rs along several responsibilities, for example per-role brief rendering, and would also break up the function over 200 lines.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading assumed briefs.rs had about 1524 production lines, so moving about 270 lines would land it under 1500. Its own code survey says the test module starts at line 2545, which means about 2544 production lines in this tree. Athena's code-health measure (total lines minus #[cfg(test)] modules) confirms 2544 at base and 2287 for the candidate. The reading's plan was sized from the request's figure rather than from the file it surveyed. One cohesive move cannot meet the request's goal of getting the file under its limits. Several groups need splitting, plus the over-long function (217 lines by the code-health measure on this tree).

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
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

- path: <state>/jobs/20261109-004033-y1jj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-004033-y1jj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo run -q -p athena-cli -- status` on the candidate: crates/athena-context/src/briefs.rs shows under 1500 production lines and is not listed over any limit; no function in briefs.rs or the new submodules exceeds 200 lines (reader_brief included).
New files under crates/athena-context/src/briefs/ (at least boundary.rs and a reader submodule) exist; briefs.rs declares each with `mod` and `pub use`s the public items, so `athena_context::briefs::` paths including boundary_words, boundary_fragments, boundary_scan, plain_boundary_text, BoundaryHit, reader_brief, ReaderInputs, planner_brief and critic_brief still resolve for other crates.
The diff only moves code, apart from visibility changes and splitting reader_brief into helpers: Grep for a tab character in the new files finds none (string continuation lines keep the base's indentation), and no string literal or brief text changes.
Run `cargo test --workspace`: all existing tests pass, including the briefs tests and any test that reads source files as text; none is deleted or weakened in the diff (test module changes limited to imports).
Run `cargo clippy --workspace --all-targets` with no warnings, and `cargo fmt --check` passes.
CHANGELOG.md has an entry describing the briefs.rs split.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
