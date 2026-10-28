# Job 20261027-054217-c896: Gate verifier start on free disk and a write probe, as an environment

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing checks that the verifier's environment can build and run tests before verification starts, a
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: verifier-environment-unusable-resource-exhausted (20261016-051427-3r97)
> Cause: Nothing checks that the verifier's environment can build and run tests before verification starts, and nothing separates an environment failure from a work failure. Cargo builds fill disk, and artifacts from earlier jobs and verifier snapsh...
> Seen 3 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261016-051415-3nbe (job 20261016-044847-hem1); prevention it proposed: Add a preflight code check before verification. It would require a minimum amount of free disk space in the verifier workspace and a trivial build or test command that succeeds. If the check fails, Athena would clean up build artifacts (suc...
> Introspection 20261016-053016-cr33 (job 20261016-044847-hem1); prevention it proposed: Two code-enforced steps. (1) When a job reaches a terminal state (merged, closed or rejected and not retryable), delete its regenerable build directories such as cargo target/. A periodic sweep should also remove these directories from idle...
> Introspection 20261026-074919-kcfs (job 20261026-064950-38jx); prevention it proposed: Add a code check before the check and verify stages. It would measure free disk against the same threshold the test guard uses (10 GB) and clear build directories of closed and idle jobs first. If space is still short, the job would be held...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the disk gate runs only at dispatch and before the checks, but the checks' cargo builds fill the disk afterwards, so the verifier (jobs 20261016-044847-hem1 and 20261026-064950-38jx) starts in an unusable environment and its failure reads as a work failure. Fix it in athena-core by running the same `disk_gate` (idle-target sweep first) plus a write probe in the verifier workspace immediately before each verifier run, failing with the retryable `environment: low disk` reason and no work verdict.

Assumptions:
- Audience: the maintainer reading `athena status` and retry logs, and the verifier who must be able to tell an environment failure from a work failure.
- Existing state (read): disk.rs enforces `[disk] min_free_gb` at dispatch (run/mod.rs ~384) and before the checks (~1975); nothing guards the verifier agent start (~2186) after the checks have built.
- Job 20261016-044847-hem1: its blocker (verifier disk full, `No space left on device`) is the class addressed by the new pre-verifier gate; its plan.rs/reading.rs/drain findings (unmet criterion 7, steward free-text refusals, broad rung word list) are unrelated to this cause and out of scope here, not re-fixed.
- Job 20261026-064950-38jx: its failing `a_nested_run_works_in_a_scratch_copy_with_fake_agents` hit the 10 GB guard on a 9 GB host; the current tree already has test_config set min_free_gb = 0 and a control test (binary.rs 143-182), so that finding no longer applies; the code_health findings (extra test file, baseline file, header width) are unrelated and out of scope.
- The write probe is a small file written and removed in the verifier workspace; a cargo build probe is not added because the gate must stay language-neutral and cheap.
- No athena.toml or defaults.toml change: the existing `[disk] min_free_gb` threshold is reused.

## Acceptance criteria

1. [content] The change description and the code comment/docs (disk.rs module docs, docs/decisions.md entry) name the root cause: the disk guard did not cover the point between the checks and the verifier, and an environment failure there was not separated from a work failure.
2. [content] run/mod.rs calls the shared disk gate (with idle-target sweep) and a workspace write probe immediately before the verifier agent runs, and on failure closes the job with an `environment: low disk` reason, no verification verdict, and the `athena retry` hint, reusing the existing LowDisk text.
3. [behavior] A new test simulating low free space only after the checks pass (e.g. via ATHENA_FREE_BYTES or a gate seam) shows the job fails with `environment: low disk` before the verifier agent is invoked and records no verifier verdict; run `cargo test --locked` and see that test fail on the base tree and pass on the change.
4. [behavior] A unit test in disk.rs shows the write probe fails on an unwritable/full workspace and succeeds on a normal one, and that the failure is classified as environment, not work, for any job (not one fixed job id); run `cargo test --locked -p athena-core disk`.
5. [behavior] With sufficient simulated free space the existing drain and binary tests still pass unchanged; run `cargo test --locked`.
6. [behavior] `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` all pass.
7. [content] CHANGELOG.md has an entry for the pre-verifier environment check.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Fixing hem1's plan.rs/reading.rs/drain.rs planning findings (unmet criterion 7, steward free-text refusals, rung word list)
- Fixing 38jx's code_health findings (extra test file, baseline file location, table header width)
- Adding athena.toml fields or changing the 10 GB default
- Periodic sweeps of terminal-state jobs beyond the existing idle-target sweep
- Running a cargo build as the probe
- Editing records/ or owner files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the gap between the check stage and the verifier, covering every job, not one instance.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and cargo test pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code via the shared disk gate, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check the early environment failure happening.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a full disk before verification yields a retryable environment failure rather than a work failure.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change reusing disk_gate and the existing threshold.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entry written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
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

16 more qualifying classes are left out.

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

The change description and the code comment/docs (disk.rs module docs, docs/decisions.md entry) name the root cause: the disk guard did not cover the point between the checks and the verifier, and an environment failure there was not separated from a work failure.
run/mod.rs calls the shared disk gate (with idle-target sweep) and a workspace write probe immediately before the verifier agent runs, and on failure closes the job with an `environment: low disk` reason, no verification verdict, and the `athena retry` hint, reusing the existing LowDisk text.
A new test simulating low free space only after the checks pass (e.g. via ATHENA_FREE_BYTES or a gate seam) shows the job fails with `environment: low disk` before the verifier agent is invoked and records no verifier verdict; run `cargo test --locked` and see that test fail on the base tree and pass on the change.
A unit test in disk.rs shows the write probe fails on an unwritable/full workspace and succeeds on a normal one, and that the failure is classified as environment, not work, for any job (not one fixed job id); run `cargo test --locked -p athena-core disk`.
With sufficient simulated free space the existing drain and binary tests still pass unchanged; run `cargo test --locked`.
`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` all pass.
CHANGELOG.md has an entry for the pre-verifier environment check.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
