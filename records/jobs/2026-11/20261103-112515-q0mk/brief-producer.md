# Job 20261103-112515-q0mk: Make a verifier timeout a distinct unjudged outcome, retried once

## Request (verbatim)

> try again on 20261027-054228-k9ex

## Reading (accepted by the user)

Retry of job 20261027-054228-k9ex (dispatched, never verified, no findings recorded): its candidate work in 7 files is kept as the starting point and finished with a rerun of a timed-out verifier plus tests. The root cause is that a verifier killed at its deadline was recorded as the generic `verifier_failure` and failed the verdict like a judged failure, so criteria were rejected unjudged (jobs 20261010-154005-jsdj, 20261018-120736-90w3, 20261018-133651-1ncn). Audience: the Athena operator reading verdicts.

Assumptions:
- Code survey: I read crates/athena-core/src/run/verification.rs (verification_stage sets a plain-string verifier_failure at the budget, snapshot lock, could-not-run, non-Success outcome and no-valid-verdict points, and already reruns once on a system_fault), crates/athena-core/src/verify.rs (VerdictInputs.verifier_failure: Option<String>; judge pushes 'the verifier did not complete: ..' and makes the verdict fail when it is set) and crates/athena-core/src/run/send_back.rs (constructs VerdictInputs with verifier_failure: None). The property is measured directly by calling judge with the new failure value and by running verification_stage with a stub verifier. The change adds no record or data file under crates/, so check_record_file and AnyRecord are unaffected. It adds no refusal rule.
- Job 20261027-054228-k9ex: its candidate changed 7 files, and all are kept as deliverables. It typed verifier_failure as Incomplete or TimedOut and gave the timeout an 'unjudged' reason. The candidate was never verified and recorded no findings, so there is nothing to fix. What was missing is that its rerun on timeout and its behavior tests are not shown, so this retry adds them.
- A timeout is detected from the verifier invocation's recorded detail or outcome. The first timeout triggers one rerun, and the second leaves the result unjudged, not a pass.
- The verdict stays non-passing when unjudged, but the reason text says 'unjudged', not 'rejected', and does not add 'returned no verdict'.
- Previous out-of-scope items stay: no change to athena.toml, defaults.toml, workflows or the verifier_timeout_s value, and no per-criterion saving or deadline scaling.

## Acceptance criteria

1. [content] The change description names the root cause: a verifier timeout was recorded as the generic `verifier_failure` in crates/athena-core/src/run/verification.rs and failed the verdict in crates/athena-core/src/verify.rs exactly like a judged failure, so criteria were rejected unjudged.
2. [content] A Grep of the diff in crates/athena-core/src/verify.rs and crates/athena-core/src/run/verification.rs shows a distinct timeout variant whose reason says 'unjudged' and is separate from 'the verifier did not complete'; it never yields a pass.
3. [behavior] Run `cargo test -p athena-core`. A new test where the verifier times out on the first run and succeeds on the second ends with the second run's verdict. A second test where it times out twice ends with the unjudged reason, not as a judged rejection. Both fail on the base commit and pass on the change.
4. [behavior] Run `cargo test -p athena-core`. A new test shows that a non-timeout verifier failure, a failing check, or a judged fail verdict still fails the verdict with its existing reason.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`; all pass with no warnings.
6. [content] outcome-check: the tests assert the verdict and reason recorded for a timed-out verifier, not only that types or wiring exist.
7. [content] CHANGELOG.md has an entry for the new outcome, and docs/records.md and docs/design/athena-v0.md are updated where they describe the verifier-failure reason.

## Expected deliverables

- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/records.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)

## Out of scope

- Re-landing the candidate work of jobs 20261010-154005-jsdj, 20261018-120736-90w3 and 20261018-133651-1ncn (run.rs trailer, docs/spec.toml, xtask/tests/spec.rs, docs edits)
- Changing athena.toml, defaults.toml or workflows, including the verifier_timeout_s value
- Saving each criterion's verdict as it is reached
- Making the deadline scale with the cargo work
- Changing the gate's reading of records

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is where the timeout is recorded and judged, not at one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests assert the recorded verdict and reason.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo test commands.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

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

The change description names the root cause: a verifier timeout was recorded as the generic `verifier_failure` in crates/athena-core/src/run/verification.rs and failed the verdict in crates/athena-core/src/verify.rs exactly like a judged failure, so criteria were rejected unjudged.
A Grep of the diff in crates/athena-core/src/verify.rs and crates/athena-core/src/run/verification.rs shows a distinct timeout variant whose reason says 'unjudged' and is separate from 'the verifier did not complete'; it never yields a pass.
Run `cargo test -p athena-core`. A new test where the verifier times out on the first run and succeeds on the second ends with the second run's verdict. A second test where it times out twice ends with the unjudged reason, not as a judged rejection. Both fail on the base commit and pass on the change.
Run `cargo test -p athena-core`. A new test shows that a non-timeout verifier failure, a failing check, or a judged fail verdict still fails the verdict with its existing reason.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`; all pass with no warnings.
outcome-check: the tests assert the verdict and reason recorded for a timed-out verifier, not only that types or wiring exist.
CHANGELOG.md has an entry for the new outcome, and docs/records.md and docs/design/athena-v0.md are updated where they describe the verifier-failure reason.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
