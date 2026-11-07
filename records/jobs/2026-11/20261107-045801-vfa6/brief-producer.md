# Job 20261107-045801-vfa6: Stop retrying a line after two attempts that did not land

## Request (verbatim)

> try again on 20261107-031917-r96w

## Reading (accepted by the user)

Retry of job 20261107-031917-r96w, narrowed to its first piece: the drain stops retrying a line (grouped by the scope guard's group_lines) once it has two attempts that did not land, collisions included, and holds it as awaiting diagnosis. The diagnosis role, schema, line recording and status display are follow-up requests.

Assumptions:
- Goal and audience: the operator wants fewer wasted retries; this job delivers only the stop rule, as the prior job 20261107-031917-r96w failed in planning (17 findings, no candidate, nothing kept) because its scope was too large for one run.
- Code survey: read crates/athena-core/src/scope_guard.rs (group_lines at line 122, line_attempts at 161) and crates/athena-cli/src/drain.rs (Landing::NotLanded, LandStep::NotLanded at ~1781-1938, where failures are counted). No new record or data file is added, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Findings of job 20261107-031917-r96w addressed: 'did not land' counts Rejected, Failed and verified-but-not-landed attempts (LandStep::NotLanded), whatever the job's state; diagnosis-record totals, paused-line follow-ups, failed-diagnosis release, status display and the Role::Diagnosis variant matches no longer apply because the diagnosis run is out of scope.
- Existing code: crates/athena-core/src/scope_guard.rs group_lines and line_attempts are reused for grouping; no second grouping implementation is added.
- The stop is a hold in the drain's per-item decision, not a new record kind; the drain's existing environment-stop handling is unchanged and does not count toward the two attempts.
- Test seam: drain tests use real records and the real drain step (as in crates/athena-cli/tests/drain.rs), not a fake that computes the count.

## Acceptance criteria

1. [behavior] Run `cargo test`: a drain test over real records shows a line with two attempts that did not land is not retried, and a second drain run still does not retry it.
2. [behavior] Run `cargo test`: a drain test shows a line with two verified-but-not-landed (collision) attempts, one with its job still open or blocked, is held and not retried.
3. [behavior] Run `cargo test`: a drain test shows a line with one failed attempt is retried as today, and an environment-cause stop does not count toward the two.
4. [behavior] Run `cargo test`, `cargo clippy` and `cargo fmt --check`: all pass.
5. [content] Grep shows drain.rs uses group_lines or line_attempts from scope_guard.rs, and no second line-grouping function is added.
6. [content] CHANGELOG.md states only what the diff does: the drain holds a line after two non-landed attempts; it does not claim a diagnosis job, status display or line field.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Follow-up request: the diagnosis role brief (crates/athena-context/prompts/diagnosis.md), its output schema, and the five-category sort with one queued follow-up
- Follow-up request: recording a line value on every job (crates/athena-data/src/records.rs) and the athena status display of open lines (status.rs, display.rs)
- Follow-up request: running diagnosis under its own run id, a paused-line retry exception, and releasing a failed diagnosis by command
- Changing scope guard signals T1 to T5 or thresholds; editing athena.toml, defaults.toml or workflows
- Backfilling old records; editing shared contract, reader or verifier prompts
- Prior job 20261107-031917-r96w's deliverables other than drain.rs, scope_guard.rs and CHANGELOG.md are dropped from this job (candidate was empty, so nothing is kept)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; the rest is split into follow-up requests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the real drain step over real records.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a twice-failed line is not retried again, not that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Collision attempts are counted, not just rejected or failed records.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

32 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

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

- path: <state>/jobs/20261107-045801-vfa6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-045801-vfa6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: a drain test over real records shows a line with two attempts that did not land is not retried, and a second drain run still does not retry it.
Run `cargo test`: a drain test shows a line with two verified-but-not-landed (collision) attempts, one with its job still open or blocked, is held and not retried.
Run `cargo test`: a drain test shows a line with one failed attempt is retried as today, and an environment-cause stop does not count toward the two.
Run `cargo test`, `cargo clippy` and `cargo fmt --check`: all pass.
Grep shows drain.rs uses group_lines or line_attempts from scope_guard.rs, and no second line-grouping function is added.
CHANGELOG.md states only what the diff does: the drain holds a line after two non-landed attempts; it does not claim a diagnosis job, status display or line field.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
