# Job 20261103-193428-df2m: Don't count needs-accepting re-reads toward the drain job limit

## Request (verbatim)

> Don't spend a batch's job slots on re-reads that end in 'needs accepting'. A batch with $22 and 4 job slots left just ended with nothing landed: all 4 slots went to old requests (from 10-16 Oct) that were read again after a settings change and came back 'needs accepting' at $0.06-0.18 each. A re-read that only produces 'needs accepting' should not count toward --job-limit, and the batch should go on to the next item. Acceptance: a test where a batch with job limit 1 meets a re-read that needs accepting and then still runs the next item.

## Reading (accepted by the user)

When a drain re-reads an old request after a settings change and the new reading needs accepting, that attempt must not use one of the --job-limit slots, so the drain goes on to the next item. Audience: the operator running `athena drain` with a job limit and budget. The fix is in the drain's job counter, with a regression test using job limit 1.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. In `run_job` (~line 1418) the order is: check stop, check `st.started >= limits.max_jobs` (JobLimit), check budget, then `st.started += 1`. It then reads or re-reads (`Source::Reread` calls `steps.reread`), then `steps.run`. `run.needs_accepting` yields `RunStep::NeedsAccepting` (~line 1537), which `attempts` maps to `ItemEnd::NeedsAccepting`, and `finish` passes the item over (~line 2411). Nothing gives the slot back, so the next item hits JobLimit. `approve_all` (~1889), `prepare` (~2008), `pick_ahead` (~2555) and the restart arithmetic (~2896) also read `st.started`. The real needs-accepting source is RealSteps::run (~line 3566). Fix: release the slot (`st.started -= 1`, saturating) when the step ends as NeedsAccepting. Money spent stays counted in the budget. The job-limit check is the one place that measures the limit.
- Code survey: crates/athena-cli/tests/drain.rs. The existing `a_run_that_needs_accepting_leaves_the_item_queued_without_a_failure` (~1305) and `a_reread_whose_scope_changed_is_left_for_a_person_and_is_not_a_failure` (~2933) cover the outcome with a stub that sets `needs_accepting`. `the_job_limit_stops_the_loop` (~1422) shows how to set `max_jobs: 1`. The new test reuses that Stub pattern: item a needs accepting, item b is runnable.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. No refusal rule is added or tightened, so no refusal-rate criterion applies.
- Only the NeedsAccepting outcome releases its slot. Failed, held and passed attempts keep counting, and the job-limit stop for those is unchanged.
- A change to docs or CHANGELOG is optional and not required. athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (Grep for `needs_accepting` near `max_jobs: 1`) drains with job limit 1 where item `a` ends as needs accepting and item `b` is runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1`, and `stop != Stop::JobLimit`. Run `cargo test -p athena-cli --test drain` and it passes.
2. [behavior] The same test, run against the unfixed `run_job` counter (as in the pre-change source in `git diff`), would stop with `Stop::JobLimit` before `b` runs. The assertions show it fails without the fix: `b` is asserted to land, which the old counter prevents.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the `RunStep::NeedsAccepting` path in `run_job` giving the started-job slot back (a decrement of `st.started` under the state lock). The JobLimit check and the other outcomes (Failed, Held, Passed) are unchanged.
4. [behavior] A job limit that still binds for ordinary jobs: `cargo test -p athena-cli --test drain` still passes `the_job_limit_stops_the_loop`, `an_urgent_queued_request_is_worked_before_an_approved_in_limit_proposal`, and the two existing needs-accepting tests, with no existing assertions weakened (`git diff` shows no removed asserts in them).
5. [behavior] `cargo test` passes for the workspace and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
6. [content] Doc comments or help text in crates/athena-cli/src/drain.rs that describe the job limit (the `--job-limit` help and the module docs near ~line 180) say a re-read that only needs accepting does not count toward it. A Grep for that wording finds it.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the cost of the re-read still counts against the budget.
- Changing how the other outcomes (failed, held, budget short) count toward the job limit.
- Auto-accepting readings whose scope differs.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test plus cargo test and clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the slot counting in run_job, not a special case for old requests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs a drain with job limit 1 and checks that the next item lands.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a batch lands the next item after a needs-accepting re-read.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test on the drain tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused counter change with one test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 42 cases in 42 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer may finish, Athena runs the cheap mechanical gate on the working copy: the decision-link consistency test (every "amends/supersedes Dxx" link in docs/decisions.md is in docs/decision-links.toml), fmt, clippy, and a scan that each behavior criterion has evidence citing a command t...
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

A new test in crates/athena-cli/tests/drain.rs (Grep for `needs_accepting` near `max_jobs: 1`) drains with job limit 1 where item `a` ends as needs accepting and item `b` is runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1`, and `stop != Stop::JobLimit`. Run `cargo test -p athena-cli --test drain` and it passes.
The same test, run against the unfixed `run_job` counter (as in the pre-change source in `git diff`), would stop with `Stop::JobLimit` before `b` runs. The assertions show it fails without the fix: `b` is asserted to land, which the old counter prevents.
`git diff` of crates/athena-cli/src/drain.rs shows the `RunStep::NeedsAccepting` path in `run_job` giving the started-job slot back (a decrement of `st.started` under the state lock). The JobLimit check and the other outcomes (Failed, Held, Passed) are unchanged.
A job limit that still binds for ordinary jobs: `cargo test -p athena-cli --test drain` still passes `the_job_limit_stops_the_loop`, `an_urgent_queued_request_is_worked_before_an_approved_in_limit_proposal`, and the two existing needs-accepting tests, with no existing assertions weakened (`git diff` shows no removed asserts in them).
`cargo test` passes for the workspace and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
Doc comments or help text in crates/athena-cli/src/drain.rs that describe the job limit (the `--job-limit` help and the module docs near ~line 180) say a re-read that only needs accepting does not count toward it. A Grep for that wording finds it.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
