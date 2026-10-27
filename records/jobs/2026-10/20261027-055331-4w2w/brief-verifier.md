# Job 20261027-055331-4w2w: verify "Rebuild and re-exec Athena after drain lands changes to its own code"

## Request (verbatim)

> Priority: urgent
> Let athena drain run unattended across its own landings. Today, when drain lands a change to Athena's own code or config schema, the running binary stays at the old build; the next landing or config read can fail (2026-10-26: 'unknown field hard_producer_effort' stopped the landing of 20261025-034237-2rep until ./setup.sh was rerun by hand), and nothing warns (the stale-build request of 2026-10-17 never landed). Fix: after drain lands a change touching crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh, it rebuilds and installs Athena from the landed tree (the same steps as ./setup.sh) and continues the run with the new binary (re-exec with the remaining budget and job limit); if the rebuild fails it stops with the stop reason 'rebuild failed' and the build output. athena status and every agent-running command (go, drain, audit, audience) print one line when the installed binary's build commit is older than HEAD for those paths.
> Acceptance:
> - A test drains two fake items where the first landing changes a crates/ file, and shows the rebuild step runs once and the second item runs under the new build commit.
> - A test shows a failed rebuild stops the drain with 'rebuild failed'.
> - A test shows status prints the stale-build line when a crates/ file changed after the build commit, and none when only docs/ or records/ changed.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

After drain lands a change touching crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh, it runs the setup.sh build/install steps, then re-execs with the remaining budget and job limit; a failed rebuild stops with 'rebuild failed' plus build output. status and agent-running commands (go, drain, audit, audience) print one stale-build line when the installed build commit is older than HEAD for those paths. Audience: the operator running drain unattended.

Assumptions:
- The build commit is already embedded in the binary (or is added at build time by the same steps as setup.sh); the producer finds how setup.sh records it.
- Rebuild and install are behind an injectable step so tests use a fake instead of running cargo.
- Re-exec carries remaining budget and job limit as arguments to the same drain command.
- The stale check compares the build commit with HEAD using git diff limited to crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh.

## Acceptance criteria

1. [behavior] Run cargo test: a test drains two fake items where the first landing changes a crates/ file; it shows the rebuild step ran exactly once and the second item ran under the new build commit.
2. [behavior] Run cargo test: a test shows a failed rebuild stops the drain with stop reason 'rebuild failed' and includes the build output.
3. [behavior] Run cargo test: a test shows status prints the stale-build line when a crates/ file changed after the build commit, and prints none when only docs/ or records/ changed.
4. [content] go, drain, audit and audience print the same one-line stale-build warning when the build commit is older than HEAD for the watched paths.
5. [content] The re-exec passes the remaining budget and job limit.
6. [behavior] cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
7. [content] CHANGELOG.md has an entry describing the self-rebuild and stale-build warning.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Changing setup.sh itself
- Rebuilding after landings that touch only docs/ or records/
- Changing config schema or the planner model settings

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for rebuild, failure and status line; fmt, clippy -D warnings and cargo test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check that the rebuild ran once and the second item ran under the new build commit, not only that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the drain continuing unattended across a landing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the named files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 4f7f478a0ab2 → candidate 41612b174926. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  13 ++
 crates/athena-cli/src/display.rs |  10 ++
 crates/athena-cli/src/drain.rs   | 204 +++++++++++++++++++++++-
 crates/athena-cli/src/main.rs    |  36 +++++
 crates/athena-cli/src/wiring.rs  |   6 +
 crates/athena-cli/tests/drain.rs | 333 ++++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs |  55 +++++++
 docs/decisions.md                |  56 +++++++
 8 files changed, 708 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1422 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
