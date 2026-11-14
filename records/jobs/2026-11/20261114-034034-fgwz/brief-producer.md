# Job 20261114-034034-fgwz: Make Hestia re-queue a cause that recurs after its fix landed

## Request (verbatim)

> From Sri (the owner), word for word: "go with a. I'd also like some fix to Hestia or the steward that allow it to catch the issues the review found. Athena fixes go first in the queue"
> 
> Background (not Sri's own words; added so the request can be read on its own):
> 
> "a" is this plan, which Sri chose to have Athena carry out itself:
> 1. Build the queued fixes first. Hold the October feature backlog for a week. Athena runs its own queued fixes (steward and Hestia) first, starting with: the misread-field bug (critic writes "None..." in request_reading_mismatch and planning stops; steward work item 20261114-022805-ws3t, request 20261114-023201-cdp5); the batch-ordering test that keeps rejecting unrelated jobs (parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn; request 20261112-005306-pa2j); the throughput clock correction (20261107-231406-f1bx); planning stopping the whole job over a reading quibble (20261108-220029-48yj); and the two items that loop on the queue (20261017-205552-y1bh $0.05 short of budget; 20261030-060410-9e5h, reading always refused): close or fix them.
> 2. Keep planning, make it cheaper: give the plan critic the earlier rounds' findings so later rounds check they were fixed. No new refusal rules unless shown to reduce rejections.
> 3. Judge the week by: landed changes per day, dollars per landed change, and queued fixes that actually ran.
> 
> "The review" is an independent review (two outside reviewers, read-only, 14 Nov) of what Athena's self-review misses. Its findings:
> 1. Steward requests are filed but never run (52 this week, none read or dispatched); nothing measures "fix queued -> fix run".
> 2. Hestia treats a failure cause as addressed forever once any landed request cites it, even if new cases follow (hestia.rs addressing_request: the landed branch has no recency check; the closed-by-person branch does).
> 3. The throughput median uses the wrong landing timestamp (f1bx), and throughput reviews keep proposing designs from it.
> 4. Hestia sweeps report nothing while errors pile up (208 "could not delete the CI branch" errors; claims without evidence 306 -> 406).
> 5. Keep-rule verdicts (e.g. the planning keep rule, sweep 20261110-104309-kk49) produce no recorded decision.
> 6. The plan critic has no memory of earlier rounds; any major finding forces another round (37 jobs stopped after 5 rounds).
> 7. Many "refuse a reading that..." gates added (18 jobs, $81) with no fall in final-verdict rejections.
> 8. fmt, clippy and the whole test run count as evidence for behavior criteria (318 of 1,733 behavior criteria in passing verifications cite only those).
> 9. Whether CI results replace local checks depends on the reading's class label, not on changed paths; a change to the CI workflow could judge itself.
> 10. A failing test on the base rejects unrelated jobs; one introspection blamed the producer.
> 11. A candidate that makes tests hang is recorded as an environment problem and resumed unchanged.
> 12. Nothing reviews landed work; introspection runs only on rejected and failed jobs; forecasts are scored against the verifier, not later outcomes.
> 13. A landed config change (planner_revision_* keys) was refused by the running Athena; the drain logged "not landed" while outcome.json says landed; nothing was introspected.

## Reading (accepted by the user)

Reads "go with a" plus the Hestia ask as one focused change: fix finding 2, so a landed request stops addressing a failure cause once a newer introspection of that cause is recorded (as the closed-by-person branch already does), in hestia.rs. Jobs 20261114-023201-cdp5, 20261112-005306-pa2j and 20261107-231406-f1bx stay queued as separate jobs, not redone here.

Assumptions:
- Goal and audience: Sri (owner) and Hestia's repeated-cause review want recurring failures to be queued again instead of hidden behind an old landed fix; the check is the unit tests and the hestia.rs doc.
- Code survey: read crates/athena-core/src/hestia.rs. addressing_request (line ~830) is called from repeated_causes (~927) and ~3052. Its landed branch (~863) accepts any landed request that cites an introspection id, with no recency check; the closed-by-person branch compares closed_at with `newest`, the newest introspection time of the group. The property is measured directly by calling addressing_request on fixture Records. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are unaffected.
- Existing code: reuse `newest` and the `o.at` outcome time already in crates/athena-core/src/hestia.rs addressing_request; extend the landed branch with the same comparison. Update the doc comments at hestia.rs lines ~49 and ~824.
- Term ladder: not used; no structural terms in this reading.
- Previous jobs: 20261114-023201-cdp5 (misread yes/no field), 20261112-005306-pa2j (drain test) and 20261107-231406-f1bx (throughput landing clock) are queued, unverified, with no candidate and no changed files. Their work is neither kept nor dropped here: they keep their own queue entries, and this reading does not touch their files (outputs.rs, planning.rs, verify.rs, display.rs, status.rs, drain.rs, throughput.rs). Sri's 'fixes first' ordering and the other plan items (cheaper critic, week metrics, 20261017-205552-y1bh and 20261030-060410-9e5h closure) are queue or operations actions outside this single code change.
- Recurrence rule: a landed request addresses the group only if no introspection of the group has `at` later than the request's landed outcome `at`; an open request still addresses it. A request landed after the newest introspection still addresses it.
- No refusal rule is added. This only lets Hestia queue more requests for recurring causes, and that higher queue rate is intended.
- Environment: none; no CI, push or records-branch reads, tests use in-memory fixtures.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/hestia.rs shows addressing_request returns None for a cause when a landed request cites it but an introspection of the cause was recorded after the landing; run `cargo test -p athena-core hestia`.
2. [behavior] A unit test shows a landed request still addresses the cause when its landing is later than the newest introspection, and an open request still addresses it even if newer introspections exist; run `cargo test -p athena-core hestia`.
3. [behavior] Existing hestia tests (closed-by-person, repeated_causes backfill) still pass; run `cargo test -p athena-core`.
4. [content] The doc comments on addressing_request and the module doc in crates/athena-core/src/hestia.rs state that a landed request stops addressing a cause once a newer introspection is recorded.
5. [content] CHANGELOG.md has a new entry describing the Hestia recurrence fix; no other deliverables are changed beyond crates/athena-core/src/hestia.rs and CHANGELOG.md.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass (recorded Athena checks on the final tree count).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixes for jobs 20261114-023201-cdp5, 20261112-005306-pa2j and 20261107-231406-f1bx and their files (outputs.rs, planning.rs, verify.rs, display.rs, status.rs, status/throughput.rs, crates/athena-cli/tests/drain.rs); they run as their own jobs.
- Queue ordering, holding the October backlog, closing 20261017-205552-y1bh and 20261030-060410-9e5h, and the week's metrics.
- Plan critic memory of earlier rounds and any new refusal rules.
- Review findings 1 and 3-13 other than the Hestia recency check, including a steward fix-queued-to-fix-run measure and Hestia sweeps reporting error pile-ups; propose these as follow-up requests.
- athena.toml, defaults.toml, workflows, docs/decisions.md and records/.

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change (Hestia recency) rather than the whole plan and thirteen findings.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus fmt, clippy, full test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run named cargo test commands on fixture records.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261114-034034-fgwz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-034034-fgwz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/hestia.rs shows addressing_request returns None for a cause when a landed request cites it but an introspection of the cause was recorded after the landing; run `cargo test -p athena-core hestia`.
A unit test shows a landed request still addresses the cause when its landing is later than the newest introspection, and an open request still addresses it even if newer introspections exist; run `cargo test -p athena-core hestia`.
Existing hestia tests (closed-by-person, repeated_causes backfill) still pass; run `cargo test -p athena-core`.
The doc comments on addressing_request and the module doc in crates/athena-core/src/hestia.rs state that a landed request stops addressing a cause once a newer introspection is recorded.
CHANGELOG.md has a new entry describing the Hestia recurrence fix; no other deliverables are changed beyond crates/athena-core/src/hestia.rs and CHANGELOG.md.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass (recorded Athena checks on the final tree count).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
