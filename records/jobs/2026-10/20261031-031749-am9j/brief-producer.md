# Job 20261031-031749-am9j: Add stack-aware plan_rebase and rebase_job_branch for landing

## Request (verbatim)

> try again on 20261031-030119-tw3v. Second half of async drain A6: rebase each verified candidate onto the latest main before it lands.
> Depends on 20261031-031744-c5yx
> Priority: urgent
> 
> The first half (20261031-031744-c5yx) adds the landing queue. This half adds the rebase the queue head runs before landing. Returning a conflicting candidate to its producer, and the thrash stop of async-drain.md section 6, are not in this job.
> 
> Design, from the round-3 plan of tw3v and its critic's findings, with the open ruling made:
> - A new `Steps::rebase` (default: clean) runs for the queue head before `Steps::land`. In production it is `Athena::rebase_for_landing`, a thin wrapper over a free function `rebase_job_branch` in crates/athena-core/src/gh/landing.rs that takes git, the state dir and the remote as arguments, so tests run it against a bare origin.
> - `plan_rebase` is stack-aware: landing commits are stacked on one local branch, so it replays the commits between the fork and the job's landing commit one at a time onto origin's main.
> - A conflict in the job's own landing commit holds that job back: not landed, main untouched, no retry, no drain stop. The next queued candidate still lands.
> - Ruling on the critic's major finding: an earlier commit on the stack that conflicts is carried as a records-only commit. Its records/ paths are kept and its code paths are dropped, so no job's records leave the branch. A test shows the held-back job's records still load from the new tip.
> - Earlier unlanded commits that replay cleanly still ride along in the next job's pull request, as they do today. Say this in CHANGELOG.md and async-drain.md; do not claim the re-check rule covers stacked content.
> - Size: the wrapper test and the second bare-origin test may be reported as read-only checks if the run is long.
> 
> Acceptance:
> - Core tests of plan_rebase: a clean move, already current, a conflict in the job's own commit, an earlier conflicting commit carried as records only (its job record still loads), and a clean earlier commit carried.
> - A bare-origin test: two jobs where main moved after the second was verified; the second lands rebased.
> - A drain test: a conflicting head is held back and the next candidate lands.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, docs/design/async-drain.md, CHANGELOG.md

## Reading (accepted by the user)

Job 20261031-030119-tw3v (A6 second half) failed in planning, so I cut it to one deliverable: core rebase logic. This adds `plan_rebase` and `rebase_job_branch` in athena-core's gh/landing.rs, replaying the stacked landing commits onto origin's main. A conflicting earlier commit is carried as records-only. The drain wiring is split off as a follow-up. It depends on 20261031-031744-c5yx, which landed the queue.

Assumptions:
- Code survey: I read crates/athena-core/src/land.rs (resync_tip exists at line 99) and grepped crates/. The producer must read crates/athena-core/src/gh/landing.rs (Athena::land_pr, landed_commit_for, after_merge, resync_onto_merge, Records::load reading from the local branch head), crates/athena-core/src/gh/mod.rs (re-exports) and crates/athena-git (replay, merge_tree, reset_keep, fetch_remote_branch). plan_rebase is new code. The behavior is measured directly by calling production plan_rebase and rebase_job_branch against real git repositories, including a bare origin, not through mocks. No record file or AnyRecord kind is added, so check_record_file and the round-trip test are unaffected. This change adds no refusal rule.
- Goal and audience: the maintainer and verifier want each verified candidate rebased onto the latest main before it lands, with no job's records lost. This job delivers only the core mechanism that the drain will call.
- Retry of job 20261031-030119-tw3v. Its candidate recorded no changed files, so there is nothing to keep. Its earlier findings are addressed as follows. The major finding (record loss) is fixed by the user's ruling: a conflicting earlier commit is carried as a records-only commit. The gh/mod.rs finding is fixed by listing it as a deliverable. The CHANGELOG finding is fixed by stating that clean earlier unlanded commits ride along, and that the re-check rule does not cover stacked content. The queue-order finding is fixed because job 20261031-031744-c5yx already documented it. The size finding is fixed by this cut.
- Job 20261031-031744-c5yx (landed, verified) supplied the LandingQueue in crates/athena-cli/src/drain.rs, docs/design/async-drain.md and CHANGELOG.md. Those files are kept as they are, except that async-drain.md and CHANGELOG.md get new notes here. Its minor finding about await_head not checking stop_requested is outside this job.
- Scope cut for the verifier timeout. This job does not touch crates/athena-cli/src/drain.rs. Steps::rebase, Athena::rebase_for_landing, the drain test of a held-back head and the max_in_flight = 1 pass-through are a follow-up request that depends on this one. For that reason the CHANGELOG and docs entries describe only the core function and what the drain will do with it.
- Governance files (athena.toml, defaults.toml, workflows, lints) need no change.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core plan_rebase`. Core tests call production plan_rebase on real git repositories. They cover a clean move onto a moved main, already current, a conflict in the job's own landing commit (the plan reports a hold with main untouched), an earlier conflicting commit carried as records only, and a clean earlier commit carried.
2. [behavior] Run `cargo test -p athena-core`. In the records-only test, the earlier job's records/ paths are present at the new tip and its code paths are dropped. Records::load from the new tip still loads that held-back job's record.
3. [behavior] Run `cargo test -p athena-core rebase_job_branch`. A test against a bare origin has two jobs stacked locally, with main moved after the second was verified. rebase_job_branch (taking git, the state dir and the remote as arguments) leaves the second job's landing commit rebased onto origin's main, and `git log` shows a linear history. A second case, a conflict in the job's own commit, leaves origin's main and the local branch unchanged.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, and the existing tests are unchanged, including the athena-cli drain tests.
5. [content] `git diff` shows changes only to crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs (the re-export), docs/design/async-drain.md and CHANGELOG.md. drain.rs is untouched.
6. [content] CHANGELOG.md has an entry for the core rebase. docs/design/async-drain.md has a note. Both say that earlier unlanded commits which replay cleanly still ride along in the next job's pull request as they do today, and that the re-check rule does not cover stacked content. Both say that a conflicting earlier commit is kept as records only. Both name the drain wiring as a follow-up.

## Expected deliverables

- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Steps::rebase, Athena::rebase_for_landing and wiring the rebase into crates/athena-cli/src/drain.rs: follow-up request A6-rebase-drain, which depends on this one
- The drain test of a held-back head with the next candidate landing, and the Athena wrapper test: follow-up with the drain wiring
- Returning a conflicting candidate to its producer, send_back/drop_landed, and the section-6 thrash stop
- Raising max_in_flight above 1 (build job 10)
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-cli/tests/drain.rs and crates/athena-core/src/run/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for plan_rebase and rebase_job_branch; fmt, clippy and the test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run real git repositories, including a bare origin, through production functions.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Cut to the core mechanism; the drain wiring is a follow-up.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Records are carried on the branch rather than dropped, so a held-back job's records are not lost.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and design doc notes are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

Run `cargo test -p athena-core plan_rebase`. Core tests call production plan_rebase on real git repositories. They cover a clean move onto a moved main, already current, a conflict in the job's own landing commit (the plan reports a hold with main untouched), an earlier conflicting commit carried as records only, and a clean earlier commit carried.
Run `cargo test -p athena-core`. In the records-only test, the earlier job's records/ paths are present at the new tip and its code paths are dropped. Records::load from the new tip still loads that held-back job's record.
Run `cargo test -p athena-core rebase_job_branch`. A test against a bare origin has two jobs stacked locally, with main moved after the second was verified. rebase_job_branch (taking git, the state dir and the remote as arguments) leaves the second job's landing commit rebased onto origin's main, and `git log` shows a linear history. A second case, a conflict in the job's own commit, leaves origin's main and the local branch unchanged.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, and the existing tests are unchanged, including the athena-cli drain tests.
`git diff` shows changes only to crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs (the re-export), docs/design/async-drain.md and CHANGELOG.md. drain.rs is untouched.
CHANGELOG.md has an entry for the core rebase. docs/design/async-drain.md has a note. Both say that earlier unlanded commits which replay cleanly still ride along in the next job's pull request as they do today, and that the re-check rule does not cover stacked content. Both say that a conflicting earlier commit is kept as records only. Both name the drain wiring as a follow-up.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
