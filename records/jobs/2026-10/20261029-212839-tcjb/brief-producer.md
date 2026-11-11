# Job 20261029-212839-tcjb: Refuse readings that defer a numbered request part or Acceptance bullet

## Request (verbatim)

> Finish the fix line for job 20261029-111202-2bpc: its scope-guard pause was lifted in records/interventions/2026-10/20261029-195133-tpxs.json. The reading gate must refuse any reading that defers a numbered request part, whether marked (N), N. or N), or any Acceptance bullet, unless the request allows a split. Address the two major critic findings recorded in that job's outcome: the parser must accept the N. and N) forms, and the stem rules must let a criterion cover the standard Steward bullet. Test the gate against the real request text of job 20261029-084645-sza7.
> 
> Acceptance:
> - The change is made as described: The reading drops or defers part of a multi-part request to a follow-up job: once caught at the final verdict, once corrected by hand
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/interventions/2026-10/20261029-195133-tpxs.json.

## Reading (accepted by the user)

Add a part-coverage check to the reading gate (a new step! in refusing_check) that refuses a reading deferring a request's numbered part, in (N), N. or N) form, or an Acceptance bullet, unless the request allows a split. This finishes job 20261029-111202-2bpc, whose pause was lifted in records/interventions/2026-10/20261029-195133-tpxs.json, and fixes its two major critic findings. It is tested against the sza7 request text, rebuilt from the brief.

Assumptions:
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance, refusing_check and its step! macro, check_governance_coverage), crates/athena-core/src/reading/topics.rs (is_bullet, request_text_acceptance) and crates/athena-core/tests/refusal_corpus.rs. The gate is refusing_check, which runs its steps in order and names the first that refuses. No part-coverage step exists yet. The change adds no record or data file under crates/, so check_record_file and AnyRecord do not apply. Coverage is measured by running the real gate on request and reading shapes in the corpus.
- Existing code: extends crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/topics.rs and crates/athena-core/tests/refusal_corpus.rs: it reuses the step! seam, is_bullet / request_text_acceptance and the corpus helpers.
- Goal and audience: the reading gate is judged by the Athena owner and by the verifier. A reading that drops or defers a requested part must be refused, as it was twice in this failure class.
- Job 20261029-111202-2bpc is retried; its candidate changed no files, so there is nothing to keep or drop. Its out_of_scope items carry over: no changes to athena.toml, no reader or verifier prompt files, and no defaults.toml, workflows or records/. Job 20261029-084645-sza7 stays rejected; its candidate files (athena.toml, checks.rs, disk.rs, run/mod.rs, CHANGELOG.md) are dropped here and used only as test text. The pause lifted in the intervention record 20261029-195133-tpxs.json is an input, and the record is not edited.
- Part forms (N), N. and N) are parsed at line start. A run counts as parts only if it numbers 1, 2, 3... from 1, has at least two entries, and each holds a distinctive word. Lettered runs (a), (b) are also accepted under the same rule, or the code comment says why they are left out. Acceptance bullets are parts too.
- Matching uses stems. The stop list is passed through the same `stems` function and includes common function words of four or more letters, such as without, before, after, into and other. The threshold is lowered for parts with three or fewer stems, so the standard Steward bullets are covered by typical criteria.
- A split is allowed only when the request itself says so, such as 'split', 'follow-up job' or 'separately'. That exemption is ignored when the same sentence has a reporting or complaint cue (said, wrong, earlier, reading, rejected).
- The sza7 request text is rebuilt from the brief, since records/ is absent in the working copy.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. A reading that moves part (3) of a request with parts (1), (2) and (3) to a follow-up job is refused by the new part-coverage check, which `refusing_check` reports by name. The same reading is accepted when the request itself allows a split.
2. [behavior] Corpus cases for `1.`/`2.`/`3.` and `1)`/`2)`/`3)` requests with no Acceptance heading, where the reading defers item 3, are refused by the part-coverage check. This answers critic finding 1 of job 20261029-111202-2bpc (N. and N) forms). Stray single numbers and quoted runs are not read as parts. Run `cargo test -p athena-core`.
3. [behavior] A test uses the sza7 request text (parts (1)-(3) plus four Acceptance bullets), rebuilt from the brief because records/ is absent. A test comment says so. The reading that defers part (3) and the status bullet is refused. A full reading that covers all parts and bullets is accepted. Run `cargo test -p athena-core`.
4. [behavior] A reading whose criteria cover the three standard Steward bullets (a test that fails before the change and passes after it; fmt, clippy and test pass; no unrelated behavior changes) is accepted. This answers critic finding 2 of job 20261029-111202-2bpc (stem rules and stop list). The stop list is built with the same `stems` function as the bullets. Run `cargo test -p athena-core`.
5. [behavior] A request that complains about an earlier reading ('the earlier reading said (3) is a follow-up; that was wrong') does not exempt part (3), so a reading that defers it is still refused. Run `cargo test -p athena-core`.
6. [content] The change adds one `step!(check_...)` line in `refusing_check` in crates/athena-core/src/reading/checks.rs, and the corpus test that requires each step to refuse some case passes. The summary or an assumption names job 20261029-111202-2bpc, says its findings are fixed, and says job 20261029-084645-sza7's candidate files stay dropped.
7. [content] Because this adds a refusal rule, the change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings. Past readings should be refused because the gate now asks that every numbered part and Acceptance bullet be covered, and a reading may not defer one unless the request allows a split.
8. [content] CHANGELOG.md has an entry for the part-coverage gate. If the reader rules text in crates/athena-context/src/briefs.rs changes, its rule-section test still passes and no reader prompt file is edited.
9. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, and no unrelated behavior changes.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- athena.toml, check_timeout_s or its schema, and the timeout work of jobs 20261029-110953-bw7k and 20261029-084645-sza7
- Files of job 20261029-110953-bw7k (display.rs, status.rs, checks.rs, disk.rs, run/mod.rs) and their two minor findings
- Files of job 20261029-084645-sza7's candidate (athena.toml, checks.rs, disk.rs, run/mod.rs, CHANGELOG.md): dropped, not reapplied
- Editing reader or verifier prompt files, defaults.toml, workflows or records/ (including the intervention record)
- Reopening or re-running jobs 20261029-111202-2bpc or 20261029-084645-sza7 themselves

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run the real gate on real request shapes, not on types or wiring.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test are an acceptance criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the shared gate for the whole class, not for the sza7 request alone.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-212839-tcjb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-212839-tcjb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core --test refusal_corpus`. A reading that moves part (3) of a request with parts (1), (2) and (3) to a follow-up job is refused by the new part-coverage check, which `refusing_check` reports by name. The same reading is accepted when the request itself allows a split.
Corpus cases for `1.`/`2.`/`3.` and `1)`/`2)`/`3)` requests with no Acceptance heading, where the reading defers item 3, are refused by the part-coverage check. This answers critic finding 1 of job 20261029-111202-2bpc (N. and N) forms). Stray single numbers and quoted runs are not read as parts. Run `cargo test -p athena-core`.
A test uses the sza7 request text (parts (1)-(3) plus four Acceptance bullets), rebuilt from the brief because records/ is absent. A test comment says so. The reading that defers part (3) and the status bullet is refused. A full reading that covers all parts and bullets is accepted. Run `cargo test -p athena-core`.
A reading whose criteria cover the three standard Steward bullets (a test that fails before the change and passes after it; fmt, clippy and test pass; no unrelated behavior changes) is accepted. This answers critic finding 2 of job 20261029-111202-2bpc (stem rules and stop list). The stop list is built with the same `stems` function as the bullets. Run `cargo test -p athena-core`.
A request that complains about an earlier reading ('the earlier reading said (3) is a follow-up; that was wrong') does not exempt part (3), so a reading that defers it is still refused. Run `cargo test -p athena-core`.
The change adds one `step!(check_...)` line in `refusing_check` in crates/athena-core/src/reading/checks.rs, and the corpus test that requires each step to refuse some case passes. The summary or an assumption names job 20261029-111202-2bpc, says its findings are fixed, and says job 20261029-084645-sza7's candidate files stay dropped.
Because this adds a refusal rule, the change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings. Past readings should be refused because the gate now asks that every numbered part and Acceptance bullet be covered, and a reading may not defer one unless the request allows a split.
CHANGELOG.md has an entry for the part-coverage gate. If the reader rules text in crates/athena-context/src/briefs.rs changes, its rule-section test still passes and no reader prompt file is edited.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, and no unrelated behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
