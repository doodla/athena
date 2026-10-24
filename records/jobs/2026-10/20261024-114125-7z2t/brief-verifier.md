# Job 20261024-114125-7z2t: verify "Add reconcile step for missing landing records at land and status"

## Request (verbatim)

> Build job 3, part 2, of the records-history plan (docs/design/records-history.md section 8.1, decision D111): a reconcile step that finds main's first-parent commits since the last landing record that have no landing record, matches each to a landing-intent record by head tree and expected base, and writes the missing landing record; with no matching intent it asks the host which pull request merged, and if the host cannot answer it reports the commit as unattributed and writes nothing. Run it at the start of athena land and athena status. Tests: a crash between the host's merge and the landing record is repaired from the intent alone; a host that is unavailable gives 'unattributed' and no record; a commit with no intent and no pull request is reported, never guessed.

## Reading (accepted by the user)

Add a bounded reconcile step in athena-core (records-history §8.1, D111) that repairs missing landing records on main's first-parent commits from landing intents, or from the host, else reports 'unattributed'. It runs at the start of `athena land` and `athena status`. The audience is the maintainer and verifier.

Assumptions:
- The walk starts after the later of the last landing record and the first landing-intent record on athena/records.
- Matching requires both head tree and expected base (parent) to equal the intent's.
- Unattributed commits are reported in output, not treated as errors that abort land or status.
- Exact module placement in athena-core is my guess from file names; I did not read the code.

## Acceptance criteria

1. [behavior] Crash repair: a test has a main first-parent commit whose tree and parent match a landing-intent and that has no landing record. Reconcile writes the landing record from the intent alone, with the host making no call. Check with `cargo test`.
2. [behavior] Host unavailable: a commit with no matching intent and an erroring host gives an 'unattributed' finding and writes no landing record. Check with `cargo test`.
3. [behavior] No guessing: a commit with no intent and no merged PR is reported unattributed and nothing is written. It is never attributed to a nearby intent or PR. Check with `cargo test`.
4. [behavior] Host fallback: with no matching intent and the host naming the merged PR, the landing record is written for that PR. Commits that already have a landing record are skipped. Check with `cargo test`.
5. [behavior] Bound: commits before the first landing-intent record on athena/records, and before the last landing record, are never reconciled. When the records branch is absent or holds no intent, reconcile does nothing and reports that. A test covers both cases. Check with `cargo test`.
6. [behavior] `athena land` and `athena status` run reconcile first, and an unattributed commit appears in their output. A test shows this. `cargo test`, `cargo clippy` and `cargo fmt --check` are clean.
7. [content] CHANGELOG.md has an entry citing docs/design/records-history.md §8.1, D111, and job 3 part 2.
8. [content] The change reuses the existing host and git helpers and record types, adds no dependency, and changes no governance file.

## Expected deliverables

- crates/athena-core/src/reconcile.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-data/src/history.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the existing history check or the cutover
- Writing records for unattributed commits
- Changes to landing-intent creation

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus clippy and fmt criteria.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing helpers; no new dependency.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check behavior via cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused reconcile step.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check repaired or reported outcomes.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Temp repo with bare origin; main commit whose tree and first parent equal an intent's head_tree_sha and expected_base; no landing; run reconcile with a counting fake host. Run: `cargo test --locked reconcile::tests::crash_between_merge_and_landing_is_repaired_from_the_intent_alone`. Observes: One Repaired finding (source Intent); records branch has a landing with host_commit = that commit and the intent id; host call count 0.
  Fails without the behavior: Without intent matching, the commit goes to the host (count > 0) and no landing is written.
- Criterion 2: Commit with no matching intent; fake host returns Err. Run: `cargo test --locked reconcile::tests::unavailable_host_gives_unattributed_and_writes_nothing`. Observes: One Unattributed finding mentioning host unavailable; landing count and records tip unchanged.
  Fails without the behavior: A swallowed error or a written record would drop the finding or raise the landing count.
- Criterion 3: Commit with no match, plus nearby intents with different tree and base; host returns Ok(None); variant where host names a PR with no intent. Run: `cargo test --locked reconcile::tests::no_intent_and_no_pull_request_is_reported_never_guessed`. Observes: Unattributed in both cases; no landing written; nearby intents remain unfulfilled.
  Fails without the behavior: A nearest-intent or nearest-PR guess would write a landing.
- Criterion 4: Intent for PR 7 whose base differs from the commit parent; host names PR 7 with the same head; a second commit already has a landing; a third is named by a declared-intervention record. Run: `cargo test --locked reconcile::tests::host_names_the_merged_pull_request_when_no_intent_matches`. Observes: Landing for PR 7 written (Repaired, source Host); host called once; the landed and intervention commits are skipped with no extra landing or host call.
  Fails without the behavior: Without the fallback the commit stays unattributed and no landing exists.
- Criterion 5: A: unmatched commits before the first intent's base and before the last landing, one after; an abandoned intent with an early base does not widen the walk; the known-gap case (earlier crashed landing, later landing) is asserted as NOT reconciled. B: no records branch. C: branch with no intent. Run: `cargo test --locked reconcile::tests::bound_skips_older_commits_and_absent_or_empty_records`. Observes: A: only the later commit gets a finding, host never called for older ones. B/C: skipped reason set, no findings, no writes, no host calls.
  Fails without the behavior: Old commits would be reported or sent to the host.
- Criterion 6: Core test calls Athena::status_with_reconcile with a fake Runner on a temp repo with an unattributed commit; CLI integration test runs the athena binary `status` with a fake gh on PATH (external_calls.rs pattern); display test renders the section; land_pr test with fake Runner and collecting Ui. Then clippy and fmt. Run: `cargo test --locked`. Observes: Commit sha and 'unattributed' appear in status output (core and CLI) and in land's collected ui lines before a later land check fails. Also run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.
  Fails without the behavior: If main.rs did not call status_with_reconcile, or land_pr did not call reconcile, those tests would miss the line.
- Criterion 7: Inspect the diff. Run: `git diff CHANGELOG.md`. Observes: New entry citing docs/design/records-history.md §8.1, D111 and job 3 part 2, noting the known gap and that a PR merged without an intent stays unattributed.
- Criterion 8: Inspect the diff; `cargo build --locked` succeeds with the unchanged lockfile. Run: `git diff --stat`. Observes: No Cargo.toml, Cargo.lock, athena.toml, defaults.toml or workflow change; code uses Git, GhClient, GitLandingRecords, LandingFacts and HistoryRecord types.

Planned files:

- crates/athena-core/src/reconcile.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-data/src/history.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 6075da669f23 → candidate d355fec01c58. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  19 +-
 crates/athena-cli/src/display.rs     |   5 +
 crates/athena-cli/src/main.rs        |   2 +-
 crates/athena-cli/tests/reconcile.rs | 138 ++++++
 crates/athena-core/src/gh.rs         | 123 ++++-
 crates/athena-core/src/lib.rs        |   4 +
 crates/athena-core/src/reconcile.rs  | 852 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs     |   5 +
 crates/athena-data/src/history.rs    |  14 +
 9 files changed, 1158 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1081 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
