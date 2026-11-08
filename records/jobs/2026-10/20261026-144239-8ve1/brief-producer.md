# Job 20261026-144239-8ve1: Split CLI run function into per-command handlers in main.rs

## Request (verbatim)

> Structural pass: crates/athena-cli/src/main.rs
> 
> Code health found crates/athena-cli/src/main.rs over a limit: function run is 549 lines, over the limit of 200.
> 
> Measured: 1290 production lines (test modules excluded); longest function run at 549 lines; 9 landings in the last 7 days; 24 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function run is 549 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/main.rs

## Reading (accepted by the user)

Refactor crates/athena-cli/src/main.rs so no function exceeds 200 lines (run is 549) and the file stays under its production-line limit, by moving each large Command arm of run into its own handler function (or a new sibling module). Behavior is unchanged. The audience is code health and the maintainers.

Assumptions:
- Code survey: read crates/athena-cli/src/main.rs; run (line 1016 to about 1697) is one match over Command with inline arms (Ask, Retry, Propose, ScoreJev, Review, AcceptProposal, RejectProposal, ApproveWork, and more), each calling open(..), an Athena method, then print_json or display::*. Helpers sit above it (lines 572-1014). Sibling modules exist: drain.rs, display.rs, wiring.rs, hooks.rs, decisions.rs in crates/athena-cli/src/.
- Approach: extract each long arm into a private fn taking the destructured fields and returning Result<u8>; keep run as a thin dispatcher. Group arms into a sibling module only if that is needed to get the file under the 1500-line landing-frequency limit. Do not change CLI output, exit codes, or clap definitions.
- No records or data files are added, so the records allow-list is unaffected. No refusal rule is added or changed.
- Governance files (athena.toml, defaults.toml, clippy config) are not touched.
- The 1290 production lines are already under 1500 and 2000; the breach is the per-function limit, so run must drop to 200 lines or fewer, and so must every new function.

## Acceptance criteria

1. [content] In crates/athena-cli/src/main.rs (and any new module), no function exceeds 200 lines; run is a dispatcher of roughly 200 lines or fewer. Check by reading the file.
2. [content] The diff is a pure restructuring: the Command enum, clap definitions, message strings and exit codes are unchanged, and the existing tests in main.rs and crates/athena-cli/tests are not modified or removed.
3. [behavior] Run `cargo test -p athena-cli`. All existing tests pass, including tests/binary.rs and tests/proposals.rs, which exercise CLI output.
4. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`. Both are clean.
5. [behavior] Run `cargo run -p athena-cli -- --help`. The output matches the pre-change output, so the subcommand list is unchanged.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/commands.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior, CLI surface, or output change
- Changes to other files over limits
- Changes to athena.toml, defaults.toml or code-health limits

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused refactor of a single file
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The long function is split by command, not trimmed by cosmetic edits
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test suite and --help rather than checking that functions exist
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Refactor stays in Rust
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

- path: <state>/jobs/20261026-144239-8ve1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144239-8ve1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In crates/athena-cli/src/main.rs (and any new module), no function exceeds 200 lines; run is a dispatcher of roughly 200 lines or fewer. Check by reading the file.
The diff is a pure restructuring: the Command enum, clap definitions, message strings and exit codes are unchanged, and the existing tests in main.rs and crates/athena-cli/tests are not modified or removed.
Run `cargo test -p athena-cli`. All existing tests pass, including tests/binary.rs and tests/proposals.rs, which exercise CLI output.
Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`. Both are clean.
Run `cargo run -p athena-cli -- --help`. The output matches the pre-change output, so the subcommand list is unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
