# Job 20261024-115923-02gf: verify "Write landing-intent and landing records in athena land --pr"

## Request (verbatim)

> try again on 20261024-115052-bzw6

## Reading (accepted by the user)

Retry job 3 part 1 of records-history: gh::land_pr writes and pushes a LandingIntent to athena/records before merging, then a Landing record (second parent = PR head) after. A small constructor builds the real records sink from a Git and its remote; Athena::land_pr (and so drain.rs) calls it, and a test covers that constructor. Audience: the maintainer and verifier checking via cargo test.

Assumptions:
- The remote is the source of truth for athena/records: commit built on the remote tip, pushed with a lease, local ref moves only after the push succeeds; rejected push fetches, rebuilds and retries a bounded number of times.
- A local records ref ahead of or diverged from the remote is reset to the remote before writing.
- No merge-requested marker. If the merge cannot be confirmed, no landing record is written, a PrReport warning names the intent, exit codes stay as today, and reconcile (job 3 part 2) repairs it.
- No fake-runner rig for Athena::land_pr; wiring is proven by testing the constructor that builds the real sink.
- display.rs and main.rs change only if the warning needs one line.
- Records writing sits in gh::land_pr behind the records sink it already takes; existing trailers, records/ and resync_onto_merge behavior are unchanged.

## Acceptance criteria

1. [behavior] A free-function gh::land_pr test with a fake host reads the records branch and finds a landing-intent record then a landing record, in that order. Intent: PR number, head sha, head tree, expected base, job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification, intent id. Run: cargo test -p athena-core.
2. [behavior] A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test.
3. [behavior] A test with a fake host that refuses the merge shows the local and remote records branch hold the intent and no landing record, and land_pr returns its existing error. Run: cargo test.
4. [behavior] A call-order test with a bare remote shows the intent is on the remote before PrHost::merge is called and the landing record after. Run: cargo test.
5. [behavior] Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded), succeeds, and local and remote tips match; the local ref moves only after a successful push. Run: cargo test.
6. [behavior] Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; the next land_pr succeeds with no divergence error. Run: cargo test.
7. [behavior] Merged-but-unconfirmed test: no landing record is written, the PrReport warning names the intent, exit codes are unchanged, and a rerun creates no duplicate records. Run: cargo test.
8. [behavior] A test of the constructor that builds the real records sink from a Git and its remote shows it returns a working sink writing to the remote records branch; Athena::land_pr calls this constructor (used by drain.rs), so a call passing no sink is not the production path. Run: cargo test.
9. [behavior] A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Run: cargo test.
10. [behavior] Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean.
11. [content] Records use athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer.
12. [content] docs/design/records-history.md states how sync_from changed (or the distinct recover path) and has a failure-path table (starting state, function, cited behavior); new functions carry doc comments citing the design section; CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Reconcile of merged-but-unconfirmed landings (job 3 part 2)
- A merge-requested marker
- A fake-runner rig for Athena::land_pr
- Changes to existing trailers, records under records/, or resync_onto_merge
- Changing exit codes in main.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new path; cargo test, clippy --all-targets and fmt --check clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check records on the branch, order, parents and failure paths, not mere existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite design sections; the design doc gets a failure-path table.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing RecordsWriter and history.rs types; no new writer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Remote-tip build with lease fixes divergence at its cause.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test with a real temp repo, a bare remote, a fake host and GitLandingRecords; run land_pr, then read athena/records. Run: `cargo test -p athena-core`. Observes: First-parent log shows the intent commit then the landing commit. Intent: PR number, head sha, head tree, expected_base equal to the remote main tip sha (asserted), job ids. Landing: PR number, head sha, host commit, merge method, commits with tree/job/verification (verification equals the job's RecordId via verification_id), intent id. Both validate.
  Fails without the behavior: With no intent or landing writes, the records branch is missing and the test fails.
- Criterion 2: Read the parents of the landing's records commit. Run: `cargo test`. Observes: Parents are [previous records tip, PR head sha]; the intent commit has one parent.
  Fails without the behavior: Without extra-parent support the second parent is absent and the assertion fails.
- Criterion 3: Fake host whose merge returns Err; read local and remote records refs. Run: `cargo test`. Observes: Both refs hold the same tip with the intent and no landing file; land_pr returns the existing error.
  Fails without the behavior: A landing written regardless of the merge would exist in the tree.
- Criterion 4: Fake merge reads the bare remote's records ref at call time; read again after land_pr returns. Run: `cargo test`. Observes: At merge time the remote has the intent and no landing; afterwards it has the landing.
  Fails without the behavior: If the intent were pushed after the merge, the remote has no intent at merge time.
- Criterion 5: A hook between build and push makes another clone push a records commit; a second case moves the remote on every attempt. Run: `cargo test`. Observes: First push rejected; commit rebuilt on the fetched tip and retry succeeds; local and remote tips equal; other writer's file kept; local ref unchanged during the rejected attempt; the always-moving case ends in a bounded Contended error.
  Fails without the behavior: A plain push without lease and rebuild fails non-fast-forward; an unbounded loop would hang.
- Criterion 6: Make a local records ref ahead of the remote, and another diverged; run the remote append and then land_pr. Run: `cargo test`. Observes: Local reset to the remote tip; write succeeds with no divergence error; leftover local commit not on the remote; next land_pr succeeds.
  Fails without the behavior: Without the reset the push is non-fast-forward or leftover commits leak.
- Criterion 7: Fake host whose merge succeeds but merged_commit stays None; run land_pr twice. Run: `cargo test`. Observes: land_pr returns Ok (the same result shape the CLI already maps today; main.rs and drain.rs exit mapping untouched in the diff); no landing record; report.warnings names the intent id; the rerun adds no second intent and no landing (counts equal).
  Fails without the behavior: Writing a landing anyway or minting a new intent on rerun changes the counts.
- Criterion 8: Call GitLandingRecords::for_remote on a repo with a bare origin, then intent() and landing() through the returned sink; an unknown remote name errors. Wiring is shown by this test plus diff inspection that Athena::land_pr passes Some(sink) from for_remote and propagates its error (no None fallback). Run: `cargo test`. Observes: The remote athena/records holds the records written through the sink; the unknown remote gives a clear error.
  Fails without the behavior: A sink writing only locally leaves the remote records branch empty and the test fails.
- Criterion 9: Repo where a commit between remote base and PR head lacks an Athena-Job trailer or verification; run the collection and land_pr. Run: `cargo test`. Observes: Error names that sha and says it lies between the remote base and the PR head; the records branch is unchanged.
  Fails without the behavior: The commit would be skipped or land_pr would proceed to write an intent.
- Criterion 10: Run the full suite; old trailer, records/ and resync_onto_merge tests are unchanged apart from `records: None` in LandPr literals and the merge return type in fakes. Run: `cargo test`. Observes: All pass; then `cargo clippy --all-targets` and `cargo fmt --check` are clean.
  Fails without the behavior: If the change altered existing trailer, records/ or resync_onto_merge behavior, those unchanged tests fail; lint or format regressions fail clippy/fmt.
- Criterion 11: Inspect the diff. Run: `git diff`. Observes: The sink builds LandingIntent, Landing, LandingCommit with MergeMethod from athena_data::history, calls validate() before writing, and writes only via RecordsWriter; no new writer type.
- Criterion 12: Inspect docs/design/records-history.md, CHANGELOG.md and new doc comments. Run: `git diff`. Observes: Design doc describes sync_from as a new reset-to-remote recover step, states the verification-id mapping, and has a failure-path table (starting state, function, cited behavior) covering rejected push, local ahead, local diverged, refused merge, unconfirmed merge and bad range commit; new functions cite design sections; CHANGELOG has an entry.

Planned files:

- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## The change

Base a566ad5c72f3 → candidate 363c7fe2fc6b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |  12 +
 crates/athena-cli/src/display.rs          |   4 +
 crates/athena-cli/tests/external_calls.rs |   3 +-
 crates/athena-core/src/gh.rs              | 723 +++++++++++++++++++++++++++++-
 crates/athena-core/src/land.rs            |  52 +++
 crates/athena-git/src/lib.rs              |  86 +++-
 crates/athena-git/src/records_branch.rs   | 341 +++++++++++++-
 docs/design/records-history.md            |  47 ++
 8 files changed, 1239 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1071 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the recorded check results: acceptance criteria not met or not reported: "A test shows the records commit carrying the landing record has the PR head sha as its second parent. Run: cargo test."; "A test with a fake host that refuses the merge shows the local and remote records branch hold the intent and no landing record, and land_pr returns its existing error. Run: cargo test."; "A call-order test with a bare remote shows the intent is on the remote before PrHost::merge is called and the landing record after. Run: cargo test."; "Rejected-push test: the remote records branch moves between build and push; the commit is rebuilt on the fetched remote tip and retried (bounded), succeeds, and local and remote tips match; the local ref moves only after a successful push. Run: cargo test."; "Leftover-local-intent test: a local records ref ahead of or diverged from the remote is reset to the remote before writing; the next land_pr succeeds with no divergence error. Run: cargo test."; "Merged-but-unconfirmed test: no landing record is written, the PrReport warning names the intent, exit codes are unchanged, and a rerun creates no duplicate records. Run: cargo test."; "A test of the constructor that builds the real records sink from a Git and its remote shows it returns a working sink writing to the remote records branch; Athena::land_pr calls this constructor (used by drain.rs), so a call passing no sink is not the production path. Run: cargo test."; "A commit between the remote base and the PR head with no landing trailer or verification gives an error naming its sha and saying it lies between the remote base and the PR head. Run: cargo test."; "Existing land and land_pr tests (trailers, records under records/, resync_onto_merge rebuild) pass unchanged; cargo test, cargo clippy --all-targets and cargo fmt --check are clean."; "Records use athena-data history.rs types (LandingIntent, Landing, LandingCommit, MergeMethod), are validated before writing, and are written through the athena-git RecordsWriter, not a new writer."; "docs/design/records-history.md states how sync_from changed (or the distinct recover path) and has a failure-path table (starting state, function, cited behavior); new functions carry doc comments citing the design section; CHANGELOG.md has an entry.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
