# Job 20261113-112836-b7sx: Refuse readings that drop or defer a part the request names

## Request (verbatim)

> try again on 20261030-060839-1njk

## Reading (accepted by the user)

Retry of job 20261030-060839-1njk, which failed in planning at round 5. Add a `check_request_parts` step, last in `refusing_check`, that refuses a reading that drops, narrows or defers a numbered, lettered or Acceptance-bullet part of the request. This time the rule is fixed up front for the request shapes that sank the plan: request-side deferral wording, alternatives and optional parts, bullet markers, Acceptance heading variants, and deliverable paths alone.

Assumptions:
- Prior result: job 20261030-060839-1njk failed in planning at round 5, with no candidate and no files changed, so nothing is kept. Its 8 findings are fixed in the design: request-side deferral uses the broad marker-negation list (criterion 4); alternatives and optional parts are grouped (criterion 5); Bullet parts have no marker and heading variants are accepted (criterion 6); a deliverable path alone no longer covers a part (criterion 3); the timeout file is named correctly (criterion 9); terms.rs is now a deliverable.
- Code survey: crates/athena-core/src/reading/checks.rs (refusing_check at ~520 runs ordered `step!` checks and returns the first refusing check name), crates/athena-core/src/reading/terms.rs (plain_request_prose at ~189), crates/athena-core/src/reading/topics.rs, crates/athena-core/src/reading/replay.rs, crates/athena-core/src/reading.rs (source-order tests split on `pub fn refusing_check(`), crates/athena-core/tests/refusal_corpus.rs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. Refusal is measured directly by calling refusing_check on fixture requests and readings.
- Existing code: crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/terms.rs and crates/athena-core/tests/refusal_corpus.rs are extended; criteria 1, 5 and 7 reuse them.
- Goal and audience: the reading stage's maintainers and the users whose requests list parts. Their readings stop deferring requested parts, and good readings are not falsely refused. Resolve ambiguity as follows: a deliverable or a criterion citing the part covers it, but a path alone does not.
- Out of scope per the earlier reading: athena.toml, defaults.toml, workflows, records/, docs/decisions.md entries, reader prompt files, verifier-side changes, the two minor findings of job 20261029-110953-bw7k, and the check-timeout code of 20261029-110953-bw7k and 20261029-084645-sza7. No governance file needs changing.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs defines `check_request_parts(reading, request)` and `refusing_check` has one `step!(check_request_parts(reading, request));` line, placed last. Its doc comment names the root cause (no comparison of request parts with deliverables and criteria, no rule that explicit parts are in scope) and cites jobs 20261029-084645-sza7, 20261029-110953-bw7k and the failed attempt 20261030-060839-1njk. It lists the known limits, including the exact Acceptance headings accepted.
2. [behavior] Refusal (request part 1): a unit test in crates/athena-core/src/reading.rs or tests/refusal_corpus.rs builds a request with parts (1), (2), (3) and a reading that leaves (3) out of its deliverables and criteria, or moves it to out_of_scope as a follow-up. It asserts `refusing_check` returns `check_request_parts` and the error says to cover the part with a deliverable and a criterion. The same reading with (3) covered is accepted. Run `cargo test -p athena-core`. The test fails with the step removed. A request Acceptance bullet that is dropped is refused too.
3. [behavior] A deliverable path alone does not cover a part (request part 1). Path words such as crates, src, core, tests, lib, mod and file extensions are ignored when matching. A corpus case has a test-file deliverable and no criterion for a dropped test part, and it is refused. Run `cargo test -p athena-core --test refusal_corpus`.
4. [behavior] Request-side deferral (part 2): a request sentence outside the part that names its marker and uses 'later', 'out of scope', 'for now', 'defer' or 'follow-up' defers it. Corpus Accept cases include 'Part (3) is out of scope for now.', 'Part 3 can come later.' and 'follow-up'. A reading that follows that wording is accepted. Run `cargo test -p athena-core --test refusal_corpus`.
5. [behavior] Alternatives and optional parts (part 3): '(a) ... or (b) ...', 'either', 'one of', 'choose' and 'pick' lists are one group covered by any member. A part starting with 'optionally', or containing 'if time permits' or 'nice to have', may be left out. Corpus Accept cases cover each, including 'Store it either as (a) a JSON file or (b) a TOML table.' with only (a) covered. Run `cargo test -p athena-core --test refusal_corpus`.
6. [behavior] Headings and markers (part 4): 'Acceptance:', 'Acceptance criteria:', '## Acceptance' and '## Acceptance criteria' all open the bullet section. Bullet parts (dash, star, and N. inside Acceptance) have marker None. Marker-cited coverage and the marker-based deferral rule apply only to enumerated parts. Corpus cases cover each heading. Run `cargo test -p athena-core --test refusal_corpus`.
7. [behavior] Placement and filtering (part 5): approved-work requests yield no parts after `## Source`, and generated and copied request text share one line filter factored out of `plain_request_prose` in crates/athena-core/src/reading/terms.rs. Existing refusals keep their check names, so the step-order source tests and the corpus self-check that every `step!` has a refusing case pass. Run `cargo test -p athena-core`.
8. [behavior] Replay (part 6): the rule runs over the recorded past requests and accepted readings, including the standard Steward bullets from steward.rs, via the replay test in crates/athena-core/src/reading/replay.rs or tests/refusal_corpus.rs. It reports zero new false refusals for accepted readings. The change refuses more readings and the higher rate is intended; expected 10% of newly refused readings, checked by `cargo test -p athena-core`.
9. [content] Jobs 20261029-110953-bw7k (landed) and 20261029-084645-sza7 (rejected) are addressed: the corpus has the sza7 shape (part 3 deferred) as a refused case and the bw7k reading, which kept all three parts, as an accepted case. The sza7 fixture's out_of_scope names crates/athena-core/src/checks.rs (the timeout file, not reading/checks.rs), display.rs, status_durations.rs, disk.rs, run/mod.rs and status.rs. `git diff` shows none of those touched.
10. [content] CHANGELOG.md has a new entry naming the root cause and the new `check_request_parts` check.
11. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as recorded by Athena's checks on the final tree.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading/terms.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading/replay.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml, workflows, records/ or docs/decisions.md entries
- Re-doing the check-timeout work of jobs 20261029-110953-bw7k (landed) and 20261029-084645-sza7 (rejected): crates/athena-cli/src/display.rs, crates/athena-cli/tests/status_durations.rs, crates/athena-core/src/disk.rs, run/mod.rs, status.rs and crates/athena-core/src/checks.rs stay as they are
- Fixing the two minor findings of job 20261029-110953-bw7k (timed-out runs absent from the status durations report; extra files outside the deliverables list)
- Editing reader prompt files
- Verifier-side changes and planner repair-round changes (the introspection's planning-stage prevention)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Corpus and unit tests plus fmt, clippy and test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run refusing_check on fixture readings.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The replay criterion checks zero false refusals on recorded shapes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix sits in refusing_check acceptance, not at one job.
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

- path: <state>/jobs/20261113-112836-b7sx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-112836-b7sx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/checks.rs defines `check_request_parts(reading, request)` and `refusing_check` has one `step!(check_request_parts(reading, request));` line, placed last. Its doc comment names the root cause (no comparison of request parts with deliverables and criteria, no rule that explicit parts are in scope) and cites jobs 20261029-084645-sza7, 20261029-110953-bw7k and the failed attempt 20261030-060839-1njk. It lists the known limits, including the exact Acceptance headings accepted.
Refusal (request part 1): a unit test in crates/athena-core/src/reading.rs or tests/refusal_corpus.rs builds a request with parts (1), (2), (3) and a reading that leaves (3) out of its deliverables and criteria, or moves it to out_of_scope as a follow-up. It asserts `refusing_check` returns `check_request_parts` and the error says to cover the part with a deliverable and a criterion. The same reading with (3) covered is accepted. Run `cargo test -p athena-core`. The test fails with the step removed. A request Acceptance bullet that is dropped is refused too.
A deliverable path alone does not cover a part (request part 1). Path words such as crates, src, core, tests, lib, mod and file extensions are ignored when matching. A corpus case has a test-file deliverable and no criterion for a dropped test part, and it is refused. Run `cargo test -p athena-core --test refusal_corpus`.
Request-side deferral (part 2): a request sentence outside the part that names its marker and uses 'later', 'out of scope', 'for now', 'defer' or 'follow-up' defers it. Corpus Accept cases include 'Part (3) is out of scope for now.', 'Part 3 can come later.' and 'follow-up'. A reading that follows that wording is accepted. Run `cargo test -p athena-core --test refusal_corpus`.
Alternatives and optional parts (part 3): '(a) ... or (b) ...', 'either', 'one of', 'choose' and 'pick' lists are one group covered by any member. A part starting with 'optionally', or containing 'if time permits' or 'nice to have', may be left out. Corpus Accept cases cover each, including 'Store it either as (a) a JSON file or (b) a TOML table.' with only (a) covered. Run `cargo test -p athena-core --test refusal_corpus`.
Headings and markers (part 4): 'Acceptance:', 'Acceptance criteria:', '## Acceptance' and '## Acceptance criteria' all open the bullet section. Bullet parts (dash, star, and N. inside Acceptance) have marker None. Marker-cited coverage and the marker-based deferral rule apply only to enumerated parts. Corpus cases cover each heading. Run `cargo test -p athena-core --test refusal_corpus`.
Placement and filtering (part 5): approved-work requests yield no parts after `## Source`, and generated and copied request text share one line filter factored out of `plain_request_prose` in crates/athena-core/src/reading/terms.rs. Existing refusals keep their check names, so the step-order source tests and the corpus self-check that every `step!` has a refusing case pass. Run `cargo test -p athena-core`.
Replay (part 6): the rule runs over the recorded past requests and accepted readings, including the standard Steward bullets from steward.rs, via the replay test in crates/athena-core/src/reading/replay.rs or tests/refusal_corpus.rs. It reports zero new false refusals for accepted readings. The change refuses more readings and the higher rate is intended; expected 10% of newly refused readings, checked by `cargo test -p athena-core`.
Jobs 20261029-110953-bw7k (landed) and 20261029-084645-sza7 (rejected) are addressed: the corpus has the sza7 shape (part 3 deferred) as a refused case and the bw7k reading, which kept all three parts, as an accepted case. The sza7 fixture's out_of_scope names crates/athena-core/src/checks.rs (the timeout file, not reading/checks.rs), display.rs, status_durations.rs, disk.rs, run/mod.rs and status.rs. `git diff` shows none of those touched.
CHANGELOG.md has a new entry naming the root cause and the new `check_request_parts` check.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as recorded by Athena's checks on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
