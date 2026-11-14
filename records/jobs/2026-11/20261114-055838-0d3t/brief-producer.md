# Job 20261114-055838-0d3t: Audit reading checks against records; relax those without evidence

## Request (verbatim)

> From Sri (the owner), word for word: "B"
> 
> Background (not Sri's own words; added so the request can be read on its own). Sri was asked how to handle two of Athena's own fixes whose readings were refused twice each by reading checks, and chose B: "Ask Athena to review its reading rules, keeping only those that have been shown to reduce rejections."
> 
> The two blocked fixes:
> - 20261107-231406-f1bx (throughput landing clock): refused because "a behavior criterion assigns a push, pull request, GitHub API call or CI write" without a complete Environment: assumption, and because "it changes a measure computed from Athena's records" without a behavior criterion run against real records with an expected range from an independent source.
> - 20261114-023201-cdp5 (critic's "None..." text read as a misread): refused because "it adds or changes a record or data file" without naming check_record_file, AnyRecord or a round-trip test in its Code survey, and because "it changes a measure computed from Athena's records" (status/planning.rs).
> 
> What is wanted: for each check that can refuse a reading (crates/athena-core/src/reading.rs and reading/), measure from records how often it refused, how often the refused request later landed after rewording, and whether jobs since the check landed were rejected at the final verdict less often than before. Keep a check only where that evidence shows it reduced rejections; relax or remove the rest, recording each decision and its numbers. Then the two blocked fixes above should pass reading.

## Reading (accepted by the user)

Goal: for the owner (Sri) and Athena's operators, measure each reading check in crates/athena-core/src/reading.rs and reading/ (refusals, later landed after rewording, rejection rate before and after), keep only checks shown to cut rejections, relax or remove the rest, and record each decision with its numbers. Then jobs 20261107-231406-f1bx (rejected; candidate dropped, to be requeued) and 20261114-023201-cdp5 (landed; work kept, untouched) readings must pass the relaxed checks.

Assumptions:
- Code survey: entry point is the reading build in crates/athena-core/src/reading.rs (build_reading, check_reading_size), which calls the checks in crates/athena-core/src/reading/checks.rs, surveys.rs, topics.rs, terms.rs, outcome.rs and retry.rs; replay.rs replays past readings against checks. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched; stored records are only read. Refusal counts are measured directly from recorded readings and refusals, not via a stand-in.
- Environment: git log and history.json are read-only, from the local repository copy and the evidence snapshot; no push, PR, GitHub API or CI write; reaches commit times, landing records and job readings only.
- Term real records: the recorded jobs, readings, refusals and landings in the evidence snapshot, not test fixtures. Term independent source: commit times from git log or history.json, which the change does not compute.
- Decision rule: keep a check only if the records show rejections at the final verdict fell after it landed; otherwise relax or remove it. Each decision goes in a new appended decision in docs/decisions.md with a Provenance bullet and its numbers; none are estimates unless labeled.
- Job 20261107-231406-f1bx: its rejected candidate (CHANGELOG.md, status.rs, status/throughput.rs) is dropped here; its verifier findings (unseen run command, unreproduced old median) are not addressed by this job, only its reading is unblocked, and the fix is requeued afterward. Job 20261114-023201-cdp5 landed; its 24 changed files are kept and not edited.
- Reference the two blocked fixes' refusals: the Environment: rule for behavior criteria that assign a push or CI write, the records-measure rule, and the check_record_file Code survey rule. These are the main candidates for relaxing, subject to the measured evidence.

## Acceptance criteria

1. [content] docs/decisions.md gains a new appended decision (no existing entry edited) with a Provenance bullet that lists, for each check that can refuse a reading in crates/athena-core/src/reading.rs and reading/, its refusal count, how many refused requests later landed after rewording, rejection rate at final verdict before and after, and keep/relax/remove.
2. [content] Each check marked relax or remove in that decision is changed or removed in crates/athena-core/src/reading.rs or reading/*.rs; each kept check has numbers in the decision showing it reduced rejections.
3. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>`: on the real records the refusal count and landed-after-rewording count it prints per check are within the expected range taken from commit times and job counts in history.json and git log (an independent source the change does not compute); the Environment: assumption names this read-only access.
4. [behavior] Run `cargo test -p athena-core`: replaying the recorded readings of 20261107-231406-f1bx and 20261114-023201-cdp5 through the reading checks (reading/replay.rs) yields no refusal; a new test per relaxed check shows the former refused reading now passes and a kept check still refuses.
5. [behavior] The change refuses fewer readings: expected 0% of newly refused readings when past readings are replayed; the lower refusal rate is intended.
6. [content] CHANGELOG.md has an entry naming the checks relaxed or removed and the measured numbers.
7. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass, as recorded by Athena on the final tree.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/outcome.rs (modify)
- crates/athena-core/src/reading/replay.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading/terms.rs (modify)
- crates/athena-core/src/reading/retry.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Re-landing the f1bx throughput fix: crates/athena-core/src/status/throughput.rs and crates/athena-core/src/status.rs candidate changes are dropped here
- Any of the 24 files changed by landed job 20261114-023201-cdp5 (outputs.rs, planning.rs, verify.rs, prompts, display.rs, fixtures and others): kept as landed, not edited
- Editing anything under records/ or landings/
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check refusal and rejection numbers from records and the two jobs' readings passing, not that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests per relaxed check and full fmt, clippy, test criterion.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each decision cites measured numbers from records.
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

36 more qualifying classes are left out.

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

- path: <state>/jobs/20261114-055838-0d3t/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-055838-0d3t/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/decisions.md gains a new appended decision (no existing entry edited) with a Provenance bullet that lists, for each check that can refuse a reading in crates/athena-core/src/reading.rs and reading/, its refusal count, how many refused requests later landed after rewording, rejection rate at final verdict before and after, and keep/relax/remove.
Each check marked relax or remove in that decision is changed or removed in crates/athena-core/src/reading.rs or reading/*.rs; each kept check has numbers in the decision showing it reduced rejections.
Run `cargo run -p athena-cli -- status --snapshot <evidence directory>`: on the real records the refusal count and landed-after-rewording count it prints per check are within the expected range taken from commit times and job counts in history.json and git log (an independent source the change does not compute); the Environment: assumption names this read-only access.
Run `cargo test -p athena-core`: replaying the recorded readings of 20261107-231406-f1bx and 20261114-023201-cdp5 through the reading checks (reading/replay.rs) yields no refusal; a new test per relaxed check shows the former refused reading now passes and a kept check still refuses.
The change refuses fewer readings: expected 0% of newly refused readings when past readings are replayed; the lower refusal rate is intended.
CHANGELOG.md has an entry naming the checks relaxed or removed and the measured numbers.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass, as recorded by Athena on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
