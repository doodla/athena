# Job 20261030-024706-j611: verify "Give OLY-B-A4-04 a mechanism: setup.sh one-command setup, with test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A4-04 a mechanism
> 
> Requirement OLY-B-A4-04 in docs/spec.toml is active and has no mechanism.
> Requirement: A clean machine reaches a working Athena with one command from committed, version-pinned, idempotent setup.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A4-04
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A4-04 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

OLY-B-A4-04 (one command from a clean machine to a working Athena) is already met in practice by `setup.sh` handing off to `xtask setup`, but its spec entry says `none`. Record that as the mechanism and add a new xtask test for the one-command property, and write a measure ending `status: not shown`.

Assumptions:
- Goal and audience: a maintainer or Athena's spec gap list/Hestia spec review judges whether OLY-B-A4-04 has a real, tested mechanism; the goal is a recorded mechanism backed by a test, not new setup machinery.
- Code survey: read setup.sh (pinned, sha256-checked rustup-init guarded by a cargo-missing check, ends in one `exec cargo run --quiet --locked --release -p xtask -- setup "$@"`), xtask/tests/setup_repeatable.rs (the OLY-B-RUN-02 test pattern: it parses docs/spec.toml, checks the mechanism names existing files, checks owner and measure, and checks the setup.sh properties), xtask/tests/spec.rs (every active requirement needs a non-empty mechanism, owner and measure; `none` is a gap that Hestia queues), and crates/athena-core/src/status.rs (BUILD_PATHS includes setup.sh and a stale-build message says to run ./setup.sh). The entry point is ./setup.sh. Rust resolution and probing run in this order: use cargo on PATH, else $CARGO_HOME/bin, else install the pinned rustup-init, then `xtask setup`. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly by reading the committed files, not through a stand-in.
- Existing code: setup.sh, xtask/src/main.rs (setup, gitleaks_present), xtask/tests/setup_repeatable.rs, xtask/tests/spec.rs, rust-toolchain.toml, athena.toml [tools], README.md and docs/spec.toml. The new test follows the pattern in setup_repeatable.rs and does not rebuild those files.
- The new test goes in xtask/tests/ (xtask has no library). It will probably be a new file xtask/tests/setup_one_command.rs, in the same style as setup_repeatable.rs.
- The test fails without the mechanism because it asserts the spec entry's mechanism is not none, names setup.sh and xtask/src/main.rs, and every file it names exists; with `none` it fails. It also checks README documents ./setup.sh as the single command.
- Measure format follows OLY-B-RUN-02: it names the cargo test command, and no athena status line prints this property, so it ends with `status: not shown`.
- The mechanism string avoids naming files that do not exist. It names setup.sh, xtask/src/main.rs, rust-toolchain.toml and xtask/tests/setup_one_command.rs.
- Environment: none. The test reads committed files only and does not use CI, the network or records. It does not run a real fresh-machine install.
- Consumers: none found for config changes, because no config file is edited. athena.toml, defaults.toml, rust-toolchain.toml and workflows are not deliverables. CHANGELOG.md is a default deliverable and gets an entry.
- Term ladder: not used. This is not a refusal rule, so no refusal-rate criterion is needed.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A4-04 has a mechanism that is not `none`, names setup.sh, xtask/src/main.rs and the new test file (all existing files), keeps owner = "Athena", and has a measure ending in `status: not shown` that names the test command.
2. [content] A new test file under xtask/tests/ (e.g. xtask/tests/setup_one_command.rs) parses docs/spec.toml and asserts OLY-B-A4-04 has a non-blank, non-`none` mechanism whose named files exist; asserts the owner; and asserts setup.sh has one locked `cargo run -p xtask -- setup` exec and a pinned, checksummed rustup-init.
3. [behavior] Run `cargo test -p xtask --test setup_one_command`: it passes with the change. A test would fail if the mechanism were reverted to `none`, and the test asserts that case.
4. [behavior] Run `cargo test -p xtask --test spec`: the existing check that every active requirement has a mechanism, owner and measure still passes.
5. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, with Athena's recorded checks on the final tree counting as evidence.
6. [content] CHANGELOG.md records this change in a new entry. No existing entry of docs/decisions.md or anything under records/ is edited.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/setup_one_command.rs (create)
- CHANGELOG.md (modify)
- xtask/tests/setup_repeatable.rs (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)

## Out of scope

- Changing the setup.sh install logic, pins or the rust-toolchain.toml channel.
- Running a real fresh-machine install or adding a CI workflow for it.
- Adding an athena status line for setup.
- Editing the other requirements in docs/spec.toml, docs/decisions.md or records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask integration test; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test parses the real docs/spec.toml and setup.sh and checks the behavior-bearing properties, not just that files exist.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement's mechanism is the setup.sh to xtask setup path, enforced by the test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a spec entry edit plus one test file.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bfe50bb78e61 → candidate e156d638568d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   1 +
 docs/spec.toml                   |   4 +-
 xtask/shard-timings.txt          |   1 +
 xtask/tests/setup_one_command.rs | 174 +++++++++++++++++++++++++++++++++++++++
 4 files changed, 178 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-024706-j611/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-024706-j611/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43317398178 for e156d638568d58d3a63e32de1f4137e4d8851605)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43317398178 for e156d638568d58d3a63e32de1f4137e4d8851605)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43317398178 for e156d638568d58d3a63e32de1f4137e4d8851605)
- ci-result: `not a check` passed (Check results for e156d638568d58d3a63e32de1f4137e4d8851605 come from the CI run https://github.com/doodla/athena/actions/runs/43317398178 (647s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, with Athena's recorded checks on the final tree counting as evidence.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
