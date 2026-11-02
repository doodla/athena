# Job 20261102-013826-mq3a: verify "Stop pending-check merge refusals firing D87; append note to decisions"

## Request (verbatim)

> try again on 20261102-005332-8gkv
> The code fix passed review; only the decision note was wrong. docs/decisions.md is append-only: do not edit the D87 entry. Add the note as a new entry at the end of the file that cites D87 and says pending-check merge refusals no longer fire its signal.
> - Mark the pending refusal on the merge call it belongs to, not on the last logged merge entry.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: urgent

## Reading (accepted by the user)

Retry of job 20261102-005332-8gkv: keep its code fix (merge 405/422 refusals logged while required checks were pending no longer match D87's hard signal), mark the pending refusal on the merge call itself, and put the decision note in a new final entry of docs/decisions.md citing D87, leaving D87 unedited. Audience: the maintainer reading decision-state records.

Assumptions:
- Previous job 20261102-005332-8gkv: its blocker (in-place edit of D87 in docs/decisions.md, tripping the append-only guard in crates/athena-cli/src/decisions.rs) is fixed by reverting that edit and appending a new final entry citing D87. Its minor finding (pending mark tied to last logged Merge entry, relying on call order) is fixed by passing the pending flag with the refusal being logged, not marking the last entry. Its nit (transport.rs changed unlisted) is fixed by listing transport.rs as a deliverable. The rest of its code fix was judged sound and is kept.
- Code survey: read crates/athena-core/src/watcher.rs (D87_SIGNAL, HostErrorSignal::matches, evaluate), crates/athena-core/src/gh/client.rs, landing.rs and transport.rs (HostErrorLog, land_pr rule-violation then Checks::Pending branch), and crates/athena-data/src/records.rs. The records validation is check_record_file(path, bytes) -> Option<AnyRecord> at records.rs:2932, over the AnyRecord enum at records.rs:2618. A new optional serde-default checks_pending field on HostError is accepted by that path, and old records such as records/host-errors/2026-11/20261101-174051-3t6g.json must still pass check_record_file unchanged. The property is measured directly: evaluate over host-error records, and the logged record from land_pr.
- The decision log guard in crates/athena-cli/src/decisions.rs rejects any change to the first bytes of docs/decisions.md, so only appending is allowed. The new entry takes the next free D number at the end of the file.
- Mark the pending refusal on the merge call it belongs to, by passing the flag with the refusal or via a handle from that call, not by flagging the last logged Merge entry.
- The previous candidate's files are kept as deliverables: CHANGELOG.md, client.rs, landing.rs, tests.rs, transport.rs, watcher.rs, records.rs and docs/decisions.md (now append-only).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core watcher`: a test shows a merge 405 host error marked as caused by pending checks does not make `evaluate` fire D87, and an unmarked 405 or 422 still fires D87 on the first job.
2. [behavior] Run `cargo test -p athena-core gh`: a landing test shows a merge refusal while checks are pending is logged with the pending mark on that same merge call's entry even when other entries are logged after it, and a refusal with checks settled is logged unmarked as before.
3. [behavior] Run `cargo test -p athena-data`: a test through check_record_file/AnyRecord shows an old-format athena.host-error/1 record (as in records/host-errors/2026-11/20261101-174051-3t6g.json) still validates and serialises unchanged.
4. [behavior] Run `cargo test -p athena-cli decisions`: the append-only guard test passes, so docs/decisions.md only gained text after its previous end.
5. [content] `git diff` of docs/decisions.md shows only added lines at the end of the file: a new entry that cites D87 and says pending-check merge refusals no longer fire its signal. No D87 line is changed.
6. [content] Grep of crates/athena-core/src/watcher.rs shows the D87_SIGNAL doc comment says pending-check refusals are excluded, and the name string "host-error: operation merge, status 405 or 422, phase landing" is unchanged.
7. [content] CHANGELOG.md has an entry naming the pending-check 405 false positive and stating that existing records/ are untouched; no file under records/ is in the diff.

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Editing any file under records/ or clearing the open D87 review
- Editing the D87 entry or any existing entry of docs/decisions.md
- Reading branch protection to find the required check name
- Changes to athena.toml, defaults.toml or .github/workflows
- Drain wiring in crates/athena-cli/src/drain.rs
- Adding signals for decisions other than D87

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a pending-check refusal does not fire D87 and a real one still does.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for watcher, landing and old-record validation.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The matcher excludes flagged records instead of special-casing one instance.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus docs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in watcher.rs. They build host-error records with the existing test helper `error(id, job, op, status)`, set `checks_pending = true` on a copy, and call the production `evaluate(&host_error_signals().unwrap(), &errors, &[])`. Run: `cargo test -p athena-core watcher`. Observes: A merge 405 and a merge 422 with checks_pending give no firing. The same errors unmarked fire D87 on the first job (one job, hard signal), with the signal name unchanged. A marked and an unmarked record mixed together fire citing only the unmarked id.
  Fails without the behavior: With `matches` ignoring checks_pending, the marked 405 fires D87 and the test fails.
  Test seam: `evaluate` and `host_error_signals` in watcher.rs, plus the test helper `error` already in the tests module. The only change is a new field in the helper's struct literal (added in step 3), so the seam exists today.
- Criterion 2: New tests in gh/tests.rs run the production `land_pr` through a real `GhClient` with a `HostErrorLog` and a new scripted `Runner` fake (`PendingMerge`), built like the existing `Refusing` and `land_through` helpers. The fake answers repos GET with ALL_METHODS and check-runs with the required run green until the merge PUT is refused, then in_progress (pending). The merge PUT fails with `GhFailure{status_code:Some(405), text:"Repository rule violations found"}`. After that refusal, one status GET fails once with 500, so a later ReadChecks entry is logged and retried (Retry attempts 3, zero delay). Cases: (a) 405 with checks pending: the entry for the 405 merge call has checks_pending, it is not the last entry, and the later ReadChecks 500 entry is unmarked. (b) An earlier merge 500 retried and then 422 pending: only the 422 entry is marked. (c) 405 with the same text while checks are settled (green): the entry is unmarked and land_pr returns the error. (d) 405 without the rule-violation text while pending: unmarked. Run: `cargo test -p athena-core gh`. Observes: The marked entry is the merge refusal that was made while checks were pending, even though entries come after it; settled-check and non-rule-violation refusals are logged without the mark, as before. Existing gh tests such as a_refusal_that_is_not_transient_is_recorded_once_and_not_retried still pass.
  Fails without the behavior: Marking the last logged Merge entry, or nothing, fails (a)/(b): the marked flag is absent on the 405/422 entry, or lands on the wrong one. Never marking fails (a) and (b); always marking fails (c) and (d).
  Test seam: Production entry points: `land_pr` (landing.rs) driven by the `Rig::run_with` helper, and `GhClient::merge` (client.rs). The fake `Runner` is injected as `GhClient::new(&fake, ..)` with `log: Some(&log)`, as `land_through` does today. The `Rig`, `Runner` trait and `land_through` pattern exist; the `PendingMerge` fake is added in step 8. The fake only answers canned replies; the pending decision comes from production `classify_checks`.
- Criterion 3: A new test in records.rs holds the literal JSON of an old-format athena.host-error/1 record: id 20261101-174051-3t6g, path records/host-errors/2026-11/20261101-174051-3t6g.json, no checks_pending, no derived field. It passes through `check_record_file(path, bytes)`, matches `AnyRecord::HostError`, asserts `checks_pending == false`, and asserts that `to_json()` equals the input text. A second assertion round-trips a record with checks_pending true and shows the field is written only then. Run: `cargo test -p athena-data`. Observes: The old record validates at its canonical path, is unmarked, and re-serialises to identical bytes; the new field appears only when true; deny_unknown_fields still rejects a stray field.
  Fails without the behavior: If the new field were required or always written, parsing or the byte comparison fails.
  Test seam: `check_record_file` and `AnyRecord::to_json` in records.rs. Both exist; the fixture is an inline string in the test (records/ is absent from the producer snapshot, so the producer copies the field layout from `to_json` output of the existing `host_error()` helper). Added in step 4.
- Criterion 4: Run the existing guard test over the modified docs/decisions.md. It checks the pinned first 230046 bytes, ascending unique ids, and a Provenance bullet in every entry, including D150. Run: `cargo test -p athena-cli decisions`. Observes: `the_decision_log_is_well_formed_and_only_appended_to` passes with D150 appended; a changed D87 byte would fail the hash check.
  Fails without the behavior: An edit inside D87 or any earlier entry changes the pinned prefix hash and the test panics; a D150 without a Provenance bullet also fails.
  Test seam: `check_log` in crates/athena-cli/src/decisions.rs reads the real docs/decisions.md; it exists and is not changed.
- Criterion 5: Read the diff of docs/decisions.md. Run: `git diff -- docs/decisions.md`. Observes: Only `+` lines after the final line of D149. The new entry is headed `## D150 · ...`, cites D87 and says that pending-check merge refusals no longer fire its signal. No `-` line and no changed hunk before the end.
- Criterion 6: Read the diff of watcher.rs. Run: `git diff -- crates/athena-core/src/watcher.rs`. Observes: The D87_SIGNAL doc comment now says pending-check refusals are excluded. The `name:` line "host-error: operation merge, status 405 or 422, phase landing" appears only as context, not as `-` or `+`.
- Criterion 7: Read the CHANGELOG diff and the list of changed files. Run: `git diff --stat`. Observes: CHANGELOG.md has a new Fixed bullet naming the pending-check 405 false positive (D87) and saying the existing records/ are untouched. No path under records/ appears in the stat.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/gh/transport.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7784767ba677 → candidate 20109a2e2cdb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   9 ++
 crates/athena-core/src/gh/client.rs    |  69 +++++++++--
 crates/athena-core/src/gh/landing.rs   |   6 +-
 crates/athena-core/src/gh/tests.rs     | 201 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/transport.rs |  15 ++-
 crates/athena-core/src/watcher.rs      |  44 +++++++-
 crates/athena-data/src/records.rs      |  32 ++++++
 docs/decisions.md                      |  30 +++++
 8 files changed, 390 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41376530242 for 20109a2e2cdbc70ce4eb32522eddc7473dfa4d28)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41376530242 for 20109a2e2cdbc70ce4eb32522eddc7473dfa4d28)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41376530242 for 20109a2e2cdbc70ce4eb32522eddc7473dfa4d28)
- ci-result: `not a check` passed (Check results for 20109a2e2cdbc70ce4eb32522eddc7473dfa4d28 come from the CI run https://github.com/doodla/athena/actions/runs/41376530242 (594s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
