# Job 20261106-012852-h7de: Raise agent ceiling to 16, record group sizes, fix stale help

## Request (verbatim)

> try again on 20261105-180339-w5h7

## Reading (accepted by the user)

Redo step 6 of docs/design/agent-and-build-slots.md section 8 from base (job 20261105-180339-w5h7 was rejected; its seven changed files are kept as deliverables and redone), and fix its major finding: `athena drain --help` still said "at most 8". Audience: the maintainer reading drain output and help to see what the ceiling allows.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs (MAX_CONCURRENCY_CEILING at 269, validation at 722 and 745, tests near 1194 and 1458), crates/athena-cli/src/main.rs:275 (drain help text; help tests at 1808 and crates/athena-cli/tests/drain.rs:2054,2097), and grep found no group_sizes anywhere yet, so recording is new. No new record or data file under crates/ is added; the group sizes live in the in-memory drain Report, so AnyRecord and check_record_file are not touched. The group size is measured directly from the plan group's item count as the batch works it.
- Job 20261105-180339-w5h7: its seven changed files (CHANGELOG.md, athena.toml, display.rs, drain.rs, tests/drain.rs, config.rs, agent-and-build-slots.md) are kept as deliverables, but its changes were discarded so the work is redone from base. Its findings are addressed: major (main.rs:275 'at most 8') fixed by building the help text from MAX_CONCURRENCY_CEILING; minor stale '1 to 8' in parallel-jobs.md and async-drain.md fixed; minor untested display::drain call fixed by a test that goes through drain() output; nit about the Built note wording fixed (only max_concurrency says 'from 1 to 16').
- Job 20261105-180334-rj4j (slots, state, run checks) stays landed and untouched.
- Introspection 20261106-012825-ay9k prevention applied: help text derives the ceiling from the constant, a test checks it, and a repo grep for the old ceiling of 8 finds no stale mention outside CHANGELOG and unrelated uses (decisions.md:3255, hestia.rs:11 'rows 1 to 8', records-history.md:302 attempts are not the ceiling).
- The design's starting value for max_concurrency is 12 (section 7.2), as read in the earlier accepted reading.
- Step 6 touches crates/athena-core/src/plan.rs only if group item counts need a helper; it may stay unchanged.

## Acceptance criteria

1. [content] crates/athena-data/src/config.rs declares MAX_CONCURRENCY_CEILING as 16; its doc comments and the athena.toml [parallel] comment say 1 to 16, and athena.toml max_concurrency is 12 (athena.toml is a deliverable for this reason).
2. [content] crates/athena-cli/src/main.rs drain help text no longer hard-codes 8: it is built from MAX_CONCURRENCY_CEILING (or states 16), and Grep for 'at most 8' and 'allows 1 to 8' / 'max_concurrency 1 to 8' in crates/, athena.toml and docs/design finds none; async-drain.md and parallel-jobs.md say 1 to 16 (or note the later change).
3. [behavior] Run `cargo test -p athena-cli`: a test renders `drain --help` and asserts it contains the current MAX_CONCURRENCY_CEILING value formatted from the constant, so help and ceiling cannot disagree.
4. [behavior] Run `cargo test -p athena-data`: tests show max_concurrency = 16 and max_in_flight = 16 validate and 17 is refused (max_concurrency with the 'from 1 to 16' error).
5. [behavior] Run `cargo test -p athena-cli --test drain`: a drain of a plan with groups of different sizes asserts the Report's group-size record lists the size of each group worked, in order, and that groups not reached are not recorded.
6. [behavior] Run `cargo test -p athena-cli`: a test calls display::drain with a report and asserts the end-of-run output contains the exact group-sizes line with the mean, and prints no such line when no group was worked.
7. [content] docs/design/agent-and-build-slots.md step 6 has a short Built note stating the ceiling 16, starting value 12, what is recorded, and that peak in-flight and the low-disk count are not measured; it does not claim max_in_flight 17 gives the 'from 1 to 16' text, and lines respect the doc wrap width.
8. [content] CHANGELOG.md has an entry for the ceiling and group-size record, and an entry under Fixed for the drain help text now following the ceiling constant.
9. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- docs/design/async-drain.md (modify)
- docs/design/parallel-jobs.md (modify)
- crates/athena-core/src/plan.rs (modify)

## Out of scope

- Disk.rs per-slot build time T, the cargo-internal-error slot delete rule and the cleaning env nit from job 20261105-180334-rj4j's findings
- Files changed by job 20261105-180334-rj4j (slots.rs, state.rs, run/checks.rs, run/mod.rs, run/tests.rs, athena-agent/src/env.rs): kept as landed
- Changing defaults.toml or [drain.pipeline] values
- Landing rebuild slot use (step 7) and producer cargo wrapper (step 4)
- Measuring peak in-flight jobs or low-disk counts from real drains
- Changing records/ or adding a record type
- Unrelated 'rows 1 to 8' and 'at most 8 attempts' mentions that are not the agent ceiling

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the ceiling, group sizes, display and help; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Help text derives from the constant rather than editing the number.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check help, refusal, recording and display output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check what the maintainer sees in help and drain output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused step-6 change plus the stale-mention fix.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that help and the ceiling agree.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261105-180339-w5h7 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-cli/src/main.rs:275): The `athena drain` help text still says the ceiling is 8. main.rs is on the deliverables list and was not changed. main.rs:274-275 reads 'drain runs up to `[parallel] max_concurrency` of them at once (athena.toml; default 1, which is one job at a time; at most 8)'. This change raised MAX_CONCURRENCY_CEILING to 16 and set athena.toml to 12. The CLI help now states a limit that is false and below the shipped value, so the maintainer reading `athena drain --help` is misled about what the change allows. The reading listed crates/athena-cli/src/main.rs (modify) as an expected deliverable, but `git diff HEAD~1 --stat` shows it untouched. Fix: change 'at most 8' to 'at most 16'.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-012852-h7de/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-012852-h7de/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-data/src/config.rs declares MAX_CONCURRENCY_CEILING as 16; its doc comments and the athena.toml [parallel] comment say 1 to 16, and athena.toml max_concurrency is 12 (athena.toml is a deliverable for this reason).
crates/athena-cli/src/main.rs drain help text no longer hard-codes 8: it is built from MAX_CONCURRENCY_CEILING (or states 16), and Grep for 'at most 8' and 'allows 1 to 8' / 'max_concurrency 1 to 8' in crates/, athena.toml and docs/design finds none; async-drain.md and parallel-jobs.md say 1 to 16 (or note the later change).
Run `cargo test -p athena-cli`: a test renders `drain --help` and asserts it contains the current MAX_CONCURRENCY_CEILING value formatted from the constant, so help and ceiling cannot disagree.
Run `cargo test -p athena-data`: tests show max_concurrency = 16 and max_in_flight = 16 validate and 17 is refused (max_concurrency with the 'from 1 to 16' error).
Run `cargo test -p athena-cli --test drain`: a drain of a plan with groups of different sizes asserts the Report's group-size record lists the size of each group worked, in order, and that groups not reached are not recorded.
Run `cargo test -p athena-cli`: a test calls display::drain with a report and asserts the end-of-run output contains the exact group-sizes line with the mean, and prints no such line when no group was worked.
docs/design/agent-and-build-slots.md step 6 has a short Built note stating the ceiling 16, starting value 12, what is recorded, and that peak in-flight and the low-disk count are not measured; it does not claim max_in_flight 17 gives the 'from 1 to 16' text, and lines respect the doc wrap width.
CHANGELOG.md has an entry for the ceiling and group-size record, and an entry under Fixed for the drain help text now following the ceiling constant.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
