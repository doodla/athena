# Job 20261029-041429-3fdr: verify "Make Athena's own writers avoid boundary words in requests and records"

## Request (verbatim)

> Priority: high
> Athena's own writers produce text its boundary scan then refuses. On 2026-10-29 a steward proposal described a recurring duty with a boundary word for the person running Athena; approving it failed at admission because the request carried that word, so the proposal could never be worked. Every writer of text that later becomes a request or a record (the steward and its proposals, Hestia's error review and root-cause requests, the introspection writer) must check its text against the boundary word list before writing and use the plain alternative (for example 'the person running Athena').
> Acceptance:
> - A test feeds each writer a fixture whose source text contains a boundary word and shows the written proposal or request contains none and passes the admission scan.
> - A test shows approve-work on such a proposal succeeds.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/steward.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/introspect.rs, CHANGELOG.md

## Reading (accepted by the user)

The steward, Hestia's error review and root-cause requests, and the introspection writer must run their text through the boundary scan before writing. Any boundary word is replaced with a plain alternative such as 'the person running Athena', so written proposals and requests pass admission and approve-work succeeds. The audience is the person running Athena, who approves proposals, and the admission scan that judges them.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (boundary_words, boundary_fragments, boundary_scan, which matches whole words case-insensitively plus fragments and returns BoundaryHit line and term), crates/athena-core/src/work_decisions.rs (approve_work builds the request from the work item and goes through normal admission), crates/athena-core/src/steward.rs (ProposedChange::from_change, render, blind_spot_change), crates/athena-core/src/hestia.rs (root_cause_request) and crates/athena-core/src/introspect.rs. Admission runs check_boundary_and_repo_files in crates/athena-core/src/reading.rs.
- The fix is one shared sanitiser, built on boundary_scan and boundary_words, that rewrites boundary words to plain alternatives (the person running Athena for the first word in the list, and similar for the others). Each writer calls it before text is written; writers that cannot rewrite safely return an error that names the word.
- No new record or data file is added under crates/, so the records allow-list (check_record_file, AnyRecord) is not touched. The shared helper may live in one of the three listed files or in athena-context next to boundary_scan, which is an extra deliverable.
- Hestia's error review writer lives in hestia.rs, and the introspection writer in introspect.rs. Fixtures must build the boundary word from fragments so no tracked file contains it, because the repository policy scan fails on tracked files.
- This change makes writers refuse or rewrite more text but adds no new admission rule, so no replay refusal-rate criterion is needed: the admission scan itself is unchanged.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A test per writer (steward proposal, Hestia error review, Hestia root-cause request, introspection writer) feeds a fixture whose source text holds a boundary word (built from fragments) and asserts the written text has no hits from boundary_scan and says 'the person running Athena' or another plain alternative.
2. [behavior] Run `cargo test -p athena-core`. A test seeds a steward proposal from a source with a boundary word, calls approve_work, and asserts it succeeds and the submitted request passes check_boundary_and_repo_files.
3. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test policy`.
4. [content] CHANGELOG.md has an entry describing that writers now keep boundary words out of proposals and requests, with no boundary word in the entry itself.
5. [content] steward.rs, hestia.rs and introspect.rs each call the shared boundary check before writing text that becomes a request or record; a Grep of the diff shows the call in all three.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)

## Out of scope

- Changing the boundary word list or the admission scan's rules
- Changing athena.toml, defaults.toml or workflows
- Rewriting records already written

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-writer tests and an approve-work test; cargo test and clippy -D warnings must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the written text against boundary_scan and approve_work succeeding.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared check used by every writer, instead of fixing only the steward proposal that failed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Tests enforce that each writer's output passes the scan.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Four unit tests, one per writer, each seeded with a boundary word built from fragments (for example `["oper","ator"].concat()`). Each asserts that `athena_context::briefs::boundary_scan` returns no hits on the written text and that the text contains 'the person running Athena'. (a) Steward proposal, steward.rs tests: call `validate_items` with a `StewardItemOutput` whose title and request hold the word, source and evidence from `records_with_job`, then `decide` with `RequestQueue` and a role with a high enough limit; read the queued request from `QueuedRequest.file` and the title from `WorkItem.title`; assert `boundary_scan` and `crate::reading::scan_request_input` (the admission scan) are empty on both; also `decide` with a role below the rung and assert the stored work item title is clean. (b) Hestia error review, hestia.rs tests: `athena_in` and `put`, seed two introspections in different jobs with one class whose cause holds the word, as in `two_introspections_of_one_cause_in_different_jobs...`; call `athena.error_review(&second, &Quiet)`; assert `Some(job)` came back (so `queue` admitted it) and the request text read from `records(&athena)` is clean. (c) Hestia root-cause request: call `root_cause_request(&cause, &records)` on a `RepeatedCause` from `repeated_causes`; assert the returned text is clean. (d) Introspection writer, introspect.rs tests: existing `athena_with(None)`, a `redactor` built from `JobConfig::at(...).config`, an empty `Records::default()` and an `Answer` whose what_happened, cause, prevention and proposed_request hold the word; call `athena.apply_answer(...)`; assert each stored `IntrospectionRecord` field is clean and the status is Recorded. Every seam is an existing function or fixture; the producer confirms reachability of `redactor`, `Records::default` and `apply_answer` from the introspect tests module. Guards: `decide` needs the rung/limit for the role; `error_review` needs two jobs in one class. Run: `cargo test -p athena-core boundary`. Observes: Four tests pass, each reporting no boundary_scan hits on the written text and the plain phrase present; (b) also shows the request was admitted.
  Fails without the behavior: With the `plain_boundary_text` calls removed from the writers, `boundary_scan` finds the word in each written text so each of the four assertions fails; the (b) test also fails because `queue` refuses the request through `refuse_banned_input`.
- Criterion 2: Test in work_decisions.rs `tests` (athena-core). Build a repository with `athena.toml` and `defaults.toml` as the `repo`/`athena_with` helpers in run_pin_tests.rs and hestia.rs do (the producer checks whether `crate::test_support::make_athena` already supplies a reader script). Commit a proposal-kind work item whose source is a committed file, or a cited job, whose text holds a boundary word built from fragments. The `Athena` uses a `FakeAgentDriver` whose reader turn returns a valid reading with a capability registered (an existing seam, used in run_pin_tests.rs); the double only returns a canned reading, and the sanitising logic under test is the production `approve_request`. Call production `athena.approve_work(id, &Quiet)`. First build the raw (unsanitised) text in the test itself and assert it has `boundary_scan` hits, so the seed is shown to trigger the failure. Run: `cargo test -p athena-core approve_work`. Observes: approve_work returns Ok. The stored job request text has an empty `crate::reading::scan_request_input` and no `boundary_scan` hits, and contains 'the person running Athena'. `crate::reading::check_boundary_and_repo_files(&reading, None)` is Ok on the stored reading. A `WorkDecision` record names the approved job.
  Fails without the behavior: Without the cleaning step in `approve_request`, `approve_work` calls `ask`, which calls `refuse_banned_input` and returns the 'request was not admitted' error; this is the 2026-10-29 failure and the test's unwrap fails.
- Criterion 3: The full test suite, which includes the per-writer tests of criterion 1, the approve-work test of criterion 2, the briefs.rs unit tests of the helper, and the athena-cli tests (work_decisions.rs, drain.rs). The verifier also runs these as separate simple commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test -p xtask --test policy`. The policy test scans tracked files, including the new tests, helper doc comments, the replacement table and the CHANGELOG entry; all boundary words in them are built from fragments. Run: `cargo test --locked`. Observes: All four commands exit 0; no existing test that compares exact text breaks, because text with no hit is returned unchanged.
  Fails without the behavior: If a writer stopped cleaning, its per-writer test or the approve-work test inside `cargo test --locked` fails. If a test or the CHANGELOG entry held a literal boundary word, `cargo test -p xtask --test policy` fails. If the helper or its callers were unformatted or left unused or wrongly typed code, fmt or clippy -D warnings fails.
- Criterion 4: Read CHANGELOG.md: the Unreleased section has a bullet saying the writers (the steward and its proposals, Hestia's error review and root-cause requests, the introspection writer, and approve-work) now rewrite boundary words to plain wording before text becomes a request or record, naming the single shared check. The policy scan reads CHANGELOG.md. Run: `cargo test -p xtask --test policy`. Observes: The policy test passes, so the entry holds no boundary word; a Grep of CHANGELOG.md for 'boundary' or 'plain' shows the new bullet.
- Criterion 5: Inspect the diff for calls to the shared helper in the three writer files. Run: `git diff -U0 -- crates/athena-core/src/steward.rs crates/athena-core/src/hestia.rs crates/athena-core/src/introspect.rs`. Observes: Each file has an added line calling `plain_boundary_text` in a non-test function: steward.rs in `StewardRequest::new` and `validate_item`; hestia.rs in `root_cause_request_at` or `try_error_review`; introspect.rs in `apply_answer`. work_decisions.rs has it in `approve_request`.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b8607d760b67 → candidate 69990e7e5d1c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  12 +++
 crates/athena-context/src/briefs.rs      | 166 +++++++++++++++++++++++++++++++
 crates/athena-core/src/hestia.rs         |  94 +++++++++++++++--
 crates/athena-core/src/introspect.rs     | 137 ++++++++++++++++++++++++-
 crates/athena-core/src/steward.rs        | 134 ++++++++++++++++++++++++-
 crates/athena-core/src/work_decisions.rs | 138 ++++++++++++++++++++++++-
 6 files changed, 662 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1718 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
