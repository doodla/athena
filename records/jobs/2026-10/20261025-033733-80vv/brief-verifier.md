# Job 20261025-033733-80vv: verify "Write a git-history file into the planner and critic snapshot"

## Request (verbatim)

> Priority: high
> Give the planner and critic read-only evidence beyond a plain copy of the code (BRIEF.md addendum of 2026-10-25): a git snapshot with history (so they can run git log, git show and git diff, read-only) and the budget printout (cargo test --locked -p athena-cli --test budgets -- --nocapture), with no writes. Planning stops and post-plan rejections came from facts only history shows, such as the shape of real pull-request ranges (job 20261024-115923-02gf broke land on every pull request).
> Acceptance:
> - A test shows the planner and critic grants allow only those read-only git commands and the budgets command, and no writes.
> - A planning brief test shows they are told the commands exist.
> - A planning run on a fixture can run git log in its snapshot.
> Files: crates/athena-core/src/run.rs, crates/athena-git/src/snapshot.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

The planner and critic get a generated history file in their read-only snapshot. It holds the last 200 base commits (subject, date, changed paths), without records/ or private paths, and comes from plain git log at snapshot time. Both briefs name the file and say what it holds. No git repository goes in the snapshot, no history is rewritten, and the grants stay unwidened. The audience is the planner and critic agents, and the maintainer who judges whether they stop planning less often.

Assumptions:
- The history file lives at a fixed path in the snapshot root, such as HISTORY.txt; the producer picks the name and the briefs cite the same constant.
- Private paths are excluded with the same filter the snapshot already uses for its file copy.
- Commits whose changed paths are all excluded are omitted or listed without paths; the producer picks one and documents it.
- The 200 commits are counted on the base commit after exclusion, not before.
- The earlier proposals (git log/show/diff grants, budgets command grant) are dropped per the user's reply. The budgets printout is out of scope.

## Acceptance criteria

1. [behavior] A test in athena-git builds a planning snapshot of a fixture repository with several commits, including one touching records/ and one touching a private path. It checks that the history file exists with one entry per commit (subject, date, changed paths), is capped at 200 commits, and contains no records/ or private path. Run: cargo test -p athena-git snapshot.
2. [behavior] The snapshot contains no .git directory and the user's repository is unchanged by snapshotting. A test checks this. Run: cargo test -p athena-git snapshot.
3. [behavior] A briefs test shows planner_brief and critic_brief each name the history file's path and say what it holds (last 200 commits, subject, date, changed paths, records/ and private excluded) and that Read and Grep are used on it. Run: cargo test -p athena-context briefs.
4. [behavior] A test in athena-core asserts the planner and critic grants are unchanged: no Bash or git commands, writes false. Run: cargo test -p athena-core.
5. [content] The time the history file adds to snapshotting is measured, and the figure (labeled as a measurement, with the repository and method) is recorded in docs/decisions.md.
6. [content] CHANGELOG.md has an entry, and docs/decisions.md records the decision. It lists the alternatives (git repository in the snapshot, widened git grants, plain copy), the choice, and cites job 20261024-115923-02gf.
7. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Giving the planner or critic Bash or git command grants
- Putting a .git directory or any repository in the snapshot
- Providing the budgets test output to the planner and critic
- Changing the producer or verifier snapshots

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the file, the briefs and the grants, plus a clean cargo test and clippy.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The snapshot time added is measured and recorded, citing job 20261024-115923-02gf.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the file's contents and the brief text, not just that the code exists.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The history file excludes records/ and private paths.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in snapshot.rs. It builds a fixture repo with .gitattributes marking private/ athena-exclude, and commits touching src/, records/, private/, plus a mixed commit. A second test makes 205 commits. It creates a PlainWithHistory snapshot and reads HISTORY.txt. Run: `cargo test -p athena-git snapshot`. Observes: HISTORY.txt exists. It has one entry per visible commit with subject, date and paths. The records-only and private-only commits are absent. 'records/' and 'private' appear nowhere in the text. The mixed commit lists only its visible path. The 205-commit case has exactly 200 entries, the newest first.
  Fails without the behavior: Without the history writer no HISTORY.txt exists. Without the classify filter the records/ or private text appears. Without the cap the 205-commit test counts 205.
- Criterion 2: Same test module. It snapshots the fixture and compares the user repo before and after. Run: `cargo test -p athena-git snapshot`. Observes: snapshot dir has no .git. `git status --porcelain`, HEAD, refs and .git/config of the user repo are the same before and after. Writing the history changed no tracked file.
  Fails without the behavior: If the snapshot ran git init or copied .git, the .git-absence assertion fails.
- Criterion 3: Unit tests in briefs.rs on planner_brief and critic_brief built from existing test inputs. Run: `cargo test -p athena-context briefs`. Observes: Each brief contains HISTORY_FILE's path, '200', 'subject', 'date', 'changed paths', 'records/', 'private', and 'Read' and 'Grep' in the history paragraph. The constant is used, not a literal. Existing brief tests that assert exact text are updated.
  Fails without the behavior: Without the paragraph the brief has no mention of the file, so the assertions fail.
- Criterion 4: Test in run.rs on the new named function `planning_grant()`, which both the planner and critic call. It asserts the full value. Run: `cargo test -p athena-core`. Observes: tools are exactly Read, Grep and Glob; commands is empty (no Bash, no git); writes is false; permission_mode is dontAsk.
  Fails without the behavior: If a git command or Bash were added, the equality assertion on commands/tools fails.
- Criterion 5: Measure by timing snapshot creation with and without the history on this repository, in a release or test build, over several runs. Record the median and spread, labelled as a measurement, with the repo, commit, and method, in docs/decisions.md. Use a temporary timing test or a one-off cargo run, not left in the tree. Run: `cargo test -p athena-git snapshot -- --nocapture`. Observes: The decisions entry holds a figure in ms (plain copy vs plain copy plus history), the repo (this one), its commit count, and the run count. The figure is labelled measured.
- Criterion 6: Read the two files after the change. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: CHANGELOG Unreleased has an entry. The decisions entry lists the alternatives (repository in snapshot, widened git grants, plain copy), the choice, and cites job 20261024-115923-02gf.
- Criterion 7: Run the three checks. The boundary and personal-data policy tests in xtask run inside cargo test. Run: `cargo test --locked`. Observes: All pass. Also run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` with clean output. The system_facts_budgets_match_computed test is unaffected because briefs sit outside the budget sets.
  Fails without the behavior: A clippy or fmt violation, or a changed always-loaded prompt, would fail these.

Planned files:

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 5516a14ebe31 → candidate e8c30aa27a93. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   6 ++
 crates/athena-context/src/briefs.rs |  48 ++++++++++
 crates/athena-core/src/run.rs       |  49 ++++++++--
 crates/athena-git/src/lib.rs        |  77 ++++++++++++++++
 crates/athena-git/src/snapshot.rs   | 174 +++++++++++++++++++++++++++++++++++-
 docs/decisions.md                   |  41 +++++++++
 6 files changed, 386 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1353 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
