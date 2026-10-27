# Job 20261027-031444-60zg: verify "Add Hestia error review and lineage-based scope guard counting"

## Request (verbatim)

> Priority: urgent
> Give a role the duty of thinking about errors across jobs, not one at a time (Sri, 2026-10-27: "Is anyone looking at the errors and thinking about them?"). Today every failure gets an introspection with a cause and prevention, but nothing groups them by cause or acts on repeats; the operator catches patterns by hand, late (four planning-cap stops before a fix; five brde attempts before a stop). Add an error review to Hestia (docs/design/hestia.md): after each introspection is recorded, group it with earlier introspections by cause (the introspection's cause and class, matched by the existing class registry or by Jev's class answer). When a cause has occurred in two or more distinct jobs and no open or landed request cites it, queue one root-cause request at urgent priority through the shared steward request builder, citing the introspection ids, and show the open repeated causes with their counts in athena status. Also make the scope guard count attempts per line of work across retries and re-asks of the same request (follow previous_job and request text lineage), not per job id.
> Acceptance:
> - A test records two introspections with the same cause on different jobs and shows exactly one urgent root-cause request is queued citing both; a third does not queue another while that request is open.
> - A test shows athena status lists repeated causes with their counts and the request that addresses each.
> - A test shows the scope guard counts a retry and a re-ask of the same request as one line, and pauses at its threshold.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry; docs/design/hestia.md describes the error review.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/scope_guard.rs, crates/athena-core/src/status.rs, docs/design/hestia.md, CHANGELOG.md

## Reading (accepted by the user)

Add an error review to Hestia that groups introspections by cause/class, queues one urgent root-cause request when a cause repeats across two or more distinct jobs without an open or landed request citing it, and shows repeated causes in athena status. Make the scope guard count attempts per line of work (previous_job and request text lineage). Audience: the operator (Sri) who catches repeats by hand today.

Assumptions:
- Grouping key is the introspection cause plus class, with the class registry or Jev's class answer resolving the class.
- The error review runs right after an introspection is recorded.
- The scope guard already has line grouping (group_lines, line_attempts); the work is to ensure counting follows previous_job and request text lineage, and to test it.
- No athena.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] Test (cargo test): two introspections with the same cause on different jobs queue exactly one urgent root-cause request citing both introspection ids via the shared steward request builder; a third introspection with that cause does not queue another while that request is open or landed.
2. [behavior] Test (cargo test): a single-job repeat, or causes already cited by an open or landed request, queue nothing.
3. [behavior] Test (cargo test): athena status output lists each open repeated cause with its count and the request addressing it.
4. [behavior] Test (cargo test): the scope guard counts a retry (previous_job) and a re-ask of the same request text as one line of work and pauses at its threshold, not per job id.
5. [behavior] cargo fmt --check, cargo clippy with -D warnings, and cargo test all pass.
6. [content] CHANGELOG.md has an entry and docs/design/hestia.md describes the error review (grouping key, trigger, dedupe, status display).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how introspections are written beyond what grouping needs
- Automatically fixing the root causes
- Changing scope guard thresholds

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests and clippy/fmt/test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check queued requests and status output.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Review is code-enforced, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Queued requests are root-cause requests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by running tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 69cb5d994f2f → candidate b781717d2567. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  11 +
 crates/athena-cli/src/display.rs      |  66 ++++
 crates/athena-core/src/hestia.rs      | 679 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/introspect.rs  |   8 +
 crates/athena-core/src/scope_guard.rs | 140 ++++++-
 crates/athena-core/src/status.rs      |  17 +
 docs/design/hestia.md                 |  46 +++
 7 files changed, 952 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1406 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy with -D warnings, and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
