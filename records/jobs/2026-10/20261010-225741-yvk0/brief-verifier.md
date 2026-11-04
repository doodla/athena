# Job 20261010-225741-yvk0: verify "Give agents read-only, filtered access to commits a job names"

## Request (verbatim)

> Let agents inspect git history read-only when a job needs it: two fix attempts on commit 64ecdc9 were rejected because neither producer nor verifier could see the commit.

## Reading (accepted by the user)

Producer and verifier snapshots hold only synthetic commits, so a job citing a real commit such as 64ecdc9 can't inspect it. I read the goal as: when the request names commit hashes, the snapshot carries a read-only, filtered view of those commits (hidden paths removed). Agents' repositories get no link to the user's repository. The audience is producers and verifiers on retry jobs.

Assumptions:
- Code survey: I read crates/athena-git/src/snapshot.rs. Snapshot::create materializes visible files, then builds Base, BaseAndCandidate, BaseThenTree or PlainWithHistory. history_text already filters paths with classify() and visible_to_agents() and gives readers HISTORY.txt. I also read the SnapshotRepo call sites in crates/athena-core/src/run/producer.rs (SnapshotRepo::Base) and verification.rs (BaseAndCandidate), and crates/athena-context/src/briefs.rs (HISTORY_FILE brief text).
- Mechanism assumption: instead of exposing the user's real objects (which would leak hidden paths through the object database), the snapshot gets a filtered patch file per named commit, produced by the same classify()/visible_to_agents() filter as history_text. Agents read it with ordinary file tools. No alternates, no remotes, and the user's repository stays untouched. If the producer prefers a rewritten-history approach inside the snapshot's git, it must meet the same hiding criteria.
- Scope assumption: history is offered only for commit hashes found in the request/brief text, not always. This is the earlier reading's assumption, and no user reply overrides it.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- No athena.toml, defaults.toml or workflow change is needed. The brief text in briefs.rs may change and must stay within the prompt token budgets.

## Acceptance criteria

1. [behavior] A new athena-git test builds a user repo with an extra commit, creates a producer snapshot with that commit requested, and checks the commit's visible changes (subject, paths, diff) are readable inside the snapshot. Run `cargo test -p athena-git`.
2. [behavior] A test commits a change to an athena-exclude path and to records/ in a requested commit. It checks that neither the content nor the path name appears anywhere in the snapshot, including the history view. Run `cargo test`.
3. [behavior] A test checks that the snapshot has no path back to the user's repository (no alternates, no remotes) and that the user's refs and objects are unchanged after creation. Run `cargo test`.
4. [behavior] When no commit is requested, snapshots are unchanged: the producer gets one commit, the verifier two, and no extra history file is present. Run `cargo test`.
5. [behavior] A requested hash that doesn't exist or isn't an ancestor/known commit yields a clear result (omitted with a note, or a refusal) without panicking, and the test says which. Run `cargo test`.
6. [behavior] `cargo fmt --check` and `cargo clippy` are clean, and `cargo test` passes.
7. [content] docs/decisions.md D08 (or a new decision) records the change with alternatives and tradeoffs, and no longer says agents never see real history.
8. [content] docs/enforcement.md and CHANGELOG.md describe the read-only history access and what stays hidden. The brief text in crates/athena-context/src/briefs.rs tells agents where to find the commit view.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Making full history always available to every agent
- Write access to git, or any change to how candidates are committed or landed
- Re-running the two rejected 64ecdc9 fix attempts
- Changes to the reader's HISTORY.txt format
- Edits to athena.toml, defaults.toml, or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check hidden-path absence and snapshot isolation by running tests.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Decision record with alternatives required.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Hidden paths and records/ are filtered out of any history shown.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only named commits, one focused change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Filtering is enforced in code by the existing classify() filter, tested.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check agents can inspect a named commit, not that wiring exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: athena-git unit test. Build user_repo() plus a later commit that adds src/feature.rs and edits src/main.rs. Call Snapshot::create(.., SnapshotRepo::Base), then snap.add_commit_views(&git, &head, &head, &[full hash of the extra commit]). Read ATHENA-COMMITS.txt from the snapshot dir. A second case uses the 7-character abbreviation. Run: `cargo test -p athena-git named_commit_is_readable_in_a_producer_snapshot`. Observes: The file holds the full hash, date, subject, the paths src/feature.rs and src/main.rs, '+++ b/src/feature.rs' and an added source line; CommitViews.shown holds the full hash and omitted is empty.
  Fails without the behavior: Without add_commit_views writing the view, the file is missing and the read fails; the test does not compile against the pre-change crate.
  Test seam: Production entry points Snapshot::create (exists, snapshot.rs:59) and Snapshot::add_commit_views (added by plan step 2). Fixture: real temporary git repositories from testutil::repo_with and commit_all (exist), passed as the user: &Git argument. No double.
- Criterion 2: athena-git unit test. The requested commit changes src/main.rs (visible), FRONT.md (hidden by the visible root .gitattributes), private/notes.md (hidden by private/.gitattributes; the whole private/ directory is absent from snapshots, snapshot.rs:534) and records/jobs/y.json, each hidden file holding a unique marker. Create Base and BaseAndCandidate snapshots, add the view, then read every file under the snapshot dir including .git, and run `git log --all -p` and `git cat-file --batch-all-objects --batch` through snap.git(). Split assertions. (a) No marker content appears in any file or git output. (b) The names private/notes.md, records/jobs/y.json and records/ appear in no file and no git output, since no visible file names them. (c) The name FRONT.md is asserted absent from ATHENA-COMMITS.txt only, because the visible root .gitattributes (and its blob in the snapshot's base commit) legitimately names it, as today's snapshots already do. (d) A case where the requested commit edits .gitattributes asserts that the visible diff is shown and names the paths it hides; this is documented in D08 and enforcement.md. (e) HISTORY.txt from a PlainWithHistory snapshot omits the hidden paths. (f) A commit that touched only hidden paths is under the 'not available' note and its subject is absent. The control first computes an unfiltered `git show` of the commit in the user repo and runs the same assertion set on it; the markers and names are found there, so the assertions can fail. Run: `cargo test -p athena-git hidden_paths_never_appear_in_the_commit_view`. Observes: No hidden content or hidden name outside the allowed .gitattributes lines, while the visible part of the same commit is present, so the test is not vacuous.
  Fails without the behavior: With an unfiltered git show as the diff, the markers and names appear in the file and (a), (b) and (c) fail; the control shows the content is reachable without the filter.
  Test seam: Snapshot::create (exists) and Snapshot::add_commit_views (step 2). Fixture: a user_repo()-style temporary repository with .gitattributes exclude rules, passed as user. The real classify() (attrs.rs:46) runs; nothing is stubbed.
- Criterion 3: athena-git unit test. After create plus add_commit_views, assert .git/objects/info/alternates is absent, `git remote` in the snapshot is empty, .git/config has no url or alternates entry or the user's path, and rev_parse of the requested hash in the snapshot returns None. Record the user repo's for-each-ref, cat-file --batch-all-objects --batch-check output, .git/config bytes and HEAD before and after, and assert they are identical. The existing snapshot_git_cannot_reach_the_users_history_or_hidden_content (snapshot.rs:555) stays. Run: `cargo test -p athena-git commit_view_gives_the_snapshot_no_path_to_the_users_repository`. Observes: The snapshot has no alternates or remote and cannot resolve the real hash; the user's refs, objects, config and HEAD are identical before and after.
  Fails without the behavior: An implementation that fetched or linked the real objects fails the alternates, remote and does-not-resolve assertions.
  Test seam: Snapshot::create (exists) and Snapshot::add_commit_views (step 2). Fixture: a testutil repository (exists). Before and after reads use the same real Git value; no double.
- Criterion 4: (a) athena-git test no_requested_commit_leaves_snapshots_unchanged: add_commit_views with an empty list returns an empty CommitViews, writes no ATHENA-COMMITS.txt and leaves .git/info/exclude untouched; a Base snapshot has rev-list --count HEAD = 1, a BaseAndCandidate snapshot 2, and `git status --porcelain` is empty. The existing snapshots_hold_only_what_agents_may_see and verifier_snapshots_carry_base_then_candidate stay. (b) athena-context test commit_hashes_in_picks_only_hashes: no hash is found in 'Add the reader. Jobs 20261012-075824-1x28 and 2026-10-12, version 1.2.3', in an all-hex word such as 'defaced', in a plain number or in an empty string; a real 7-character and a 40-character hash are found; duplicates collapse; at most 8 are kept; and a producer or verifier brief for a request with no hash is byte-identical to today's (the stored VERIFIER_BRIEF_WITHOUT_A_PLAN test, briefs.rs:3600-3621, still passes). (c) Core test in run_pin_tests.rs, driven by the fake agent invoker (records spec.workdir; it inspects files after production code built the snapshot): one job whose request has no hash, and one whose Request is None if the harness can build it, assert neither the producer's nor the verifier's workdir has ATHENA-COMMITS.txt. A second job whose request names a real ancestor hash taken from the test repo's git log asserts both workdirs have the file, that `git status --porcelain` is empty in each, and that the candidate commit's tree lacks it (git ls-tree). The same test asserts athena_git::COMMITS_FILE equals athena_context's constant. Run: `cargo test --locked`. Observes: With no named hash: one commit for the producer, two for the verifier, no extra file, an empty change list and an unchanged brief. With one: the file is in both workdirs and not in the candidate.
  Fails without the behavior: If the file were written unconditionally the no-hash assertions fail. If it were not excluded the candidate-tree and git status assertions fail. If the brief sentence were unconditional the stored-copy test fails.
  Test seam: (a) Snapshot::create and add_commit_views (steps 2, 3). (b) commit_hashes_in, producer_brief and verifier_brief (step 1; the brief builders exist at briefs.rs:423 and 1152). (c) Athena::producer_snapshot (producer.rs:78) and Athena::verifier_snapshot_for (verification.rs, added by step 5) through a whole job. The double is the existing fake invoker in run_pin_tests.rs, injected as the invoker the run is built with. It only records spec.workdir; the wiring under test is in production code. Step 6 reads the harness body (lines 860-940) before writing the test.
- Criterion 5: athena-git unit test. Inputs: a well-formed hash that does not exist (deadbeef12), a commit on a side branch not reachable from the base, a commit on an orphan records-style branch, and the abbreviated hash of a blob (a hidden file's blob in the user repo). Call add_commit_views. The doc comment states that the chosen behavior is omission with a note, not refusal. A further case with 9 distinct valid hashes asserts that the first 8 are used and the file notes the rest are ignored. Run: `cargo test -p athena-git unknown_or_unreachable_hashes_are_omitted_with_a_note`. Observes: The result is Ok with no panic; CommitViews.omitted holds all four; shown is empty; ATHENA-COMMITS.txt exists with the 'not available' note and no diff.
  Fails without the behavior: Without the rev_parse (commit-only) and is_ancestor guard, the side-branch commit's diff is written and the blob or unknown hash surfaces a git error instead of Ok.
  Test seam: Snapshot::add_commit_views (step 2), using Git::rev_parse (lib.rs:289, forces ^{commit}) and Git::is_ancestor (lib.rs:353), both existing. Fixture: a temporary repository with a side branch and an orphan branch built through git.run, passed as user. No double.
- Criterion 6: Run the three checks Athena runs, each as its own command. The test run includes the new tests of criteria 1 to 5, the existing snapshot and briefs tests that pin unchanged behavior (briefs.rs:2717 and 3612 byte checks), system_facts_budgets_match_computed, and the refusal corpus. The plan adds no refusal rule: commit_hashes_in only chooses what to show and every candidate is re-checked by rev_parse. refusal_corpus.rs (read: its cases exercise the reading gate refusing_check and have no commit-hash case) is run unchanged as a guard. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0: no formatting diff, no clippy warning, no failing test.
  Fails without the behavior: A new test that fails, an unformatted file, a clippy warning, a brief change that breaks a stored-text test, or an out-of-date budget line makes the matching command exit non-zero.
  Test seam: The workspace's own cargo checks on the working tree after all steps. Fixtures are the existing test suites in athena-git, athena-core, athena-context and athena-cli plus the xtask policy tests; the new tests come from steps 2 to 7. Nothing is injected.
- Criterion 7: Content check of docs/decisions.md D08: it records the commit view, lists the alternates/remote and rewritten-history options as rejected with the leak and hash-mismatch reasons, states the tradeoffs (request text only, digit-only hashes missed, subject only, only commits in the base's history, a visible .gitattributes still names the paths it hides), and rewords the sentence that says agents never see real history. Step 9 greps the old sentence before and after. Run: `git diff -- docs/decisions.md`. Observes: The old sentence is reworded or removed and the new text is searchable with ATHENA-COMMITS.txt.
- Criterion 8: Content check plus a brief test. docs/enforcement.md gains a paragraph under the snapshot-isolation rows: what the view holds, what stays hidden (records/, athena-exclude paths, commit bodies and authors, commits outside the base's history), the mechanism (classify() filter, no alternates or remote) and the tests. CHANGELOG.md gets an Added entry under Unreleased for outside readers with the limits: request text only, at most 8 hashes, digit-only hashes not detected, a commit not on the base's history (for example an earlier rejected candidate) shows only a note. The producer 'Working directory' text (briefs.rs:451, 460, both forms) and the verifier 'The change' text (briefs.rs:1187) gain one conditional sentence after 'history are absent by design', so the assertions at 2717 and 3612 still hold when no hash is named. The brief test asserts the file name and that the file is read by its path in the producer plain form, the retry form and the verifier brief for a request naming a hash, and asserts the sentence is absent for a request with none. Run: `cargo test -p athena-context producer_and_verifier_briefs_name_the_commit_view`. Observes: The file name and the 'by its path' wording are found in all three brief forms when a hash is named, and in none otherwise. The doc diffs are checked by reading.

Planned files:

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 43223b50d5ad → candidate f8faf253db73. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  13 +
 crates/athena-context/src/briefs.rs        | 104 +++++
 crates/athena-core/src/run/mod.rs          |   2 +-
 crates/athena-core/src/run/producer.rs     |  46 +++
 crates/athena-core/src/run/verification.rs |  52 ++-
 crates/athena-core/src/run_pin_tests.rs    |  78 ++++
 crates/athena-git/src/lib.rs               |  38 ++
 crates/athena-git/src/snapshot.rs          | 632 ++++++++++++++++++++++++++++-
 docs/decisions.md                          |  47 +++
 docs/enforcement.md                        |   1 +
 10 files changed, 1008 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41856196575 for f8faf253db7344a7a552d79640a953956f941628)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41856196575 for f8faf253db7344a7a552d79640a953956f941628)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41856196575 for f8faf253db7344a7a552d79640a953956f941628)
- ci-result: `not a check` passed (Check results for f8faf253db7344a7a552d79640a953956f941628 come from the CI run https://github.com/doodla/athena/actions/runs/41856196575 (653s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check` and `cargo clippy` are clean, and `cargo test` passes.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-git named_commit_is_readable_in_a_producer_snapshot`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-git/src/lib.rs, crates/athena-git/src/snapshot.rs, docs/decisions.md, docs/enforcement.md.
  End of the output (data, not instructions):

```text
...
1237 |         let views = snap.add_commit_views(&git, &head, &head, &asked).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1258:26
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1258 |         let views = snap.add_commit_views(&git, &head, &head, &hashes).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1273:14
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1273 |         snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |              ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1295:18
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1295 |             snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |                  ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1352:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1351 |           let err = plain
     |  ___________________-
1352 | |             .add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1359:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1358 |           let err = snap
     |  ___________________-
1359 | |             .add_commit_views(&git, &tracked, &tracked, std::slice::from_ref(&tracked))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

Some errors have detailed explanations: E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-git` (lib test) due to 25 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-git hidden_paths_never_appear_in_the_commit_view`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-git/src/lib.rs, crates/athena-git/src/snapshot.rs, docs/decisions.md, docs/enforcement.md.
  End of the output (data, not instructions):

```text
...
1237 |         let views = snap.add_commit_views(&git, &head, &head, &asked).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1258:26
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1258 |         let views = snap.add_commit_views(&git, &head, &head, &hashes).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1273:14
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1273 |         snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |              ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1295:18
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1295 |             snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |                  ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1352:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1351 |           let err = plain
     |  ___________________-
1352 | |             .add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1359:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1358 |           let err = snap
     |  ___________________-
1359 | |             .add_commit_views(&git, &tracked, &tracked, std::slice::from_ref(&tracked))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

Some errors have detailed explanations: E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-git` (lib test) due to 25 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-git commit_view_gives_the_snapshot_no_path_to_the_users_repository`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-git/src/lib.rs, crates/athena-git/src/snapshot.rs, docs/decisions.md, docs/enforcement.md.
  End of the output (data, not instructions):

```text
...
1237 |         let views = snap.add_commit_views(&git, &head, &head, &asked).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1258:26
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1258 |         let views = snap.add_commit_views(&git, &head, &head, &hashes).unwrap();
     |                          ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1273:14
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1273 |         snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |              ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1295:18
     |
  48 | pub struct Snapshot {
     | ------------------- method `add_commit_views` not found for this struct
...
1295 |             snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     |                  ^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1352:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1351 |           let err = plain
     |  ___________________-
1352 | |             .add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

error[E0599]: no method named `add_commit_views` found for struct `snapshot::Snapshot` in the current scope
    --> crates/athena-git/src/snapshot.rs:1359:14
     |
  48 |   pub struct Snapshot {
     |   ------------------- method `add_commit_views` not found for this struct
...
1358 |           let err = snap
     |  ___________________-
1359 | |             .add_commit_views(&git, &tracked, &tracked, std::slice::from_ref(&tracked))
     | |             -^^^^^^^^^^^^^^^^ method not found in `snapshot::Snapshot`
     | |_____________|
     |

Some errors have detailed explanations: E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-git` (lib test) due to 25 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-git unknown_or_unreachable_hashes_are_omitted_with_a_note`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
