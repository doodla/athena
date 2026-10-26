# Job 20261026-124004-g1w0: verify "Give every steward-channel request an Acceptance section and Files line"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in steward.rs builds each writer's text (structural pass, design review, RequestQueue::apply with an introspection-style request and a blind-spot change) and runs it through reading::request_text_acceptance and request_text_touches, plus PlanEntry::missing(). Run: `cargo test --locked steward_requests_from_every_writer_pass_the_plan_parse`. Observes: Each text yields at least one acceptance bullet and a non-empty file list; missing() does not contain acceptance.
  Fails without the behavior: A writer omitting Acceptance bullets or files gives empty acceptance or touches, so missing() contains acceptance and the test fails.
- Criterion 2: New test in code_health.rs builds a FileHealth fixture breaching lines, function length and weekly landings, calls structural_text. Run: `cargo test --locked structural_text_lists_acceptance_naming_the_breached_limits`. Observes: An Acceptance: section whose bullets name each limit with its measured value, say no behaviour change with existing tests passing, and fmt, clippy and test pass; a Files: line names the path.
  Fails without the behavior: Without the bullets, limit numbers or Files line, the assertions fail.
- Criterion 3: New test in code_health.rs runs the hook on a landed big file, closes the queued job with the close_job helper, runs the hook again. Run: `cargo test --locked structural_pass_is_queued_again_after_the_old_one_is_closed`. Observes: The second run queues a new structural job for the same path, different from the first.
  Fails without the behavior: If closed passes counted as open, the second run queues nothing and the test fails.
- Criterion 4: Builder unit tests in steward.rs: new returns Err for empty acceptance, blank bullets, empty files or multi-line title; render always contains Acceptance: and Files:. The all-writers test covers every caller. Run: `cargo test --locked steward_request`. Observes: Constructor errors for each missing part; valid input renders both sections.
  Fails without the behavior: If the builder accepted empty acceptance or files the constructor tests fail; if a writer bypassed it the all-writers test fails.
- Criterion 5: Run the existing open-request dedupe test together with the new reopen test. Run: `cargo test --locked code_health`. Observes: An open pass suppresses a duplicate and a closed pass does not.
  Fails without the behavior: If an open pass stopped suppressing, structural_pass_queues_nothing_for_a_file_with_an_open_request fails; if a closed pass counted as open, the reopen test fails.
- Criterion 6: Run the three checks; the new tests are part of the suite. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0, including the new writer, bullet and reopen tests.
  Fails without the behavior: Unformatted code fails fmt; any warning fails clippy; a writer lacking acceptance or files, a missing bullet, or a dedupe that ignores closing fails the new tests under cargo test.
- Criterion 7: Inspect the diff for a new Unreleased entry. Run: `git diff CHANGELOG.md`. Observes: Entry says steward requests carry an Acceptance: section and Files: line via one shared builder, the structural-pass bullets, and the reopen-after-close dedupe.

Planned files:

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 4c0048bd7c22 → candidate c84f65a48eb0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  13 ++
 crates/athena-core/src/code_health.rs | 157 ++++++++++++++---
 crates/athena-core/src/reading.rs     |  43 +++--
 crates/athena-core/src/steward.rs     | 307 +++++++++++++++++++++++++++++++++-
 4 files changed, 481 insertions(+), 39 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1327 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
