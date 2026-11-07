# Job 20261030-060835-ryvc: Require a measured stage timeline in readings of speed requests

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request named a fix (move the call earlier) and a timing figure, but nothing required the readin
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: speed-change-requested-without-checking-measured-stage-timeline (20261025-110542-wa68)
> Cause: The request named a fix (move the call earlier) and a timing figure, but nothing required the reading or planner to check that figure against the real stage order and durations before planning around it. The class already exists for a speed...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261025-110857-eqhd (job 20261025-110525-8hzb); prevention it proposed: For any request that claims a delay or speed-up, the reading stage must first produce a timeline from the code and the recorded run timings: each stage in order, its measured duration, and where the claimed delay sits. The reading must stat...
> Introspection 20261025-183009-xw7y (job 20261025-034733-c453); prevention it proposed: Add a measure-first step for any request to speed up a pipeline or check. Before the reading is accepted, Athena runs or reads a timing breakdown (compile versus run, per test binary or stage) and puts it in the reading. The reading must th...
> Introspection 20261025-190659-jvd7 (job 20261025-190606-eebk); prevention it proposed: Add a required measurement step to any request or reading whose goal is speed. Before the reading fixes a method, it must record a per-stage or per-binary timeline from a real run. It must also try each candidate lever, such as a different ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Goal: stop speed or delay requests being planned from the request's own figures (jobs 20261025-110525-8hzb, 20261025-034733-c453, 20261025-190606-eebk). Add one reading-acceptance check, `check_speed_timeline`, that refuses a reading of such a request unless it has a `Timeline:` assumption naming the crates/ code read and a measured duration per stage. Audience: the reader, planner and verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: check_reading_acceptance calls refusing_check, which runs `step!` lines in order (check_prior_result_and_scope, check_criteria_evidence, check_code_survey, ...) and receives the request text; check_code_survey (reading.rs ~3910) is the model for a check on assumptions. Reading records are validated by build_reading; this change adds no record or data file, so check_record_file and AnyRecord are not involved. The property (a refused reading) is measured directly by calling refusing_check on real request and reading shapes, via crates/athena-core/tests/refusal_corpus.rs, which fails when a step! has no corpus case. Reader rules live in READING_RULES in crates/athena-context/src/briefs.rs.
- Root cause: the reading stage only checked a request's figures if the reader chose to, so readings planned around stated timings (move the call earlier, 12 minutes of checks, split compile versus run) without a stage timeline from the code. Fix at the reading gate so it covers the whole class, not one job.
- A speed request is detected from the request text by whole-word cues such as faster, speed up, slow, delay, latency, cut ... minutes, head start, timing. The rule requires an assumption starting `Timeline:` that names a crates/ path and gives at least two stages each with a duration figure (a number with s, ms, min or minutes) and where the claimed delay sits, or says no recorded timing exists.
- Prior jobs: 20261025-110525-8hzb (failed in planning; its request misplaced where the delay sits) and 20261025-034733-c453 (failed in planning; no measured compile/run split) had no candidate, so no files are kept; this change addresses their cause, the missing timeline, and does not redo their work. 20261025-190606-eebk's findings (6 minute target unmet, production behavior changed in secrets.rs and checks.rs, files outside the list) are addressed by dropping that candidate's secrets.rs and checks.rs changes; its CHANGELOG.md and docs/decisions.md edits are not kept as they were, and new entries are written for this change only.
- Expected 5% of newly refused readings: only readings of speed requests lacking a Timeline: assumption are refused; the higher refusal rate is intended.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new corpus case in crates/athena-core/tests/refusal_corpus.rs gives a speed request (e.g. 'move the call earlier to save 12 minutes') and a reading with no `Timeline:` assumption, and asserts refusing_check refuses it with `check_speed_timeline`; the same test fails if the step! line is removed.
2. [behavior] Run `cargo test -p athena-core`: a reading of the same speed request with a `Timeline:` assumption naming a crates/ path and per-stage durations is accepted, and a non-speed request without a timeline is accepted unchanged.
3. [content] crates/athena-core/src/reading.rs has `check_speed_timeline` added as one `step!(check_speed_timeline(reading, request))` line in refusing_check, with a doc comment naming the root cause and jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, and stating the refusal rate is intended (expected 5% of newly refused readings).
4. [content] READING_RULES in crates/athena-context/src/briefs.rs lists the new rule and names `check_speed_timeline`, and the reader brief tells the reader what a Timeline: assumption must hold; the briefs.rs tests for the rule list pass.
5. [content] docs/decisions.md records the decision with alternatives (prompt-only guidance, planner-stage check, reading gate check) and the cause; CHANGELOG.md has an entry.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass; no existing test is removed, ignored or weakened.
7. [content] crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs are unchanged by this job.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speeding up any check, test or pipeline stage itself
- Re-doing the early pull request move of job 20261025-110525-8hzb or the nextest work of job 20261025-034733-c453
- Keeping the changes of job 20261025-190606-eebk to crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs (dropped)
- Changing athena.toml, defaults.toml or CI workflow files
- Changing planner or critic prompts

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the reading gate where speed requests are accepted, covering the class.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Corpus and unit tests, fmt, clippy and test criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by a check in refusing_check, not prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on refusal and acceptance cases.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check plus its rule line, tests and records.
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

- path: <state>/jobs/20261030-060835-ryvc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060835-ryvc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a new corpus case in crates/athena-core/tests/refusal_corpus.rs gives a speed request (e.g. 'move the call earlier to save 12 minutes') and a reading with no `Timeline:` assumption, and asserts refusing_check refuses it with `check_speed_timeline`; the same test fails if the step! line is removed.
Run `cargo test -p athena-core`: a reading of the same speed request with a `Timeline:` assumption naming a crates/ path and per-stage durations is accepted, and a non-speed request without a timeline is accepted unchanged.
crates/athena-core/src/reading.rs has `check_speed_timeline` added as one `step!(check_speed_timeline(reading, request))` line in refusing_check, with a doc comment naming the root cause and jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, and stating the refusal rate is intended (expected 5% of newly refused readings).
READING_RULES in crates/athena-context/src/briefs.rs lists the new rule and names `check_speed_timeline`, and the reader brief tells the reader what a Timeline: assumption must hold; the briefs.rs tests for the rule list pass.
docs/decisions.md records the decision with alternatives (prompt-only guidance, planner-stage check, reading gate check) and the cause; CHANGELOG.md has an entry.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass; no existing test is removed, ignored or weakened.
crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs are unchanged by this job.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
