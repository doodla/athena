# Job 20261111-064834-q749: Refuse readings that defer a numbered request part or Acceptance bullet

## Request (verbatim)

> try again on 20261029-212839-tcjb

## Reading (accepted by the user)

Retry of 20261029-212839-tcjb: add one part-coverage step to refusing_check that refuses a reading deferring a numbered part ((N), N., N)) or an Acceptance bullet unless the request explicitly allows a split. Fixes its two major critic findings (real Steward bullets incl. long-title 'as described'; split exemption on common words) and the minor ones, with fixtures from real Steward output and the sza7 request text.

Assumptions:
- Goal and audience: the reading gate's maintainers and later Steward jobs, which must no longer see multi-part requests silently split; the gate is judged by the corpus tests and a replay of past readings.
- Code survey: crates/athena-core/src/reading/checks.rs (refusing_check at ~line 518 chains step!(check_...) lines; new check added as one more step! line), crates/athena-core/src/reading/topics.rs (is_bullet, request_text_acceptance, parsing), crates/athena-core/src/reading.rs, crates/athena-core/src/steward.rs:620-624 (StewardRequest::from_change generates the standard bullets), crates/athena-core/tests/refusal_corpus.rs (every step must refuse some case). No record or data file is added under crates/, so check_record_file/AnyRecord are not involved. Coverage is measured directly by running the new check over reading and request text.
- Prior job 20261029-212839-tcjb failed in planning, not code: no candidate exists, so its deliverables are re-listed here and nothing is kept from it. Its major finding 1 (threshold refused the real Steward bullet, demo used invented text) is fixed by taking fixtures from steward.rs:621-623 (the three bullets: 'The change is made as described: <title>', 'No unrelated behavior changes; the existing tests pass', 'fmt, clippy and test pass') including a long title, and by capping or relaxing the stem overlap so that case is accepted. Major finding 2 (split exemption on 'split'/'separately') is fixed by counting only permission phrasing (modal plus job or part noun, e.g. 'may be split into separate jobs', '(N) can be a follow-up job') with ordinary-sense cases still refused. Minor findings: 'distinctive stem' is defined in a code comment (at least one non-stop stem of a minimum length) and pinned by a unit test; numbered lines in generator-copied cited text are excluded as check_defined_terms does; the 5% figure is labeled an estimate with its basis from the corpus Accept cases; crates/athena-agent/src/fake.rs is added only if tests need it (not expected).
- Job 20261029-111202-2bpc is the job being finished (its pause was lifted in records/interventions/2026-10/20261029-195133-tpxs.json); its critic findings (N. and N) forms; stem rules for the standard Steward bullet) are fixed here. Job 20261029-084645-sza7's candidate files stay dropped, and its request text is rebuilt from the brief because records/ is absent.
- The existing parser used by plan, steward and hestia acceptance is left unchanged; a separate request_parts parser is added. Quoted (>) lines and fenced blocks are skipped.
- Refusal rate: expected 5% of newly refused readings is an estimate until measured from the corpus Accept cases; the doc comment should state the measured figure if the replay or corpus run gives one.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a reading that moves part (3) of a request with parts (1),(2),(3) to a follow-up job is refused by the new part-coverage check, named by refusing_check; the same reading is accepted when the request explicitly permits a split (e.g. 'part (3) may be split into a separate job').
2. [behavior] Corpus cases with `1.`/`2.`/`3.` and `1)`/`2)`/`3)` requests and no Acceptance heading, where the reading defers item 3, are refused; stray single numbers, '1. Yes 2. No' lists, quoted runs and generator-copied numbered lists are not read as parts. Run `cargo test -p athena-core`.
3. [behavior] A test using the sza7 request text (parts (1)-(3) plus four Acceptance bullets, rebuilt from the brief, with a test comment saying so) refuses a reading that defers part (3) and the status bullet, and accepts a reading covering all parts and bullets. Run `cargo test -p athena-core`.
4. [behavior] A fixture built from the real Steward bullets (copied from steward.rs:621-623, or via StewardRequest::from_change, with a long title) is accepted when the reading's criteria cover them, and a refuse twin where a bullet is dropped is refused. This job's own request and accepted reading are an Accept case. Run `cargo test -p athena-core`.
5. [behavior] The split exemption fires only on permission phrasing: requests using 'split' or 'separately' in an ordinary sense ('split checks.rs into modules', 'test each form separately') still have a deferral refused, and a complaint ('the earlier reading said (3) is a follow-up; that was wrong') does not exempt it. Run `cargo test -p athena-core`.
6. [content] crates/athena-core/src/reading/checks.rs adds exactly one `step!(check_...)` line in refusing_check, a code comment defines 'distinctive stem', and the doc comment states 'expected 5% of newly refused readings' labeled as an estimate with its basis. Because the gate now asks every numbered part and Acceptance bullet be covered, newly refused past readings are intended.
7. [content] CHANGELOG.md has an entry for the part-coverage gate. If the reader rules in crates/athena-context/src/briefs.rs change, its rule-section test still passes and no prompt file is edited; athena.toml, defaults.toml, workflows and records/ are unchanged.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, and no unrelated behavior changes.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)

## Out of scope

- athena.toml, check_timeout_s or its schema, and the timeout work of jobs 20261029-110953-bw7k and 20261029-084645-sza7
- Files of job 20261029-110953-bw7k (display.rs, status.rs, checks.rs, disk.rs, run/mod.rs) and their two minor findings
- Files of job 20261029-084645-sza7's candidate (athena.toml, checks.rs, disk.rs, run/mod.rs, CHANGELOG.md): dropped, not reapplied
- Editing reader or verifier prompt files, defaults.toml, workflows or records/ (including the intervention record)
- Reopening or re-running jobs 20261029-111202-2bpc or 20261029-084645-sza7 themselves
- Planning-stage changes proposed by introspection 20261111-064803-f4da (keep best plan, repair round)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the gate refusing and accepting real-shaped readings, not wiring.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion plus corpus cases.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands.
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

- path: <state>/jobs/20261111-064834-q749/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-064834-q749/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core --test refusal_corpus`: a reading that moves part (3) of a request with parts (1),(2),(3) to a follow-up job is refused by the new part-coverage check, named by refusing_check; the same reading is accepted when the request explicitly permits a split (e.g. 'part (3) may be split into a separate job').
Corpus cases with `1.`/`2.`/`3.` and `1)`/`2)`/`3)` requests and no Acceptance heading, where the reading defers item 3, are refused; stray single numbers, '1. Yes 2. No' lists, quoted runs and generator-copied numbered lists are not read as parts. Run `cargo test -p athena-core`.
A test using the sza7 request text (parts (1)-(3) plus four Acceptance bullets, rebuilt from the brief, with a test comment saying so) refuses a reading that defers part (3) and the status bullet, and accepts a reading covering all parts and bullets. Run `cargo test -p athena-core`.
A fixture built from the real Steward bullets (copied from steward.rs:621-623, or via StewardRequest::from_change, with a long title) is accepted when the reading's criteria cover them, and a refuse twin where a bullet is dropped is refused. This job's own request and accepted reading are an Accept case. Run `cargo test -p athena-core`.
The split exemption fires only on permission phrasing: requests using 'split' or 'separately' in an ordinary sense ('split checks.rs into modules', 'test each form separately') still have a deferral refused, and a complaint ('the earlier reading said (3) is a follow-up; that was wrong') does not exempt it. Run `cargo test -p athena-core`.
crates/athena-core/src/reading/checks.rs adds exactly one `step!(check_...)` line in refusing_check, a code comment defines 'distinctive stem', and the doc comment states 'expected 5% of newly refused readings' labeled as an estimate with its basis. Because the gate now asks every numbered part and Acceptance bullet be covered, newly refused past readings are intended.
CHANGELOG.md has an entry for the part-coverage gate. If the reader rules in crates/athena-context/src/briefs.rs change, its rule-section test still passes and no prompt file is edited; athena.toml, defaults.toml, workflows and records/ are unchanged.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, and no unrelated behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
