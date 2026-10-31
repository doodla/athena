# Job 20261030-060759-7x7m: Recompute scope-guard pauses and classify auth stops as environment

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped,
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: scope-guard-counts-environment-stop-as-line-spend-the person running Athena-lifts-pause (20261028-065858-bztk)
> Cause: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped, and it does not know how much work is left. A stop caused by the environment (low disk) looks the same as a stop caused by the work being t...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261028-065847-kmx7 (job 20261028-055601-hg0n); prevention it proposed: Record a stop reason on every attempt. Have the scope guard exclude attempts that ended on an environment stop (low disk, container reclaim) from the spend and attempt counts, or count them separately. When an attempt has a kept candidate t...
> Introspection 20261028-103312-b53e (job 20261028-065911-m6m5); prevention it proposed: Make the scope guard record the stop reason for each attempt and the state of the kept candidate. Spend from an environment stop, such as low disk, must not count toward the line's progress-less spend. If a candidate is kept and has passed ...
> Introspection 20261028-124847-gf2n (job 20261028-103330-3d47); prevention it proposed: Compute the pause instead of storing it. On each guard evaluation (and whenever the guard's rule changes), recompute the line's spend from the per-attempt stop reasons under the current rule, and clear or keep the pause on the result. If a ...
> Introspection 20261029-195138-3pqb (job 20261029-111202-2bpc); prevention it proposed: Record a stop reason with each attempt's spend (environment, mechanical plan finding, real critic or verifier failure, producer failure). The guard should count only the reasons that signal runaway work. Store with each pause the rule versi...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the guard stores a pause as a file and classifies a stop only by the closing-reason prefix. A pause therefore outlives rule changes or environment-only spend, and auth/usage stops (401, 20261028-103330-3d47) count as work failures. Fix: derive the pause from per-attempt stop reasons on every check, and classify auth/usage stops as environment. Jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47 are addressed as described in the assumptions.

Assumptions:
- Code survey: crates/athena-core/src/scope_guard.rs (attempt_of classifies Verdict::Environment only by ENVIRONMENT_PREFIX on the closing reason; try_watch_line writes the pause file and, when no signal fires, removes only the flag file; line_pause reads the stored pause file; t3_basis and counted already exclude environment stops), crates/athena-core/src/disk.rs (is_environment_reason accepts only LOW_DISK and CHECK_TIMEOUT; resumable_stop), crates/athena-core/src/run/mod.rs line 680 and run/producer.rs line 206 (the 'the producer did not complete' closing reasons with no environment prefix; run/mod.rs line 695 refuses a paused line). Entry points are Athena::go and Athena::retry via line_pause. Tests are the unit tests in scope_guard.rs, measured directly on AttemptRecord/request_records output and on line_pause results. No record or data file is added, so check_record_file and AnyRecord are not touched.
- Goal and audience: the person running Athena must not see a line paused because of spend caused by the environment (low disk, expired login, usage limit). Audience is the owner who lifts pauses and the verifier running cargo test.
- Job 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47: their replay-feature candidates (check-reading, athena.replay/1) are a different task whose jobs were stopped by the environment. That work is dropped here: the files they changed (CHANGELOG.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/store.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs, docs/records.md) are excluded in out_of_scope, except run/mod.rs and CHANGELOG.md, which this change may touch. Their low-disk and 401 findings are the failure class fixed here, not re-run. Job 20261028-103330-3d47 had no candidate.
- The existing environment-stop handling for low disk is kept; the change extends the classification and makes pauses derived, not stored. A pause file whose recomputation no longer fires is removed or ignored.
- Only changed behaviour is the guard; no reading or refusal rule for readings is added, so the replay-percent criterion does not apply.

## Acceptance criteria

1. [content] The change description (CHANGELOG.md entry) names the root cause: the guard stores a pause and classifies attempts only by a closing-reason prefix, so environment stops and stale pauses count as line spend; and it cites the jobs 20261028-055601-hg0n, 20261028-065911-m6m5, 20261028-103330-3d47.
2. [content] In crates/athena-core/src/scope_guard.rs or disk.rs, a single classification function maps a closing reason (low disk, check timeout, expired login/401, usage limit) to Verdict::Environment; Grep shows attempt_of uses it and the producer-failure reasons in crates/athena-core/src/run/producer.rs and run/mod.rs carry the environment prefix for these causes.
3. [behavior] Run `cargo test -p athena-core scope_guard`: a new test builds a line whose attempts all ended on environment stops (low disk, and a 401 auth stop) with spend above the T3 pause ratio, and asserts no pause; the same test fails on the base commit.
4. [behavior] Run `cargo test -p athena-core scope_guard`: a new test writes a stale pause file for a line whose recomputed signals are none (or only environment stops) and asserts line_pause returns None and the stored pause is cleared, while a line with real runaway spend or two same-criterion rejections stays paused.
5. [behavior] Run `cargo test`: the whole workspace test suite passes, including the existing scope_guard tests.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## Out of scope

- The reading replay feature (athena check-reading, athena.replay/1 record, replay hold) from jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47; their changes to crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/store.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs and docs/records.md are dropped
- Changing athena.toml, defaults.toml or workflows
- Changing the T1 to T5 thresholds
- Automatic disk cleanup or reservation, and token refresh
- Lifting existing pauses by hand

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix classification and pause derivation in the guard, not one job's pause.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail on base and pass with the change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that environment-only spend no longer pauses a line.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the scope guard and the stop-reason classification.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 7 cases in 7 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Land b5hm first and extend it. Before the checks stage, and again when free disk drops below the guard, Athena should delete regenerable artifacts itself: the verify-side build folder of finished baseline checks, the workspace target, and the uv and npm caches. If that is still not enough, the job s...

22 more qualifying classes are left out.

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

The change description (CHANGELOG.md entry) names the root cause: the guard stores a pause and classifies attempts only by a closing-reason prefix, so environment stops and stale pauses count as line spend; and it cites the jobs 20261028-055601-hg0n, 20261028-065911-m6m5, 20261028-103330-3d47.
In crates/athena-core/src/scope_guard.rs or disk.rs, a single classification function maps a closing reason (low disk, check timeout, expired login/401, usage limit) to Verdict::Environment; Grep shows attempt_of uses it and the producer-failure reasons in crates/athena-core/src/run/producer.rs and run/mod.rs carry the environment prefix for these causes.
Run `cargo test -p athena-core scope_guard`: a new test builds a line whose attempts all ended on environment stops (low disk, and a 401 auth stop) with spend above the T3 pause ratio, and asserts no pause; the same test fails on the base commit.
Run `cargo test -p athena-core scope_guard`: a new test writes a stale pause file for a line whose recomputed signals are none (or only environment stops) and asserts line_pause returns None and the stored pause is cleared, while a line with real runaway spend or two same-criterion rejections stays paused.
Run `cargo test`: the whole workspace test suite passes, including the existing scope_guard tests.
Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
