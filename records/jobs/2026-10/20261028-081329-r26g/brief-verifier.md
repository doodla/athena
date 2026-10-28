# Job 20261028-081329-r26g: verify "Group scope-guard lines only by retry link, not cited job ids"

## Request (verbatim)

> Priority: high
> The scope guard groups a request into another line of work when its text merely cites that line's jobs as evidence: on 2026-10-28 a separate disk-and-resume fix that cited two jobs of a paused line was refused as part of that line. Group a request into a line only by its retry link (previous_job) or an explicit follow-up of a named job, not by any job id in its text; requests that cite jobs as evidence start their own line.
> Acceptance:
> - A test shows a request citing a paused line's jobs as evidence is not paused, and a retry of that line is.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/scope_guard.rs, CHANGELOG.md

## Reading (accepted by the user)

In scope_guard.rs, stop joining a request to a line because its text mentions another job's id; group only by the retry link (and the existing same-text re-ask). A request citing a paused line's jobs as evidence starts its own line and is not paused. Audience: the operator whose unrelated fix was refused.

Assumptions:
- 'Explicit follow-up of a named job' is the existing retry link ('try again on <job>'); no new follow-up syntax is added.
- The same-text re-ask link stays, as the request only removes id-in-text grouping.
- The quotes field may be removed or left unused; either is fine if clippy is clean.

## Acceptance criteria

1. [behavior] A new unit test in scope_guard.rs shows a request whose text cites jobs of a paused line is in its own line (group_lines) and is not refused as paused; run `cargo test -p athena-core scope_guard`.
2. [behavior] A test shows a retry (retry_of link) of the paused line is still grouped into it and refused; run `cargo test -p athena-core scope_guard`, and the existing retry_refuses_paused_line passes.
3. [content] group_lines and request_records in scope_guard.rs no longer link a request to another by job ids found in its text (diff shows the quotes link removed from grouping); doc comments describe grouping by retry link and same-text re-ask only.
4. [behavior] Tests that relied on quote grouping (grouping_joins_retries_and_quoting_requests and any others) are updated to the new rule; `cargo test` passes for the whole workspace.
5. [behavior] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
6. [content] CHANGELOG.md has an entry describing that citing job ids as evidence no longer groups a request into that line (git diff shows it).

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## Out of scope

- The separate disk-and-resume fix
- Changing pause thresholds or signals
- Changing the same-text re-ask rule
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Remove the id-in-text grouping rule rather than special-casing the one refused request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise grouping and the pause outcome for evidence-citing vs retry requests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in scope_guard.rs and the changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New rig test `citing_a_paused_lines_jobs_as_evidence_starts_its_own_line` in the rig tests module of scope_guard.rs, next to `retry_refuses_paused_line` (line 1167). Setup uses the existing helper `rig(0.8, 0.01, &[Some(0), Some(0), None])` (line 847; fake producer and verifier verdict list, no new seam). Ask and `go` job a, then ask `try again on {a}` as b and `go` it; two same-criterion rejections pause the line naturally (T4). Then ask c with text like `fix disk and resume handling; evidence: jobs {a} and {b} were paused` (no 'try again on' prefix, so retry_of is None). Production entry points: `request_records(&r.records())`, `group_lines`, `Athena::line_pause`, `Athena::go`. No double contains the logic under test. Run: `cargo test -p athena-core scope_guard`. Observes: `group_lines` gives `[[0,1],[2]]`; requests[2].retry_of and same_text_as are None; `line_pause` for c is None; `r.go(&c)` does not return an error containing 'paused'.
  Fails without the behavior: With the old quotes scan, c joins line [0,1,2], `line_pause` returns Some and `go(&c)` is refused with 'paused'; the grouping and pause assertions fail.
- Criterion 2: Same new test, continued: after c, ask `d = try again on {b}` and call `r.go(&d)`. Also the existing `retry_refuses_paused_line` runs. Entry points `Athena::go`, `line_pause`, `group_lines`; the pause is the natural one from the setup, not injected. Run: `cargo test -p athena-core scope_guard`. Observes: `go(&d)` is Err with text containing 'paused' and 'T4'; `group_lines` puts d in line 0 with a and b; `retry_refuses_paused_line` passes.
  Fails without the behavior: If grouping dropped the `retry_of` link along with `quotes`, d would start its own line, `go` would proceed, and the refusal assertion would fail.
- Criterion 3: Read the diff of scope_guard.rs and grep it. Check that the `quotes` field, the `quotes: ids...text.contains(...)` block, the `ids` binding and `.chain(r.quotes.iter())` are gone, and that the doc comments on `RequestRecord`, `group_lines` and `request_records` name only the retry link and the same-text re-ask. Run: `git diff -- crates/athena-core/src/scope_guard.rs`. Observes: The diff shows those removals and the new doc text; no occurrence of `quotes` remains in the file (Grep for `quotes` in scope_guard.rs returns nothing).
  Fails without the behavior: If the id-in-text link were still there, the diff would still show `quotes` and the text scan, and the criterion 1 test would also fail.
- Criterion 4: Rewrite `grouping_joins_retries_and_quoting_requests` as `grouping_joins_retries_only` using `req(id, retry_of)` (helper at line 1371, quotes parameter removed): a, x, b (retry of a), c (no link), d (retry of a missing id). Expected `[[0,2],[1],[3],[4]]`. Update callers of `req`, including `grouping_joins_a_re_ask_to_the_request_it_repeats`. Run the whole workspace suite. Run: `cargo test --locked`. Observes: All workspace tests pass, including the updated grouping tests and every rig test that ran before.
  Fails without the behavior: The old test, left as written, would not compile against the new `req`/`RequestRecord` or would expect c joined to b; the workspace run would fail.
- Criterion 5: Run the exact checks Athena runs on the result, after the edits. They guard an incomplete removal: a `quotes` field kept but never read, or an `ids` binding left unused in `request_records`, is a dead-code or unused warning that clippy turns into an error; unformatted edits fail fmt. Entry points are the cargo tools themselves; no seam or double is involved. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0, with no dead-code or unused-variable warning from the removed field or binding, and no formatting diff.
  Fails without the behavior: If grouping stopped reading `quotes` but the field or the `ids` binding stayed, clippy -D warnings fails with dead_code/unused_variables; if the edits were not rustfmt-clean, `cargo fmt --check` fails.
- Criterion 6: Read the diff of CHANGELOG.md. Under `## [Unreleased]` / `### Fixed` (existing section near line 28) there is a new bullet saying that citing job ids as evidence no longer groups a request into that line, and that only the retry link and a same-text re-ask do. It names the 2026-10-28 refusal of the disk-and-resume fix and notes that docs/design/scope-guard.md is updated. Run: `git diff -- CHANGELOG.md`. Observes: Only added lines, with the new Unreleased bullet describing the change.
  Fails without the behavior: Without the entry the diff of CHANGELOG.md is empty.

Planned files:

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## The change

Base b87662de4210 → candidate a86e26bf908f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  9 ++++
 crates/athena-core/src/scope_guard.rs | 96 ++++++++++++++++++++++-------------
 docs/design/scope-guard.md            |  7 +--
 3 files changed, 75 insertions(+), 37 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1527 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
