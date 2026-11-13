# Job 20261030-060839-1njk: Refuse readings that drop a part the request names

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading stage does not compare its list of deliverables with the parts the request names, so not
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-defers-requested-part-to-follow-up (20261029-111116-8ade)
> Cause: The reading stage does not compare its list of deliverables with the parts the request names, so nothing catches a part that goes missing. It also has no rule that a request's explicit parts are in scope for the job unless the user agrees t...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261029-111106-c3pq (job 20261029-110953-bw7k); prevention it proposed: Before a reading is accepted, run a check that extracts the numbered or lettered parts of the request, such as (1), (2) and (3). It requires each part to map to at least one deliverable and one acceptance criterion in the reading. It refuse...
> Introspection 20261029-111141-1661 (job 20261029-084645-sza7); prevention it proposed: Add a coverage check to reading acceptance. It extracts the request's acceptance bullets and requires every one to map to at least one reading criterion. A reading that defers, drops or narrows a bullet is refused, unless the request itself...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: reading acceptance (`refusing_check` in crates/athena-core/src/reading/checks.rs) never compares the request's numbered or lettered parts with the reading's deliverables and criteria, and nothing says an explicit part stays in scope unless the request defers it. Add a `check_request_parts` step that refuses a reading which defers, drops or narrows a part. This addresses jobs 20261029-084645-sza7 and 20261029-110953-bw7k, whose request reading deferred part (3).

Assumptions:
- Goal and audience: the reader stage and the maintainers who judge readings. The goal is that a request's explicit parts cannot go missing from an accepted reading.
- Code survey: crates/athena-core/src/reading/checks.rs (`check_reading_acceptance` calls `refusing_check`, whose ordered `step!` list is the gate; `check_governance_coverage`, `check_scope_limit`), crates/athena-core/src/reading/topics.rs (`request_text_acceptance`, `is_bullet` parse bullets and `N.` lines only), crates/athena-core/src/reading/outcome.rs (`request_words`), crates/athena-core/tests/refusal_corpus.rs (real request shapes, number-dot/number-paren/letter-paren parts, asserts which check refuses). No existing parts-coverage check was found in crates/. No record or data file is added, so `check_record_file`, `AnyRecord` and the round-trip test are not touched. The property is measured directly by calling `refusing_check` on real request and reading shapes.
- Existing code: crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/topics.rs and crates/athena-core/tests/refusal_corpus.rs. The criteria extend `refusing_check` and the corpus, and reuse `request_text_acceptance`/`is_bullet` for bullets.
- Match rule: a part is covered when its significant words (stems, stop list as in the existing checks) appear in a deliverable path or a criterion. A part named only in out_of_scope counts as deferred, unless the request text itself defers it.
- The prompts are not edited, because the reader prompt has 2 tokens of headroom. The rule is enforced by mechanism in code.
- Consumers: none found. No config file is changed.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs defines `check_request_parts(reading, request)` and `refusing_check` has one `step!(check_request_parts(reading, request));` line. Its doc comment names the root cause (no comparison of request parts with deliverables and criteria, no rule that explicit parts are in scope) and cites jobs 20261029-084645-sza7 and 20261029-110953-bw7k.
2. [behavior] A unit test in crates/athena-core/src/reading.rs or tests/refusal_corpus.rs builds a request with parts (1), (2), (3) and a reading that leaves (3) out of its deliverables and criteria or moves it to out_of_scope as a follow-up. It asserts `refusing_check` returns `check_request_parts`, and the same reading with (3) covered is accepted. Run `cargo test -p athena-core`. The test fails with the step removed.
3. [behavior] The corpus in crates/athena-core/tests/refusal_corpus.rs gains cases for `(N)`, `N.`, `N)` and lettered `a)` parts, and for request acceptance bullets. They cover a part deferred, dropped or narrowed, and a part the request itself defers (accepted). The corpus self-check that every `step!` has a refusing case passes under `cargo test -p athena-core --test refusal_corpus`.
4. [behavior] A reading that moves a request part to a follow-up is refused unless the request text itself defers it, for example 'later' or 'follow-up'. The error names the part and says to cover it with a deliverable and a criterion. Check with `cargo test -p athena-core`.
5. [behavior] Refusal rate: the change refuses more readings and the higher rate is intended; expected 10% of newly refused readings. This is checked by the replay test of past readings in the athena-core tests, run with `cargo test -p athena-core`.
6. [content] Jobs 20261029-110953-bw7k and 20261029-084645-sza7 are addressed. The corpus carries the sza7 shape as a refused case (part 3 deferred), and the bw7k reading, which kept all three parts, as an accepted case. Their candidate files (athena.toml, display.rs, status_durations.rs, checks.rs, disk.rs, run/mod.rs, status.rs) are named in out_of_scope as already landed or rejected and not touched.
7. [content] CHANGELOG.md has a new entry naming the root cause and the new check.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as recorded by Athena's checks on the final tree.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml, workflows, records/ or docs/decisions.md entries
- Re-doing the check-timeout work of jobs 20261029-110953-bw7k (landed) and 20261029-084645-sza7 (rejected): crates/athena-cli/src/display.rs, crates/athena-cli/tests/status_durations.rs, crates/athena-core/src/disk.rs, run/mod.rs, status.rs and checks.rs timeout code stay as they are
- Fixing the two minor findings of job 20261029-110953-bw7k (timed-out runs absent from the status durations report; extra files outside the deliverables list); they do not bear on this cause
- Editing reader prompt files
- Verifier-side changes

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion runs a refusal test on the real sza7 shape, so it checks the behavior and not the wiring.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and corpus tests; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the reading acceptance gate, not in one job's reading.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a code check, not prose.
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

- path: <state>/jobs/20261030-060839-1njk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060839-1njk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/checks.rs defines `check_request_parts(reading, request)` and `refusing_check` has one `step!(check_request_parts(reading, request));` line. Its doc comment names the root cause (no comparison of request parts with deliverables and criteria, no rule that explicit parts are in scope) and cites jobs 20261029-084645-sza7 and 20261029-110953-bw7k.
A unit test in crates/athena-core/src/reading.rs or tests/refusal_corpus.rs builds a request with parts (1), (2), (3) and a reading that leaves (3) out of its deliverables and criteria or moves it to out_of_scope as a follow-up. It asserts `refusing_check` returns `check_request_parts`, and the same reading with (3) covered is accepted. Run `cargo test -p athena-core`. The test fails with the step removed.
The corpus in crates/athena-core/tests/refusal_corpus.rs gains cases for `(N)`, `N.`, `N)` and lettered `a)` parts, and for request acceptance bullets. They cover a part deferred, dropped or narrowed, and a part the request itself defers (accepted). The corpus self-check that every `step!` has a refusing case passes under `cargo test -p athena-core --test refusal_corpus`.
A reading that moves a request part to a follow-up is refused unless the request text itself defers it, for example 'later' or 'follow-up'. The error names the part and says to cover it with a deliverable and a criterion. Check with `cargo test -p athena-core`.
Refusal rate: the change refuses more readings and the higher rate is intended; expected 10% of newly refused readings. This is checked by the replay test of past readings in the athena-core tests, run with `cargo test -p athena-core`.
Jobs 20261029-110953-bw7k and 20261029-084645-sza7 are addressed. The corpus carries the sza7 shape as a refused case (part 3 deferred), and the bw7k reading, which kept all three parts, as an accepted case. Their candidate files (athena.toml, display.rs, status_durations.rs, checks.rs, disk.rs, run/mod.rs, status.rs) are named in out_of_scope as already landed or rejected and not touched.
CHANGELOG.md has a new entry naming the root cause and the new check.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as recorded by Athena's checks on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
