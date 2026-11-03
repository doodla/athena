# Job 20261102-223326-2x9t: Don't count environment stops toward the drain's repeated-failure rule

## Request (verbatim)

> The drain stopped a run for 'repeated failure' when one of the two failures was an environment stop: the build-inputs retry ended with 'environment: producer login expired' (a 401 from the model service), and the drain counted it with the earlier rejection as two failures of the item.
> - An attempt that ends in an environment stop (login expired, usage limit, low disk, network) does not count toward the drain's repeated-failure rule for its item; the item stays queued for a later run.
> - Use the one place that already classifies environment stops for the scope guard; do not add a second list.
> - A drain test: an item with one rejection and one login-expired attempt does not stop the run, and the run goes on to the next item.
> - CHANGELOG.md gets an entry.
> - Before planning, list what else reads the repeated-failure count, with a test or a reason for each.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

The drain counts any failed attempt toward its two-attempt limit, so a rejection plus a "producer login expired" stop ends the run as "repeated failure". An attempt that ends in an environment stop will no longer count; the item stays queued and the run goes on. Audience: people who run the drain and read its stop reason.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs: `attempts` counts `failures` against ATTEMPTS and returns ItemEnd::Failed, which `set_stop`s Stop::RepeatedFailure; `run_job` turns any run outcome that is not passed, needs-accepting or held into RunStep::Failed, with the run's reason in `line.reason`. The environment classifier is `athena_core::disk::is_environment_reason` / `StopCause::of` in crates/athena-core/src/disk.rs (PRODUCER_AUTH = "environment: producer login expired"), already used by crates/athena-core/src/scope_guard.rs and crates/athena-core/src/run/mod.rs. The change reuses it and adds no second list. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- Existing code: crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/scope_guard.rs.
- Design: in `run_job`, a run whose reason is an environment stop yields a new non-failure step (like Held/Waiting), which `attempts` maps to an item end that leaves the item queued, notes the reason via pass_over and lets the run go on. Environment stops already reach this path with outcome Failed and a reason.
- Readers of the repeated-failure count found by search: (1) `failures`/ATTEMPTS in Drain::attempts (the rule being fixed); (2) the Stop::RepeatedFailure display in crates/athena-cli/src/display.rs and the report fields in drain.rs (read only after the stop is set; covered by the existing drain.rs tests the_same_item_failing_twice_is_a_repeated_failure_naming_the_item); (3) crates/athena-core/src/steward.rs `repeated_failures`, a per-class count of Rejected/Failed jobs fed to the steward, which also counts environment-stop jobs. The producer decides whether to exclude them there too, with a test, or records why not.
- The goal (environment stops not counted) is in scope and is not an out_of_scope item.
- No refusal rule is added; the change refuses fewer runs as repeated failure, not more readings.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs: an item whose first attempt is rejected and whose second ends with reason "environment: producer login expired" does not end the run with Stop::RepeatedFailure; the item stays queued and the drain goes on to the next item and works it. Run: cargo test -p athena-cli --test drain.
2. [behavior] The existing drain tests the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable still pass: two real failures still stop the run. Run: cargo test -p athena-cli --test drain.
3. [content] The drain decides an attempt is an environment stop by calling athena_core::disk::is_environment_reason (or StopCause); a Grep of crates/athena-cli finds no new list of environment reason strings.
4. [content] The drain.rs module docs state that an attempt ending in an environment stop does not count toward Stop::RepeatedFailure and that the item stays queued.
5. [content] CHANGELOG.md has a new entry saying that environment stops (login expired, usage limit, low disk, network) no longer count toward the drain's repeated-failure rule.
6. [content] The final report lists every reader of the repeated-failure count (at least drain.rs attempts, display.rs, steward.rs repeated_failures), each with a test or a stated reason; if steward.rs is changed, a steward test shows an environment-stop job is not counted.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Changing the scope guard's own counting or the environment classifier's list in disk.rs
- Changing how `athena retry` resumes environment stops
- Reworking the drain's other stop reasons
- Adding a repository-wide fmt, clippy or test run as its own criterion

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-cli.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain test plus existing tests for the two-failure case.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The drain reuses the shared environment classifier and the other readers of the count are checked.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the drain tests and check the run goes on to the next item.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: The existing disk.rs classifier is reused instead of a new list.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain plus a test and a changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 40 cases in 40 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one repair round within the same job. The verifier is only start...
- 20 cases in 18 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make the disk guard and build cleanup one ordered step. Before the free-space check, and again on any low-disk hold, Athena removes the regenerable builds of held, interrupted and finished jobs, then measures free space. Only the unfreeable shortfall holds a job. Held jobs resume automatically once ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

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

A new test in crates/athena-cli/tests/drain.rs: an item whose first attempt is rejected and whose second ends with reason "environment: producer login expired" does not end the run with Stop::RepeatedFailure; the item stays queued and the drain goes on to the next item and works it. Run: cargo test -p athena-cli --test drain.
The existing drain tests the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable still pass: two real failures still stop the run. Run: cargo test -p athena-cli --test drain.
The drain decides an attempt is an environment stop by calling athena_core::disk::is_environment_reason (or StopCause); a Grep of crates/athena-cli finds no new list of environment reason strings.
The drain.rs module docs state that an attempt ending in an environment stop does not count toward Stop::RepeatedFailure and that the item stays queued.
CHANGELOG.md has a new entry saying that environment stops (login expired, usage limit, low disk, network) no longer count toward the drain's repeated-failure rule.
The final report lists every reader of the repeated-failure count (at least drain.rs attempts, display.rs, steward.rs repeated_failures), each with a test or a stated reason; if steward.rs is changed, a steward test shows an environment-stop job is not counted.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
