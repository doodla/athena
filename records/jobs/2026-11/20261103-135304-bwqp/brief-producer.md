# Job 20261103-135304-bwqp: Don't count drain environment stops toward repeated failure

## Request (verbatim)

> try again on 20261102-223326-2x9t

## Reading (accepted by the user)

Retry of job 20261102-223326-2x9t (rejected, no candidate, no findings recorded): the drain's two-attempt rule in drain.rs `attempts` counts every non-landed attempt, so a rejection plus a "producer login expired" stop ends the run as Stop::RepeatedFailure. An attempt that ends in an environment stop will not count, the item stays queued, and the run goes on to the next item. Audience: people who run the drain and read its stop reason.

Assumptions:
- Previous job 20261102-223326-2x9t: it recorded no findings and no candidate, so there is nothing to fix or keep. This is a fresh attempt at the same request, with the same deliverables and out_of_scope. Its candidate changed no files, so nothing is carried over.
- Code survey: read crates/athena-cli/src/drain.rs. `Drain::attempts` (around line 1725) counts `failures` after each attempt that did not land and returns ItemEnd::Failed at ATTEMPTS, which sets Stop::RepeatedFailure (around line 2379). `run_job` builds RunStep::Failed from `run.reason` and `run.outcome`, so the reason is known at that point; RunStep::Failed does not yet carry it. The existing precedents `LandStep::Waiting`, `Held` and `NeedsAccepting` already mean 'not a failure, not counted'.
- Code survey: read crates/athena-core/src/disk.rs. `is_environment_reason(&str)` (line 432) is the one classifier, used by scope_guard.rs (line 547) and run/mod.rs (line 598). The drain should call it, with no new list of reason strings.
- Code survey: the other readers of a failure count are not the drain's rule. `steward::repeated_failures` (crates/athena-core/src/steward.rs line 712) groups rejected or failed jobs by class of work for the steward and counts per class, not per item. display.rs (line 619) only formats Stop::RepeatedFailure for the stop line. I expect to change neither and to state that reason in the final report. The change touches them only if the producer finds environment-stop jobs wrongly counted there, and then needs a test.
- Code survey: no record or data file is added under crates/, so the records allow-list, `check_record_file` and `AnyRecord` are not involved. The behavior is measured directly by the drain tests in crates/athena-cli/tests/drain.rs, which use the stub Steps with fixed run reasons.
- A rejected attempt followed by an environment-stop attempt leaves the item queued: the drain returns an ItemEnd that is not Failed, notes the reason, and moves to the next item. A later run tries the item again.
- This adds no new refusal rule and tightens none: it makes the drain stop less often, so the replay of past readings has nothing to refuse.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs: item A's first attempt is rejected and its second ends with reason 'environment: producer login expired'. The run does not end with Stop::RepeatedFailure, A stays queued and is not in the landed list, and the drain goes on to item B and lands it. Run: cargo test -p athena-cli --test drain.
2. [behavior] The existing drain tests the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable still pass, so two real failures still stop the run. Run: cargo test -p athena-cli --test drain.
3. [behavior] A drain test where the first attempt is the environment stop (login expired) and the second is a rejection also does not end the run as RepeatedFailure. Run: cargo test -p athena-cli --test drain.
4. [content] crates/athena-cli/src/drain.rs decides an attempt is an environment stop by calling athena_core::disk::is_environment_reason on the run's reason. A git diff and a Grep of crates/athena-cli show no new list of environment reason strings, and disk.rs is unchanged.
5. [content] The drain.rs module docs state that an attempt ending in an environment stop (login expired, usage limit, low disk, network) does not count toward Stop::RepeatedFailure and that the item stays queued.
6. [content] CHANGELOG.md has a new entry saying that environment stops no longer count toward the drain's repeated-failure rule.
7. [content] The final report lists every reader of the repeated-failure count (at least drain.rs attempts, display.rs line ~619 and steward.rs repeated_failures), each with a test or a stated reason. If steward.rs is changed, a steward test shows an environment-stop job is not counted.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Changing the scope guard's own counting or the environment classifier's list in crates/athena-core/src/disk.rs
- Changing how `athena retry` resumes environment stops
- Reworking the drain's other stop reasons
- Adding a repository-wide fmt, clippy or test run as its own criterion
- Any candidate files from job 20261102-223326-2x9t: it changed none, so none are kept

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new drain tests are in tests/drain.rs; they are named as criteria, not as a repository-wide run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The drain uses the one classifier, is_environment_reason, so every environment stop is covered, not just login expiry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run drain tests that check the stop reason and what happens to the next item.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The module docs and CHANGELOG entry explain the rule to an outside reader.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: It reuses the existing classifier and adds no second list.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain; steward.rs and display.rs change only if a reader needs it.
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

A new test in crates/athena-cli/tests/drain.rs: item A's first attempt is rejected and its second ends with reason 'environment: producer login expired'. The run does not end with Stop::RepeatedFailure, A stays queued and is not in the landed list, and the drain goes on to item B and lands it. Run: cargo test -p athena-cli --test drain.
The existing drain tests the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable still pass, so two real failures still stop the run. Run: cargo test -p athena-cli --test drain.
A drain test where the first attempt is the environment stop (login expired) and the second is a rejection also does not end the run as RepeatedFailure. Run: cargo test -p athena-cli --test drain.
crates/athena-cli/src/drain.rs decides an attempt is an environment stop by calling athena_core::disk::is_environment_reason on the run's reason. A git diff and a Grep of crates/athena-cli show no new list of environment reason strings, and disk.rs is unchanged.
The drain.rs module docs state that an attempt ending in an environment stop (login expired, usage limit, low disk, network) does not count toward Stop::RepeatedFailure and that the item stays queued.
CHANGELOG.md has a new entry saying that environment stops no longer count toward the drain's repeated-failure rule.
The final report lists every reader of the repeated-failure count (at least drain.rs attempts, display.rs line ~619 and steward.rs repeated_failures), each with a test or a stated reason. If steward.rs is changed, a steward test shows an environment-stop job is not counted.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
