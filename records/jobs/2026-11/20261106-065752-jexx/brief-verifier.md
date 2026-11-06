# Job 20261106-065752-jexx: verify "Write test stand-in scripts from a child process to avoid ETXTBSY"

## Request (verbatim)

> Make the shared stand-in script helper immune to 'Text file busy' without retries at the call sites.
> 
> Tests write small executable stand-in scripts and then run them, sometimes through production code. On Linux, if another test thread forks while this process holds the script open for writing, running the script fails with ETXTBSY. A retry at each run site is not enough, because many runs happen inside production code.
> 
> Write the script's bytes from a child process instead: the helper starts a short-lived child (for example sh -c 'cat > tmp && chmod 755 tmp && mv tmp final', with the bytes on stdin) and waits for it. The test process never holds a write handle to the script, so no fork in another thread can inherit one.
> 
> Acceptance:
> - every test that writes an executable stand-in uses this one helper (xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-git/src/snapshot.rs)
> - a stress test runs many write-then-exec cycles while other threads fork; it passes with the helper, and a variant that writes in-process is shown in the recorded check output to hit ETXTBSY at least once (or the test documents how often it does on this machine)
> - the decision that covers crates/athena-git/src/snapshot.rs gets an appended entry saying it was checked and stands, with its link
> - xtask/shard-timings.txt lists any new test file
> - CHANGELOG states only what the diff does
> 
> Files changed: the shared test helper, the tests above, docs/decisions.md and docs/decision-links.toml for the entry, xtask/shard-timings.txt, CHANGELOG.md.

## Reading (accepted by the user)

Add one shared test helper that writes an executable stand-in script via a short-lived `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child fed on stdin, so the test process never holds a write handle. Switch all six listed test sites to it, add a fork-stress test, and record the decision, shard-timing and CHANGELOG entries. Audience: developers and CI running the Linux test suite.

Assumptions:
- Code survey: read crates/athena-git/src/snapshot.rs (tests around lines 495-552, set_permissions at 438-478), crates/athena-git/src/lib.rs (testutil module at 1093), crates/athena-agent/src/claude.rs (retry helper at ~45, stand-in at ~618-645), crates/athena-core/src/checks.rs (~998-1009), crates/athena-core/src/reading.rs (~8141-8156, already matches 'Text file busy'), crates/athena-core/src/test_support.rs, xtask/src/main.rs (stand_in_claude ~833), crates/athena-cli/tests/reconcile.rs (~140). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Entry point is the helper called by each test; the property is measured directly by exercising real write-then-exec cycles, not through a stand-in.
- No existing helper is shared across xtask, athena-cli tests, athena-agent, athena-core and athena-git, so the shared helper needs a home every one of them can dev-depend on; I assume a small new dev-only workspace crate (or an existing one if the dependency graph allows), which means Cargo.toml and Cargo.lock change.
- The stress test lives in the helper's home (a new test file or module), forks from other threads (e.g. spawning `true`) while the main thread loops write-then-exec; the in-process variant (std::fs::write then exec) is reported with its measured ETXTBSY count on this machine, printed in test output.
- Existing ETXTBSY retry code in claude.rs and reading.rs may be removed or left; I leave production code untouched and only change test code, apart from removing now-redundant test-only retries if they sit in test modules.
- The decision covering crates/athena-git/src/snapshot.rs is D08 (docs/decision-links.toml line ~180); the appended entry says it was checked and stands, with its link.
- Non-Linux/non-unix cfg gating is kept as existing tests are unix-only.

## Acceptance criteria

1. [content] Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs shows no test code writing an executable stand-in itself; each such test calls the one shared helper.
2. [content] The helper's source starts a child process (sh) with the script bytes on stdin that runs cat to a temp file, chmod 755, then mv to the final path, and waits for it; it never opens the script for writing in the test process.
3. [behavior] Run `cargo test` for the stress test: many write-then-exec cycles with other threads forking pass with the helper with zero ETXTBSY, and an in-process-write variant prints/records its ETXTBSY count on this machine in the test output (at least once, or the test documents how often it hit).
4. [behavior] `cargo test --locked` passes across the workspace, `cargo clippy` is clean and `cargo fmt --check` passes.
5. [content] docs/decisions.md has an appended entry for the decision covering crates/athena-git/src/snapshot.rs saying it was checked and stands, and docs/decision-links.toml carries its link; the repo's decision-link check passes under cargo test.
6. [content] xtask/shard-timings.txt lists any new test file or unit added by the change (and Cargo.toml/Cargo.lock list any new crate).
7. [content] CHANGELOG.md has an entry that describes only what the diff does, no more.
8. [content] No secrets, credentials or personal data are committed; the helper and test are documented for an outside reader.

## Expected deliverables

- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retries at run sites or in production code
- Changing production behavior of claude.rs or other non-test code
- Non-Linux platform support
- athena.toml, defaults.toml and workflow changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: stress test added; cargo test and clippy must pass
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier runs cargo test and checks the recorded ETXTBSY count
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check real write-then-exec under forking, not file existence
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fixes the cause (held write handle) instead of retrying
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one helper, mechanical call-site swaps
- strangers: Code, commits, docs, and records are written for an outside reader. Here: helper documented for outside readers
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: helper and tests in Rust
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: no secrets or personal data
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: uses std::process and sh rather than a new crate dependency for the write itself
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: the outcome is no ETXTBSY under fork stress
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3ea65de79101 → candidate 1d02574855c2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |   9 ++
 Cargo.lock                                     |  12 +++
 Cargo.toml                                     |   2 +
 crates/athena-agent/Cargo.toml                 |   1 +
 crates/athena-agent/src/claude.rs              |   4 +-
 crates/athena-cli/Cargo.toml                   |   1 +
 crates/athena-cli/tests/reconcile.rs           |   4 +-
 crates/athena-core/Cargo.toml                  |   3 +
 crates/athena-core/src/checks.rs               |   4 +-
 crates/athena-core/src/reading.rs              |  21 +---
 crates/athena-git/Cargo.toml                   |   3 +
 crates/athena-git/src/snapshot.rs              |   7 +-
 crates/athena-testkit/Cargo.toml               |  14 +++
 crates/athena-testkit/src/lib.rs               | 109 ++++++++++++++++++++
 crates/athena-testkit/tests/stand_in_stress.rs | 136 +++++++++++++++++++++++++
 docs/architecture.md                           |   1 +
 docs/decision-links.toml                       |   7 ++
 docs/decisions.md                              |  10 ++
 xtask/Cargo.toml                               |   1 +
 xtask/shard-timings.txt                        |   3 +
 xtask/src/main.rs                              |   4 +-
 21 files changed, 323 insertions(+), 33 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-065752-jexx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-065752-jexx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42196696407 for 1d02574855c22c8ae9ffa6dd5f549b17828de324)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42196696407 for 1d02574855c22c8ae9ffa6dd5f549b17828de324)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42196696407 for 1d02574855c22c8ae9ffa6dd5f549b17828de324)
- ci-result: `not a check` passed (Check results for 1d02574855c22c8ae9ffa6dd5f549b17828de324 come from the CI run https://github.com/doodla/athena/actions/runs/42196696407 (682s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --locked` passes across the workspace, `cargo clippy` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
