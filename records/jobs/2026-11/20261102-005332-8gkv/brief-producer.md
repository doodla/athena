# Job 20261102-005332-8gkv: Stop pending-check merge 405s from firing D87's hard signal

## Request (verbatim)

> try again on 20261102-001719-1fva

## Reading (accepted by the user)

Retry of job 20261102-001719-1fva (dispatched, never verified, no findings, no candidate recorded). Landing merge refusals (405/422) that happened while required checks were pending must not count toward D87 "merge commits not allowed"; real refusals with checks complete still fire on the first job. Audience: the maintainer reading decision-state records.

Assumptions:
- Previous result: job 20261102-001719-1fva had no candidate and no findings, so nothing is kept or dropped; this retry does the whole change fresh with the same deliverables and out_of_scope as that job.
- Code survey: read crates/athena-core/src/watcher.rs (D87_SIGNAL, load, HostErrorSignal::matches, evaluate; matching is exact on operation, phase and status), crates/athena-core/src/gh/landing.rs, gh/client.rs and gh/tests.rs (merge refusals are logged as host-error records at landing; pending-check handling exists in gh/early.rs, landing.rs, client.rs), and crates/athena-data/src/records.rs. The records validation is check_record_file, which parses into AnyRecord (enum at records.rs:2618, check_record_file at :2932). If HostError gains an optional field (e.g. checks pending), it must be added with serde default so old records still deserialize, AnyRecord accepts it, and the existing round-trip/validation tests plus a new check_record_file test on an old-format host-error JSON cover it. The property is measured directly by running evaluate over HostError values and by landing tests in gh/tests.rs, not via a stand-in.
- Assumed mechanism: at the merge refusal, landing already knows whether checks were pending; the refusal is recorded with a distinguishing marker (or message_class) and HostErrorSignal::matches skips it, keeping D87_SIGNAL's name string unchanged.
- Existing records under records/ are never edited, and the open D87 review (20261031-085139-1y06) is not cleared.
- This narrows a firing rule (fewer firings), not a refusal rule on readings, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core watcher`: a new test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and a 405/422 without that marking still fires D87 on the first job.
2. [behavior] Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged distinguishably (or not as a D87-matching error), while a refusal with checks complete is logged as before.
3. [behavior] Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates unchanged after any records.rs change.
4. [content] Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment states pending-check refusals are excluded, and the signal name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged so record 20261031-085139-1y06 still matches.
5. [content] CHANGELOG.md has an entry naming the pending-check 405 false positive and stating existing records/ are untouched; docs/decisions.md D87 notes the exclusion.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Editing any file under records/ or clearing the open D87 review
- Reading branch protection to find the required check name
- Changes to athena.toml, defaults.toml or .github/workflows
- Drain wiring in crates/athena-cli/src/drain.rs
- Adding new decisions' signals other than D87

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New watcher, gh and records tests; fmt, clippy -D warnings and cargo test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the signal's misclassification of pending-check refusals, not just the one open review.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing the firing behavior.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the D87 signal path.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Exclusion is enforced in code in matches/evaluate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
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

Run `cargo test -p athena-core watcher`: a new test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and a 405/422 without that marking still fires D87 on the first job.
Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged distinguishably (or not as a D87-matching error), while a refusal with checks complete is logged as before.
Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates unchanged after any records.rs change.
Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment states pending-check refusals are excluded, and the signal name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged so record 20261031-085139-1y06 still matches.
CHANGELOG.md has an entry naming the pending-check 405 false positive and stating existing records/ are untouched; docs/decisions.md D87 notes the exclusion.
Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
