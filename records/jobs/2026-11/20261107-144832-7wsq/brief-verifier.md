# Job 20261107-144832-7wsq: verify "Link producer CARGO_TARGET_DIR to the slot's warm target dir"

## Request (verbatim)

> try again on 20261106-202436-ch54

## Reading (accepted by the user)

Retry of job 20261106-202436-ch54 (dispatched, never verified, no findings, no candidate recorded): in the producer's cargo wrapper, while a slot is held, link the job's CARGO_TARGET_DIR path to <slots_dir>/<n>/target with athena_agent::env::BuildLink, run cargo, drop the link. Unslotted subcommands are unchanged. Audience: the operator whose disk fills with two concurrent jobs; the verifier checks via tests.

Assumptions:
- Code survey: crates/athena-cli/src/slot_exec.rs run() reads CargoShim::from_env, finds the real cargo, takes a Producer slot via SlotQueue::acquire only for build/test/clippy/check, then runs the child. crates/athena-core/src/run/checks.rs link_slot_target is the model (prepare, BuildLink::create(job_dir, path, slot_dir), warning and unlinked build on failure). crates/athena-agent/src/env.rs BuildLink::create runs check_build_dirs (D46, lexical) and Drop removes the link; CargoShim has dir (= job_dir/SHIM_DIR_NAME), slots_dir, note. crates/athena-core/src/state.rs slot_target(n) = build_slots_dir/n/target; Slot::index() in crates/athena-core/src/slots.rs. No record or data file is added, so no records validation is touched.
- Job directory for BuildLink::create is derived as the parent of shim.dir; the slot target is slots_dir/<index>/target (created if missing, as make_target_dir does), matching the checks' layout. CARGO_TARGET_DIR is read through the lookup closure; absent or outside the job directory means a warning on stderr and cargo runs unlinked.
- Prior job 20261106-202436-ch54: it had no recorded findings or candidate, so nothing is kept or dropped; this job redoes the whole work from the base. Its deliverable list is kept as is, including crates/athena-agent/src/env.rs, which changes only if a small helper or doc tweak is needed (unlikely).
- Slot size-limit deletion (slot_max_gb) is not in the wrapper's settings; it is left out, as before.
- No governance files (athena.toml, defaults.toml, workflows) need changing.

## Acceptance criteria

1. [behavior] A test in crates/athena-cli/src/slot_exec.rs: a slotted `build` run with a fake cargo observes the job's CARGO_TARGET_DIR path as a symlink resolving to <slots_dir>/<n>/target during the command, and the path is gone after run returns. Check: cargo test -p athena-cli slot_exec.
2. [behavior] A test: two slotted runs one after the other in different job directories resolve to the same slot target directory, and a file written by the first run is still there in the second. Check: cargo test -p athena-cli slot_exec.
3. [behavior] A test: unslotted subcommands (fmt, metadata) create no link, leave the slot directory untouched, and CARGO_TARGET_DIR is unchanged. Check: cargo test -p athena-cli slot_exec.
4. [behavior] D46 still holds: BuildLink::create is called with the job directory; a test shows a CARGO_TARGET_DIR outside the job directory is refused (warning, cargo runs unlinked) while an inside path is linked. Check: cargo test -p athena-cli and cargo test -p athena-agent.
5. [behavior] cargo test (workspace) passes, cargo clippy --all-targets is clean, cargo fmt --check passes.
6. [content] CHANGELOG.md gains an entry stating only what the diff does: the producer's slotted cargo commands build in the slot's target directory via a temporary link; unslotted commands unchanged.
7. [content] git diff shows changes only to crates/athena-cli/src/slot_exec.rs, CHANGELOG.md and possibly crates/athena-agent/src/env.rs; none to athena.toml, defaults.toml, workflows or crates/athena-core/src/run/checks.rs.

## Expected deliverables

- crates/athena-cli/src/slot_exec.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/env.rs (modify)

## Out of scope

- Changing checks.rs link_slot_target or the checks' behaviour
- Slot size-limit deletion in the wrapper
- Changes to athena.toml, defaults.toml or workflows
- Removing the producer's private build directory in the drain

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; workspace cargo test, clippy and fmt --check required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests observe the link during the command and its absence after.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in one module plus the changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are checkable with cargo test/clippy/fmt and git diff.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the cause (producer builds in private dir) by reusing the checks' linking mechanism.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cefdfcf90e13 → candidate 2b4f714c13c6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   4 +
 crates/athena-cli/src/slot_exec.rs | 191 ++++++++++++++++++++++++++++++++++++-
 2 files changed, 194 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-144832-7wsq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-144832-7wsq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42384396530 for 2b4f714c13c66381dfe1edc837d781bedf15fe25)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42384396530 for 2b4f714c13c66381dfe1edc837d781bedf15fe25)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42384396530 for 2b4f714c13c66381dfe1edc837d781bedf15fe25)
- ci-result: `not a check` passed (Check results for 2b4f714c13c66381dfe1edc837d781bedf15fe25 come from the CI run https://github.com/doodla/athena/actions/runs/42384396530 (739s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test (workspace) passes, cargo clippy --all-targets is clean, cargo fmt --check passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
