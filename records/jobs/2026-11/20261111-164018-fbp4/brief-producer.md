# Job 20261111-164018-fbp4: Report each role's count and source in the role-need replay

## Request (verbatim)

> try again on 20261111-145400-dx3z

## Reading (accepted by the user)

Retry of job 20261111-145400-dx3z, which ended with an empty diff because the base already fixes the cause (role_need reads Records::all_invocations; the steward, audience and auditor are never retire candidates). I read the request as: confirm that, and add the one missing piece, a replay that lists each role's run count and the data source it came from, with a test over known-active roles.

Assumptions:
- Code survey: I read crates/athena-core/src/role_need.rs (RoleNeed::compute counts via Records::all_invocations; used, retire_candidates and not_measured hold only (role, count) pairs with no source; tests include known_active_roles_are_never_retire_candidates_and_an_unused_role_is and this_module_reads_usage_only_through_all_invocations) and grep'd store.rs and docs/system-facts.md. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. Retry of job 20261111-145400-dx3z: its producer found the earlier fix already in the base and changed nothing, so that fix is kept, not redone.
- Job 20261111-145400-dx3z: its earlier work (system-facts section, store.rs doc comment, on-demand roles reported as not measured, tests, CHANGELOG entry) is already present and is kept unchanged. The gap left is that the output does not say which data source each count came from. This job adds that. Files that job's candidate changed: none. Its listed docs/spec.toml, status.rs, lib.rs, store.rs, system-facts.md and shard-timings.txt are dropped as already done or unneeded.
- Goal: Athena maintainers read the role-need replay and see that the steward, audience and auditor are counted and not flagged. The measure adds no refusal rule, so there is no refused-readings share to state.
- If the producer again changes nothing, its report must say why: already done (with evidence), blocked (by what), or gave up (why).
- Existing code: crates/athena-core/src/role_need.rs and crates/athena-core/src/store.rs (Records::all_invocations) are extended, not replaced.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core role_need`: a test builds Records via from_parts with a steward, audience and auditor run that appear in no job's invocations, and asserts each is counted with a nonzero count, labelled with the source it came from (outside job records), and is not a retire candidate.
2. [behavior] Run `cargo test -p athena-core role_need`: a replay test over a list of known-active roles asserts that each role reports its count and source, that no known-active role is flagged, and that a mixed record set yields both flagged and unflagged outcomes.
3. [content] crates/athena-core/src/role_need.rs gives every role in the output a count and a data-source label, and still reads usage only through Records::all_invocations (the existing source-check test passes).
4. [content] CHANGELOG.md has an entry naming the root cause (the usage measure read a partial run source) and the per-role count and source output.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass, including the existing all_invocations_covers_every_loaded_invocation and status_role_need tests.

## Expected deliverables

- crates/athena-core/src/role_need.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_role_need.rs (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- OLY-10-02 improvement_cycle and status_build_measurement.rs (job 20261029-233058-fveq work, neither changed nor reverted)
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- A generic refusal rule or gate on new measures
- Changes to prompts, athena.toml or defaults.toml
- Reworking the producer's structured-outcome reporting (introspection 20261111-163944-gnda design change)
- docs/system-facts.md, store.rs, status.rs, lib.rs and docs/spec.toml: already done in the base, kept as is

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run the measure on mixed and known-active record sets and check the outcome.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on top of the existing fix.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 18 cases in 18 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to fail on the known bad input, here the current duplicated te...
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

- path: <state>/jobs/20261111-164018-fbp4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-164018-fbp4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core role_need`: a test builds Records via from_parts with a steward, audience and auditor run that appear in no job's invocations, and asserts each is counted with a nonzero count, labelled with the source it came from (outside job records), and is not a retire candidate.
Run `cargo test -p athena-core role_need`: a replay test over a list of known-active roles asserts that each role reports its count and source, that no known-active role is flagged, and that a mixed record set yields both flagged and unflagged outcomes.
crates/athena-core/src/role_need.rs gives every role in the output a count and a data-source label, and still reads usage only through Records::all_invocations (the existing source-check test passes).
CHANGELOG.md has an entry naming the root cause (the usage measure read a partial run source) and the per-role count and source output.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass, including the existing all_invocations_covers_every_loaded_invocation and status_role_need tests.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
