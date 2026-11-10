# Job 20261110-124307-bn3c: Add reading-coverage check that scores each bullet by its own words

## Request (verbatim)

> try again on 20261029-111121-fzx0

## Reading (accepted by the user)

Retry of job 20261029-111121-fzx0: add a reading-acceptance check that refuses a reading omitting a request "Acceptance:" bullet (naming it) unless the request allows a split. Coverage scores each bullet by the share of its own content words found in its best-matching criterion, never discarding prose-repeated words, with a threshold tuned on recorded cases. Audience: the Athena owner and verifier.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs (build_reading, request/reading types), crates/athena-core/src/reading/checks.rs (refusing_check, one step! line per check, ending at check_reading_references), crates/athena-core/tests/refusal_corpus.rs (requires a corpus case per step! in which that check refuses). No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not touched. Coverage is measured directly by calling the check on request text and reading criteria.
- Retry of 20261029-111121-fzx0, addressed: (1) the major soundness finding is fixed by dropping the prose-word stripping and the stem-uniqueness rule; each bullet is scored against its best-matching criterion by the share of the bullet's own content words (with simple stemming that maps flagged/flag together) found there; (2) the unmet command-evidence criterion is fixed by criteria that name commands the verifier can run; (3) the job-ID minor is fixed by naming 20261029-084645-sza7 as the 2026-10-29 rejected job in the doc comment and CHANGELOG; (4) the fixture nit is fixed by using the real recorded shapes.
- Kept from the earlier candidate: all five prior deliverables, and the edits to crates/athena-agent/src/fake.rs and crates/athena-cli/tests/drain.rs, which are now listed as deliverables because the fake reader and drain helper must repeat request bullets to pass the rule; they are kept only if the suite needs them.
- Fixtures come from recorded request shapes, not invention: sza7 (the motivating omission, part 3 moved to a follow-up, must be refused naming the bullet), ttnw (criterion 'a new test with no Steward or Hestia requests queued asserts the plan order is identical…' must cover bullet 'a test: with no improvement requests queued, ordering is unchanged') and bp5e (criterion 'its doc comment names the root cause' must cover 'The root cause is named in the change description…'), both of which must be accepted. The producer copies their request and bullet text verbatim; the threshold is tuned so these three come out right.
- The measured refusal rate is obtained by replaying the new rule over the recorded requests and readings available to the producer (the three above plus any others reachable); the figure is stated in the test output, the CHANGELOG and the intended-refusal criterion, in place of the old unmeasured 10% estimate. Because records/ is absent from the reading copy, the reference of N=10 is used as the estimate until measured.
- A request allows a split only when its non-bullet text affirmatively says so; negated wording such as 'Do not split' does not count. Readings that ask a question or decline, and requests with no Acceptance heading, are not checked.
- Reader prompts are not changed (2 tokens of headroom).

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/reading/coverage.rs (or reading.rs) calls the coverage check with the recorded sza7 request and its reading that omits the duration bullet, asserts an Err naming that bullet, and asserts a reading covering every bullet returns Ok. Run: cargo test -p athena-core.
2. [behavior] A replay test loads the recorded ttnw and bp5e request/reading shapes (bullet and criterion text copied verbatim from the recorded jobs) and asserts zero refusals on them, since every bullet is covered. Run: cargo test -p athena-core coverage.
3. [behavior] A test shows prose words repeated in a bullet are still judged: a bullet whose only distinguishing words are repeated in the request prose is refused when no criterion has them, and accepted when a criterion has them. Run: cargo test -p athena-core coverage.
4. [behavior] A test shows a request that affirmatively allows a split does not refuse a reading covering fewer bullets, while 'Do not split this job' does not allow it. Run: cargo test -p athena-core coverage.
5. [behavior] The check is one step! line in refusing_check in crates/athena-core/src/reading/checks.rs, with a new case in crates/athena-core/tests/refusal_corpus.rs in which it is the refusing check. Run: cargo test -p athena-core --test refusal_corpus; confirm the step! line with Grep.
6. [behavior] Running cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
7. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 10% of newly refused readings (an estimate), replaced by the measured replay figure if the producer measures one; the replay test or its output states the measured rate over the recorded requests, and no newly refused reading that covers all its bullets exists. Run: cargo test -p athena-core -- --nocapture.
8. [content] CHANGELOG.md has an Unreleased entry naming the new check, that it refuses more readings, that the higher refusal rate is intended, and the expected percent of newly refused readings (cite the measured figure if available).
9. [content] The doc comment of the check names job 20261029-084645-sza7 (the 2026-10-29 job rejected for a split the reading chose), links check_reading_acceptance, and says a split may be proposed only as a question, never by dropping a bullet.
10. [content] Every file changed by the earlier candidate is either a deliverable here or removed: git diff shows fake.rs and drain.rs changed only if the suite needs them.

## Expected deliverables

- crates/athena-core/src/reading/coverage.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing reader prompts (the reader prompt set has only 2 tokens of headroom).
- Auto-splitting jobs or changing the small-job default beyond forbidding dropped bullets.
- Changes to athena.toml, defaults.toml or CI workflows.
- Building a general replay harness over all records beyond the recorded cases used as fixtures and the measured rate.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that coverage-complete readings are accepted and omissions refused, using recorded shapes.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and cargo test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name runnable commands.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check in one new module plus wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-111121-fzx0 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 6 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo run -q -p athena-cli -- check-reading --request <state>/jobs/20261029-111121-fzx0/evidence/records/jobs/2026-11/20261107-031902-ttnw/request.json --reading <state>/jobs/20261029-111121-fzx0/evidence/records/jobs/2026-11/20261107-031902-ttnw/reading-1.json`
> major (crates/athena-core/src/reading/coverage.rs:303-357 (specific_stems, uncovered_bullets)): Soundness: the coverage rule throws away the bullet words that the request's prose repeats, then judges coverage on the incidental words left over. As a result it wrongly refuses readings that cover every bullet, and it catches the motivating case only through an accident of word forms. I ran the candidate's check-reading on both readings the replay newly refused. ttnw is refused for 'a test: with no improvement requests queued, ordering is unchanged', yet its criterion says 'a new test with no Steward or Hestia requests queued asserts the plan order is identical to the order before the change'. bp5e is refused for 'The root cause is named in the change description…', yet its criterion says 'its doc comment names the root cause'. Both jobs' verification.json record request_reading_mismatch: null. After removing prose words, the only word left to judge was 'unchanged' for ttnw and 'description' for bp5e, and the criteria lacked it. The sza7 refusal also rests on one leftover word, 'flags', because the prose says 'flagged' and the two forms reduce to different stems ('flag', 'flagg'); this is from reading the code, not a run. Had the bullet said 'flagged', the rule would fall back to the bullet's other unique words, 'duration' would match criterion 4's 'measured duration', and the omission would be accepted. So 2 of the 2 replayed refusals are wrong, and the true catch is fragile. A better alternative: score each bullet against its best-matching criterion by the share of the bullet's own content words found there, without removing prose words, and tune the threshold on the replay set so that ttnw and bp5e are accepted and sza7 is refused.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

- path: <state>/jobs/20261110-124307-bn3c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-124307-bn3c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test in crates/athena-core/src/reading/coverage.rs (or reading.rs) calls the coverage check with the recorded sza7 request and its reading that omits the duration bullet, asserts an Err naming that bullet, and asserts a reading covering every bullet returns Ok. Run: cargo test -p athena-core.
A replay test loads the recorded ttnw and bp5e request/reading shapes (bullet and criterion text copied verbatim from the recorded jobs) and asserts zero refusals on them, since every bullet is covered. Run: cargo test -p athena-core coverage.
A test shows prose words repeated in a bullet are still judged: a bullet whose only distinguishing words are repeated in the request prose is refused when no criterion has them, and accepted when a criterion has them. Run: cargo test -p athena-core coverage.
A test shows a request that affirmatively allows a split does not refuse a reading covering fewer bullets, while 'Do not split this job' does not allow it. Run: cargo test -p athena-core coverage.
The check is one step! line in refusing_check in crates/athena-core/src/reading/checks.rs, with a new case in crates/athena-core/tests/refusal_corpus.rs in which it is the refusing check. Run: cargo test -p athena-core --test refusal_corpus; confirm the step! line with Grep.
Running cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
The change refuses more readings and the higher refusal rate is intended: expected 10% of newly refused readings (an estimate), replaced by the measured replay figure if the producer measures one; the replay test or its output states the measured rate over the recorded requests, and no newly refused reading that covers all its bullets exists. Run: cargo test -p athena-core -- --nocapture.
CHANGELOG.md has an Unreleased entry naming the new check, that it refuses more readings, that the higher refusal rate is intended, and the expected percent of newly refused readings (cite the measured figure if available).
The doc comment of the check names job 20261029-084645-sza7 (the 2026-10-29 job rejected for a split the reading chose), links check_reading_acceptance, and says a split may be proposed only as a question, never by dropping a bullet.
Every file changed by the earlier candidate is either a deliverable here or removed: git diff shows fake.rs and drain.rs changed only if the suite needs them.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
