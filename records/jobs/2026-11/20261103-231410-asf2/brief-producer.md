# Job 20261103-231410-asf2: Run planned 'fails without the fix' demos and show them to the verifier

## Request (verbatim)

> Let the verifier see the 'fails without the fix' run. Many acceptance criteria require a recorded run showing the new test fails on the old code, but the verifier receives no producer report and may not change files, so it can never see that run. Two tries of 'don't count needs-accepting re-reads toward the job limit' (jobs 20261103-193428-df2m and 20261103-222737-zzsr) were rejected only on this, although the code was correct and all checks passed. Fix it in one place: after the producer finishes, Athena itself runs each planned 'fails without the fix' demonstration (apply the planned revert to a scratch copy of the candidate, run the named test, record the command, exit code and output tail) and puts that record in the verifier's brief under 'Checks Athena ran'. Acceptance: a test where a plan has one such demonstration and the verifier brief contains its recorded failing run; a test where the revert does not make the test fail and the record says so.

## Reading (accepted by the user)

After the producer finishes, Athena runs each planned demonstration that has a fails_without on a scratch copy of the candidate with the fix reverted, and records command, exit code and output tail under 'Checks Athena ran on this tree' in the verifier brief. Goal: stop correct changes (retries of jobs 20261103-193428-df2m and 20261103-222737-zzsr) failing because the verifier can never see the failing run; audience is the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/run/verification.rs (verification_stage builds VerifierInputs.checks from check_runs plus extra 'not a check' entries, then verifier_brief_with_plan), crates/athena-context/src/briefs.rs (line ~1140 renders 'Checks Athena ran on this tree' from inputs.checks), crates/athena-core/src/admission.rs (a behavior criterion's demonstration must set fails_without, free text) and crates/athena-core/src/run/tests.rs (plan fixtures with fails_without). The new run happens before the verifier starts; its result is measured directly by running the named test command in a scratch copy, not inferred. No record or data file is added under crates/, so check_record_file/AnyRecord need no change.
- The 'planned revert' is read as: in a scratch copy of the candidate, restore the base version of the candidate's changed non-test files, keep the new tests, run the demonstration's command, and record command, exit code and output tail. If the plan's demonstration has no fails_without, nothing runs. No new plan field or planner prompt change (the planner/verifier prompts have almost no token headroom).
- The record is added as an extra checks entry like 'criteria-left-to-athena-checks' (not a check, fails nothing); it says whether the test failed (expected) or passed (revert did not make it fail), and the final verdict still rests on the recorded fmt/clippy/test checks.
- Prior jobs: 20261103-193428-df2m and 20261103-222737-zzsr changed drain.rs, tests/drain.rs, main.rs and CHANGELOG.md for the needs-accepting job-limit behaviour. Their code was judged correct; their only finding (missing failing run) is addressed here by this fix, and re-running that work is a separate follow-up request, so those four files are excluded in out_of_scope (work not kept in this job).
- Scope kept to one deliverable (Athena running and briefing the demonstration); wall time forecast stays under verifier_timeout_s of 2400s.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core` (filtered to the new test): a test where a plan has one demonstration with fails_without and the reverted copy makes the named test fail; the verifier brief it builds contains that demonstration's recorded run (command, non-zero exit code, output tail) under 'Checks Athena ran on this tree'.
2. [behavior] Run `cargo test -p athena-core` (filtered to the new test): a test where reverting does not make the named test fail; the brief's record says the test passed without the fix, and it is not presented as a failing run.
3. [behavior] Run `cargo test -p athena-context`: briefs.rs renders the demonstration record in the brief; a rendering test checks the text appears with command, exit code and output tail.
4. [content] The demonstration runs in a scratch copy: Grep/diff of verification.rs (or the checks stage) shows the candidate tree and the candidate commit are not modified, and the scratch copy is removed afterwards.
5. [content] The record is an entry that fails nothing ('not a check'), so the final verdict still rests on the recorded fmt, clippy and test checks.
6. [behavior] Outcome-check/tested: `cargo fmt --check`, `cargo clippy` and `cargo test` pass as recorded under 'Checks Athena ran on this tree'.
7. [content] CHANGELOG.md has an entry saying the verifier now sees Athena's recorded 'fails without the fix' run.

## Expected deliverables

- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Re-doing the needs-accepting job-limit work of jobs 20261103-193428-df2m and 20261103-222737-zzsr: crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and crates/athena-cli/src/main.rs are not changed here; that is a follow-up request once this lands.
- Giving the verifier a way to change files or run reverts itself.
- Passing the producer's report to the verifier.
- Adding a machine-readable revert field to plans or changing planner/critic/verifier prompts.
- Changes to athena.toml, defaults.toml, workflows or other governance files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both the failing-run and not-failing cases; fmt, clippy and test pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are checked by cargo test runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change; drain work deferred.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the verifier's blind spot once, rather than per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the record appearing in the brief, not wiring.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is the verifier seeing the failing run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by Athena running the demonstration, not by prose.
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
- 9 cases in 7 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Make "fails without the fix" a check that Athena runs itself. For a bug-fix job, Athena would run the new or changed tests against the base tree's production code, with the job's test files overlaid, and record the failing output as a named check. It would then run them on the final tree. The verifi...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

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

Run `cargo test -p athena-core` (filtered to the new test): a test where a plan has one demonstration with fails_without and the reverted copy makes the named test fail; the verifier brief it builds contains that demonstration's recorded run (command, non-zero exit code, output tail) under 'Checks Athena ran on this tree'.
Run `cargo test -p athena-core` (filtered to the new test): a test where reverting does not make the named test fail; the brief's record says the test passed without the fix, and it is not presented as a failing run.
Run `cargo test -p athena-context`: briefs.rs renders the demonstration record in the brief; a rendering test checks the text appears with command, exit code and output tail.
The demonstration runs in a scratch copy: Grep/diff of verification.rs (or the checks stage) shows the candidate tree and the candidate commit are not modified, and the scratch copy is removed afterwards.
The record is an entry that fails nothing ('not a check'), so the final verdict still rests on the recorded fmt, clippy and test checks.
Outcome-check/tested: `cargo fmt --check`, `cargo clippy` and `cargo test` pass as recorded under 'Checks Athena ran on this tree'.
CHANGELOG.md has an entry saying the verifier now sees Athena's recorded 'fails without the fix' run.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
