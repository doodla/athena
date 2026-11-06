# Job 20261106-095820-edjm: verify "Finish ETXTBSY fix: all test stand-ins via shared child-process helper"

## Request (verbatim)

> try again on 20261106-065752-jexx

## Reading (accepted by the user)

Retry of job 20261106-065752-jexx for developers and CI running the Linux suite: a new test-only crate athena-testkit holds one helper writing executable stand-ins from an `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child. Every test site, including the missed xtask gitleaks fake, uses it, and a scan test fails on any in-process executable write in test code.

Assumptions:
- Job 20261106-065752-jexx findings addressed: (1) major, the xtask test a_cached_gitleaks_is_skipped_only_when_it_reports_the_pinned_version still wrote in-process via std::fs::write + set_executable, so it is converted to the helper; (2) minor, the CHANGELOG overclaim is fixed by writing the entry from the final diff; (3) minor, docs/architecture.md was changed outside the deliverables, and it is now a listed deliverable (a crate-table row for athena-testkit is kept); (4) nit, the in-process ETXTBSY count was visible only with --nocapture, so the stress test now prints it in a way that appears in recorded output (for example, it asserts nothing on the count but the test run uses println via a documented command, or the count is written into the test's passing output through a non-captured channel). The earlier candidate is the starting point: its 21 changed files are all kept as deliverables.
- Code survey: grep over crates/ and xtask shows the test-side executable writes at xtask/src/main.rs:809 (set_executable helper) and :837, crates/athena-core/src/reading.rs:8144, crates/athena-core/src/checks.rs:1009, crates/athena-agent/src/claude.rs:641 and crates/athena-cli/tests/reconcile.rs:140. The other set_permissions hits in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-git/src/snapshot.rs lines 445-516 are production code and stay. No records or data file under crates/ is added, so the records allow-list (check_record_file, AnyRecord) is not touched.
- The scan test lives in the new crate athena-testkit (tests/), scans test code in the workspace, ignores production code and the helper itself, and has a unit check that a planted violation string is detected.
- The helper crate is named athena-testkit as in the earlier candidate; the follow-up text's 'athena-testutil' is read as the same shared helper.
- The in-process variant's ETXTBSY count depends on scheduling; the test reports it and does not assert a minimum.

## Acceptance criteria

1. [content] Grep for `set_permissions` and `from_mode(0o755)` in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs finds no test code that writes an executable; only production code matches. The xtask gitleaks-version test calls athena_testkit::write_executable.
2. [content] The helper source in crates/athena-testkit/src/lib.rs spawns sh with the script bytes on stdin running cat to a temp file, chmod 755, then mv to the final path, waits for it, and never opens the script for writing in the test process.
3. [behavior] Run `cargo test --locked -p athena-testkit`: the scan test passes on the repo and a unit test shows it flags a planted in-process executable write; the stress test passes with 0 ETXTBSY for the helper, and the in-process variant's ETXTBSY count appears in the recorded output.
4. [behavior] Run `cargo test --locked` across the workspace; it passes, including the converted tests in xtask, athena-cli, athena-agent, athena-core and athena-git, and the decision-link check. `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] docs/decisions.md has an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands, and docs/decision-links.toml carries its link.
6. [content] xtask/shard-timings.txt lists every new test file or unit, and Cargo.toml and Cargo.lock list the new crate and its dependents.
7. [content] CHANGELOG.md has one entry that describes only what the final diff does. docs/architecture.md gets only the crate-table row for athena-testkit.
8. [content] The diff has no retries at run sites, no production-behavior change in claude.rs or other non-test code, no secrets or personal data, and the helper and tests are documented for an outside reader.

## Expected deliverables

- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (create)
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
- docs/architecture.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retries at run sites or in production code
- Changing production behavior of claude.rs or other non-test code
- Non-Linux platform support
- athena.toml, defaults.toml and workflow changes
- Production-code set_permissions uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-git/src/snapshot.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper, stress test and scan test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New crate has passing tests; cargo test and clippy are required clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands the verifier runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: A scan test covers every site instead of patching only the one that surfaced.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check ETXTBSY counts and the scan failing on a planted violation.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Helper and tests are documented for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule that stand-ins go through the helper is enforced by the scan test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1fb611f1071d → candidate 21fa3ff4ee8b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  10 +
 Cargo.lock                                         |  14 +
 Cargo.toml                                         |   2 +
 crates/athena-agent/Cargo.toml                     |   1 +
 crates/athena-agent/src/claude.rs                  |   4 +-
 crates/athena-cli/Cargo.toml                       |   1 +
 crates/athena-cli/tests/reconcile.rs               |   4 +-
 crates/athena-core/Cargo.toml                      |   3 +
 crates/athena-core/src/checks.rs                   |   4 +-
 crates/athena-core/src/reading.rs                  |  20 +-
 crates/athena-git/Cargo.toml                       |   3 +
 crates/athena-git/src/snapshot.rs                  |   9 +-
 crates/athena-testkit/Cargo.toml                   |  16 +
 crates/athena-testkit/src/lib.rs                   | 146 +++++++
 .../tests/no_in_process_exec_writes.rs             | 486 +++++++++++++++++++++
 crates/athena-testkit/tests/stand_in_stress.rs     |  99 +++++
 docs/architecture.md                               |   1 +
 docs/decision-links.toml                           |   7 +
 docs/decisions.md                                  |  15 +
 xtask/Cargo.toml                                   |   1 +
 xtask/shard-timings.txt                            |   4 +
 xtask/src/main.rs                                  |   7 +-
 22 files changed, 819 insertions(+), 38 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-095820-edjm/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-095820-edjm/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42202600571 for 21fa3ff4ee8ba751efd2286b0beb9806aa435e3d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42202600571 for 21fa3ff4ee8ba751efd2286b0beb9806aa435e3d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42202600571 for 21fa3ff4ee8ba751efd2286b0beb9806aa435e3d)
- ci-result: `not a check` passed (Check results for 21fa3ff4ee8ba751efd2286b0beb9806aa435e3d come from the CI run https://github.com/doodla/athena/actions/runs/42202600571 (682s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked` across the workspace; it passes, including the converted tests in xtask, athena-cli, athena-agent, athena-core and athena-git, and the decision-link check. `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
