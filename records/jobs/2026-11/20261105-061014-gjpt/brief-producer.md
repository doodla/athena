# Job 20261105-061014-gjpt: Revise warm-baseline-build design note on measured CI and local paths

## Request (verbatim)

> try again on 20261104-123932-23d3

## Reading (accepted by the user)

Retry of job 20261104-123932-23d3: rewrite docs/design/warm-baseline-build.md (plus CHANGELOG entry) with an Existing code section reusing routing.rs, ci-routing.md and ci.yml, measured per-path durations and local share (October window versus November), the corrected 'mostly CI wait' claim, and only new recommendations tagged against what exists. Audience: the next throughput review and maintainers.

Assumptions:
- Retry of rejected job 20261104-123932-23d3: its findings are addressed as follows: the false 'mostly CI wait' claim is corrected from the October records; the guessed CI duration is replaced by the median of the 52 recorded CI run durations and local durations from records; the already-built recommendations (PR cache restore, test shards) are marked 'already present' or dropped; job counts are re-taken from verification.json records (50 of 58 November jobs reused CI); the note splits the review's own window, not only November. Its two changed files, docs/design/warm-baseline-build.md and CHANGELOG.md, are kept as deliverables and revised.
- Existing code: crates/athena-core/src/routing.rs, docs/design/ci-routing.md and .github/workflows/ci.yml are existing and not created by this job; the design reuses or extends them.
- Goal: a correct, evidence-based design note; audience is the next throughput review and maintainers judging the design. The request text must cite review 20261029-155931-wfkr.
- Records under records/ may be absent from the producer's copy; numbers it cannot measure are labeled estimates.

## Acceptance criteria

1. [content] The design has an Existing code section that reuses or extends crates/athena-core/src/routing.rs, docs/design/ci-routing.md and .github/workflows/ci.yml, listing each as already present (target-cache restore on pull-request runs, sharded test job, CI-result polling up to ci_wait_s, local fallback) with file and step or function names.
2. [content] The Problem section gives measured durations for the CI-reuse path (median of the recorded 'CI run <id> (<n>s)' durations in brief-verifier records) and the local-run path (from verification.json records and THROUGHPUT.md), each number with its source or an 'estimate' label, and states the local-run share both for the review window (279 landings up to 2026-10-29, mostly local) and for November (about 50 of 58 reused CI).
3. [content] The note checks the claim that the 14.1-minute median is mostly CI wait against the records, and corrects it: the October review window ran checks mostly locally, and the later bounded CI polling (job 20261101-061448-sdya) already moved the stage away from 14.1; the note says the next review's comparison to 14.1 must be split by CI versus local.
4. [content] Every recommendation is tagged 'new' or 'already present'; none re-proposes restoring the target cache on pull-request runs or splitting test across jobs, which ci.yml and ci-routing.md already do. The remaining recommendation follows the measured cause, scales the 8.0-minute prediction to the local-fallback share, and says plainly if most jobs would not benefit.
5. [content] Alternatives considered include a warm local baseline, warming or extending the CI-result path and CI target cache, shared target dir, sccache and no change, compared with reasons; the note corrects the freshness claim (stable Cargo uses file mtimes, workspace crates rebuild, only third-party deps reuse).
6. [content] The note cites review 20261029-155931-wfkr and states how the next throughput review measures the saving (median minutes per landed job over the last 20, checks stage versus 14.1 and 25.4, split by CI versus local).
7. [content] git diff shows changes only to docs/design/warm-baseline-build.md and CHANGELOG.md; rust-toolchain.toml, athena.toml, defaults.toml, Cargo.lock, crates/, xtask/ and .github/workflows/ are unchanged.

## Expected deliverables

- docs/design/warm-baseline-build.md (create)
- CHANGELOG.md (modify)

## Out of scope

- rust-toolchain.toml (read as a cache key input only)
- Implementing the warm baseline build or CI-reuse changes in crates/ or xtask/
- Changes to athena.toml, defaults.toml, Cargo.lock or .github/workflows/
- Changing docs/design/ci-routing.md or routing.rs (cited only)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every Problem-section number carries a source or an estimate label.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc plus a changelog entry.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the corrected analysis and per-path numbers, not file existence.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Recommendation rests on the measured cause, the local-fallback share.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261104-123932-23d3 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/design/warm-baseline-build.md section 2 (paragraph after the table)): The claim that the 14.1-minute median is mostly CI wait is false, and records that would have shown this were readable. Review 20261029-155931-wfkr covers 279 landings up to 2026-10-29. Among the 434 October verification.json files in the evidence snapshot, only 16 have reused:true, and only 3 of those are dated before the review (Oct 10 twice, Oct 16), plus Oct 27. The window ran checks locally for almost every job. The bounded push-run polling was added later, by job 20261101-061448-sdya on 2026-11-01. The note's '15 percent is a lower bound for older months' hides this. A better approach: report the October local share from the records, and state that the CI-reuse change, not a warm baseline, has already moved the stage away from 14.1. The next review's comparison against 14.1 depends on that.
> major (docs/design/warm-baseline-build.md section 2 table): The CI-path duration is a guess (8 to 15 minutes) and one October sample, although 52 measured CI run durations were in the records. Grep 'CI run \S+ \(\d+s\)' over records/jobs/2026-11 matches 52 brief-verifier.md files, with durations from 400 s to 690 s. The local path likewise has 8 records, but only one was used. The default 'evidence' rule says a number that could be measured should be measured. A median of the recorded durations per path would have been simple and correct.
> major (docs/design/warm-baseline-build.md section 4 'Recommendation' and section 5 item 2): The headline recommendation is to restore the target cache on pull-request branches and split test across jobs, but the CI workflow already does both. ci-routing.md line 128: 'Pull-request runs restore it and do not save it.' The target-cache restore steps in .github/workflows/ci.yml (e.g. lines 157-161, 260-264, 369-373) have no event condition, so they run on every event, including push runs on athena/ci/**. The test job uses a shard strategy (ci.yml line 344, 'test shard 0'). The note says restore on pull-request runs is something 'section 4.1 currently does not do'. That misreads the document the note cites, so the recommended next step is work already built.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading asks the note to split "the 14.1 minutes by path". The note does not split the review's 14.1-minute window. It counts November jobs and then says the 14.1-minute median "is dominated by the CI-wait path". The review's window was 279 landings up to 2026-10-29. In the evidence snapshot, only 3 October verification records dated before that review reuse a CI result (20261010-154013-a9gz, 20261010-225741-yvk0, 20261016-005805-p806; 20261027-054251-33v7 makes a fourth). Almost all of the 14.1 minutes was the local path.

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

- path: <state>/jobs/20261105-061014-gjpt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-061014-gjpt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The design has an Existing code section that reuses or extends crates/athena-core/src/routing.rs, docs/design/ci-routing.md and .github/workflows/ci.yml, listing each as already present (target-cache restore on pull-request runs, sharded test job, CI-result polling up to ci_wait_s, local fallback) with file and step or function names.
The Problem section gives measured durations for the CI-reuse path (median of the recorded 'CI run <id> (<n>s)' durations in brief-verifier records) and the local-run path (from verification.json records and THROUGHPUT.md), each number with its source or an 'estimate' label, and states the local-run share both for the review window (279 landings up to 2026-10-29, mostly local) and for November (about 50 of 58 reused CI).
The note checks the claim that the 14.1-minute median is mostly CI wait against the records, and corrects it: the October review window ran checks mostly locally, and the later bounded CI polling (job 20261101-061448-sdya) already moved the stage away from 14.1; the note says the next review's comparison to 14.1 must be split by CI versus local.
Every recommendation is tagged 'new' or 'already present'; none re-proposes restoring the target cache on pull-request runs or splitting test across jobs, which ci.yml and ci-routing.md already do. The remaining recommendation follows the measured cause, scales the 8.0-minute prediction to the local-fallback share, and says plainly if most jobs would not benefit.
Alternatives considered include a warm local baseline, warming or extending the CI-result path and CI target cache, shared target dir, sccache and no change, compared with reasons; the note corrects the freshness claim (stable Cargo uses file mtimes, workspace crates rebuild, only third-party deps reuse).
The note cites review 20261029-155931-wfkr and states how the next throughput review measures the saving (median minutes per landed job over the last 20, checks stage versus 14.1 and 25.4, split by CI versus local).
git diff shows changes only to docs/design/warm-baseline-build.md and CHANGELOG.md; rust-toolchain.toml, athena.toml, defaults.toml, Cargo.lock, crates/, xtask/ and .github/workflows/ are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
