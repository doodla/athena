# Job 20261026-124004-g1w0: Give every steward-channel request an Acceptance section and Files line

## Request (verbatim)

> Priority: urgent
> Every request Athena writes for itself must be schedulable by its own planner. Today all 30 requests that `athena plan` refuses ("the plan lacks acceptance") were written on the steward channel by Athena: introspection prevention follow-ups (for example 20261026-075017-vpg6, 20261026-123222-genk) and the eight code-health structural passes queued by the hook in crates/athena-core/src/code_health.rs (for example 20261026-123041-8pt5). None carries an `Acceptance:` section with bullets, so the learning loop records lessons that never run. Fix the root cause at the writers, not the parser: every steward-channel request writer (the code-health structural pass and design review, the introspection follow-up writer in steward.rs, and any other writer that builds an athena.request/1 on the steward channel) emits an `Acceptance:` section of `- ` bullets and a `Files:` line, in the format athena plan parses. For a structural pass, acceptance is: the file is under the limits it breached as measured by code_health; no behavior change, the existing tests pass; fmt, clippy and test pass. Add one shared builder for steward requests so a new writer cannot skip the format.
> Also: make the code-health hook's dedupe treat a closed or refused earlier structural pass as not open, so closing the old refused passes lets the hook queue them again in the new format on the next landing.
> Acceptance:
> - A test runs each steward-channel request writer and passes its text through the same parse athena plan uses; none is refused for missing acceptance, and each names its files.
> - A test shows a structural-pass request for a fixture file lists acceptance bullets naming the breached limits.
> - A test shows that after a structural pass is closed, the next hook run queues a new one for a file still over its limits.
> - fmt, clippy with -D warnings, and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/code_health.rs, crates/athena-core/src/steward.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Add one shared steward-request builder (in steward.rs) that always emits an `Acceptance:` section of `- ` bullets and a `Files:` line in the format `athena plan` parses. Route the structural-pass, design-review and introspection follow-up writers through it. Make the structural-pass dedupe ignore closed passes. The audience is the drain/plan loop, which today refuses 30 Athena-written requests.

Assumptions:
- Goal: every request Athena writes for itself is schedulable by `athena plan`. The audience is the drain loop and the person reading its refusals.
- The parse `athena plan` uses for request-text acceptance lives mainly in reading.rs and plan.rs. The test calls it directly, and reading.rs changes only if a small exported helper is needed.
- The introspection follow-up writer builds its text through ProposedChange.request, applied by RequestQueue in steward.rs, with text composed in introspect.rs. The shared builder goes in steward.rs and callers in introspect.rs and code_health.rs use it.
- Closing the old refused passes is an operational step by the user, not part of the change.
- Per the 'Fix the root cause at the writers, not the parser' instruction, the plan parser's behavior is not loosened.

## Acceptance criteria

1. [behavior] A test builds the text from each steward-channel writer (structural pass, design review, introspection follow-up, and any other writer found building a steward request) and runs it through the same acceptance and files parse that `athena plan` uses. None is refused for missing acceptance and each yields a non-empty file list. Run `cargo test`.
2. [behavior] A test builds a structural-pass request for a fixture file over each limit. The text has an `Acceptance:` section whose `- ` bullets name the breached limits as measured (lines, function length, weekly landings). The bullets also say no behavior change with existing tests passing, and that fmt, clippy and test pass. The text has a `Files:` line naming the file. Run `cargo test`.
3. [behavior] A test queues a structural pass for a file over its limits and closes that job. The next hook run (`code_health_after_landing`) queues a new structural pass for the same file. Run `cargo test`.
4. [content] A single shared builder for steward requests exists, and every steward-channel writer constructs its text through it. A writer cannot omit `Acceptance:` bullets or `Files:`. The code or tests make this enforced rather than relying on convention.
5. [content] Closed, including refused, earlier structural passes count as not open in the dedupe. A still-open pass for the same file still suppresses a duplicate, and the existing test for that keeps passing.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry describing the steward request format fix and the dedupe change.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Closing the existing 30 refused requests
- Changing the plan parser to accept requests that lack acceptance
- Restructuring the files that the passes name, such as run.rs
- The introspection follow-up job contents (reading-input and no-change guard)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each writer, the structural-pass bullets and the reopen-after-close dedupe; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the writers via a shared builder, not in the parser.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the writers' output through the real parse and check the hook queues again after close.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The shared builder enforces the format, and a test covers every writer.
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

A test builds the text from each steward-channel writer (structural pass, design review, introspection follow-up, and any other writer found building a steward request) and runs it through the same acceptance and files parse that `athena plan` uses. None is refused for missing acceptance and each yields a non-empty file list. Run `cargo test`.
A test builds a structural-pass request for a fixture file over each limit. The text has an `Acceptance:` section whose `- ` bullets name the breached limits as measured (lines, function length, weekly landings). The bullets also say no behavior change with existing tests passing, and that fmt, clippy and test pass. The text has a `Files:` line naming the file. Run `cargo test`.
A test queues a structural pass for a file over its limits and closes that job. The next hook run (`code_health_after_landing`) queues a new structural pass for the same file. Run `cargo test`.
A single shared builder for steward requests exists, and every steward-channel writer constructs its text through it. A writer cannot omit `Acceptance:` bullets or `Files:`. The code or tests make this enforced rather than relying on convention.
Closed, including refused, earlier structural passes count as not open in the dedupe. A still-open pass for the same file still suppresses a duplicate, and the existing test for that keeps passing.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has an entry describing the steward request format fix and the dedupe change.

## Report

End with the producer report as your structured output.
