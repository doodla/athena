# Job 20261104-153614-pzfw: Reuse main's newest green CI run with the base's fingerprint

## Request (verbatim)

> Baseline reuse never finds main's result. In crates/athena-core/src/run/checks.rs the default-branch path asks main's CI for a run on the base's own content commit, but that commit exists only on the working branch: main holds a different squash commit with the same governed content. So every job since the fix still re-ran the 20-minute local baseline ('main: no usable CI result for 609bd60... on main', 4 Nov 15:35 UTC, while main's tip fe1df4f had a green ci run). Change the default-branch path to take main's newest green CI run whose commit has the same governed-content fingerprint as the base, instead of looking up the base's own commit.
> 
> Acceptance:
> - a test where main's tip is a squash commit with the base's fingerprint and a green CI run, and the base is a different commit: the baseline is reused from that run
> - a test where main's tip has a different fingerprint: the baseline runs locally as before
> - the existing baseline-reuse tests still pass

## Reading (accepted by the user)

In `reused_baseline` (crates/athena-core/src/run/checks.rs), the default-branch path stops asking main's CI about the base's own content commit. It now takes main's tip (the local remote-tracking ref of the default branch) and uses its green CI run when that commit has the same governed-content fingerprint as the base. Otherwise the baseline runs locally. The audience is the job runner and maintainers, who need the 20-minute baseline skipped.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (reused_baseline: step 1 is the landed job's candidate run; step 2 is the default-branch push run looked up at content_commit(base) via green_run, after a fingerprint(base) check; baseline_stage/take_baseline falls back to a local run on Err), crates/athena-core/src/run/mod.rs (with_ci_source_on reads one run with no wait, via a test seam), crates/athena-core/src/reconcile.rs (local_default_branch reads origin/HEAD) and crates/athena-core/src/run/tests.rs (commit_landed_squash, ci_reuse_run_with, CiReplies::only_for, baseline_reads, and the three existing baseline_* tests around lines 2915-3100). No record or data file is added, so the records allow-list, check_record_file and AnyRecord are untouched.
- Main's tip is resolved from the local remote-tracking ref (origin/<default branch>) with no network fetch. The request says 'newest green CI run' on main, so the simplest reading is main's tip first, then a bounded walk back over its first-parent ancestors (a small cap) until a commit with the base's fingerprint is found. Each candidate goes through the existing green_run, so the finished, green and covers-fmt/clippy/test rules, and the no-wait one-read rule, all still apply.
- Fingerprints are compared with the existing fingerprint(git, commit, commit) function against the base's own fingerprint, as step 1 does. If the commit is not in the local object store, that case yields a reason string and falls back to a local run.
- The existing default-branch test (baseline_reused_from_the_default_branch_run_runs_nothing_locally) uses a main run on the commit under the records. It should keep passing, since that commit is main's tip with the same fingerprint, and may need a small setup adjustment so origin/main points at it.
- No workflow, athena.toml or defaults.toml change is needed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs has main's tip as a squash commit that is not the base's content commit but has the same governed-content fingerprint, with a green CI run on main for that tip. It asserts the baseline has reused_from set to the run address, all checks are reused and passed, baseline_ran is false, and the host read was for the squash SHA on branch main. Run `cargo test -p athena-core baseline_reused`.
2. [behavior] A new test has main's tip with a different governed-content fingerprint, even if it has a green run. It asserts the baseline runs locally (baseline_ran true, reused_from None) and the output line gives the fingerprint-mismatch reason. Run `cargo test -p athena-core baseline_runs_locally`.
3. [behavior] The existing baseline tests still pass unchanged in meaning: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_runs_locally_when_no_green_ci_result_exists, and the ci_reuse_* tests. Run `cargo test -p athena-core`.
4. [content] In crates/athena-core/src/run/checks.rs, the default-branch step of reused_baseline no longer passes the base's own content commit to green_run on the default branch. It passes main's commit chosen by matching fingerprint with the base's, and the doc comment of reused_baseline describes this.
5. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the change touches only the reuse path, so the landed-candidate path and the local fallback are unchanged.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)

## Out of scope

- Changing the landed-candidate path (step 1) of reused_baseline.
- Fetching or polling the host for new main commits, or waiting for an unfinished run.
- Changing CI workflow files, athena.toml or defaults.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the match and mismatch cases; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the baseline being reused or run locally, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The lookup key is changed from the working-branch commit to a fingerprint match on main, rather than patching one case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the default-branch step.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier runs.
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
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

27 more qualifying classes are left out.

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

A new test in crates/athena-core/src/run/tests.rs has main's tip as a squash commit that is not the base's content commit but has the same governed-content fingerprint, with a green CI run on main for that tip. It asserts the baseline has reused_from set to the run address, all checks are reused and passed, baseline_ran is false, and the host read was for the squash SHA on branch main. Run `cargo test -p athena-core baseline_reused`.
A new test has main's tip with a different governed-content fingerprint, even if it has a green run. It asserts the baseline runs locally (baseline_ran true, reused_from None) and the output line gives the fingerprint-mismatch reason. Run `cargo test -p athena-core baseline_runs_locally`.
The existing baseline tests still pass unchanged in meaning: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_runs_locally_when_no_green_ci_result_exists, and the ci_reuse_* tests. Run `cargo test -p athena-core`.
In crates/athena-core/src/run/checks.rs, the default-branch step of reused_baseline no longer passes the base's own content commit to green_run on the default branch. It passes main's commit chosen by matching fingerprint with the base's, and the doc comment of reused_baseline describes this.
`cargo clippy --all-targets` and `cargo fmt --check` are clean, and the change touches only the reuse path, so the landed-candidate path and the local fallback are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
