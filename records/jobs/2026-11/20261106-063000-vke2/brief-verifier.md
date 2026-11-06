# Job 20261106-063000-vke2: verify "Add shared stand-in script helper that avoids ETXTBSY; reconcile D08"

## Request (verbatim)

> try again on 20261106-042920-9e47

## Reading (accepted by the user)

Retry of job 20261106-042920-9e47: add one shared test-support crate (athena-testutil) that writes executable stand-ins via temp name, close, chmod, rename and retries runs on ETXTBSY, move all listed tests onto it, and this time reconcile D08 (snapshot.rs is governed) with a new docs/decisions.md entry plus a decision-links.toml link. Audience: maintainers whose jobs are rejected by flaky tests.

Assumptions:
- Code survey: read docs/decision-links.toml header and docs/decisions.md D156 (the [[governs]] rule, which maps D08 to crates/athena-git/src/snapshot.rs and crates/athena-core/src/run/candidate.rs). Test code lives in xtask/src/main.rs, crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs and crates/athena-git/src/snapshot.rs. The new crate adds no record or data file, so check_record_file and AnyRecord are not touched. The property is measured directly by a stress test in crates/athena-testutil that writes and runs real scripts while other threads fork.
- Prior job 20261106-042920-9e47 failed only on its recorded finding: 'decisions not reconciled: D08 (crates/athena-git/src/snapshot.rs)'. It is addressed by adding a D157 entry in docs/decisions.md saying D08 was checked and stands (the change touches only the snapshot.rs test module, not snapshot behavior), and a [[link]] in docs/decision-links.toml with older D08, newer D157, status amended, whose line matches the phrase in the new entry (line numbers of existing links unaffected because the entry is appended at the end of the log).
- The earlier candidate's 17 changed files are kept as the starting point: all 16 earlier deliverables plus docs/architecture.md (one row for the new crate), which is now a deliverable. The only new work is the decision entry and link.
- Entry id is the next free id after D156 (assumed D157); the producer confirms the last id in docs/decisions.md.
- Scope stays on test code: non-test code in snapshot.rs, pin.rs and toolconfig.rs does not change.

## Acceptance criteria

1. [content] Grep of crates/athena-testutil/src/lib.rs shows one helper that writes an executable script via a temp file, close, chmod, then rename, and a run helper that retries on ETXTBSY; doc comments explain the ETXTBSY cause for an outside reader.
2. [content] Grep finds no write-then-from_mode(0o755) stand-in pattern left in xtask/src/main.rs (stand_in_claude), crates/athena-cli/tests/reconcile.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs or the tests of crates/athena-git/src/snapshot.rs; each calls the helper crate, which is a workspace member in Cargo.toml and a dev-dependency in the athena-agent, athena-core, athena-git, athena-cli and xtask manifests.
3. [behavior] Run `cargo test -p athena-testutil`: a stress test writes and runs at least 200 scripts while other threads repeatedly fork/spawn processes, and none of the runs fails with ETXTBSY.
4. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`: all pass or are clean.
5. [content] docs/decisions.md ends with a new entry that says D08 was checked and stands, and docs/decision-links.toml has a [[link]] with older = "D08" and newer = that entry's id, whose line matches the phrase in the entry, so the decisions-reconciled check passes.
6. [content] CHANGELOG.md has an entry describing only what the diff does (shared stand-in helper, listed tests moved onto it, ETXTBSY retry, stress test, D08 reconcile entry), with no unrelated claims.
7. [content] The diff touches only crates/athena-testutil, the manifests and Cargo.lock, the six listed test files, docs/architecture.md, docs/decisions.md, docs/decision-links.toml and CHANGELOG.md; athena.toml, defaults.toml and workflows are unchanged.

## Expected deliverables

- crates/athena-testutil/Cargo.toml (create)
- crates/athena-testutil/src/lib.rs (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- xtask/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- docs/architecture.md (modify)

## Out of scope

- Non-test code that materialises files with permissions (pin.rs, snapshot.rs non-test code, toolconfig.rs)
- Changing athena.toml, defaults.toml or workflows
- Changing D08's substance or crates/athena-core/src/run/candidate.rs
- Adding a general pre-dispatch decision-mapping step (the introspection's prevention idea)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Helper and stress test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New crate has a passing stress test; workspace tests and clippy must pass.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Helper is a few lines of std; no crate fits the need, recorded in the helper docs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Stress test runs scripts under concurrent forks.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is no ETXTBSY failures plus the decisions check passing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the ETXTBSY cause.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper crate, call-site swaps, one decision entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 26a10e895f35 → candidate cd7a0c3dd979. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   7 ++
 Cargo.lock                           |  12 +++
 Cargo.toml                           |   2 +
 crates/athena-agent/Cargo.toml       |   1 +
 crates/athena-agent/src/claude.rs    |   4 +-
 crates/athena-cli/Cargo.toml         |   1 +
 crates/athena-cli/tests/reconcile.rs |   7 +-
 crates/athena-core/Cargo.toml        |   3 +
 crates/athena-core/src/checks.rs     |   9 +-
 crates/athena-core/src/reading.rs    |  17 +---
 crates/athena-git/Cargo.toml         |   3 +
 crates/athena-git/src/snapshot.rs    |  10 +-
 crates/athena-testutil/Cargo.toml    |  17 ++++
 crates/athena-testutil/src/lib.rs    | 187 +++++++++++++++++++++++++++++++++++
 docs/architecture.md                 |   1 +
 docs/decision-links.toml             |   7 ++
 docs/decisions.md                    |  18 ++++
 xtask/Cargo.toml                     |   1 +
 xtask/shard-timings.txt              |   2 +
 xtask/src/main.rs                    |   4 +-
 20 files changed, 275 insertions(+), 38 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-063000-vke2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-063000-vke2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176001411 for cd7a0c3dd97977973db107f5c790424f2e6cbaab)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176001411 for cd7a0c3dd97977973db107f5c790424f2e6cbaab)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176001411 for cd7a0c3dd97977973db107f5c790424f2e6cbaab)
- ci-result: `not a check` passed (Check results for cd7a0c3dd97977973db107f5c790424f2e6cbaab come from the CI run https://github.com/doodla/athena/actions/runs/42176001411 (665s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`: all pass or are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
