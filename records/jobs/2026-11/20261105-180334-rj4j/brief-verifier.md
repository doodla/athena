# Job 20261105-180334-rj4j: verify "Give each build slot a warm, size-limited target directory"

## Request (verbatim)

> Slot-owned warm target directories (step 5). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - each build slot owns one target directory that outlives jobs; a job's build uses its slot's directory for the command's duration
> - after each command the slot directory keeps third-party output and drops workspace-crate output
> - a slot directory over its size limit is deleted and rebuilt
> - tests cover reuse between two jobs and the size limit
> Depends on 20261105-180325-s9pg

## Reading (accepted by the user)

Build step 5 of docs/design/agent-and-build-slots.md, following job 20261105-180325-s9pg (landed, pass). Each build slot owns a persistent target directory under the state directory; a local check's build uses it for one command, then workspace-crate output is cleaned, and a directory over the limit is deleted and rebuilt. Audience: the maintainer, who judges it by the reuse and size-limit tests and the second-build time.

Assumptions:
- Code survey: read crates/athena-core/src/slots.rs (SlotQueue::acquire returns a Slot guard holding slots/slot-<n>.lock; Slot has no index yet), crates/athena-core/src/run/checks.rs (SnapExec::exec takes the slot after the disk gate and holds it past the snapshot restore, then calls run_check), crates/athena-agent/src/env.rs ({job_dir}/target/{side} is the per-job target path), crates/athena-core/src/state.rs (slots_dir at line 332) and crates/athena-core/src/disk.rs (measure_build, Figures). The slot must expose its index so a directory <state>/build-slots/<n>/target can be linked to {job_dir}/target/<side> for one command. No new record or data file under crates/ is added, so no check_record_file/AnyRecord change; the property is measured directly by tests that run two jobs through the same slot and by a test with a directory over the limit.
- Prior job 20261105-180325-s9pg is landed and passed; its findings are addressed as follows. Extra-file changes (drain.rs, checks.rs, introspect.rs, reading.rs, routing.rs, verify.rs, close.rs, send_back.rs, tests) are kept as landed and are not touched again. Baseline and fails-without reruns not taking a slot stays a known gap, out of scope here. The shared hold-marker-shape nit is out of scope. The generic hold-wording note is kept. The verifier's stale shared target dir was an environment problem and does not apply.
- Cleaning uses cargo clean -p for each workspace member, as in design section 5.3 step 3. cargo clean is a maintained cargo feature, so nothing is bought or built beyond that. The size limit is a new PipelineConfig parameter slot_max_gb with a default; athena.toml and defaults.toml are not changed (the design says the CARGO_PROFILE_DEV_DEBUG/CARGO_INCREMENTAL settings are a separate maintainer job).
- The env.rs symlink is lexical inside the job directory; if its check resolves links, the symlink is placed under the job directory and passes as the design requires.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core slots: a test runs two jobs' checks one after the other through the same slot and asserts both used the same slot directory path under build-slots/<n>/target, and that third-party output written by the first is still present for the second.
2. [behavior] Run cargo test -p athena-core: after a command, a test asserts the slot directory keeps third-party output and workspace-crate output is gone (cargo clean -p per workspace member, or an equivalent shown by the test).
3. [behavior] Run cargo test -p athena-core: a test makes a slot directory larger than slot_max_gb and asserts it is deleted and recreated empty before the next use, and one under the limit is kept.
4. [content] Each of max_check_slots build slots owns one directory <state>/build-slots/<n>/target created in crates/athena-core/src/state.rs that is not removed when a job ends; the job's target path is linked to it only for the command's duration (read crates/athena-core/src/run/checks.rs and crates/athena-agent/src/env.rs).
5. [content] PipelineConfig in crates/athena-data/src/config.rs gains slot_max_gb with a default and validation, with a test, and docs/design/agent-and-build-slots.md section 8 step 5 is marked Built with the choices made and what was measured or not; CHANGELOG.md has an entry.
6. [behavior] Run cargo test --locked, cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check: all pass.
7. [content] athena.toml and defaults.toml are unchanged, and the diff shows the effect can be read: the second build's time in the same slot is recorded or reported by measure-build in disk.rs, or the design note says it is not yet.

## Expected deliverables

- crates/athena-core/src/slots.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-agent/src/env.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/design/agent-and-build-slots.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Producer cargo wrapper and build-slot subcommand (step 4)
- Landing rebuild slot use (step 7)
- Changes to athena.toml or defaults.toml, including CARGO_PROFILE_DEV_DEBUG and CARGO_INCREMENTAL settings
- Wrapping the baseline run and fails-without reruns in a slot
- Re-changing files landed by job 20261105-180325-s9pg that are not listed as deliverables: crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-cli/tests/status_durations.rs, crates/athena-core/src/{checks.rs,introspect.rs,reading.rs,routing.rs,verify.rs,status.rs}, crates/athena-core/src/run/{close.rs,send_back.rs}, crates/athena-data/src/records.rs (kept as landed)
- Changing the sample metrics in records/
- Redesigning the shared hold-marker file shape

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover reuse between two jobs and the size limit; test, clippy and fmt criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check slot-directory reuse and the size-limit deletion happening, plus the second-build time.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test runs that exercise reuse, cleaning and deletion.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Cleaning and the size limit are enforced in code and tested, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One step of the build order; governance files left to the maintainer.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Uses cargo clean -p; the slot directory is built because no tool offers it (design 5.2).
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 084d4122af51 → candidate 01590c42c2e8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   8 ++
 crates/athena-agent/src/env.rs       |  93 +++++++++++++-
 crates/athena-core/src/run/checks.rs | 125 ++++++++++++++++++-
 crates/athena-core/src/run/mod.rs    |   8 +-
 crates/athena-core/src/run/tests.rs  | 230 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/slots.rs      | 188 +++++++++++++++++++++++++++-
 crates/athena-core/src/state.rs      | 139 +++++++++++++++++++++
 crates/athena-data/src/config.rs     |  30 +++--
 docs/design/agent-and-build-slots.md |  36 ++++++
 9 files changed, 838 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180334-rj4j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180334-rj4j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42099280737 for 01590c42c2e87a1a2496a363103dfeb2596062b8)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42099280737 for 01590c42c2e87a1a2496a363103dfeb2596062b8)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42099280737 for 01590c42c2e87a1a2496a363103dfeb2596062b8)
- ci-result: `not a check` passed (Check results for 01590c42c2e87a1a2496a363103dfeb2596062b8 come from the CI run https://github.com/doodla/athena/actions/runs/42099280737 (655s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test --locked, cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
