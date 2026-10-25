# Job 20261017-152733-efyd: verify "Push the candidate commit and reuse CI's result in the checks step"

## Request (verbatim)

> CI routing is built but inert: docs/design/ci-routing.md section 10 says the candidate commit is never pushed, so routing finds no CI result and every check still runs locally (a one-line config job still took 29 minutes). Wire section 5.2 steps 1-3 and 5: after the producer finishes, push the candidate commit so CI runs on it, wait for CI's run on that exact SHA within a bounded time, and have the checks step reuse that result, with the local fallback already built. The verifier's reuse is a separate request.

## Reading (accepted by the user)

The job run currently passes `NoCi` with a one-poll wait, so every check runs locally. The change pushes the candidate commit to the job's work branch after the producer finishes, then waits a bounded time for the `ci` run on that exact SHA, and passes a real `GhClient` source to `run_routed`. Local fallback stays as built. The audience is the Athena operator, who wants a job's checks to take CI's time rather than 29 minutes locally.

Assumptions:
- The push happens in the checks step of run.rs, after the candidate commit is created (around run.rs:906) and before the routed checks (run.rs:1016). The SHA passed to `run_routed` (`c`) is that commit.
- The real CI source is `GhClient` in gh.rs, which already implements `CiSource`, and the pushing remote is `GitRemote::push`. The repository is detected from the origin remote.
- Pushing needs a remote and credentials. When there is no remote, no gh access, or the push fails, the job logs the reason and falls back to local checks. It is not blocked.
- The wait bound is a conservative constant or config value, because section 9 data does not exist yet. A value near typical CI duration is acceptable.
- Nested runs and `build`-kind jobs keep their present behavior: nested runs skip checks, and build jobs never reuse CI.
- A workflow file changed in the candidate still forces the local fallback, per 5.3.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: it passes, with new tests showing the checks step receives a CI source backed by the platform and not `NoCi`. They also show the candidate commit is pushed before CI is read, the same SHA is used for both, and the wait uses the configured bound rather than one poll.
2. [behavior] Run `cargo test`: tests with fake push and CI sources show a green CI run for the candidate SHA means no local check runs and the check records say they came from CI with the run address. A run for another SHA, a missing run after the bounded wait, and a push failure each run the three checks locally exactly once. A red run fails and does not run locally.
3. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both are clean.
4. [content] The push goes through the existing guarded push path (`check_push`, the work branch only). No new push mechanism is added, and nothing is pushed to the default branch.
5. [content] The wait bound is a named, documented value (a constant or a config field) with a stated basis, and the run output says whether the checks came from CI or ran locally. If `athena.toml` gets a field, it is named and the schema/load tests are updated.
6. [content] docs/design/ci-routing.md section 10 items 4 and 5 are updated: the push and wait are no longer listed as 'not done'. The verifier's reuse is still listed as not done, and no speedup is claimed without a measurement.
7. [content] The `fmt`, `clippy` and `test` checks are not changed in definition. The verifier's input and prompts are not changed.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- docs/design/ci-routing.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- The verifier's reuse of the CI result, and the reuse statement in the verifier's input (step 4 of 5.2), which the request defers.
- Measurement of speedup and the section 9 baseline.
- Cache and `timeout-minutes` changes in ci.yml.
- Performing the workflow's own run for `build` jobs.
- Changes to the landing path (`athena land --pr`) beyond reusing its push.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover push, wait, reuse and fallback, and cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that CI results are reused or the fallback runs, using fake sources, not that wiring exists.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: It reuses the existing GhClient, GitRemote::push and resolve, and builds nothing new.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: It is one focused change: steps 1-3 and 5. The verifier's reuse is excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is the NoCi stub and the missing push, and both are replaced.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design doc is updated to match the code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the test, clippy and fmt commands.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The hidden design details come from ci-routing.md sections 5.2, 5.3 and 10, and from run.rs and routing.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 3970d5d18ab8 → candidate 630ceff2be9f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   7 ++
 crates/athena-core/src/gh.rs      |  49 +++++++++
 crates/athena-core/src/routing.rs | 211 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run.rs     |  39 +++++--
 docs/design/ci-routing.md         |  20 ++--
 5 files changed, 308 insertions(+), 18 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (877 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
