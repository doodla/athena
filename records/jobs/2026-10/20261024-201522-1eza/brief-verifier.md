# Job 20261024-201522-1eza: verify "Add `athena check --attribution` records-only attribution check"

## Request (verbatim)

> Build job 5, part 1, of the records-history plan (docs/design/records-history.md sections 5.2, 6 and 10, decision D111): an attribution check that reads only athena/records, never commit-message trailers. It walks main's first-parent commits from the commit the first landing-intent record expects as its base onward, and requires each to be named as host_commit by a landing record whose head tree equals that commit's tree and whose expected base is its parent; a commit with a matching landing-intent but no landing record yet is pending for 15 minutes from the intent and fails after; any other commit fails as unattributed. Commits before that base are out of scope here (the old history check and, later, the legacy rule judge them). Expose it as a command (for example athena check --attribution) and add a non-blocking CI job on pushes to main and on schedule, beside the existing full-history check, listing .github/workflows/ci.yml as a governance deliverable. Tests: a properly landed squash passes; an unrecorded commit fails naming it; a pending one passes within 15 minutes and fails after; a records-branch commit that is not a fast-forward is reported.

## Reading (accepted by the user)

Add a check reading only the athena/records branch (never commit trailers) that walks main's first-parent commits from the first landing-intent's expected base and requires each to be attributed by a matching landing record, with a 15-minute pending window. Expose it as `athena check --attribution` and add a non-blocking CI job that explicitly fetches athena/records. Audience: the maintainer and CI.

Assumptions:
- Records data models (landing-intent, landing, host_commit, expected_base, head tree) already exist in athena-data/athena-git from earlier jobs.
- The check time is injectable so tests control the 15-minute window.
- A missing athena/records ref or no landing-intent is an explicit failure/error, not silent success.
- CHANGELOG.md gets an entry if the repo keeps one per change.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test with a properly landed squash (landing-intent plus landing whose host_commit is the commit, head tree equal to the commit's tree, expected base equal to its parent) passes the attribution check.
2. [behavior] Run `cargo test`: a main commit with no landing record and no matching intent makes the check fail, and the report names that commit's sha as unattributed.
3. [behavior] Run `cargo test`: a commit with a matching landing-intent and no landing passes when the check time is within 15 minutes of the intent's creation time, and fails after 15 minutes.
4. [behavior] Run `cargo test`: a records-branch history containing a commit that is not a fast-forward of its predecessor is reported by the check.
5. [behavior] Run `cargo run -- check --help`: `--attribution` is listed, and combining it with --history or --pr-head is rejected.
6. [content] The check reads only athena/records records and main's first-parent graph; nothing parses commit-message trailers; commits before the first intent's expected base are not judged. Code has doc comments citing records-history.md sections 5.2, 6, 10 and D111.
7. [content] .github/workflows/ci.yml has a new attribution job beside the history job: continue-on-error true, runs only on push to main and schedule, uses full-history checkout, explicitly fetches athena/records (e.g. `git fetch origin athena/records:athena/records`) before running `athena check --attribution`, pins actions by SHA like existing jobs, and is not a required check. The command reads the branch from the local or remote-tracking ref that was fetched.
8. [behavior] Run `cargo clippy`, `cargo fmt --check` and `cargo test`: all clean and passing for the workspace.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (create)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Legacy-rule judging of commits before the base
- Changes to the existing full-history check
- Making the attribution job a required check
- Job 5 part 2 and later records-history jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for pass, unrecorded, pending, and non-fast-forward cases; clippy and fmt clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust within existing crates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by CI job and command exit status.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests and the command rather than checking file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in attribution.rs: temp repo with base and squash commit; write intent (expected_base=base, head_tree=squash tree) and landing (host_commit=squash, intent=id) with RecordsWriter; run check just after the landing. Run: `cargo test -p athena-gate attribution`. Observes: Report has no findings and the squash is counted attributed.
  Fails without the behavior: If the match logic is missing or compares tree/base wrongly, the squash is flagged and the test fails.
- Criterion 2: Unit test: add a main commit with no intent and no landing; run the check. Run: `cargo test -p athena-gate attribution`. Observes: Report fails with an Unattributed finding containing the commit's full sha; rendered text names it.
  Fails without the behavior: Without the unattributed rule the commit passes silently and the finding assertion fails.
- Criterion 3: Unit test: commit with matching intent and no landing; run at created_at+14m, +15m and +16m. Run: `cargo test -p athena-gate attribution`. Observes: At +14m and +15m no findings and the commit is pending; at +16m a finding names the commit and expired intent.
  Fails without the behavior: No pending window fails the +14m case; no expiry passes the +16m case and fails the test.
- Criterion 4: Unit tests in attribution.rs and records_branch.rs: build a records history with a second-root commit and another dropping a prior file; also a normal branch with an extra-parent commit. Run: `cargo test -p athena-gate attribution`. Observes: Report has a records-history finding naming the offending records commit; the normal branch has none.
  Fails without the behavior: Without the audit the rewritten history yields no finding and the test fails.
- Criterion 5: Binary tests run the real binary: check --help; check --attribution --history; check --attribution --pr-head a b; --attribution in a repo without athena/records. Run: `cargo run -- check --help`. Observes: Help lists --attribution; via cargo test -p athena-cli --test binary the combinations exit non-zero with a clap conflict message, and the no-records case exits 2 with a finding.
  Fails without the behavior: Without conflicts_with the combinations are accepted and the test fails; without the flag the help assertion fails.
- Criterion 6: Inspect the diff for doc comments citing records-history.md 5.2, 6, 10 and D111, no trailer/log_grep/Change::trailers use in attribution.rs, and the pre-base test. Run: `git diff`. Observes: Doc comments present, no trailer calls; the pre-base unrecorded commit test passes with no finding for it (cargo test -p athena-gate attribution).
  Fails without the behavior: If the walk started before the base, the pre-base test would report that commit and fail.
- Criterion 7: Read the workflow diff. Run: `git diff`. Observes: attribution job beside history: continue-on-error true; if push to main or schedule; checkout fetch-depth 0; step git fetch origin athena/records:athena/records before athena check --attribution; actions pinned by the same 40-hex SHAs; comment says not required.
  Fails without the behavior: Missing fetch or wrong if condition would be absent in the diff, so the review fails.
- Criterion 8: Run the workspace gates; each must exit 0. The new tests, the clippy -D warnings gate and fmt would fail on any formatting, lint or test regression introduced by the change (e.g. a failing attribution test, an unused import, unformatted code). Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: No warnings; then cargo fmt --all --check shows no diff and cargo test --locked passes across the workspace, including the new attribution and binary tests.
  Fails without the behavior: If the attribution logic were wrong or the code unformatted or lint-dirty, cargo test, fmt --check or clippy would exit non-zero.

Planned files:

- crates/athena-gate/src/attribution.rs (create)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- .github/workflows/ci.yml (modify)
- CHANGELOG.md (modify)

## The change

Base a1dcb9cc2a5a → candidate ac4392dc5452. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml                |  22 ++
 CHANGELOG.md                            |  13 +
 crates/athena-cli/src/display.rs        |  52 +++
 crates/athena-cli/src/main.rs           |  25 ++
 crates/athena-cli/tests/binary.rs       |  45 +++
 crates/athena-gate/src/attribution.rs   | 623 ++++++++++++++++++++++++++++++++
 crates/athena-gate/src/lib.rs           |   1 +
 crates/athena-git/src/records_branch.rs | 187 ++++++++++
 8 files changed, 968 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1130 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
