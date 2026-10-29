# Job 20261029-073932-9648: verify "Require a mechanism, owner and measure for each active spec requirement"

## Request (verbatim)

> Priority: urgent
> Make docs/spec.toml, Athena's own requirements, say how each requirement is met, so an active requirement can never sit without a mechanism (Sri, 2026-10-29: the brief is temporary and informs requirements). Today each requirement has a statement, kind, status and source quotes, but nothing records what meets it, so the requirement that Athena gets measurably faster and owns its acceleration went two weeks with no mechanism. Add to every active requirement: mechanism (the code path, role or check that meets it, or none), owner (the role responsible) and measure (how it is shown, and where in athena status). The Hestia sweep flags any active requirement with no mechanism or a failing measure and queues one request for it through the shared steward request builder (urgent for requirements about speed, per OP-23); athena status lists the requirements without a mechanism. Fill the new fields for the existing requirements from the current code; where nothing meets one, say none.
> Acceptance:
> - Every active requirement in docs/spec.toml has mechanism, owner and measure, and the spec test fails when one is missing.
> - A test with a fixture spec shows the sweep queues one request for a requirement whose mechanism is none, and none on a second run while it is open.
> - athena status lists active requirements with no mechanism.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: docs/spec.toml, xtask/tests/spec.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Add mechanism, owner and measure to every active requirement in docs/spec.toml, enforce them in the spec test, make the Hestia sweep queue one deduplicated steward request per requirement lacking a mechanism (urgent for speed), and list such requirements in athena status.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (default_checks, run_sweep, repeated_causes, root_cause_request building via crate::steward::StewardRequest), crates/athena-core/src/steward.rs (StewardRequest::new, queued_work) and crates/athena-core/src/status.rs, plus xtask/tests/spec.rs (serde Spec/Requirement with deny_unknown_fields, so new fields must be added to the struct). The sweep checks run in run_sweep over default_checks; the new check parses docs/spec.toml, dedups against open requests by citing the requirement id in request text (as addressing_request does), and needs no new record type under crates/, so check_record_file and AnyRecord are untouched.
- Goal: make the spec enforce that each active requirement has a meeting mechanism; audience is Sri and the Hestia sweep and status readers.
- The spec parser in athena-core is a small serde struct mirroring the xtask one, since xtask tests cannot be imported; athena-core likely already has the toml crate.
- A failing measure is determined by a simple recorded measure check where feasible; otherwise only the missing or none mechanism triggers a request, and the measure check is limited to the field being present. Superseded or non-active requirements are exempt.
- Mechanism text is filled from current code by the producer, which is judgment-heavy across 135 requirements.
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [content] Grep of docs/spec.toml shows every active [[requirement]] (135 entries today) carries mechanism, owner and measure; where nothing meets one, mechanism is the literal "none". The measure names where it appears in athena status.
2. [behavior] Run `cargo test -p xtask --test spec`: it passes on docs/spec.toml, and new in-memory negative cases fail an active requirement missing mechanism, owner or measure.
3. [behavior] Run `cargo test -p athena-core hestia`: a fixture-spec test shows the sweep queues exactly one request for an active requirement whose mechanism is none, built via StewardRequest, and none on a second run while the first is open. A requirement about speed gets an urgent request (OP-23). A failing measure is also flagged.
4. [behavior] Run `cargo test -p athena-core status`: a fixture test shows athena status lists active requirements with no mechanism, and lists none when all have one.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (workspace): all pass.
6. [content] CHANGELOG.md has an entry describing the mechanism, owner and measure fields, the sweep check and the status listing.
7. [content] The requirement that Athena gets measurably faster and owns its acceleration records a mechanism (or none) and an owner, so it is visible, not omitted.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Changing BRIEF.md or the source quotes in the spec
- Building the acceleration mechanism itself
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require fmt, clippy -D warnings and tests passing.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The spec test and sweep enforce the rule rather than prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Fixture tests check queueing, idempotence and status listing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Fields and docs are written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change within the five named files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Spec test over the real file plus a hestia parse of the same file. `check(&real_spec(), &real_brief())` in xtask/tests/spec.rs gets a rule: for each Status::Active requirement, mechanism blank, owner blank/'none', measure blank/'none', or a `status:` marker with nothing after it each push an error '<id>: no mechanism|owner|measure'. A new test `every_active_requirement_has_mechanism_owner_and_measure` asserts the count of active requirements equals the count with all three fields set, and lists the ids whose mechanism is the literal 'none'. A hestia test `the_real_spec_has_every_field_and_no_failing_measure` (include_str!('../../../docs/spec.toml') through production `parse_spec`) asserts no active requirement has a blank field and every `status:` label is in STATUS_MEASURE_LABELS. Entry point: `check` and `parse_spec`; guards: active status only; seam: both read the real file, exist today (check) or step 4 (parse_spec). Run: `cargo test -p xtask --test spec && cargo test -p athena-core hestia`. Observes: All active requirements (the existing 135 plus OLY-A-22 and OLY-A-23) pass; every measure names where it appears in athena status (a known label) or says 'status: not shown' with the test or check that shows it; the 'none' mechanism ids are listed in the test output.
  Fails without the behavior: A missing or blank field on any active requirement makes `the_real_spec_passes` fail with '<id>: no mechanism' (or owner, measure), and the hestia real-spec test fail on the blank or unknown-label measure.
- Criterion 2: In-memory negative cases on the pure function `check(&Spec, &str)` from xtask/tests/spec.rs, built from `real_spec()`. Test `an_active_requirement_missing_mechanism_owner_or_measure_fails` clears each field in turn on an active requirement, also sets owner 'none' and a measure 'status:' with no label, and asserts an error naming the id and field; the same blanks on a requirement set to Status::Superseded yield no error. Requirement gains `mechanism`, `owner`, `measure` as `#[serde(default)] String`, so a missing field is a check error, not a parse error. The pinned-heading test `only_the_pinned_late_headings_are_citable` asserts 'Athena owns acceleration' and 'Speed first' resolve and '2026-10-25: Evidence labels Jev' still reports 'no heading' (the old test is updated to use that heading). The seam (Requirement fields, PINNED_HEADINGS in `sections()`) is added in steps 2 and 3; `check` is a pure function today. Run: `cargo test -p xtask --test spec`. Observes: The real spec passes; each negative case fails with a message naming the id and the field; superseded requirements are exempt; late non-pinned headings stay rejected.
  Fails without the behavior: Without the rule, clearing a field on an active requirement produces no error and the negative test's assertion fails; without the status exemption the superseded case fails.
- Criterion 3: Tests in hestia.rs `mod tests` using the existing `athena_in(dir, state)` fixture (a real git repo with a fake driver). A helper writes a small fixture docs/spec.toml into that repo and commits it with the repo's git, as the existing `put` helper does for records. Production entry points: `Athena::spec_review(&Quiet)`, `spec_gaps`, `parse_spec`, `StewardRequest::new`, `Athena::close`. Cases: (a) one active requirement with mechanism 'none': the first `spec_review` queues exactly one request job whose text, reloaded from records, has an 'Acceptance:' bullet block, a 'Files:' line, and the line 'Spec requirement: <id>' (shows it passed through StewardRequest); a second `spec_review` queues nothing while that job is open; after `close` it queues one again. (b) a requirement with measure 'status: throughput' and mechanism none queues text starting 'Priority: urgent'; the learning-effectiveness-style statement containing 'faster' with measure 'status: repeated' queues at Normal. (c) a requirement with a mechanism but measure 'status: nowhere' (unknown label) or 'none' is queued with reason 'failing measure'; a measure 'status: not shown' with a named test is not queued. (d) superseded requirements and fully specified ones queue nothing; missing docs/spec.toml queues nothing. (e) with seven fixture gaps, one pass queues 5, speed first. Fixtures only supply spec text and a repo; queueing, dedupe and priority logic are all production code. `athena_in`, `queue` and `close` exist today; `spec_review`, `parse_spec`, `spec_gaps` are added in steps 4 to 6. Run: `cargo test -p athena-core hestia`. Observes: Exactly one queued request per gap on the first run, zero on the second while open, a fresh one after close; urgent only for a speed-measure gap; failing measures flagged; the learning-effectiveness-style 'faster' statement stays Normal; the pass line appears in review_passes output.
  Fails without the behavior: Without `spec_review` nothing is queued and the 'exactly one' assert fails; without the open-request dedupe the second run adds a request; without the speed rule the priority assert sees normal; without the measure rule case (c) queues nothing.
- Criterion 4: Test `athena_status_lists_requirements_without_a_mechanism` in hestia.rs mod tests (matches the 'status' filter via its name), with the same `athena_in` fixture as the existing repeated-cause status test. Commit a fixture docs/spec.toml with one mechanism-none requirement and one measure-only gap; call the production `athena.status()`; assert `view.spec_gaps` holds both ids, `next` has 'requirement <id> has no mechanism' for the first and 'has a failing measure' (not 'no mechanism') for the second. Commit a corrected spec and assert `spec_gaps` is empty and no such `next` line; a repo with no docs/spec.toml gives an empty list and no error. A status.rs unit test checks the label table drift (fields exist in StatusView, label texts appear in display.rs). `Athena::status` exists today; the field and fill are added in step 6. Also the CLI test file crates/athena-cli/tests/lifecycle.rs is not changed. Run: `cargo test -p athena-core status`. Observes: The listing appears with the none requirement, labelled by its true reason, disappears once every mechanism and measure is valid, and a missing spec does not error.
  Fails without the behavior: Without the status change `view.spec_gaps` does not compile; if it were not filled from the spec the 'lists the requirement' assert fails; if it listed complete requirements the empty-list assert fails.
- Criterion 5: Run the three checks Athena runs, as separate commands, including the boundary scan in xtask/tests/policy.rs over the new spec text and the budgets test over any prompt files (none are changed). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit zero; Cargo.lock is unchanged so --locked passes.
  Fails without the behavior: Formatting drift, a lint warning or any failing test fails the matching command; a new dependency would fail --locked.
- Criterion 6: Inspect the diff of CHANGELOG.md for an Unreleased entry in the style of the neighbouring entries, naming the mechanism, owner and measure fields, the spec test rule, the two new requirements, the sweep's spec review (one deduped request per gap, urgent for speed, per-pass cap) and the athena status listing. Run: `git diff HEAD -- CHANGELOG.md`. Observes: A new entry under Unreleased that names all of those parts.
  Fails without the behavior: A missing entry shows an empty diff for CHANGELOG.md; an entry that omits a part is visibly incomplete.
- Criterion 7: Two tests. In xtask/tests/spec.rs, `the_acceleration_requirement_records_a_mechanism_and_an_owner` looks up id OLY-A-22 in the real spec (source heading '2026-10-13: Athena owns acceleration'), asserts its status is Active, the statement says Athena gets measurably faster and owns its acceleration, mechanism is non-blank and names `throughput_review`, owner is 'Hestia', and measure contains 'status: throughput'; it also looks up OLY-A-23 and asserts non-blank mechanism (or the literal 'none') and owner. In hestia.rs, `the_acceleration_requirements_are_classed_as_speed` parses the real spec with `parse_spec` and asserts `is_speed` is true for OLY-A-22 and OLY-A-23 and false for the learning-effectiveness requirement; a fixture copy of OLY-A-22 with mechanism 'none' queues at Priority urgent. Seams: `sections()` PINNED_HEADINGS (step 3) and `is_speed` (step 5); the real spec is read from disk. Run: `cargo test -p xtask --test spec && cargo test -p athena-core hestia`. Observes: The acceleration requirement exists with a real mechanism (`Athena::throughput_review`), an owner and a throughput measure, so it is visible, and is classed as speed (urgent) if its mechanism is ever none.
  Fails without the behavior: If OLY-A-22 were absent the lookup fails; if its mechanism or owner were blank both the lookup test and the general rule fail; if the speed class were taken from keywords the learning-effectiveness assertion would fail.

Planned files:

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b19aeb8da0b0 → candidate f611a53e38e8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  15 +
 crates/athena-core/src/hestia.rs | 697 ++++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs |  52 +++
 docs/spec.toml                   | 446 ++++++++++++++++++++++++-
 xtask/tests/spec.rs              | 233 ++++++++++++-
 5 files changed, 1424 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1658 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (workspace): all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
