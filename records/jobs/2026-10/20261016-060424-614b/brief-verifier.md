# Job 20261016-060424-614b: verify "Make planning a per-item pass that flags vague requests; add athena plan"

## Request (verbatim)

> try again on 20261016-044847-hem1

## Reading (accepted by the user)

Retry hem1: replace default-filled plan fields with a deterministic per-item planning pass, hold back underspecified items with a named reason, and add an `athena plan` command that prints the coordinated plan. Per the user's reply, the pass reads unread requests (runs the reader) so the drain can work the queue rather than refusing them and leaving them queued. Audience is the user and the verifier judging whether the gate fires on real vague requests.

Assumptions:
- The typed request (retry hem1) governs; the introspection's proposed verifier-preflight/unverified-environment work is a separate change and is out of scope here.
- The planning pass is deterministic Rust code (no model call); reading unread requests uses the existing reader step already in the drain.
- Unread requests are read first (in drain and in `athena plan`), then planned from the reading; an item refused after reading is recorded with a reason and does not sit unread in the queue forever.
- The earlier verifier's disk-full blocker was an environment fault, not a defect; the design of the rejected attempt is otherwise kept.
- Previous finding: every behavior criterion's evidence must cite the commands that ran.

## Acceptance criteria

1. [behavior] Run `cargo test`: a new test shows a request with no bullets and no path-like words (e.g. 'make it better') is flagged underspecified after reading, the flag names the missing field, and the item is not in Plan::order.
2. [behavior] Run `cargo test`: a new test shows a request's first line is no longer copied into acceptance; only explicit acceptance bullets or a reading's criteria fill it.
3. [behavior] Run `cargo test`: a new test shows a non-proposal job's rung is inferred from its request or reading, or flagged missing; it is never silently Implementation, and an item with no determinable rung is refused.
4. [behavior] Run `cargo test`: tests assert open_questions are filled for a read and an unread request, including a touched path that does not exist and an unresolved referenced job.
5. [behavior] Run `cargo test`: a test shows two open items naming the same path get a dependency or different groups, and items with disjoint touches share a group.
6. [behavior] Run `cargo test`: a drain test with RealSteps over a records fixture with one specified and one unread vague queued item shows the unread item is read by the pass (not refused merely for being unread), the vague one lands in Report::refused with a reason and is not run as a job, and the specified one runs. A further test shows an unread but specifiable request is read and then runs, so the queue does not fill with unread refused items.
7. [behavior] Run `cargo run -- plan` (or the equivalent new command): it prints order, parallel groups, and each item's goal, rung, acceptance, open questions, dependencies, touches, plus flagged items with reasons; with nothing open it says so explicitly. The verifier's evidence cites the commands that ran and their output.
8. [content] Files touched outside plan.rs, reading.rs and drain.rs (display.rs, main.rs, etc.) are named in the docs or decision entry; the docs/decisions.md tradeoff about the gate rarely firing is updated, including how unread requests are handled.
9. [content] `cargo clippy` is clean and `cargo fmt --check` passes; no secrets or personal data are added.
10. [content] Docs state plainly that the planning pass is deterministic code (not a model call) and whether plan::trace/GapLog audit is wired to records or still follow-up.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Verifier preflight, disk cleanup, and an 'unverified, environment' verdict state (separate change).
- Calling a model inside the planning pass.
- Changes to athena.toml, defaults.toml, or workflows.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the refusal gate firing on vague requests and the plan being printed, not that fields exist.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test and cargo run commands and require cited output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each behavior; clippy and fmt --check required.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entry written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Plan command is exercised with cargo run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the defaulting at its source in plan entry construction, and the unread-refusal deadlock in drain.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 2a23e5a35473 → candidate 528670614df8. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |  15 ++
 README.md                         |  15 +-
 crates/athena-cli/src/display.rs  |  99 +++++++++++-
 crates/athena-cli/src/drain.rs    | 213 +++++++++++++++++++++----
 crates/athena-cli/src/main.rs     |  42 ++++-
 crates/athena-cli/tests/drain.rs  | 269 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/plan.rs    |  72 ++++++++-
 crates/athena-core/src/reading.rs | 318 ++++++++++++++++++++++++++++++++++----
 docs/architecture.md              |   7 +
 docs/decisions.md                 |  36 ++++-
 10 files changed, 1003 insertions(+), 83 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (747 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
