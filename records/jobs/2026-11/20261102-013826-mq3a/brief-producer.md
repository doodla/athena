# Job 20261102-013826-mq3a: Stop pending-check merge refusals firing D87; append note to decisions

## Request (verbatim)

> try again on 20261102-005332-8gkv
> The code fix passed review; only the decision note was wrong. docs/decisions.md is append-only: do not edit the D87 entry. Add the note as a new entry at the end of the file that cites D87 and says pending-check merge refusals no longer fire its signal.
> - Mark the pending refusal on the merge call it belongs to, not on the last logged merge entry.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: urgent

## Reading (accepted by the user)

Retry of job 20261102-005332-8gkv: keep its code fix (merge 405/422 refusals logged while required checks were pending no longer match D87's hard signal), mark the pending refusal on the merge call itself, and put the decision note in a new final entry of docs/decisions.md citing D87, leaving D87 unedited. Audience: the maintainer reading decision-state records.

Assumptions:
- Previous job 20261102-005332-8gkv: its blocker (in-place edit of D87 in docs/decisions.md, tripping the append-only guard in crates/athena-cli/src/decisions.rs) is fixed by reverting that edit and appending a new final entry citing D87. Its minor finding (pending mark tied to last logged Merge entry, relying on call order) is fixed by passing the pending flag with the refusal being logged, not marking the last entry. Its nit (transport.rs changed unlisted) is fixed by listing transport.rs as a deliverable. The rest of its code fix was judged sound and is kept.
- Code survey: read crates/athena-core/src/watcher.rs (D87_SIGNAL, HostErrorSignal::matches, evaluate), crates/athena-core/src/gh/client.rs, landing.rs and transport.rs (HostErrorLog, land_pr rule-violation then Checks::Pending branch), and crates/athena-data/src/records.rs. The records validation is check_record_file(path, bytes) -> Option<AnyRecord> at records.rs:2932, over the AnyRecord enum at records.rs:2618. A new optional serde-default checks_pending field on HostError is accepted by that path, and old records such as records/host-errors/2026-11/20261101-174051-3t6g.json must still pass check_record_file unchanged. The property is measured directly: evaluate over host-error records, and the logged record from land_pr.
- The decision log guard in crates/athena-cli/src/decisions.rs rejects any change to the first bytes of docs/decisions.md, so only appending is allowed. The new entry takes the next free D number at the end of the file.
- Mark the pending refusal on the merge call it belongs to, by passing the flag with the refusal or via a handle from that call, not by flagging the last logged Merge entry.
- The previous candidate's files are kept as deliverables: CHANGELOG.md, client.rs, landing.rs, tests.rs, transport.rs, watcher.rs, records.rs and docs/decisions.md (now append-only).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core watcher`: a test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and an unmarked 405 or 422 still fires D87 on the first job.
2. [behavior] Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged with the pending mark on that same merge call's entry even when other entries are logged after it, and a refusal with checks settled is logged unmarked as before.
3. [behavior] Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates and serialises unchanged.
4. [behavior] Run `cargo test -p athena-cli decisions`: the append-only guard test passes, so docs/decisions.md only gained text after its previous end.
5. [content] `git diff` of docs/decisions.md shows only added lines at the end of the file: a new entry that cites D87 and says pending-check merge refusals no longer fire its signal. No D87 line is changed.
6. [content] Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment says pending-check refusals are excluded, and the name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged.
7. [content] CHANGELOG.md has an entry naming the pending-check 405 false positive and stating that existing records/ are untouched; no file under records/ is in the diff.

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Editing any file under records/ or clearing the open D87 review
- Editing the D87 entry or any existing entry of docs/decisions.md
- Reading branch protection to find the required check name
- Changes to athena.toml, defaults.toml or .github/workflows
- Drain wiring in crates/athena-cli/src/drain.rs
- Adding signals for decisions other than D87

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a pending-check refusal does not fire D87 and a real one still does.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for watcher, landing and old-record validation.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The matcher excludes flagged records instead of special-casing one instance.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus docs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261102-005332-8gkv raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (docs/decisions.md:1917 (inside the D87 entry); guard in crates/athena-cli/src/decisions.rs:217-241): Editing D87 in place breaks the append-only decision log, so cargo test fails `cargo test --locked` fails with 'docs/decisions.md: the first 230046 bytes changed: entries are append-only, add new ones at the end'. CI run 41372212254 records the same test failure. Fix: revert the in-place bullet and add the note at the end of docs/decisions.md, for example as a later note or new entry that cites D87.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 15 cases in 13 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make Athena own the build-folder lifecycle in one place that every run path calls, not only drain start. Before any retry or new build starts, and before the checks stage, Athena removes the build folders of jobs that are interrupted, abandoned, finished or superseded. A job's retry deletes its own ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

Run `cargo test -p athena-core watcher`: a test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and an unmarked 405 or 422 still fires D87 on the first job.
Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged with the pending mark on that same merge call's entry even when other entries are logged after it, and a refusal with checks settled is logged unmarked as before.
Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates and serialises unchanged.
Run `cargo test -p athena-cli decisions`: the append-only guard test passes, so docs/decisions.md only gained text after its previous end.
`git diff` of docs/decisions.md shows only added lines at the end of the file: a new entry that cites D87 and says pending-check merge refusals no longer fire its signal. No D87 line is changed.
Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment says pending-check refusals are excluded, and the name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged.
CHANGELOG.md has an entry naming the pending-check 405 false positive and stating that existing records/ are untouched; no file under records/ is in the diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
