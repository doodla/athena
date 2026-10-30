# Job 20261030-060812-etgx: Hold jobs on low disk before checks instead of failing them

## Request (verbatim)

> Priority: normal
> 
> The free-disk guard that runs before the checks still ends the job as failed and needs a manual retry. Disk ran out three times, Oct 23 to 26, each needing hand cleanup
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261027-082518-jmr6 (key low-disk-guard-before-checks-ends-job-failed-needs-manual-retry), queued at 2026-10-27T08:25:18.472254844Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> job 20261025-033738-yz2d
> Request:
> ```
> Run athena propose automatically (BRIEF.md addendum of 2026-10-25): it turns a failure class seen across enough jobs into a defaults proposal, but it only runs by hand, nobody runs it, and no proposal exists although classes have recurred 5 to 27 times. Run it after each introspection is recorded (and at the start of athena drain), so proposals appear in athena status, and within the steward's limit are worked like other approved items.
> ```
> Read as: Run defaults proposals automatically after introspection and at drain (code). Make `Athena::propose` run without a separate command: after each introspection is recorded, and once at the start of `athena drain`, so pending proposals show in `athena status` and in-limit ones are approved and worked by drain's existing steward path. Audience: the maintainer who reads status and the verifier running cargo test.
> Outcome: failed. environment: low disk: 1.5 GB free, 10.0 GB required before the checks; this is the environment, not the work. Free disk space, then `athena retry 20261025-033738-yz2d`
> 
> 
> ## Evidence
> 
> ### 1. job 20261025-033738-yz2d
> Request:
> ```
> Run athena propose automatically (BRIEF.md addendum of 2026-10-25): it turns a failure class seen across enough jobs into a defaults proposal, but it only runs by hand, nobody runs it, and no proposal exists although classes have recurred 5 to 27 times. Run it after each introspection is recorded (and at the start of athena drain), so proposals appear in athena status, and within the steward's limit are worked like other approved items.
> ```
> Read as: Run defaults proposals automatically after introspection and at drain (code). Make `Athena::propose` run without a separate command: after each introspection is recorded, and once at the start of `athena drain`, so pending proposals show in `athena status` and in-limit ones are approved and worked by drain's existing steward path. Audience: the maintainer who reads status and the verifier running cargo test.
> Outcome: failed. environment: low disk: 1.5 GB free, 10.0 GB required before the checks; this is the environment, not the work. Free disk space, then `athena retry 20261025-033738-yz2d`
> 
> 
> ### 2. path records/introspections/2026-10/20261026-140204-9xvr.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261026-140204-9xvr",
>   "at": "2026-10-26T14:02:04.452956473Z",
>   "trigger": "failure",
>   "job": "20261025-033738-yz2d",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "failed",
>   "what_happened": "Job 20261025-033738-yz2d (\"Run defaults proposals automatically after introspection and at drain\") ended failed. The cause was the environment, not the work. Before the checks ran, the disk had 1.5 GB free and 10.0 GB was required. The message tells the the person running Athena to free disk space and run `athena retry 20261025-033738-yz2d`. The event lists no unmet acceptance criteria, no blocker or major findings, and no failed checks. The work was never judged.",
>   "failure_class": "20261026-140223-4xmw",
>   "new_class": true,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "Athena measures free disk only at the point just before the checks, after the producer has already run. It owns the space that fills up (job working copies, build output, state directories) but has no routine that reclaims it, such as clearing finished jobs' build output. Nothing measures or limits disk use across the drain either. The guard turns exhaustion into a failed job that needs a manual retry, not a held or deferred job that resumes once space is back. Lost work and the person running Athena effort are therefore a normal result of a long drain, not a rare event.",
>   "prevention_kind": "design-change",
>   "prevention": "Check free disk before dispatch, not only before the checks. If it is below the minimum, first run an automatic cleanup of space Athena owns: build output and working copies of finished, landed or closed jobs, and caches past a set age. If space is still short, put the job and the drain on hold in a resumable state, not \"failed\", and show the hold in status. The drain would then continue by itself on the next pass once space returns. A job stopped this way must never count as a work failure. It must not need a manual `athena retry`, and it must never lead to a verifier or producer verdict.",
>   "proposed_request": "Make Athena handle low disk without an the person running Athena. Outcome: a drain never ends a job as failed because the disk is full, and it frees its own space first.\n\nAcceptance:\n- Before dispatching any job, Athena checks free disk against the configured minimum (default 10 GB). The check does not wait until just before the checks.\n- If free space is below the minimum, Athena first deletes space it owns: build output and working copies of jobs that are finished, landed or closed, oldest first. It never touches a running job or any committed record.\n- If space is still below the minimum after cleanup, the job stays queued or held in a resumable state with a clear reason shown in `athena status`. It is not marked failed. The drain then resumes the job by itself on a later pass once space is back, with no `athena retry`.\n- A demonstration shows (a) cleanup freeing enough space so the job runs, (b) space that stays short producing a hold and not a failure, and (c) the held job resuming and finishing after space is added. Each must fail without the new behavior.\n- An environment hold never appears as a work failure in introspection or in the failure counts.\n- Update the changelog and docs as the repo defaults require. Then retry job 20261025-033738-yz2d (\"Run defaults proposals automatically after introspection and at drain\") on the fixed build.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.081256,
>     "input_tokens": 2,
>     "output_tokens": 1866
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 3. path records/failure-classes/2026-10/20261026-140223-4xmw.json
> ```
> {
>   "schema": "athena.failure-class/1",
>   "id": "20261026-140223-4xmw",
>   "at": "2026-10-26T14:02:23.462766213Z",
>   "name": "low-disk-ends-job-failed-no-automatic-cleanup",
>   "description": "The free-disk guard fires before the checks and the job ends failed with an environment message, and Athena frees no space itself, so an the person running Athena must clear disk by hand and retry.",
>   "status": "proposed",
>   "proposed_by": "20261026-140204-9xvr"
> }
> ```
> 
> ### 4. path records/interventions/2026-10/20261026-140252-pr5w.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261026-140252-pr5w",
>   "at": "2026-10-26T14:02:52.050469143Z",
>   "what": "Mistake: ran 6239 and yz2d in parallel on a disk allowance that fits about one job's build; yz2d failed before its checks on low disk (1.5 GB free) and 6239's baseline test failed, costing a retry and about 40 minutes.",
>   "why": "The plan's 'could run in parallel' groups ignore disk; nothing checks free disk against the combined build size of jobs started together, only per job before checks.",
>   "class_of_work": "other",
>   "category": "mistake",
>   "job": "20261025-033738-yz2d",
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 5. path records/interventions/2026-10/20261025-214444-69eb.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261025-214444-69eb",
>   "at": "2026-10-25T21:44:44.03762937Z",
>   "what": "Disk filled again (100%, 2026-10-25 21:41) while job 20261025-202447-xgk9 wrote its verdict: that job's state held a 16 GB target directory and the repository's own target/debug held 12 GB from the brain's timing runs; the verifier's failing verdict was lost. Freed 28 GB (the repository's target/debug, the job's target, two stale scratch worktrees) and made the disk-space guard request (qfax) urgent.",
>   "why": "One job's build can need about 16 GB while the session's allowance is about 39 GB; without a guard, any other build fills it.",
>   "class_of_work": "build",
>   "category": "fixed-environment",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 6. path records/interventions/2026-10/20261023-183217-nth6.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261023-183217-nth6",
>   "at": "2026-10-23T18:32:17.804117243Z",
>   "what": "Mistake: did not watch disk; two jobs' 11-12 GB build caches filled the allowance and job zvmr died mid-commit.",
>   "why": "Athena should delete a job's build cache when it closes and check free disk before dispatch.",
>   "class_of_work": "build",
>   "category": "other",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 7. path GIT-LOG.txt
> This path is not in the repository at the current commit.
> 
> Acceptance:
> - A test shows an introspection that brings a class to the threshold produces a pending proposal without a separate command.
> - A class already proposed or rejected is not proposed again (existing rule kept).
> - athena status lists the new proposal.
> - A test shows an introspection that brings a class to the threshold produces a pending proposal without a separate command.
> - A class already proposed or rejected is not proposed again (existing rule kept).
> - athena status lists the new proposal.
> 
> Files: crates/athena-core/src/introspect.rs, crates/athena-core/src/propose.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

The free-disk guard before the checks (and before the verifier) closes the job failed with `environment: low disk` and needs `athena retry`. Change it so that low space after cleanup leaves the job held and resumable from its built candidate, shown in `athena status`. A later drain pass resumes it by itself once space is back, with no retry and no work-failure count. This follows up job 20261025-033738-yz2d (which hit this guard) and job 20261025-202447-xgk9 (an earlier disk-full loss, closed as superseded). The audience is the maintainer running drains and the verifier running cargo test.

Assumptions:
- Code survey: crates/athena-core/src/disk.rs already holds a job at dispatch (gate, sweep_idle_targets, write_hold, LowDisk) and does idle-build cleanup. crates/athena-core/src/run/mod.rs check_and_verify (about line 1180) and crates/athena-core/src/run/verification.rs (about line 130) still close the job failed with an `environment: low disk` reason before the checks and before the verifier. crates/athena-core/src/disk.rs resumable_stop and is_environment_reason, plus ResumeMark in crates/athena-core/src/state.rs, mark which failed jobs `athena retry` resumes. crates/athena-cli/src/drain.rs gives each item two attempts and treats a dispatch hold (`run.held`) as not a failure.
- Reading of the work: the two late guards keep the built candidate and write the resume mark, then end the job in a held state that is not Failed. That state shows the reason in `athena status`, never counts as a work failure in introspection or failure classes, and does not use up a drain attempt. A later drain pass re-checks disk and resumes from the candidate with no new producer run. The existing dispatch hold, the cleanup sweep and the `athena retry` path for old failed environment jobs stay.
- No new record schema or data file under crates/ is planned. If the producer adds one, it must be covered by check_record_file, AnyRecord and the round-trip test.
- Job 20261025-033738-yz2d (automatic proposing) is not re-done here. Its candidate's 8 files are not part of this change and are excluded below; that work is neither kept nor dropped by this change. Job 20261025-202447-xgk9 was closed as superseded and had no candidate, so nothing carries over from it.
- Disk is simulated in tests with the existing ATHENA_FREE_BYTES hook (disk.rs FREE_BYTES_ENV) and with_min_free_gb in crates/athena-cli/tests/common/mod.rs. This adds no refusal rule on readings, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held and not Failed: no `athena retry` is needed, no verifier or producer verdict is recorded, and the outcome text names `environment: low disk` with the free and required amounts.
2. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows a later drain pass, once the simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
3. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows space that stays short across passes leaves the job held and uses up no drain attempt. Each of these tests fails on the pre-change code.
4. [behavior] Run `cargo test -p athena-core`. A test shows a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection or failure classes, and that the idle-build cleanup still runs first and lets the job proceed when it frees enough space.
5. [behavior] Run `cargo test -p athena-cli --test binary` or the status tests. `athena status` shows the held job with its low-disk reason and the amounts.
6. [content] Grep of crates/athena-core/src/run/mod.rs and crates/athena-core/src/run/verification.rs shows the low-disk path no longer calls `self.failure(` with a `before the checks` or `before the verifier` retry message.
7. [content] CHANGELOG.md has an entry for the hold-and-auto-resume behavior, and the module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe the new behavior instead of the old fail-and-retry.
8. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. All pass (the standing `tested` default).
9. [content] Governance files are unchanged: athena.toml, defaults.toml, workflows, and the Cargo and toolchain files are not in the diff.

## Expected deliverables

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Automatic proposing after introspection (the work of job 20261025-033738-yz2d): its 8 changed files (CHANGELOG.md aside) are not part of this change. These are crates/athena-cli/src/main.rs, crates/athena-cli/tests/introspection.rs, crates/athena-cli/tests/proposals.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/proposals.rs, and its edits to drain.rs and tests/drain.rs. That job is rerun separately.
- The test-suite speedup and the other work of job 20261025-202447-xgk9 (its docs/decisions.md, secrets.rs, checks.rs and test-file scope)
- Guarding the combined disk use of jobs started in parallel, or planning around disk
- Changing the [disk] min_free_gb default or config schema, or editing athena.toml, defaults.toml or workflows
- Deleting committed records or touching a running job's files in the cleanup
- Removing `athena retry` or its handling of old failed environment jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing tests plus clippy and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the job held, then resumed, with no retry, by running tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the late guards that still fail the job, rather than only retrying the one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The hold and resume behavior is enforced in code and tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the disk guard; the proposing work and other jobs are excluded.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a drain no longer ends a job failed on low disk and resumes it by itself.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

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

Run `cargo test -p athena-cli --test drain`. A new test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held and not Failed: no `athena retry` is needed, no verifier or producer verdict is recorded, and the outcome text names `environment: low disk` with the free and required amounts.
Run `cargo test -p athena-cli --test drain`. A new test shows a later drain pass, once the simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
Run `cargo test -p athena-cli --test drain`. A new test shows space that stays short across passes leaves the job held and uses up no drain attempt. Each of these tests fails on the pre-change code.
Run `cargo test -p athena-core`. A test shows a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection or failure classes, and that the idle-build cleanup still runs first and lets the job proceed when it frees enough space.
Run `cargo test -p athena-cli --test binary` or the status tests. `athena status` shows the held job with its low-disk reason and the amounts.
Grep of crates/athena-core/src/run/mod.rs and crates/athena-core/src/run/verification.rs shows the low-disk path no longer calls `self.failure(` with a `before the checks` or `before the verifier` retry message.
CHANGELOG.md has an entry for the hold-and-auto-resume behavior, and the module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe the new behavior instead of the old fail-and-retry.
Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. All pass (the standing `tested` default).
Governance files are unchanged: athena.toml, defaults.toml, workflows, and the Cargo and toolchain files are not in the diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
