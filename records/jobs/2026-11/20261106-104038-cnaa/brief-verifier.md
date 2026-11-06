# Job 20261106-104038-cnaa: verify "Add shared child-process script writer; use it in all stand-in tests"

## Request (verbatim)

> Make tests that write and run stand-in scripts immune to 'Text file busy', with one shared helper and no source scan.
> 
> Tests write small executable stand-in scripts and run them, sometimes through production code. On Linux, if another test thread forks while this process holds the script open for writing, running it fails with ETXTBSY. This has failed correct jobs.
> 
> Add one shared test helper that writes the script's bytes from a short-lived child process (for example sh -c 'cat > tmp && chmod 755 tmp && mv tmp final', bytes on stdin) and waits for it, so the test process never holds a write handle. Make every test that writes an executable stand-in use it.
> 
> Do not add a test that scans source text for patterns; reviewers found such scans miss cases. List each converted call site in the job report instead.
> 
> Acceptance:
> - the helper exists, and every test in the workspace that writes a file and then makes it executable (by set_permissions, from_mode, set_mode, a wrapper such as xtask's set_executable, or an external chmod) uses it; the report lists each one by file and test name
> - a stress test runs many write-then-exec cycles through the helper while other threads fork, and passes
> - if a converted file is governed by a decision in docs/decisions.md, an appended entry says it was checked and stands, with its link
> - any new test file is listed in xtask/shard-timings.txt
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the shared helper, the tests that write stand-ins, docs/decisions.md and docs/decision-links.toml if needed, xtask/shard-timings.txt if needed, CHANGELOG.md.

## Reading (accepted by the user)

Add one shared test helper that writes an executable script via a short-lived `sh -c 'cat > tmp && chmod 755 tmp && mv tmp final'` child (bytes on stdin, waited on), so no test process holds a write handle. Convert every test that writes a file then makes it executable, add a fork-stress test, and record the D08 check for the governed athena-git snapshot.rs. The audience is maintainers whose correct jobs fail on ETXTBSY.

Assumptions:
- Code survey: I grepped all *.rs under crates/ and xtask/. Test sites that write a file and then chmod: crates/athena-core/src/reading.rs (~8185, in the stub checker test helper next to check_stub's 'Text file busy' retry), crates/athena-core/src/checks.rs (~1009), crates/athena-agent/src/claude.rs (~641, near retry_busy), crates/athena-cli/tests/reconcile.rs (~140, fake gh), crates/athena-git/src/snapshot.rs (~514, test user_repo, run.sh fixture), xtask/src/main.rs (tests at ~837 and ~1023 via set_executable). The producer must re-grep to find any missed site, including ones that use set_executable or an external chmod.
- Code survey: Non-test uses stay unchanged: crates/athena-core/src/toolconfig.rs, pin.rs and crates/athena-core/src/snapshot.rs set modes for production, and xtask set_executable at ~415, ~490 and ~550 is production code. crates/athena-agent/src/policy.rs and crates/athena-cap-change/src/lib.rs only mention chmod as a policy string. Production set_executable in xtask can stay; only the xtask test sites change.
- Code survey: Existing test-sharing mechanisms: athena-core has a `test-support` feature (crates/athena-core/src/test_support.rs, lib.rs:120) that athena-cli enables, and athena-git has `pub(crate) mod testutil` (lib.rs:1093). The helper must be reachable from athena-core, athena-agent, athena-git, athena-cli tests and xtask. The producer chooses the home, for example a `test-support` feature in athena-git (which xtask already depends on) or a small dev-only crate. It must be a plain Rust fn that takes a path and bytes. No records or data file is added under crates/, so check_record_file and AnyRecord are not involved.
- Code survey: The change adds no refusal rule, so no refusal-rate criterion applies. The property is measured directly: the stress test runs the helper's write-then-exec cycles while other threads fork, and the existing sleep-retry code in reading.rs check_stub and claude.rs retry_busy is not the measure. Retry code in production, such as retry_busy in claude.rs, is kept unless it is test-only.
- docs/decision-links.toml has a [[governs]] D08 entry covering crates/athena-git/src/snapshot.rs. Converting the test there triggers the D156 reconciliation, so a new entry is appended to docs/decisions.md saying D08 was checked and stands, plus an `amended` [[link]] in docs/decision-links.toml whose line matches the new phrase. Other converted files are not in a [[governs]] table, which the producer should recheck.
- A new stress test is placed inside the helper's own crate or an existing test module, so no new test file is made and xtask/shard-timings.txt stays unchanged. If a new test file is made instead, it is added there.
- Cargo.toml dev-dependency or feature edits for sharing the helper are permitted, but no lint, toolchain or other governance config changes.

## Acceptance criteria

1. [content] A single shared helper exists that writes script bytes by spawning a child process (sh with bytes on stdin: cat to a tmp file, chmod 755, mv into place) and waits for it; the test process never opens the script for writing. Confirm by reading it.
2. [content] Every test in the workspace that writes a file and then makes it executable (set_permissions, from_mode, set_mode, xtask set_executable, or an external chmod) calls the helper. This covers at least reading.rs, checks.rs, claude.rs, reconcile.rs, athena-git snapshot.rs and the xtask tests. Grep of test code finds no remaining write-then-chmod pairs. Production set_executable uses are untouched.
3. [content] A stress test runs many write-then-exec cycles through the helper while other threads fork, and asserts every exec succeeds.
4. [behavior] Run `cargo test` for the workspace. All tests pass, including the stress test and every converted test. Run `cargo clippy` (clean) and `cargo fmt --check`.
5. [content] No new test scans source text for patterns. The change adds no source-scanning test.
6. [content] docs/decisions.md has an appended entry saying D08 was checked for the crates/athena-git/src/snapshot.rs conversion and stands. docs/decision-links.toml has a matching `amended` [[link]] with older D08 and the new entry as newer. Its `line` matches the entry's phrase line in docs/decisions.md. Any other governed converted file is handled the same way.
7. [content] Any new test file is listed in xtask/shard-timings.txt; otherwise that file is unchanged.
8. [content] CHANGELOG.md has an entry that states only what the diff does: the helper, the converted tests and the stress test.
9. [content] The job report lists each converted call site by file and test name. Because the reader checks only the diff, the verifier compares that list with a Grep of the diff.

## Expected deliverables

- crates/athena-git/src/lib.rs (modify)
- crates/athena-git/src/script_writer.rs (create)
- crates/athena-git/Cargo.toml (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-git/src/snapshot.rs (modify)
- xtask/src/main.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- A test that scans source text for patterns
- Changing production code paths (toolconfig.rs, pin.rs, core snapshot.rs, xtask production set_executable)
- Removing the existing ETXTBSY retry code unless it is test-only and made redundant
- Changes to athena.toml, defaults.toml, workflows or lint configuration

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The helper and stress test are Rust; the helper only shells out to sh as the request requires.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The stress test and cargo test, clippy and fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A behavior criterion runs cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The stress test exercises write-then-exec under concurrent forks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The helper carries a doc comment explaining ETXTBSY and why the child writes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper and mechanical conversions only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that exec succeeds under concurrent forks, not that the helper exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 64bec1c17489 → candidate ead6ae070c80. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  10 +++
 Cargo.lock                             |   1 +
 crates/athena-agent/Cargo.toml         |   1 +
 crates/athena-agent/src/claude.rs      |   4 +-
 crates/athena-cli/tests/reconcile.rs   |   7 +--
 crates/athena-core/src/checks.rs       |   7 +--
 crates/athena-core/src/reading.rs      |   4 +-
 crates/athena-git/src/lib.rs           |   1 +
 crates/athena-git/src/script_writer.rs | 108 +++++++++++++++++++++++++++++++++
 crates/athena-git/src/snapshot.rs      |   8 +--
 docs/decision-links.toml               |   7 +++
 docs/decisions.md                      |  14 +++++
 xtask/src/main.rs                      |   7 +--
 13 files changed, 153 insertions(+), 26 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-104038-cnaa/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-104038-cnaa/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42213223731 for ead6ae070c806842093d3d742819620e89aa80fd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42213223731 for ead6ae070c806842093d3d742819620e89aa80fd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42213223731 for ead6ae070c806842093d3d742819620e89aa80fd)
- ci-result: `not a check` passed (Check results for ead6ae070c806842093d3d742819620e89aa80fd come from the CI run https://github.com/doodla/athena/actions/runs/42213223731 (532s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` for the workspace. All tests pass, including the stress test and every converted test. Run `cargo clippy` (clean) and `cargo fmt --check`.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
