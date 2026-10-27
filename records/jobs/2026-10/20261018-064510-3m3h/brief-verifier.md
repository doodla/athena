# Job 20261018-064510-3m3h: verify "Run boundary scan in producer and on request input at admission"

## Request (verbatim)

> Have the producer run the boundary scan (`cargo test -p xtask --test policy`) on its changed files before it finishes, and fix any banned word it finds. Also make Athena run that same scan on the request text and on any files the request names before it admits the job, so that a banned word in the input is reworded first. The banned-word list in the brief was not enough by itself: in job 20261017-182453-a5sf the brief warned about 'operator' at lines 58-60, but docs/design/job-planning.md still failed on that word.

## Reading (accepted by the user)

Two changes so banned boundary words are caught early: the producer brief tells it to run `cargo test -p xtask --test policy` on its changes and fix findings before finishing, and Athena scans the request text and any files it names with the same scan before admitting the job, so a hit is reworded first. Audience: the user and the verifier, who saw job a5sf fail on a word in docs/design/job-planning.md.

Assumptions:
- `Admits the job` means the intake step before a producer is dispatched, not the post-change admission in admission.rs; the hook goes where the request is first handled (likely run/mod.rs).
- Reword-first means Athena reports the hit and the request is not admitted until reworded; it does not silently edit the user's text.
- The producer's ability to run cargo test is already allowed; this change only adds the instruction to the brief.
- docs/design/job-planning.md may still need rewording if it fails the scan on this branch.

## Acceptance criteria

1. [content] The producer brief built in crates/athena-context/src/briefs.rs tells the producer to run `cargo test -p xtask --test policy` on its changed files before finishing and to reword any banned word found; a unit test asserts this text is in the producer brief.
2. [behavior] A new test shows a request text containing a banned boundary word (built from fragments) is flagged before admission with the word and its location, while a clean request passes. Run `cargo test` and see the test pass.
3. [behavior] A new test shows a file named in the request that contains a banned word is flagged before admission, with its path and line, and a clean named file is not. Run `cargo test`.
4. [content] Admission-time scanning reuses the same word list as the policy scan (boundary_words/boundary_fragments or a shared source), not a third copy; the existing test that keeps the brief list equal to the scanner list still passes.
5. [behavior] `cargo test --locked` passes in full, including athena_refers_to_no_one_who_runs_it_or_where, and `cargo clippy` is clean; no new tracked file contains a banned word.
6. [content] CHANGELOG.md notes the new behavior in plain language for an outside reader.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- xtask/tests/policy.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Changing the banned-word list itself
- Rewriting the scan in policy.rs beyond exposing it for reuse
- Automatic editing of user request text

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for brief text, request scan, and named-file scan; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the scan on planted words and check flagging happens.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The scan is enforced at admission by code, not prose only.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the gap that input and brief word lists alone let a banned word through.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and comments written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base fae52db1a8f8 → candidate d11c39fc4643. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   7 ++
 crates/athena-context/src/briefs.rs | 135 +++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs   |  71 +++++++++++++++++++
 crates/athena-core/src/run/mod.rs   |  84 ++++++++++++++++++++++
 docs/enforcement.md                 |   2 +-
 xtask/tests/policy.rs               |  29 ++++++++
 6 files changed, 326 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1431 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --locked` passes in full, including athena_refers_to_no_one_who_runs_it_or_where, and `cargo clippy` is clean; no new tracked file contains a banned word.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
