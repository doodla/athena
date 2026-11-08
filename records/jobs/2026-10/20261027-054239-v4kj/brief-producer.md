# Job 20261027-054239-v4kj: Require a stated check method for subjective-quality outcomes

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The system has no default for how to verify subjective quality such as how clear a page is to a newc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: acceptance-method-unspecified-needs-user-answer (20261014-163954-85d6)
> Cause: The system has no default for how to verify subjective quality such as how clear a page is to a newcomer. Requests state the outcome without a verification method. Athena's reading step does not require an acceptance check for such outcomes...
> Seen 3 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261014-163945-t9c3 (job 20261014-163844-c2kt); prevention it proposed: Add a standing default: when a deliverable is a page or document whose quality is subjective, the acceptance check is an independent model reader that had no part in building it. It reads as a newcomer, rates the page 1 to 5, and blocks shi...
> Introspection 20261014-165951-resy (job 20261014-163844-c2kt); prevention it proposed: When a reading's acceptance criterion depends on a subjective or model-judged rating, admission should require the reading to state the check as a rubric before the build starts. It should name the reader persona and prompt, the rating dime...
> Introspection 20261015-064256-fmsv (job 20261015-064153-wekz); prevention it proposed: Add defaults that apply when a request has a quality outcome or a loop. (1) The outcome is the check behind the work, not the activity. Audience-facing quality is judged by a fresh model reader, with no part in building the work, who rates ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reading gate (`refusing_check` in crates/athena-core/src/reading/checks.rs) accepts a reading whose outcome is subjective quality, such as a page being clear to a newcomer, with no criterion saying how that is judged. This produced jobs 20261014-163844-c2kt and 20261015-064153-wekz, whose verifiers then failed or asked. I add a new `check_subjective_method` step that refuses such a reading until a criterion names an independent model reader, a persona, a 1–5 rating and a pass threshold, so the whole class is prevented at reading time.

Assumptions:
- Goal and audience: the owner and the verifier of future jobs. A reading that states a subjective outcome must also state how it is checked, so no job ends in 'needs user answer' or a verifier failure on an unverifiable outcome.
- Code survey: read crates/athena-core/src/reading/checks.rs (`refusing_check` runs ordered `step!` checks; a new check is one step line), crates/athena-core/src/reading/surveys.rs (`check_criteria_evidence` shows the shape of a check, error text and tests), crates/athena-core/tests/refusal_corpus.rs (every step needs a corpus case in which it refuses), and crates/athena-context/src/briefs.rs (reading rules list each check). No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test do not apply. The property is measured directly by calling `check_subjective_method` on a built Reading and through `refusing_check`.
- Existing code: extends crates/athena-core/src/reading/surveys.rs and crates/athena-core/src/reading/checks.rs; reuses `has_phrase` and the `reading_with_criterion` test helper in crates/athena-core/src/reading.rs.
- The check triggers on a fixed lowercase whole-word list in the request or summary (for example newcomer, clear, readable, understandable, useful, easy to follow, quality of a page or explainer). It is satisfied by one criterion that has an independent or fresh reader/model, a rating scale and a pass threshold. The corpus tests tune this list.
- The reader prompt has 3 tokens of headroom, so no prompt or defaults.toml text is added; the mechanism is the check and its error message. The brief's reading-rules line in briefs.rs may gain one line, if its tests allow.
- Governance: athena.toml, defaults.toml and CI workflows are not changed; no criterion needs them.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/surveys.rs defines `pub fn check_subjective_method`, with a doc comment that names the root cause (no required verification method for subjective quality) and jobs 20261014-163844-c2kt and 20261015-064153-wekz; Grep for `fn check_subjective_method` finds it.
2. [content] crates/athena-core/src/reading/checks.rs `refusing_check` has exactly one new line `step!(check_subjective_method(...))`, and the check_reading_acceptance doc comment lists it.
3. [behavior] Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading whose request or summary says a page must be clear to a newcomer and whose criteria give no method; `check_subjective_method` refuses it and the error names the missing independent reader, rating and threshold. The same test fails if the new step is removed from `refusing_check`.
4. [behavior] Run `cargo test -p athena-core`: a unit test shows a reading with the same subjective outcome and a criterion naming an independent model reader that had no part in building it, a newcomer persona, a 1 to 5 rating and a minimum score is accepted; a reading with no subjective wording (for example a plain refactor) is unaffected.
5. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus has new cases built from the shapes of jobs 20261014-163844-c2kt and 20261015-064153-wekz, expecting `Expected::Refuse("check_subjective_method")`, and at least one accepted counterpart; the corpus test that every step has a refusing case passes.
6. [content] Expected 5% of newly refused readings: the rule refuses more readings and the higher refusal rate is intended, but it triggers only on subjective-quality wording, so it must not refuse readings of ordinary code requests.
7. [content] Job 20261014-163844-c2kt's findings are addressed by moving the fix to admission: its newcomer.rs gate, prompt asset and ship-path wiring are dropped and not redone here (see out_of_scope). Job 20261015-064153-wekz's findings about `athena land --pr` and retry budget are dropped as a different cause. The check's error text tells the reader to state the method before the build.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root: all pass.
9. [content] CHANGELOG.md has an entry naming the root cause and the new check; Grep for `check_subjective_method` in CHANGELOG.md matches.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Job 20261014-163844-c2kt candidate files are dropped, not kept: CHANGELOG.md newcomer entry, Cargo.lock, crates/athena-cap-study/Cargo.toml, crates/athena-cap-study/prompts/newcomer.md, crates/athena-cap-study/src/lib.rs, crates/athena-cap-study/src/newcomer.rs, docs/architecture.md, docs/design/explainer-site.md. Wiring a newcomer gate into the pages ship path.
- Job 20261015-064153-wekz candidate files are dropped, not kept: README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/steward.rs, docs/decisions.md. Drain landing through `athena land --pr`.
- Changes to athena.toml, defaults.toml, CI workflows, lint configuration or prompt files
- Rewriting the firstmate explainer or calibrating a model reader against human ratings

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the reading gate where the missing method originates, not at either job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced as a refusing check in `refusing_check`, not prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and refusal-corpus cases; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test cases that show refusal and acceptance.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The new check requires the outcome's check to be named.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, its tests and a changelog entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment names the root cause and the jobs.
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
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

- path: <state>/jobs/20261027-054239-v4kj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054239-v4kj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/surveys.rs defines `pub fn check_subjective_method`, with a doc comment that names the root cause (no required verification method for subjective quality) and jobs 20261014-163844-c2kt and 20261015-064153-wekz; Grep for `fn check_subjective_method` finds it.
crates/athena-core/src/reading/checks.rs `refusing_check` has exactly one new line `step!(check_subjective_method(...))`, and the check_reading_acceptance doc comment lists it.
Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading whose request or summary says a page must be clear to a newcomer and whose criteria give no method; `check_subjective_method` refuses it and the error names the missing independent reader, rating and threshold. The same test fails if the new step is removed from `refusing_check`.
Run `cargo test -p athena-core`: a unit test shows a reading with the same subjective outcome and a criterion naming an independent model reader that had no part in building it, a newcomer persona, a 1 to 5 rating and a minimum score is accepted; a reading with no subjective wording (for example a plain refactor) is unaffected.
Run `cargo test -p athena-core --test refusal_corpus`: the corpus has new cases built from the shapes of jobs 20261014-163844-c2kt and 20261015-064153-wekz, expecting `Expected::Refuse("check_subjective_method")`, and at least one accepted counterpart; the corpus test that every step has a refusing case passes.
Expected 5% of newly refused readings: the rule refuses more readings and the higher refusal rate is intended, but it triggers only on subjective-quality wording, so it must not refuse readings of ordinary code requests.
Job 20261014-163844-c2kt's findings are addressed by moving the fix to admission: its newcomer.rs gate, prompt asset and ship-path wiring are dropped and not redone here (see out_of_scope). Job 20261015-064153-wekz's findings about `athena land --pr` and retry budget are dropped as a different cause. The check's error text tells the reader to state the method before the build.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root: all pass.
CHANGELOG.md has an entry naming the root cause and the new check; Grep for `check_subjective_method` in CHANGELOG.md matches.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
