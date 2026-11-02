# Job 20261102-005332-8gkv: verify "Stop pending-check merge 405s from firing D87's hard signal"

## Request (verbatim)

> try again on 20261102-001719-1fva

## Reading (accepted by the user)

Retry of job 20261102-001719-1fva (dispatched, never verified, no findings, no candidate recorded). Landing merge refusals (405/422) that happened while required checks were pending must not count toward D87 "merge commits not allowed"; real refusals with checks complete still fire on the first job. Audience: the maintainer reading decision-state records.

Assumptions:
- Previous result: job 20261102-001719-1fva had no candidate and no findings, so nothing is kept or dropped; this retry does the whole change fresh with the same deliverables and out_of_scope as that job.
- Code survey: read crates/athena-core/src/watcher.rs (D87_SIGNAL, load, HostErrorSignal::matches, evaluate; matching is exact on operation, phase and status), crates/athena-core/src/gh/landing.rs, gh/client.rs and gh/tests.rs (merge refusals are logged as host-error records at landing; pending-check handling exists in gh/early.rs, landing.rs, client.rs), and crates/athena-data/src/records.rs. The records validation is check_record_file, which parses into AnyRecord (enum at records.rs:2618, check_record_file at :2932). If HostError gains an optional field (e.g. checks pending), it must be added with serde default so old records still deserialize, AnyRecord accepts it, and the existing round-trip/validation tests plus a new check_record_file test on an old-format host-error JSON cover it. The property is measured directly by running evaluate over HostError values and by landing tests in gh/tests.rs, not via a stand-in.
- Assumed mechanism: at the merge refusal, landing already knows whether checks were pending; the refusal is recorded with a distinguishing marker (or message_class) and HostErrorSignal::matches skips it, keeping D87_SIGNAL's name string unchanged.
- Existing records under records/ are never edited, and the open D87 review (20261031-085139-1y06) is not cleared.
- This narrows a firing rule (fewer firings), not a refusal rule on readings, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core watcher`: a new test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and a 405/422 without that marking still fires D87 on the first job.
2. [behavior] Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged distinguishably (or not as a D87-matching error), while a refusal with checks complete is logged as before.
3. [behavior] Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates unchanged after any records.rs change.
4. [content] Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment states pending-check refusals are excluded, and the signal name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged so record 20261031-085139-1y06 still matches.
5. [content] CHANGELOG.md has an entry naming the pending-check 405 false positive and stating existing records/ are untouched; docs/decisions.md D87 notes the exclusion.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Editing any file under records/ or clearing the open D87 review
- Reading branch protection to find the required check name
- Changes to athena.toml, defaults.toml or .github/workflows
- Drain wiring in crates/athena-cli/src/drain.rs
- Adding new decisions' signals other than D87

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New watcher, gh and records tests; fmt, clippy -D warnings and cargo test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the signal's misclassification of pending-check refusals, not just the one open review.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing the firing behavior.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the D87 signal path.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Exclusion is enforced in code in matches/evaluate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in the watcher module call `evaluate` over HostError values with the real `host_error_signals()` / `load(&D87_SIGNAL)`. (a) A merge/landing/405 error with `checks_pending: true`, and the same with 422, gives no firing. (b) The same errors with `checks_pending: false` give one D87 firing on the first job (one distinct job, threshold 1). (c) A mix of a flagged and an unflagged error from one job fires citing only the unflagged id. Run: `cargo test -p athena-core watcher`. Observes: The new tests pass. The existing `derive_host_error` 405 test (watcher.rs:472, `load(&D87_SIGNAL).unwrap().matches(&derived)`) still passes.
  Fails without the behavior: Without the `!checks_pending` guard in `HostErrorSignal::matches`, test (a) sees a firing and fails. Without the field, the tests do not compile. Test (b) guards against over-excluding: an implementation that skips every 405/422 fails it.
  Test seam: Production entry points: `watcher::evaluate` and `HostErrorSignal::matches`. Input is plain `HostError` values built by the existing test helper `error(id, job, op, status)` at watcher.rs:426, extended with the new field. No double is used. The seam exists today; the field is added in step 1 and the tests in step 6.
- Criterion 2: Landing tests in gh/tests.rs drive `land_pr` through a real `GhClient` with a `HostErrorLog`, using the existing `land_through` / `Refusing` runner / `Rig` seam (tests.rs:2354-2426). Extend `Refusing` with optional merge-refusal text ("Repository rule violations found") and an option to answer the check-runs read with an in-progress optional run once a merge has been attempted. Test A: the 405 refusal, with checks pending afterwards, gives Ok(report) with checks Pending and one log entry (Merge, 405, MethodNotAllowed) with `checks_pending == true`. Test B: the same refusal with checks still green (settled) gives Err and one entry with `checks_pending == false`. Test C: a 405 "Pull request is not mergeable" refusal while checks are pending stays an error and the entry is unflagged. Test D: `HostErrorLog::mark_last_pending` marks only the latest Merge entry. Run: `cargo test -p athena-core gh`. Observes: The log entries returned by `log.entries()` carry the flag only in test A. The existing tests at tests.rs:1169-1207 (wait, settled error, other refusal) and 2429-2475 (500 retries, non-transient statuses recorded once) pass unchanged, so refusals with checks complete are logged as before.
  Fails without the behavior: Without the `note_pending_merge_refusal` call in the `land_pr` Pending branch, or without the `GhClient` override, test A sees `checks_pending == false`. Without the rule-violation and Pending guards, B or C sees a flagged entry.
  Test seam: Production entry points: `gh::land_pr` (via `Rig::run_with`) and `GhClient::merge` / `HostErrorLog`. The fixture is the `Refusing` runner passed as `&dyn Runner` to `GhClient`. It only scripts HTTP replies and refusal text; the pending decision and the marking are production code (`classify_checks`, `land_pr`, `GhClient`). The `land_through` helper exists. The `Refusing` extension and the new tests are added in step 7.
- Criterion 3: Records tests. (a) A hard-coded old-format athena.host-error/1 JSON literal, with the same fields the 3t6g record has (schema, id, at, status_code 405, operation merge, job, phase landing, message_class method-not-allowed) and no `checks_pending`, goes through `check_record_file("records/host-errors/2026-11/<id>.json", bytes)`. It returns Ok(Some(AnyRecord::HostError(h))) with `h.checks_pending == false`, and `AnyRecord::to_json()` of it contains no `checks_pending` text. (b) A record with `checks_pending: true` round-trips through `AnyRecord::parse` and writes `"checks_pending": true`. (c) An unknown extra field is still refused (the existing deny_unknown_fields test, kept). Run: `cargo test -p athena-data`. Observes: The old-format record validates unchanged and serializes without the new key. The new-format record round-trips. The existing host-error tests (records.rs:3291, 3313) pass with the struct literal updated.
  Fails without the behavior: If the field were required (no `serde(default)`), the old-format JSON would fail with a parse error in `check_record_file`. If `skip_serializing_if` were missing, the 'no checks_pending text' assertion would fail.
  Test seam: Production entry points: `athena_data::records::check_record_file` and `AnyRecord::parse` / `to_json`. The input is a JSON literal in the test, with no double. The seam exists today (records.rs:2932, 2704). The test is added in step 6.
- Criterion 4: Read the D87_SIGNAL doc comment and name in watcher.rs. The name line `name: "host-error: operation merge, status 405 or 422, phase landing"` is unchanged. The existing watcher test that loads the signal and the 1y06 review fixture keep passing as the mechanical check. Run: `cargo test -p athena-core watcher`. Observes: Grep of crates/athena-core/src/watcher.rs for `pending` in the lines above `pub const D87_SIGNAL` finds the new doc sentence. A grep for the exact name string finds it once, unchanged. The existing watcher tests that use the D87 name (the `d87.decision` / firing tests near watcher.rs:582-628) pass.
- Criterion 5: Content check. CHANGELOG.md under [Unreleased] / Fixed gets an entry that names the pending-check merge 405 false positive, gives the root cause (landing logged the refusal that it treated as a wait, and the signal matched on operation, phase and status alone) and the fix, and states that existing records/ are untouched and review 20261031-085139-1y06 is not cleared. The D87 entry in docs/decisions.md (line 1907) gets a note that refusals during pending checks are excluded. The tool is Grep; any CI test that scans CHANGELOG/decisions format also stays green under `cargo test`. Run: `cargo test`. Observes: A grep of CHANGELOG.md for `405` and `records/` finds the new entry. A grep of docs/decisions.md for `pending` near D87 finds the note. The full test run, including any docs/format tests, passes.
- Criterion 6: Run the three Athena checks. Fix all HostError struct literals (watcher.rs:436, records.rs:3286, transport.rs:323, watcher.rs:254) so everything compiles. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All pass, including the existing watcher test deriving a 405 host error from an intervention (watcher.rs:472).
  Fails without the behavior: A missed `checks_pending` literal fails to compile. A clippy lint on the new method fails the check.
  Test seam: The commands run the whole workspace. The seams are the ones above.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 2600b2403b36 → candidate 0388b27731b1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           | 10 ++++
 crates/athena-core/src/gh/client.rs    | 10 ++++
 crates/athena-core/src/gh/landing.rs   | 10 +++-
 crates/athena-core/src/gh/tests.rs     | 87 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/gh/transport.rs | 10 ++++
 crates/athena-core/src/watcher.rs      | 59 +++++++++++++++++++++--
 crates/athena-data/src/records.rs      | 44 +++++++++++++++++
 docs/decisions.md                      |  5 ++
 8 files changed, 228 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41372212254 for 0388b27731b1a1bf3f3b083f79c0d493db7bd9a7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41372212254 for 0388b27731b1a1bf3f3b083f79c0d493db7bd9a7)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41372212254 for 0388b27731b1a1bf3f3b083f79c0d493db7bd9a7)
- ci-result: `not a check` passed (Check results for 0388b27731b1a1bf3f3b083f79c0d493db7bd9a7 come from the CI run https://github.com/doodla/athena/actions/runs/41372212254 (535s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass; `cargo test` passes including existing watcher tests (the 405 method-not-allowed derivation test).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
