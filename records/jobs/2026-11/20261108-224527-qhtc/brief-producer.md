# Job 20261108-224527-qhtc: Make both snapshot locks attempt a write at every path

## Request (verbatim)

> try again on 20261027-054321-rxa7

## Reading (accepted by the user)

Retry of job 20261027-054321-rxa7 (failed in planning; no candidate kept). Fix the class in both copies of the two-place-probe lock, `Snapshot::lock` in athena-git and `lock` in athena-core's snapshot.rs, via one shared routine that attempts a write at every path and names the writable one. The audience is the Athena operator, who should not get verdicts voided by a partly locked snapshot.

Assumptions:
- Code survey: read crates/athena-git/src/snapshot.rs (Snapshot::lock line 161, probe_write line 182, set_write_bits, set_immutable) and crates/athena-core/src/snapshot.rs (lock line 133, probe_write line 155); both probe only a root file and one other file. Callers: crates/athena-core/src/run/verification.rs already turns a lock error into Incomplete before the verifier runs; that stays unchanged. No records or data files are added, so no record validation is touched.
- Job 20261027-054321-rxa7 failed at plan critique round 1 with no candidate, so nothing is kept; its findings are addressed: (a) the second copy in athena-core is fixed here; (b) tests bind root by building the partial lock with immutable/attempted-write or an injected writable path, not mode bits alone; (c) a no-leftover-probe check is made on the real lock then unlock path; (d) the error distinguishes 'accepted a write' from 'could not be checked'; (e) the CHANGELOG states only what the diff does and does not blame earlier jobs.
- Shared routine: `writable_paths` is a pub function in athena-git (athena-core depends on athena-git) that both locks call; athena-core's probe_write is deleted.
- The suite may run as root, so tests must use an attempted write and fail against the old two-place probe.
- Verification.rs, run_pin_tests.rs and the rerun policy are dropped from scope unless a compile fix needs them; the earlier-listed deliverables there are not kept as planned changes.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`: a new test leaves one nested path (e.g. .git/objects) writable in an otherwise locked tree, constructed so it binds root too (attempted write or injected writable path, not mode bits alone), and asserts `Snapshot::lock` returns an error naming exactly that path. It fails against the old two-place probe.
2. [behavior] Run `cargo test -p athena-core`: a new test in crates/athena-core/src/snapshot.rs does the same for the evidence-snapshot `lock`, asserting the error names the nested writable path, and it fails against the old probe when run as root.
3. [content] crates/athena-core/src/snapshot.rs `lock` calls the shared athena-git routine, and its own `probe_write` is deleted (Grep finds no `fn probe_write` there); athena-git's `lock` uses the same routine over every directory and file including .git.
4. [behavior] Run `cargo test -p athena-git`: a test shows that after a successful `lock()` then `unlock()` no `.athena-lock-probe` file exists anywhere in the tree and the work-tree tree hash equals its value before the lock.
5. [behavior] Run `cargo test -p athena-git`: a test shows the lock error text differs between a path that accepted a write and a path that could not be checked (e.g. 'could not be listed').
6. [behavior] Run `cargo test -p athena-git` and `cargo test -p athena-core`: existing tests still pass, including `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` and the snapshot-change rerun tests in run_pin_tests.rs.
7. [content] CHANGELOG.md [Unreleased] has an entry stating that both locks now attempt a write at every path and fail naming the writable one; it does not assert that the earlier two-place probe caused jobs 20261015-031159-xx7n or 20261015-033631-x3a1.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Changing the rerun-once policy or tree comparison in verification.rs beyond surfacing a lock failure
- Hashing gitignored files such as target/
- studies/firstmate-work-handling/*, xtask/src/site.rs, docs/design/explainer-site.md (earlier candidate changes dropped)
- records/, governance files (athena.toml, defaults.toml, workflows) and prompts

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Both lock copies fixed via one shared routine.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests attempt real writes, as root too.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check a partly locked snapshot is refused.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two source files plus CHANGELOG.
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
- 11 cases in 11 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the final verdict. For a job whose criteria name specific job ids, the check reads the live records and lists, per id, whether an outcome record exists and whether a closed job cites a commit hash found in git log. It reports any gap to the producer while it can sti...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261108-224527-qhtc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-224527-qhtc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-git`: a new test leaves one nested path (e.g. .git/objects) writable in an otherwise locked tree, constructed so it binds root too (attempted write or injected writable path, not mode bits alone), and asserts `Snapshot::lock` returns an error naming exactly that path. It fails against the old two-place probe.
Run `cargo test -p athena-core`: a new test in crates/athena-core/src/snapshot.rs does the same for the evidence-snapshot `lock`, asserting the error names the nested writable path, and it fails against the old probe when run as root.
crates/athena-core/src/snapshot.rs `lock` calls the shared athena-git routine, and its own `probe_write` is deleted (Grep finds no `fn probe_write` there); athena-git's `lock` uses the same routine over every directory and file including .git.
Run `cargo test -p athena-git`: a test shows that after a successful `lock()` then `unlock()` no `.athena-lock-probe` file exists anywhere in the tree and the work-tree tree hash equals its value before the lock.
Run `cargo test -p athena-git`: a test shows the lock error text differs between a path that accepted a write and a path that could not be checked (e.g. 'could not be listed').
Run `cargo test -p athena-git` and `cargo test -p athena-core`: existing tests still pass, including `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` and the snapshot-change rerun tests in run_pin_tests.rs.
CHANGELOG.md [Unreleased] has an entry stating that both locks now attempt a write at every path and fail naming the writable one; it does not assert that the earlier two-place probe caused jobs 20261015-031159-xx7n or 20261015-033631-x3a1.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
