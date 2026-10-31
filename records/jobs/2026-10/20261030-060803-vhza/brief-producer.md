# Job 20261030-060803-vhza: Find why the steward's decisions source is empty and fix or document it

## Request (verbatim)

> Priority: normal
> 
> Make the decisions source readable to the steward
> 
> The steward could not read decisions: Source 'decisions' is empty in this copy: records/decision-states and records/default-expectations contain no files, though introspection 20261029-135005-vb9w cites recorded decision D84. Find out why, and make it readable, or record why it cannot be.
> 
> Acceptance:
> - The cause of the failure to read decisions is found and written down
> - decisions is readable by the steward, or the notes say why it cannot be
> - fmt, clippy and test pass
> 
> Files: records/decision-states, records/default-expectations

## Reading (accepted by the user)

The steward's `decisions` source (defined in crates/athena-core/src/steward.rs) reported empty. The job finds the cause, writes it down, and makes decisions readable to the steward, or records why it cannot be. Audience: the steward and the maintainer reading its introspections.

Assumptions:
- Code survey: crates/athena-core/src/steward.rs (SOURCES lists `decisions` with two record directories; GitView::workspace writes every file from RecordSet::load, then creates any missing source directory as empty), crates/athena-gate/src/records.rs (RecordSet::load copies every blob under records/ at HEAD after check_record_file), crates/athena-data/src/records.rs (AnyRecord paths for decision-state and default-expectations records; round-trip tests around lines 3372 and 3405), crates/athena-core/src/store.rs (Records::load). Order: load records at HEAD, write files, then mkdir missing source dirs, so an empty source means nothing is committed under those paths at HEAD, not a copy bug.
- Those two record directories do not exist in the snapshot I read (records/ is hidden from readers by design), so I could not confirm directly that no decision records are committed. Working hypothesis: no decision-state or default-expectations records are committed at the base commit (they are written but not landed, or D84 lives in docs/design only). The producer must verify this with git log/show on the record paths and with Grep over crates/ for the writers (Athena::record_default_expectations, decision-state writer) before choosing a fix.
- If the cause is a missing writer or an unlanded record, the fix is in code (steward workspace or the writer). If decisions are legitimately empty, the fix is to make the steward's empty-source message say so and to record that in docs. No new record file is added under crates/, so no AnyRecord change is expected.
- The user's stated outcome is the goal: reading decisions works or the notes say why not. The cause note goes in CHANGELOG.md and, if useful, docs/records.md.

## Acceptance criteria

1. [content] The change writes down the cause of the empty `decisions` source in a committed file (CHANGELOG.md or docs/records.md), citing the code path (crates/athena-core/src/steward.rs and crates/athena-gate/src/records.rs) that produces the empty directory. A reader can check this with git diff.
2. [content] Either the diff changes crates/athena-core/src/steward.rs (or the record writer) so the steward's workspace holds decision records when they exist at HEAD, with a test in the same crate that builds a view containing a decision-state record and asserts the file appears under the decisions source paths, or the committed notes state why decisions cannot be read and what would change that.
3. [behavior] A test that the decisions source is populated when a decision-state record exists in the view and stays an empty directory when none exists; run `cargo test -p athena-core steward` and it passes.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the change.
5. [content] No file under records/ is edited, and no governance file (athena.toml, defaults.toml) is changed.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)
- docs/records.md (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-gate/src/records.rs (modify)

## Out of scope

- Editing or fabricating files under records/
- Changing athena.toml or defaults.toml
- Redesigning the other steward sources (jobs, events, introspections, class-registry, git-log, code)
- Changing the decision-state or default-expectations record schemas

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Any code change is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A test covers the decisions source behavior; cargo test and clippy must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The job finds why the source is empty rather than only silencing the message.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs a test that shows the decisions source populated, not just a wiring check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the steward being able to read decisions, or a recorded reason it cannot.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier runs fmt, clippy and test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the decisions source.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
- 11 cases in 10 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle. Before the checks stage, it should automatically delete the finished producer build folder and any other regenerable artifacts (verify-side builds, caches) once they are no longer needed. If free space is still below the guard, it should hold the job instead of failing ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

22 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change writes down the cause of the empty `decisions` source in a committed file (CHANGELOG.md or docs/records.md), citing the code path (crates/athena-core/src/steward.rs and crates/athena-gate/src/records.rs) that produces the empty directory. A reader can check this with git diff.
Either the diff changes crates/athena-core/src/steward.rs (or the record writer) so the steward's workspace holds decision records when they exist at HEAD, with a test in the same crate that builds a view containing a decision-state record and asserts the file appears under the decisions source paths, or the committed notes state why decisions cannot be read and what would change that.
A test that the decisions source is populated when a decision-state record exists in the view and stays an empty directory when none exists; run `cargo test -p athena-core steward` and it passes.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the change.
No file under records/ is edited, and no governance file (athena.toml, defaults.toml) is changed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
