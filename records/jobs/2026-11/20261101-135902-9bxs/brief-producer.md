# Job 20261101-135902-9bxs: Judge PRs by required checks; let held-job records pass head check

## Request (verbatim)

> Two problems stop drain landings from counting, both seen on the disk fix's pull request.
> 1. The drain reports a landed job as 'not landed: pull request is Open, checks Red' when the only red check is one GitHub does not require (the pull request head check, marked not yet required in .github/workflows/ci.yml), and then stops the run for a repeated failure. Judge the pull request by the checks main requires, as recorded for the required-check names; a red optional check is a note, not a failure.
> 2. The pull request head check (crates/athena-gate/src/invariant.rs, 'adds the records of job ... without its verification') refuses the records a job writes when it is held for low disk, because a held job has no verification yet. A commit that only records a hold, or a held job's progress, must pass that check, as other records-only commits do.
> - A drain test: a pull request whose required check is green and an optional check is red is reported landed or waiting, never failed, and the run does not stop.
> - An invariant test: a commit adding a held job's records with no verification passes the pull request head check; a commit adding a finished job's records with no verification still fails.
> - CHANGELOG.md gets an entry.
> Priority: high

## Reading (accepted by the user)

Fix two blockers for drain landings, for the maintainer running the drain. (1) The drain judges a pull request only by the required check ("fmt, clippy, tests"), so a red optional check such as the pull request head check is a note, not a failure. (2) The pre-merge classification no longer returns NoVerification for a held job's records, while a finished job's closing records without verification still fail.

Assumptions:
- Code survey: read crates/athena-core/src/gh/client.rs (PrHost::checks at ~405 calls classify_checks at ~766, which turns any failed check-run into Checks::Red and has no notion of required names; CI_AGGREGATE_JOB = "fmt, clippy, tests" at ~516), crates/athena-core/src/gh/early.rs (EARLY_RUN_JOB), crates/athena-core/src/gh/landing.rs (~169-340, the landing wait and the 'checks red' standing text), and crates/athena-gate/src/invariant.rs (closing_jobs ~234, Uncovered::NoVerification ~278, classify_pre_merge ~311, which returns NoVerification when a job's closing records lack verification.json). The entry points are the landing wait in gh/landing.rs and classify_pre_merge; the producer must locate how the drain maps Red to 'not landed' and the repeated-failure stop.
- The required-check name set is what main requires: the single job named 'fmt, clippy, tests' (see xtask/tests/ci_shards.rs REQUIRED_CHECK_NAME). The producer should reuse that existing constant or recorded name instead of adding a second list; optional checks (the pull request head, attribution of main) are ignored when judging red, and appear only as a note.
- A 'held job' is one whose state is a low-disk hold (crates/athena-core/src/disk.rs, state.rs). The invariant fix must identify it from the job's own records in the commit tree (for example a hold record present and no outcome or candidate record), so a finished job (candidate or outcome records) without verification still yields NoVerification.
- No record or data file is added under crates/, so the records validation allow-list (check_record_file, AnyRecord round-trip) is not touched, unless the producer finds the hold marker is a new record kind, which this reading does not expect.
- The change adds no new refusal; it narrows two existing ones, so the replay refusal rate should fall. Expected 0% of newly refused readings.
- Governance: .github/workflows/ci.yml, athena.toml and defaults.toml need no change; the 'not yet required' marking stays as is.

## Acceptance criteria

1. [behavior] A drain/gh test (cargo test -p athena-core) where the required check 'fmt, clippy, tests' is green and an optional check (the pull request head) is red reports landed or waiting, never failed, and the run does not stop for a repeated failure.
2. [behavior] A drain test (cargo test -p athena-core) shows that a red required check is still reported as failed, and a pending required check stays waiting, so the behavior is not weakened.
3. [behavior] An invariant test (cargo test -p athena-gate) shows a commit adding a held job's records with no verification passes classify_pre_merge (no Uncovered::NoVerification), and a commit adding a finished job's closing records with no verification still returns NoVerification.
4. [content] The red-optional-check case is surfaced as a note in the drain or landing output (visible in the test's assertion on the report), not as the 'not landed: pull request is Open, checks Red' error.
5. [content] CHANGELOG.md has an entry that describes both fixes (required-checks-only judgment, held-job records pass the pull request head check).
6. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check pass for the workspace.
7. [content] The required check names are taken from one existing source (the 'fmt, clippy, tests' constant in the code), and not duplicated as an unrelated second list; verified by Grep of the diff.
8. [content] Expected 0% of newly refused readings: the change only narrows refusals, so the higher refusal rate is not intended.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Making the pull request head check required, or editing .github/workflows/ci.yml
- Changing the disk-hold logic itself or the low-disk threshold
- Changes to the drain's thrash-stop rule for genuine repeated failures
- Changes to records/ or branch-protection settings on GitHub

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new drain and invariant tests and passing cargo test, clippy, fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check outcomes (landed or waiting report, pass or fail of the head check) through named tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Judge by required checks generally, and classify held jobs by their records, not one instance.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Both rules are enforced in code and tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change covering the two stated problems.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 38 cases in 38 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict step in code. Before the verifier runs, Athena runs the repo gate (fmt, clippy, test) itself and records the exact commands and exit codes. It also checks that every behavior criterion has a demonstration that names a command in that record. If the gate fails or a criterion has no ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
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

A drain/gh test (cargo test -p athena-core) where the required check 'fmt, clippy, tests' is green and an optional check (the pull request head) is red reports landed or waiting, never failed, and the run does not stop for a repeated failure.
A drain test (cargo test -p athena-core) shows that a red required check is still reported as failed, and a pending required check stays waiting, so the behavior is not weakened.
An invariant test (cargo test -p athena-gate) shows a commit adding a held job's records with no verification passes classify_pre_merge (no Uncovered::NoVerification), and a commit adding a finished job's closing records with no verification still returns NoVerification.
The red-optional-check case is surfaced as a note in the drain or landing output (visible in the test's assertion on the report), not as the 'not landed: pull request is Open, checks Red' error.
CHANGELOG.md has an entry that describes both fixes (required-checks-only judgment, held-job records pass the pull request head check).
cargo test, cargo clippy --all-targets and cargo fmt --check pass for the workspace.
The required check names are taken from one existing source (the 'fmt, clippy, tests' constant in the code), and not duplicated as an unrelated second list; verified by Grep of the diff.
Expected 0% of newly refused readings: the change only narrows refusals, so the higher refusal rate is not intended.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
