# Job 20261101-222517-3kgp: Commit job records from the real landing queue after each landing (A8)

## Request (verbatim)

> try again on 20261031-031939-wz33

## Reading (accepted by the user)

Retry of job 20261031-031939-wz33 (A8, D145, job 8): make the real drain path commit each landed job's records from the landing queue right after its landing, in landing order, and confirm and enforce the append-only conflict rule. No default or no-op hook may remain as the only implementation. At max_in_flight = 1 the drain behaves as today. The audience is the maintainer, who must see real records committed after real landings.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs (trait Steps at ~500, with Steps::land at ~513; Lander and PrLander at ~2780; RealSteps::land at ~3307) and crates/athena-core/src/run/close.rs (land_or_block ~204, hold_for_disk ~177). I also read crates/athena-core/src/commits.rs (commit_records, commit_records_with, commit_records_in at ~131-260). The entry point is drain_concurrent. Today a job's closing files, score and outcome are committed at close in land_or_block, before the job joins the landing queue. The new behaviour is measured directly through drain_concurrent with real records. No new record type or data file is added, so the records allow-list (check_record_file, AnyRecord) is unchanged.
- Previous result, job 20261031-031939-wz33: its candidate is dropped and rebuilt from the base. Its hook Steps::commit_records is replaced by a real implementation. Its findings are addressed as follows. Finding 1 (hook only, no real override): the real Athena-backed Steps defers the close-time records commit for a job that will land and commits those records after its landing, from the landing queue. A test covers this. Finding 2 (the append-only conflict rule left undone): it gets its own implementation, test and criterion. Finding 3 (the wall-clock-sleep flakiness risk): the new tests order jobs with channels or barriers, not sleeps. Finding 4 (store.rs unchanged): store.rs changes only if the work needs it.
- The 'month-shard conflict reading' in docs/design/async-drain.md (~line 221) is confirmed against the code and the doc is updated to state it, with no 'not done' notes left. If the reading proves wrong, the doc states what the code does.
- Jobs that do not land commit their records as they do today. A failed or stale job keeps its close-time behaviour, because there is no landing to follow.
- Existing code: crates/athena-cli/src/drain.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/commits.rs, crates/athena-core/src/store.rs and docs/design/async-drain.md are extended, not replaced.
- Prerequisite A6 (20261030-013555-f5qm) is taken as present or addressed in this job only as far as records ordering needs it. The A6 rebase and conflict-return work stays out of scope.

## Acceptance criteria

1. [behavior] A test through drain_concurrent with at least three jobs in flight and real Athena-backed landing and records code (not a stub Steps override). Run: cargo test -p athena-cli --test drain. The test shows the records commits made after landing appear in git log in landing order, each after its own landing and none before it.
2. [behavior] A test shows that a job which does not land gets no records commit from the landing queue. Run: cargo test -p athena-cli --test drain.
3. [behavior] A test shows the append-only conflict rule: a records commit that conflicts with existing records is refused or resolved as the design doc states, and no existing record file is deleted or rewritten (git diff over the records shows only added files). Run: cargo test.
4. [behavior] With max_in_flight = 1 the existing drain tests pass unchanged. git diff shows no edits to existing test bodies. A test through drain_concurrent with max_concurrency 2 and max_in_flight 1 pins today's records commit behaviour.
5. [content] Grep of crates/athena-cli/src/drain.rs and crates/athena-core shows the records-after-landing step has a real implementation in the production path. No requested hook or trait method has only its default no-op implementation.
6. [content] docs/design/async-drain.md states both parts of build job 8 as built: the commit from the landing queue and the append-only conflict rule. It has no leftover 'not built', 'not confirmed' or 'later work' note for either part.
7. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass across the workspace.
8. [content] CHANGELOG.md has an entry saying records are now committed from the landing queue after each landing, with no behaviour change at max_in_flight = 1.
9. [content] Outcome check: a maintainer reading the new tests can see the records commit sequence beside the landing sequence for at least three jobs, and the tests use no wall-clock sleeps to fix the order.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/commits.rs (modify)

## Out of scope

- Raising max_in_flight above 1 or running several producers at once (build job 10)
- The landing queue's rebase, conflict return and thrash stop (A6, job 20261030-013555-f5qm)
- Disk guard, check slots, read-ahead and recovery of leftover in-flight rows
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Changes to record formats, records/ and crates/athena-core/src/gh/landing.rs
- The previous candidate's hook-only commit_records in drain.rs, its tests and its doc note: dropped and rebuilt, not kept

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require tests through the real drain path with real records commits.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The maintainer-readable commit-versus-landing sequence criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The real close-time records commit is moved, not just hooked.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name the commands the verifier runs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The append-only rule is enforced and tested, not prose-only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261031-031939-wz33 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-cli/src/drain.rs:633 (default hook, no real override); docs/design/async-drain.md:236-242): The requested behaviour is built only as a hook. No real records are committed from the landing queue, so production record ordering is unchanged at every bound. The only definition of Steps::commit_records in the codebase is the default `Ok(())` at drain.rs:633. A grep finds no override in the real Athena-backed Steps, and the only overrides are the test stub `Recording` in tests/drain.rs. The candidate's own design note says 'Not built here: the real steps keep the default. Today `go` still commits a job's closing files and its score and outcome records at close (land_or_block in run/close.rs), before the job joins the landing queue, so those commits follow close order.' The request says 'commit records from the landing queue after each landing'. The user-accepted reading says 'the drain commits each landed job's records from the landing queue right after its landing', and its assumption expects the producer to wire the commit to the real landing step. The cited design (async-drain.md section 8 and build job 8) says each job writes its records locally without committing them and the queue commits them after landing. None of that happens in a real drain. The tests commit synthetic priority records from a stub, not the job's real records. Build job 9 depends on job 8, yet the essential move of the close-time commit is deferred to unnamed 'later work'. Simpler, correct alternative: have the real steps defer the close-time records commit for a job that will land, and commit them in the real Steps::commit_records override. Pin bound 1 with the existing real-steps tests.
> major (docs/design/async-drain.md:241): Build job 8's second half, 'confirm the append-only conflict rule', is explicitly left undone. async-drain.md:221 says the month-shard conflict reading 'is to be confirmed by build job 8 before it is relied on', and job 8 at line 344-345 says 'confirm the append-only conflict rule'. The candidate's note at line 241 says 'The month-shard conflict reading is not confirmed', so the doc's requirement is still open after the job meant to close it.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

A test through drain_concurrent with at least three jobs in flight and real Athena-backed landing and records code (not a stub Steps override). Run: cargo test -p athena-cli --test drain. The test shows the records commits made after landing appear in git log in landing order, each after its own landing and none before it.
A test shows that a job which does not land gets no records commit from the landing queue. Run: cargo test -p athena-cli --test drain.
A test shows the append-only conflict rule: a records commit that conflicts with existing records is refused or resolved as the design doc states, and no existing record file is deleted or rewritten (git diff over the records shows only added files). Run: cargo test.
With max_in_flight = 1 the existing drain tests pass unchanged. git diff shows no edits to existing test bodies. A test through drain_concurrent with max_concurrency 2 and max_in_flight 1 pins today's records commit behaviour.
Grep of crates/athena-cli/src/drain.rs and crates/athena-core shows the records-after-landing step has a real implementation in the production path. No requested hook or trait method has only its default no-op implementation.
docs/design/async-drain.md states both parts of build job 8 as built: the commit from the landing queue and the append-only conflict rule. It has no leftover 'not built', 'not confirmed' or 'later work' note for either part.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass across the workspace.
CHANGELOG.md has an entry saying records are now committed from the landing queue after each landing, with no behaviour change at max_in_flight = 1.
Outcome check: a maintainer reading the new tests can see the records commit sequence beside the landing sequence for at least three jobs, and the tests use no wall-clock sleeps to fix the order.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
