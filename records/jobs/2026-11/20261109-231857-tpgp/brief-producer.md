# Job 20261109-231857-tpgp: Retry 20261027-233442-hyv6: drain fix request and verified reopened

## Request (verbatim)

> try again on 20261027-233442-hyv6

## Reading (accepted by the user)

Retry of job 20261027-233442-hyv6: keep its drain fix-request work (steward.rs, drain.rs, pin.rs, tests/drain.rs, CHANGELOG.md), fix its findings, and make the reopened-returns-to-Queued citation (c45) checked. The study's model, intro lead-in and glossary already say reopened returns the item to Queued, so those are verified, not rewritten.

Assumptions:
- Prior job 20261027-233442-hyv6 addressed. (a) Major c45 finding: re-cite c45 in claims.json from bin/fm-teardown.sh where the reopen path sets the row to Queued, read from the pinned source (via the study's source-reading tooling), and confirm the study validates with the pinned source, not 'conventions only'. If the pinned source is absent in this copy, the quote must be one that the checker can match, and the CHANGELOG must say what was not checked. (b) Minor duplicate-key finding: tighten the steward.rs finding marker to include the claim id or cited source path and lines, not only file and kind, so a new contradiction in the same file is queued while the same one is skipped. (c) Minor doc-comment finding: move the study_text doc comment back directly above study_text in drain.rs. (d) Minor scope finding: pin.rs and crates/athena-cli/tests/drain.rs are kept and listed as deliverables.
- Part 1 of the original request is already in the base: model.json t9 goes ready -> reopened with t12 reopened -> queued, and the intro.json glossary says the item goes back to Queued. The producer should only change model.json or intro.json if Grep for 'final state' or terminal-success finds a leftover; the lead-in in intro.json line 54 should be checked for 'Done and Reopened both end the task'.
- Code survey: read crates/athena-cli/src/drain.rs (study_text at ~3496, source check and report handling), crates/athena-core/src/steward.rs (StewardRequest builder, source_fix_request, finding_marker, citing_request) and crates/athena-core/src/pin.rs (SourceReport, SourceProblem). The new studies/ files are not records under crates/, so no check_record_file or AnyRecord change is needed.
- Goal: the drain source check stops repeating the same contradiction. Audience: the maintainer who reads drain reports and the verifier who runs cargo.
- Environment: none beyond the local repository; no CI workflow, push or history read.

## Acceptance criteria

1. [content] claims.json claim c45 cites bin/fm-teardown.sh lines where the reopen path returns the row to Queued, with a quote that is present in those lines; git diff shows the lines changed from 3894-3896 or the CHANGELOG states the check result.
2. [content] Grep for 'final state' and 'both end the task' in studies/firstmate-work-handling finds no claim that reopened is terminal; model.json shows reopened -> queued.
3. [behavior] Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study.
4. [behavior] Run `cargo test -p athena-cli` and `cargo test -p athena-core`: the drain test with a fake source check reporting a contradiction shows exactly one fix request queued through StewardRequest citing the statement and source, none on a second run while it is open, and a different contradiction in the same file is still queued.
5. [content] drain.rs has the study_text doc comment directly above `pub fn study_text`, checked with git diff.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry covering the study correction, the c45 citation and the drain fix-request queueing.

## Expected deliverables

- studies/firstmate-work-handling/claims.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)

## Out of scope

- Other studies' source checks
- Changing the source check model call itself
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the source check passes and the duplicate request is not repeated.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Drain test criterion checks queuing behavior.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-233442-hyv6 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (studies/firstmate-work-handling/claims.json:345-348): The study's only citation for the return to Queued (c45) was never checked or re-cited, and nothing shows the source check passing on this study. c45 cites bin/fm-teardown.sh lines 3894-3896 with the quote "Queued". The evidence snapshot (20261104-062715-yv4m reading-1.json:38) shows those lines were taken from a request and never read from the source. claims.json is an expected and planned deliverable but did not change. The CHANGELOG says the citation is unchecked. `cargo test -p athena-cap-study every_study_follows_the_conventions_and_the_matching_one_validates -- --nocapture` printed 'conventions only' here because the pinned source is absent from the snapshot. Until the citation is checked on the host, the repeated drain report this request is about may not be fixed, and criteria 1 and 3 stay unmet.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

- path: <state>/jobs/20261109-231857-tpgp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-231857-tpgp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

claims.json claim c45 cites bin/fm-teardown.sh lines where the reopen path returns the row to Queued, with a quote that is present in those lines; git diff shows the lines changed from 3894-3896 or the CHANGELOG states the check result.
Grep for 'final state' and 'both end the task' in studies/firstmate-work-handling finds no claim that reopened is terminal; model.json shows reopened -> queued.
Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study.
Run `cargo test -p athena-cli` and `cargo test -p athena-core`: the drain test with a fake source check reporting a contradiction shows exactly one fix request queued through StewardRequest citing the statement and source, none on a second run while it is open, and a different contradiction in the same file is still queued.
drain.rs has the study_text doc comment directly above `pub fn study_text`, checked with git diff.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has an entry covering the study correction, the c45 citation and the drain fix-request queueing.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
