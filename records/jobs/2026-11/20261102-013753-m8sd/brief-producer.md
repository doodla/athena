# Job 20261102-013753-m8sd: Check each criterion against D43 and append-only rules at reading

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Athena builds criteria from the request text and does not test each one against the decisions that g
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: criterion-contradicts-athena-default-or-invariant (20261023-183245-dd7r)
> Cause: Athena builds criteria from the request text and does not test each one against the decisions that govern how criteria are judged. D43 says what kind of evidence a behavior criterion may use. Nothing makes the reading stage check a criterio...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261024-045710-yn3g (job 20261024-045548-7qht); prevention it proposed: Add a check at reading acceptance. For every criterion it reads the declared type (behavior or content) and the named evidence. It compares them with the rules from D43 and the other criterion-judging defaults. It rejects a reading, with th...
> Introspection 20261102-013736-k4jv (job 20261102-005332-8gkv); prevention it proposed: At reading acceptance, run a code check that scans criteria and deliverables for edits to append-only files (docs/decisions.md existing entries, records/) and refuses or rewrites them as appending a new decision that supersedes or amends D8...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reading stage fixes criteria from the request text, and no check compares them with the defaults that govern judging (D43 evidence rules, append-only decisions and records). Add one reading-acceptance check, registered as a `step!` in `refusing_check`, that refuses a behavior criterion whose evidence the verifier cannot produce and a criterion or deliverable that edits existing docs/decisions.md entries or records/ in place. Prior jobs 20261024-045548-7qht and 20261102-005332-8gkv are the two instances.

Assumptions:
- Goal and audience: Athena's reading gate, judged by maintainers and by the verifier that later judges each criterion; prevent criteria that contradict governing defaults from being accepted.
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance and refusing_check run the checks in order through step! lines), crates/athena-core/src/reading/surveys.rs (check_criteria_evidence, the model for a criterion-text check), crates/athena-core/tests/refusal_corpus.rs (needs a case where the new check refuses) and crates/athena-context/src/briefs/reader.rs (reading rules list). The new check adds no record or data file, so check_record_file, AnyRecord and round-trip tests are not involved. The property is measured directly by calling the check on built readings.
- Existing code: extend crates/athena-core/src/reading/checks.rs and crates/athena-core/src/reading/surveys.rs; reuse the producer_evidence_phrase style of phrase matching; D43 in docs/decisions.md and the append-only guard in crates/athena-cli/src/decisions.rs define the rules mirrored.
- Rules enforced: (1) a behavior criterion must not require a command the verifier cannot run (cargo fmt without --check, git commit/push, cargo install, environment variables); (2) no criterion or modify deliverable may edit existing entries of docs/decisions.md in place or any file under records/; the error says to append a new decision that cites or amends the old one. Matching is on criterion text and deliverable paths.
- The new check name is added to the reader rules list in crates/athena-context/src/briefs/reader.rs; the reader prompt is at 1998 of 2000 tokens, so an existing rule line is shortened rather than grown. docs/decisions.md gets a new appended entry only (no in-place edits), with docs/decision-links.toml updated if the repo's link test requires it.
- Environment: none; no CI, push or records read. Only local cargo commands.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a unit test builds readings with (a) a behavior criterion requiring `cargo fmt` without --check, and (b) a criterion editing an existing D87 entry in place in docs/decisions.md; each is refused by the new check, naming the criterion; it fails with the check removed and passes with it.
2. [behavior] Run `cargo test -p athena-core`: tests show that criteria such as 'cargo test passes', 'cargo fmt --check passes' and a deliverable that only appends a new decision to docs/decisions.md are still accepted.
3. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a case where the new check is the refusing one, and the test that every step! line has such a case passes.
4. [content] Grep crates/athena-core/src/reading/checks.rs shows a `step!(...)` line for the new check inside refusing_check, with a doc comment that names the root cause (criteria are not tested against D43 and the append-only rules) and jobs 20261024-045548-7qht and 20261102-005332-8gkv.
5. [content] The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings.
6. [content] The reader rules in crates/athena-context/src/briefs/reader.rs list the new check by name, and the brief token budget test still passes.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked` and `cargo test --workspace --locked`; all pass, including the decisions append-only guard.
8. [content] CHANGELOG.md has an entry, and docs/decisions.md changes only by appending new text at the end.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing verify.rs D43 judging itself (already amended by D115 in job 20261024-045548-7qht; its files verify.rs, run.rs, decisions.md, decision-links.toml and CHANGELOG.md are kept as landed)
- Job 20261102-005332-8gkv: its candidate (gh client, landing, transport, watcher, records.rs, tests) was rejected and is dropped here; its blocker (in-place D87 edit) is the class prevented, not re-implemented
- Editing any existing decision entry or any file under records/
- Changes to athena.toml, defaults.toml or .github/workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run tests that fail without the check and pass with it.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed at the reading gate where criteria are first fixed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 20 cases in 20 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the verifier, at reading acceptance and again after the producer finishes. It would (1) validate every measure label a job adds or edits in docs/spec.toml against STATUS_MEASURE_LABELS by running the real-spec test; (2) require each new behavior test to fail on the ...
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

- path: <state>/jobs/20261102-013753-m8sd/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261102-013753-m8sd/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a unit test builds readings with (a) a behavior criterion requiring `cargo fmt` without --check, and (b) a criterion editing an existing D87 entry in place in docs/decisions.md; each is refused by the new check, naming the criterion; it fails with the check removed and passes with it.
Run `cargo test -p athena-core`: tests show that criteria such as 'cargo test passes', 'cargo fmt --check passes' and a deliverable that only appends a new decision to docs/decisions.md are still accepted.
Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a case where the new check is the refusing one, and the test that every step! line has such a case passes.
Grep crates/athena-core/src/reading/checks.rs shows a `step!(...)` line for the new check inside refusing_check, with a doc comment that names the root cause (criteria are not tested against D43 and the append-only rules) and jobs 20261024-045548-7qht and 20261102-005332-8gkv.
The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings.
The reader rules in crates/athena-context/src/briefs/reader.rs list the new check by name, and the brief token budget test still passes.
Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked` and `cargo test --workspace --locked`; all pass, including the decisions append-only guard.
CHANGELOG.md has an entry, and docs/decisions.md changes only by appending new text at the end.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
