# Job 20261106-042847-ab51: Raise drain max_concurrency ceiling from 8 to 16

## Request (verbatim)

> try again on 20261106-022423-vdtn

## Reading (accepted by the user)

Retry of job 20261106-022423-vdtn: raise MAX_CONCURRENCY_CEILING in athena-data config from 8 to 16 so max_concurrency accepts 1 to 16 and 17 is refused with a 1-to-16 message, updating help (derived from the constant), docs and CHANGELOG. The prior xtask test failures looked unrelated (CI claude --version); this attempt runs full cargo test and fixes anything the ceiling change breaks. Audience: the operator running athena drain and reviewers.

Assumptions:
- Code survey: crates/athena-data/src/config.rs defines MAX_CONCURRENCY_CEILING (line 269, =8), used by the [parallel] max_concurrency check (~722), the error attribute (~588), and the max_in_flight check (~745); MAX_CHECK_SLOTS_CEILING (line 91) is a separate constant and stays 8. crates/athena-cli/src/main.rs (~553-563) fills the drain --help ceiling from the constant, with a test at ~1853. No record or data file is added, so check_record_file/AnyRecord are not involved. The ceiling is measured directly by Config::parse in tests.
- Job 20261106-022423-vdtn: its candidate changed CHANGELOG.md, athena.toml, config.rs, and the three design docs; all are kept as work to redo from base. Its findings are addressed: (a) CHANGELOG nit: the entry goes under 'Changed', not 'Added'; (b) the xtask repin test failure (claude --version failed in CI, xtask untouched) is environmental and unrelated, so xtask is not changed; the producer runs the full cargo test and reports any test the ceiling change genuinely breaks; (c) the test check failure is otherwise unreadable, so no further cause is assumed.
- max_in_flight shares MAX_CONCURRENCY_CEILING, so its range also becomes 1 to 16; docs stating 'max_in_flight outside 1 to 8' (async-drain.md ~264) are updated, but max_check_slots stays 1 to 8.
- Prior candidate's athena.toml change was only the range comment (line 192); value max_concurrency = 2 stays. No README.md or crates/athena-cli/src/main.rs edit is expected since help is derived from the constant.
- Changing Athena to store failing test names from test output (the introspection's proposal) is a separate request and is not done here.
- Historical CHANGELOG entries and docs/decisions.md records that say 1 to 8 are history and stay.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-data: the range test shows max_concurrency = 16 loads and 17 is refused with an error containing '1 to 16'; all pass.
2. [behavior] Run cargo test -p athena-cli: help test passes and states the ceiling as 16 (stated via the constant); cargo run -- drain --help shows 16 and not 8.
3. [behavior] Run cargo test --locked from the repo root, cargo clippy --all-targets and cargo fmt --check: all clean; any test newly failing because of the ceiling is fixed in this diff, and xtask is untouched unless a test genuinely depends on the ceiling.
4. [content] Grep of docs/design/async-drain.md, docs/design/parallel-jobs.md and athena.toml finds no remaining 'max_concurrency'/max_in_flight range stated as 1 to 8; they say 1 to 16. max_check_slots keeps 1 to 8.
5. [content] athena.toml diff changes only the comment; max_concurrency = 2 is unchanged.
6. [content] CHANGELOG.md has one new entry under 'Changed' that states only what the diff does (ceiling 8 to 16, error and help text, docs) and does not mention plan group size recording; older entries are untouched.
7. [content] docs/design/agent-and-build-slots.md step 6 text reflects the ceiling now being 16 where it describes the built drain, without implementing other steps.
8. [content] No code or docs add plan group size recording.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- docs/design/async-drain.md (modify)
- docs/design/parallel-jobs.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Recording plan group sizes
- Changing max_concurrency = 2 in athena.toml
- Rewriting historical CHANGELOG entries or docs/decisions.md records
- Other build steps of agent-and-build-slots.md
- Changing MAX_CHECK_SLOTS_CEILING
- Changing Athena to store failing test names from test output
- Editing xtask code (the repin test failure was environmental)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust constant plus tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full cargo test, clippy and fmt required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test and drain --help.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check 16 loads and 17 refused, not just that the constant changed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the ceiling; no group-size recording.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Help and error derive from the constant; docs grep covers all restatements.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and docs written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

30 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-042847-ab51/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-042847-ab51/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test -p athena-data: the range test shows max_concurrency = 16 loads and 17 is refused with an error containing '1 to 16'; all pass.
Run cargo test -p athena-cli: help test passes and states the ceiling as 16 (stated via the constant); cargo run -- drain --help shows 16 and not 8.
Run cargo test --locked from the repo root, cargo clippy --all-targets and cargo fmt --check: all clean; any test newly failing because of the ceiling is fixed in this diff, and xtask is untouched unless a test genuinely depends on the ceiling.
Grep of docs/design/async-drain.md, docs/design/parallel-jobs.md and athena.toml finds no remaining 'max_concurrency'/max_in_flight range stated as 1 to 8; they say 1 to 16. max_check_slots keeps 1 to 8.
athena.toml diff changes only the comment; max_concurrency = 2 is unchanged.
CHANGELOG.md has one new entry under 'Changed' that states only what the diff does (ceiling 8 to 16, error and help text, docs) and does not mention plan group size recording; older entries are untouched.
docs/design/agent-and-build-slots.md step 6 text reflects the ceiling now being 16 where it describes the built drain, without implementing other steps.
No code or docs add plan group size recording.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
