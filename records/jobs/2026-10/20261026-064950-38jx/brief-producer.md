# Job 20261026-064950-38jx: Add code-health check, status table, and 10-landing design review

## Request (verbatim)

> try again on 20261026-042558-5p65

## Reading (accepted by the user)

Retry of 20261026-042558-5p65. Add code_health.rs measuring per-file production lines, longest function, 7-day churn, failure citations and test run time; show it in athena status with thresholds; queue one structural-pass request per breaching file; run after each landing; queue a self-contained design-review request every 10 landings. Audience is Sri and the verifier.

Assumptions:
- No design-review role exists in the repo; the queued request carries its whole content, as the user said.
- Dependence on request 20261017-205558-dn2e is not assumed; this job defines the review text.
- Dedup and cadence are derived from records and open requests, not a counter, so running both hooks for one job does not double count.
- With no earlier design-review request, the cadence baseline is the current landing count, so the first review does not fire at once on existing history.
- Function length is measured by parsing source (syn if added, or a line-based scan); the choice is recorded in docs/decisions.md.
- No athena.toml field is added; thresholds are constants.
- The check hooks into both the finished-job path (steward note_finished) and Athena::land_pr.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
2. [behavior] Run cargo test: tests show a breach of each threshold (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
3. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; running again, or with an open structural-pass request for that file, queues none.
4. [behavior] Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th.
5. [behavior] Run cargo test: the queued design-review text names what to review (the code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
6. [behavior] Run cargo test: with no earlier review request, the first design review does not fire at once on existing history.
7. [behavior] Run cargo test: a test of the Athena::land_pr path shows the code-health check is invoked and would fail without the hook; the finished-job hook is likewise shown.
8. [behavior] Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time.
9. [behavior] cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.
10. [content] CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Expected deliverables

- crates/athena-core/src/code_health.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Adding athena.toml fields
- Changing steward::tick or steward-count behavior
- Building a general design-review role or prompt beyond the queued request text
- Refactoring files flagged by the check

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each behavior; clippy and tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and assert observed outcomes.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Thresholds and cadence enforced in code, not prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Function-length measurement weighs syn against a manual scan and records the call.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Breaches queue work automatically, so no one has to point them out.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A check with no files measured is not treated as healthy.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures and queued text contain no personal data.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
Run cargo test: tests show a breach of each threshold (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; running again, or with an open structural-pass request for that file, queues none.
Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th.
Run cargo test: the queued design-review text names what to review (the code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
Run cargo test: with no earlier review request, the first design review does not fire at once on existing history.
Run cargo test: a test of the Athena::land_pr path shows the code-health check is invoked and would fail without the hook; the finished-job hook is likewise shown.
Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time.
cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.
CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Report

End with the producer report as your structured output.
