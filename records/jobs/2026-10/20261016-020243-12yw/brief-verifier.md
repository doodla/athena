# Job 20261016-020243-12yw: verify "Make athena drain work cli-channel (ask --later) queued requests"

## Request (verbatim)

> try again on 20261016-005953-582g

## Reading (accepted by the user)

queued_work in steward.rs keeps only channel "steward", so requests from `ask --later` (channel "cli") are never drained. Change it to a channel-set filter (steward and cli), keep ordering and retry unchanged, fix the stale docs, and prove it with tests using fresh ids. Audience: the user who queues requests and expects drain to work them.

Assumptions:
- `ask --later` records channel "cli"; the producer must confirm this in the code before filtering.
- Other channels stay excluded.
- Gate commands (test, clippy, fmt) are a standing definition of done, listed as one criterion each at the end.
- Earlier failures were unmet criteria: no job ids in non-test code, docs corrected in all places, gate results reported. The producer should map each criterion to evidence before reporting.

## Acceptance criteria

1. [behavior] A unit test in steward.rs, using fresh job ids, shows queued_work returns unread Queued requests from both the steward and cli channels, oldest first (ties by job id), and excludes non-Queued jobs and other channels. Run `cargo test -p athena-core queued_work`.
2. [behavior] A drain test with a cli-channel queued request (fresh id) shows drain lists and works it like a steward item. Run `cargo test -p athena-cli drain`.
3. [behavior] Existing drain ordering and retry/thrash tests pass unedited. Run `cargo test`.
4. [content] The change in queued_work is a channel-set filter with no job-id special case. `git diff` shows none of the ids 20261015-215658-6ejp, 20261015-215701-cqn8, 20261015-215703-px13, 20261015-225907-kb92 in non-test code.
5. [content] No doc comment still says drain takes only the steward channel: REQUEST_CHANNEL and queued_work in steward.rs, drain.rs, and docs/decisions.md. Grep for the old wording finds none outside historical entries.
6. [content] A new docs/decisions.md entry and a CHANGELOG.md line record the change and why.
7. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` pass, and `cargo test` passes in full.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Changing drain ordering, retry, or thrash rules
- Draining channels other than steward and cli
- Building the completion gate proposed by the introspection
- Editing athena.toml, defaults.toml, or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both channels; cargo test and clippy pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared selection function, not a per-id case.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests run drain with a cli-channel request.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entry written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused filter change plus its docs and tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the test, clippy and fmt commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 739d3b9cd8fa → candidate 5fdadfbd8d9d. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |  6 ++++
 README.md                         |  4 +--
 crates/athena-cli/src/drain.rs    |  2 +-
 crates/athena-cli/src/main.rs     |  2 +-
 crates/athena-cli/tests/drain.rs  | 49 ++++++++++++++++++++-----
 crates/athena-core/src/steward.rs | 76 ++++++++++++++++++++++++++++++++-------
 docs/decisions.md                 | 21 ++++++++++-
 docs/design/roles-and-ladder.md   |  3 +-
 8 files changed, 136 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (718 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "A unit test in steward.rs, using fresh job ids, shows queued_work returns unread Queued requests from both the steward and cli channels, oldest first (ties by job id), and excludes non-Queued jobs and other channels. Run `cargo test -p athena-core queued_work`."; "A drain test with a cli-channel queued request (fresh id) shows drain lists and works it like a steward item. Run `cargo test -p athena-cli drain`."; "Existing drain ordering and retry/thrash tests pass unedited. Run `cargo test`.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
