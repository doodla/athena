# Job 20261107-031925-mrpk: Make the throughput review measure wasted work, not only landed time

## Request (verbatim)

> Make the throughput review measure wasted work, not only the time of work that landed.
> 
> The throughput review measures minutes per landed job. On 4 to 6 Nov, 58% of attempts did not land and it could not see them; four of its seven runs did nothing because an earlier design request was still open.
> 
> Change the review so that:
> - it reports attempts per landed change, money spent on attempts that did not land, and the share of changes that landed on the first attempt, over the newest 20 landings and the 20 before them
> - it triggers on a rise in any of these, as well as on the median time
> - it does not skip a run because an earlier design request is open; it notes the open request instead
> - it only queues a request when the change is larger than the noise between recent windows, and says so
> 
> Acceptance:
> - a test: a window where half of attempts did not land reports attempts per landing of 2 and a first-attempt share of 0.5 from sample records
> - a test: a rise in wasted spend triggers a review with no change in median time
> - a test: an open earlier design request no longer stops the review
> - athena status shows the three measures
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the throughput review in Hestia, status display, tests, CHANGELOG.md.

## Reading (accepted by the user)

Add attempts per landed change, spend on attempts that did not land, and first-attempt share to the throughput section (newest 20 landings vs the 20 before). Hestia's throughput review triggers on a rise in any of them, notes rather than skips on an open design request, and queues a request only when the change exceeds recent-window noise. Audience: the maintainer reading `athena status` and the review.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs: `throughput_due(landed, median, latest)` (~line 1341) is the trigger, called from the throughput review in `Athena::review_passes` (~line 1711); right after it, `open_throughput_request` (~line 1728) currently stops the review when a design request is open, so the order is: trigger check, then the open-request skip, then the model proposal, then slowdown/stage growth. Read crates/athena-core/src/status.rs: `Throughput::compute(&Records)` (~line 984) builds the section over `THROUGHPUT_WINDOW` = 20 newest landed jobs, already tracks `unlanded_jobs`/`unlanded_minutes`, and `Throughput::text()` is what `athena status` prints via crates/athena-cli/src/display.rs. The new measures are computed directly from job/attempt records and outcome totals (cost) in `Throughput::compute`, not through the minutes stand-in. Read crates/athena-data/src/records.rs: `ThroughputReview` (~line 2188) and `ThroughputTrigger` (Cadence, Worsened) are accepted by `AnyRecord::parse` / `check_record_file` (line 3111) via `AnyRecord::ThroughputReview(r) => r.validate()`; the change adds fields with serde defaults (so old records still parse) and new trigger variants, and is covered by `ThroughputReview::validate` plus a round-trip test through `AnyRecord::parse` in records.rs tests. No new record file type or allow-list entry is added.
- Prior-window comparison reuses the previous review's stored measures (new defaulted fields on ThroughputReview) or recomputes the 20-before window from records; the producer picks the simpler one.
- Noise is estimated as the spread (e.g. max-min or median absolute deviation) of the measure across recent 20-landing windows; a request is queued only when the change exceeds it, and the review note says whether it did.
- Wasted spend = cost of attempts on jobs/attempts that did not land, from outcome totals; first-attempt share = landed changes with exactly one attempt / landed changes.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new status.rs test builds sample records where half of attempts in the window did not land and asserts attempts per landing == 2.0 and first-attempt share == 0.5.
2. [behavior] Run `cargo test -p athena-core`: a new hestia.rs test shows `throughput_due` (or its successor) returns a trigger when wasted spend rises beyond the previous review and noise while median minutes is unchanged; the same inputs without the rise return None.
3. [behavior] Run `cargo test -p athena-core`: a new hestia.rs test shows an open earlier design request no longer stops the review: the review runs, and its record note mentions the open request.
4. [behavior] Run `cargo test -p athena-core`: a test shows a change within the noise between recent windows queues no request and the note says it was within noise, while a larger change queues one.
5. [behavior] Run `cargo test -p athena-cli` and `cargo run -- status` (or the status_snapshot test): the throughput output shows attempts per landed change, money spent on attempts that did not land, and first-attempt share.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass; an old ThroughputReview JSON without the new fields still parses via `AnyRecord::parse`, and a new one round-trips.
7. [content] CHANGELOG.md has entries naming only what the diff does (the three measures, new triggers, open-request note, noise rule); no unrelated claims.
8. [content] The doc comment of hestia.rs and the Throughput review section of docs/design/hestia.md no longer say an open design request skips the review (Grep for the old wording finds none).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)
- crates/athena-core/src/snapshot.rs (modify)

## Out of scope

- Changing the slowdown request's 25% rule or the stage-growth comparison beyond what the noise rule needs.
- Changing athena.toml, defaults.toml, workflows or prompt files.
- Rewriting existing records under records/.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each behavior; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests asserting values 2.0 and 0.5, triggers and no-skip.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Status output exercised via test or cargo run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the review, status and record.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks that wasted work is seen and triggers the review.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for outside readers.
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

- path: <state>/jobs/20261107-031925-mrpk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-031925-mrpk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a new status.rs test builds sample records where half of attempts in the window did not land and asserts attempts per landing == 2.0 and first-attempt share == 0.5.
Run `cargo test -p athena-core`: a new hestia.rs test shows `throughput_due` (or its successor) returns a trigger when wasted spend rises beyond the previous review and noise while median minutes is unchanged; the same inputs without the rise return None.
Run `cargo test -p athena-core`: a new hestia.rs test shows an open earlier design request no longer stops the review: the review runs, and its record note mentions the open request.
Run `cargo test -p athena-core`: a test shows a change within the noise between recent windows queues no request and the note says it was within noise, while a larger change queues one.
Run `cargo test -p athena-cli` and `cargo run -- status` (or the status_snapshot test): the throughput output shows attempts per landed change, money spent on attempts that did not land, and first-attempt share.
Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass; an old ThroughputReview JSON without the new fields still parses via `AnyRecord::parse`, and a new one round-trips.
CHANGELOG.md has entries naming only what the diff does (the three measures, new triggers, open-request note, noise rule); no unrelated claims.
The doc comment of hestia.rs and the Throughput review section of docs/design/hestia.md no longer say an open design request skips the review (Grep for the old wording finds none).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
