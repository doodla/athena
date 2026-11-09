# Job 20261028-043810-h0ws: verify "Skip Priority lines in the Queue a request commit subject"

## Request (verbatim)

> When building the 'Queue a request:' commit subject in crates/athena-core/src/reading.rs (and the 'Dropped:' subject, wherever it is built from the request text), skip leading lines that match the 'Priority: <level>' marker parsed in crates/athena-core/src/plan.rs and use the first remaining non-empty line. Add a test with a request whose first line is 'Priority: urgent' showing that the subject names the request, not the priority. Add a CHANGELOG.md entry.
> 
> Acceptance:
> - The change is made as described: Queued and dropped request commits are titled only 'Priority: <level>' because the subject takes the request's first line
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/reading.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

`Athena::queue` in reading.rs builds "Queue a request: <first line>" from the request's raw first line, so a request starting "Priority: urgent" is titled by its priority. The fix uses the existing `plan::title_line` helper, which skips Priority and blank lines, and adds a test and a CHANGELOG entry. The audience is anyone reading the records log.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs `queue` (about lines 895-932) takes `request.text.lines().next()` truncated to 60 chars for the subject. crates/athena-core/src/plan.rs already has `pub fn title_line` (line 114), which skips Priority: lines and blanks and has its own tests. crates/athena-core/src/store.rs `title()` already uses it. crates/athena-core/src/manage.rs close builds its subjects from `job.title()`, which is `title_line`-based when there is no reading. No records or data file is added, so no record allow-list is affected.
- The 'Dropped:' subject is probably already correct, because it comes from `job.title()` in store.rs, which uses `title_line`. The producer should confirm this with a Grep and change it only if some path still takes the raw first line. Nothing under crates/athena-gate needs to change.
- The 60-character truncation stays. `title_line` returns the first line when every line is a Priority or blank line, and that fallback is kept.
- The test goes in the existing tests module of reading.rs. It calls `queue` through the existing test helpers, with a request like 'Priority: urgent\n\nFix the widget', and reads the commit subject back with git log.
- No governance files change. CHANGELOG.md is the only non-code deliverable.

## Acceptance criteria

1. [content] In crates/athena-core/src/reading.rs, the `queue` subject uses `crate::plan::title_line(&request.text)` (then the 60-char cap) instead of `lines().next()`; check the diff.
2. [behavior] A new test in reading.rs queues a request whose first line is 'Priority: urgent' followed by a real request line. It asserts the commit subject is 'Queue a request: <real line>' and does not contain 'Priority'. Check with `cargo test -p athena-core` and the test name in the diff.
3. [content] The diff shows no raw first-line subject left for a Dropped commit. The Dropped subject path (manage.rs via `job.title()`) is either shown to use `title_line` or changed to use it.
4. [content] CHANGELOG.md has a new entry under the unreleased section saying that queued and dropped request commit subjects skip leading Priority lines.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` all pass, and the existing `title_line` tests in plan.rs still pass unchanged.
6. [content] The diff touches only reading.rs, CHANGELOG.md, and plan.rs or manage.rs if needed; no other behavior changes.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/manage.rs (modify)

## Out of scope

- Changing how Priority is parsed or how priority affects plan ordering
- Changing the 60-character subject cap
- Changing titles taken from a reading
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test and clippy criteria
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that reuses the existing title_line helper
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Checks that the Dropped path also uses the shared helper, not only the Queue path
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the actual commit subject
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are in Rust
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are commands the verifier can run
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a3ebd6d9643a → candidate 16d4f048db82. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  5 +++
 crates/athena-core/src/reading.rs | 71 ++++++++++++++++++++++++++++++++++-----
 2 files changed, 67 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-043810-h0ws/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-043810-h0ws/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42553186098 for 16d4f048db822452c9fce9bd44411278978a2936)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42553186098 for 16d4f048db822452c9fce9bd44411278978a2936)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42553186098 for 16d4f048db822452c9fce9bd44411278978a2936)
- ci-result: `not a check` passed (Check results for 16d4f048db822452c9fce9bd44411278978a2936 come from the CI run https://github.com/doodla/athena/actions/runs/42553186098 (760s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` all pass, and the existing `title_line` tests in plan.rs still pass unchanged.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
