# Job 20261014-035456-267v: verify "Route external calls through one bounded-retry wrapper, with a guard tes"

## Request (verbatim)

> Audit every place Athena calls GitHub (gh CLI or the API) or another external service, and route all of them through a single wrapper with bounded retry. The wrapper retries transient failures (HTTP 5xx, timeouts, connection resets) up to 3 times with exponential backoff and jitter, fails immediately on 4xx and auth errors, and logs each retry with the command name and status. Build on the 5xx retry added in job 86p4 rather than duplicating it. Add a check, as a lint rule or test, that fails when code calls gh or an HTTP client outside the wrapper. Add tests that inject a transient 500 into 'athena land --pr' and the other GitHub-calling commands. Each test should assert that a one-off 500 is retried and succeeds, and that persistent 500s fail after the limit with a clear error.

## Reading (accepted by the user)

Extend the existing 5xx retry in crates/athena-core/src/gh.rs (job 86p4) into the single wrapper: add jitter, timeout and connection-reset classification, and a per-retry log line with command and status. Add a source-scanning test that fails when `Command::new("gh")` or an HTTP client appears outside the wrapper, and tests that inject a one-off and a persistent 500 into each GitHub-calling command, including `land --pr`.

Assumptions:
- The audit found one GitHub call site: GhCli in crates/athena-core/src/gh.rs, the only Command::new("gh"). I grepped for reqwest, ureq and curl in crates/ and found no HTTP client. The only curl is in xtask (tool download, which already passes --retry 3).
- Other external processes (git, the claude agent binary) are not 'external services' for this task. The claude binary's busy-retry in athena-agent stays as it is.
- The guard is a test (a source scan) rather than a clippy lint, because disallowed-methods cannot ban Command::new("gh") by argument. No clippy config changes.
- Jitter is deterministic-injectable (a sleeper or jitter seam) so tests do not really sleep. No new crate is added if a small hash- or time-based jitter suffices; otherwise add the rand crate, per buy-first.
- Retry logging goes to stderr via the existing output path. Each line names the command (the REST path or operation) and the status.
- Non-HTTP transient failures (timeouts, connection resets) are detected from gh's stderr text, since gh gives no status for them.

## Acceptance criteria

1. [content] Retry lives in one place in crates/athena-core/src/gh.rs, built on the existing Retry and is_transient code with no second retry loop. It makes at most 3 attempts with exponential backoff plus jitter.
2. [content] Transient means HTTP 5xx, timeouts and connection resets. 4xx and auth errors (401/403, 'not logged in') fail on the first attempt with no retry.
3. [content] Each retry writes a log line containing the command name, the status or failure kind, and the attempt number.
4. [behavior] Run `cargo test`. A source-scan test passes on the tree and fails when a fixture file outside the wrapper contains Command::new("gh") or an HTTP client call.
5. [behavior] Run `cargo test`. For `land --pr` and each other GitHub-calling command (default branch, PR create/read, merged-commit and similar), a test that injects one 500 sees a retry followed by success. A test that injects persistent 500s sees a failure after 3 attempts with an error naming the command and status.
6. [behavior] Run `cargo test`. Unit tests show a 404 and an auth failure are not retried, and that timeouts and connection resets are retried.
7. [behavior] `cargo clippy` reports no warnings and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/external_calls.rs (create)
- crates/athena-core/tests/external_calls_guard.rs (create)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- README.md (modify)

## Out of scope

- Retry for git, the claude agent binary or xtask downloads.
- Changing the landing logic or the HostError record schema.
- Adding new GitHub-calling commands.
- Changes to CI workflows, clippy config or other governance files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests; cargo test and clippy must pass.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Extend the existing retry; use a maintained crate for jitter only if needed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One wrapper plus a guard test, not per-call patches.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The guard is an enforced test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, centred on gh.rs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running cargo test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base dfa92f8718fa → candidate ed962064aba6. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                     |  13 +
 README.md                                        |   2 +-
 crates/athena-cli/tests/external_calls.rs        | 293 +++++++++++++++++++++++
 crates/athena-core/src/gh.rs                     | 195 ++++++++++++++-
 crates/athena-core/tests/external_calls_guard.rs |  97 ++++++++
 5 files changed, 596 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (459 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
