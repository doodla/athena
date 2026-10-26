# Job 20261026-023720-62fj: Let producers leave full-suite criteria to Athena's checks

## Request (verbatim)

> Priority: urgent
> Most jobs still lose about 7 minutes to a producer send-back over a criterion only Athena's own checks can show: readers write criteria such as 'cargo clippy --all-targets and cargo fmt --check are clean, and cargo test for the workspace passes', producers no longer run the full suite (d73effa), so the producer's self-check reports it unmet and the run is sent back (job 20261026-015416-94rc lost 431 s this way). When a criterion is satisfied by Athena's own recorded checks (fmt, clippy, the full test suite), the producer's self-check marks it 'left to Athena's checks' instead of unmet, and the verifier still judges it from the recorded checks.
> Acceptance:
> - A test shows a producer report that leaves a full-suite criterion to Athena's checks is not sent back, while a report that omits a targeted-test criterion still is.
> - The verifier still fails the job when the recorded full-suite check fails.
> Files: crates/athena-core/src/run.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md

## Reading (accepted by the user)

The producer self-check (unmet_criteria in run.rs) stops sending a report back when a criterion is marked 'left to Athena's checks' and is a full-suite, fmt or clippy criterion. The producer brief (briefs.rs) tells producers how to mark it. Targeted-test criteria that are omitted or unmet still go back, and the verifier still judges the left-over criteria from the recorded checks. Audience: Athena's run loop and its verifier, which pay for the wasted send-back.

Assumptions:
- Marking uses the existing met/unmet status plus a fixed evidence prefix "left to Athena's checks", so the CriterionStatusOutput enum and its JSON schema in outputs.rs stay unchanged (the request lists only run.rs, briefs.rs and CHANGELOG.md). A new enum variant would also touch outputs.rs.
- Only criteria that name Athena's own recorded checks (cargo fmt --check, cargo clippy, the full or workspace cargo test) may be left this way. A criterion about a targeted test or any other behavior must still be reported met, so an omitted or unmet targeted-test criterion is still sent back. The criterion text must be recognizable as a full-suite check, not just marked by the producer.
- Criteria left to Athena's checks are passed on to the verifier to judge from the recorded checks, the way a still-incomplete report is flagged today (run.rs near line 1947). The verifier fails the job when the recorded full-suite check fails, which is existing behavior and must keep working.
- The job is urgent because it cuts a redo-work send-back that costs about 7 minutes per job.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/run.rs shows unmet_criteria returns nothing for a report that marks a full-suite criterion (clippy, fmt --check and workspace cargo test) as left to Athena's checks, and still returns a targeted-test criterion that the report omits. Run: cargo test -p athena-core.
2. [behavior] A test shows a criterion marked 'left to Athena's checks' that is not an Athena-check criterion (for example a targeted-test or content criterion) is still returned as unmet and sent back. Run: cargo test -p athena-core.
3. [behavior] A test shows the verifier still fails the job when the recorded full-suite check fails, even though the producer left that criterion to Athena's checks. Run: cargo test -p athena-core.
4. [content] The producer brief's criteria_check section in crates/athena-context/src/briefs.rs tells the producer to mark a criterion that only Athena's recorded checks (fmt, clippy, the full test suite) can show as left to Athena's checks, with the exact marker text, and says targeted-test criteria must still be reported met with evidence. A briefs.rs test asserts that wording.
5. [content] CHANGELOG.md has an entry describing the change: self-check no longer sends back criteria left to Athena's checks.
6. [behavior] cargo fmt --check and cargo clippy --all-targets are clean, and cargo test passes for the workspace (Athena's recorded checks).

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/outputs.rs (modify)

## Out of scope

- Changing how the verifier judges criteria or how Athena records fmt, clippy and test checks.
- Re-running the full suite in the producer (decision d73effa stands).
- The scope-guard work from job 20261026-015416-94rc and its cosmetic docs finding.
- Changing the number of send-backs (PRODUCER_SEND_BACKS) or the send-back text beyond what the new marking needs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code and Rust tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover the self-check and the brief; cargo test, clippy and fmt --check stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the self-check outcome (sent back or not) and the verifier's failure, not that types exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The checks are the send-back decision for a full-suite criterion versus a targeted-test criterion, and the verifier still failing on a failed recorded check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the self-check, the brief and the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog entry are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the self-check rule that wrongly counts Athena-owned checks as unmet, not in a retry or timeout.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

A unit test in crates/athena-core/src/run.rs shows unmet_criteria returns nothing for a report that marks a full-suite criterion (clippy, fmt --check and workspace cargo test) as left to Athena's checks, and still returns a targeted-test criterion that the report omits. Run: cargo test -p athena-core.
A test shows a criterion marked 'left to Athena's checks' that is not an Athena-check criterion (for example a targeted-test or content criterion) is still returned as unmet and sent back. Run: cargo test -p athena-core.
A test shows the verifier still fails the job when the recorded full-suite check fails, even though the producer left that criterion to Athena's checks. Run: cargo test -p athena-core.
The producer brief's criteria_check section in crates/athena-context/src/briefs.rs tells the producer to mark a criterion that only Athena's recorded checks (fmt, clippy, the full test suite) can show as left to Athena's checks, with the exact marker text, and says targeted-test criteria must still be reported met with evidence. A briefs.rs test asserts that wording.
CHANGELOG.md has an entry describing the change: self-check no longer sends back criteria left to Athena's checks.
cargo fmt --check and cargo clippy --all-targets are clean, and cargo test passes for the workspace (Athena's recorded checks).

## Report

End with the producer report as your structured output.
