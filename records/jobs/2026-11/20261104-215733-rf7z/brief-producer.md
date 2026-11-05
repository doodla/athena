# Job 20261104-215733-rf7z: Land in the same step when checks turn green during the poll

## Request (verbatim)

> Merge as soon as a pull request's checks are green. Today the batch rebases, re-checks and merges in one locked step, but after polling the pull request's checks (about 10 minutes) it returns Landing::Waiting and comes back to the job later, instead of merging when the checks turn green during the poll. That costs about 10 minutes per job.
> 
> Files changed: crates/athena-cli/src/drain.rs and its tests.
> 
> Acceptance:
> - a test where the checks go green during the poll and the job lands in the same step, without returning Waiting
> - a test where the checks are still running at the end of the poll keeps today's behaviour

## Reading (accepted by the user)

Make the drain's locked rebase, re-check and land step merge a pull request as soon as its checks go green during the poll, rather than returning Landing::Waiting and retrying about 10 minutes later. Still-pending checks at the end of the poll keep today's Waiting behaviour. The audience is the person running the drain, who loses about 10 minutes per job.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs. Drain::land takes the landing lock, runs recheck_after_rebase, then calls Steps::land and maps Landing::Landed, NotLanded or Waiting to a LandStep. PrLander::land calls athena.land_pr with crate::wiring::checks_poll, and landing_from_report maps an open pull request with Checks::Pending to Landing::Waiting. I also read crates/athena-core/src/gh/landing.rs (land_pr) and crates/athena-core/src/gh/early.rs (wait_for_checks). wait_for_checks already stops polling at the first non-pending read, and land_pr merges when the result is Green and merge_via_rest is on. So the stated bug may be in drain.rs or wiring.rs, or already fixed in core; the producer must find where Waiting is returned despite a green poll and reproduce it first.
- The drain.rs test World/Steps fake has a `waiting` set that makes land return Waiting; the new tests extend that fake or call landing_from_report and Drain::land directly. The tests drive a scripted checks sequence (pending then green) with a counting fake and no real sleep.
- No new record or data file under crates/ is added, so the records validation allow-list (check_record_file, AnyRecord) is not touched.
- The fix is confined to crates/athena-cli/src/drain.rs and its tests as the request says. A minimal change to crates/athena-cli/src/wiring.rs or crates/athena-core/src/gh is allowed only if the root cause is there, and then it is a listed deliverable.
- Merging only on green is preserved: a Red or still-Pending result never reaches the merge. This is not a new refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/src/drain.rs scripts checks that are pending at first and green partway through the poll. It shows the job reports Landing::Landed (the job line is 'landed') in the same land step, with no Landing::Waiting returned and no second land call. Check with `cargo test -p athena-cli drain`.
2. [behavior] A new test scripts checks that are still pending when the poll ends. It shows the result is still Landing::Waiting, the job line still starts 'waiting to merge', and the item is not retried. This is today's behaviour. Check with `cargo test -p athena-cli drain`.
3. [behavior] The existing tests a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job and a_merge_refused_for_rule_violations_while_checks_are_pending_is_reported_as_waiting still pass unchanged. Red checks still give NotLanded. Check with `cargo test -p athena-cli`.
4. [behavior] `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] The diff touches only crates/athena-cli/src/drain.rs, plus any file where the root cause is found. Comments or docs near the Landing::Waiting handling say that a poll which turns green merges in the same step.
6. [content] Outcome check: the diff shows the green-during-poll path ending in a merge in one locked step, rather than only adding types or wiring. Each new test would fail on the old code for the green-during-poll case.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the poll length or interval
- Changing the merge gate: merge only when every check is green and merge_via_rest is on
- Changing the rebase and re-check rule
- Changes to athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests, with cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the job lands or waits, by running the tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the job merges in the same step when the checks go green.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain.rs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer must locate where Waiting is returned despite a green poll.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments are written for an outside reader.
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

28 more qualifying classes are left out.

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

- path: <state>/jobs/20261104-215733-rf7z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-215733-rf7z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli/src/drain.rs scripts checks that are pending at first and green partway through the poll. It shows the job reports Landing::Landed (the job line is 'landed') in the same land step, with no Landing::Waiting returned and no second land call. Check with `cargo test -p athena-cli drain`.
A new test scripts checks that are still pending when the poll ends. It shows the result is still Landing::Waiting, the job line still starts 'waiting to merge', and the item is not retried. This is today's behaviour. Check with `cargo test -p athena-cli drain`.
The existing tests a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job and a_merge_refused_for_rule_violations_while_checks_are_pending_is_reported_as_waiting still pass unchanged. Red checks still give NotLanded. Check with `cargo test -p athena-cli`.
`cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
The diff touches only crates/athena-cli/src/drain.rs, plus any file where the root cause is found. Comments or docs near the Landing::Waiting handling say that a poll which turns green merges in the same step.
Outcome check: the diff shows the green-during-poll path ending in a merge in one locked step, rather than only adding types or wiring. Each new test would fail on the old code for the green-during-poll case.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
