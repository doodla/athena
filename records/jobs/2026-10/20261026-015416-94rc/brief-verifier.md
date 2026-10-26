# Job 20261026-015416-94rc: verify "Add scope guard line grouping and thrash signals T1-T5; accept D127"

## Request (verbatim)

> Priority: urgent
> Build job 1 of the scope guard (docs/design/scope-guard.md section 13, decision D127): the line-of-work grouping and the five thrash signals T1 to T5 (section 5) as pure functions over job records, with no I/O. Also mark D127 accepted in docs/decisions.md, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 and 2026-10-25) with two changes: escalations in section 8 go to whoever holds Sri's delegation (Sri only for what the brief reserves to Sri), and the build order starts with jobs 1 and 5 (rules only) before the value scoring and model scorer.
> Acceptance:
> - Unit tests build records for the trailer, test-suite speedup, prompt-budget and cut A cases (section 5.1) and assert which signals fire at which attempt; the T4 test fails if the same-criterion rule is removed.
> - D127 is marked accepted with the two changes, and scope-guard.md section 8 and 13 say so.
> Files: crates/athena-core/src/scope_guard.rs, crates/athena-core/src/lib.rs, docs/decisions.md, docs/design/scope-guard.md, CHANGELOG.md

## Reading (accepted by the user)

Add a pure, I/O-free scope_guard module in athena-core that groups job records into lines of work and computes T1 to T5 per section 5, with replay tests for the four named cases. Mark D127 accepted (brain under Sri's delegation) with the two stated changes, and update scope-guard.md sections 8 and 13 to match. The audience is the Athena owner and the later watcher job (job 5), which will call these functions.

Assumptions:
- Job records are modelled as small input structs in scope_guard.rs (attempt number, verdict, unmet criterion text, cost, agent minutes, plan size) rather than an existing store type, to keep the functions pure; the exact shape is the producer's choice.
- The per-job-kind forecast p90 for T3 is passed in as an input value.
- Case figures not known from the design (per-attempt costs) use even splits, as section 5.1 says.
- No change to athena.toml, defaults.toml or workflows is needed.
- The 'urgent' priority is a user label only; it does not cut wasted time, so no urgent reason is given.

## Acceptance criteria

1. [behavior] cargo test -p athena-core scope_guard passes. Tests build records for the trailer, test-suite speedup, prompt-budget and cut A cases and assert which of T1 to T5 fire and before which attempt, matching the section 5.1 table (speedup: T4 fires after the 2nd rejection; cut A: T1 flag with no pause because criteria differ; prompt-budget: T5 flag at 60 minutes).
2. [behavior] A T4 test fails if the same-criterion comparison is removed or made always-true: a case with two rejections of different criteria must not fire T4 (checkable by reading the test and, if desired, mutating the comparison and running cargo test -p athena-core scope_guard).
3. [content] Line grouping puts a request, its retries and any request that quotes it into one line. T4 normalises criterion text to lower case with numbers removed and returns a maybe result (flag, not pause) on an unclear match. The thresholds in section 5 (T1 3rd attempt, T2 1.5x or 3rd planning round, T3 1.0x/1.5x p90 and $5, T4 two rejections, T5 60/120 minutes) are named constants.
4. [content] scope_guard.rs does no I/O: it uses no std::fs, process, network or clock calls, and takes time and spend as input data. It is exported from crates/athena-core/src/lib.rs.
5. [content] docs/decisions.md D127 status reads accepted, says it was decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 and 2026-10-25), and records both changes: section 8 escalations go to whoever holds Sri's delegation (Sri only for what the brief reserves to Sri), and the build order starts with jobs 1 and 5 (rules only) before value scoring and the model scorer.
6. [content] docs/design/scope-guard.md status line, section 8 (Goes to Sri) and section 13 (build order) state the acceptance and the two changes; CHANGELOG.md has an entry for the module and the decision.
7. [behavior] cargo clippy --all-targets and cargo fmt --check are clean, and cargo test for the workspace passes.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/design/scope-guard.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Jobs 2 to 7 of the build plan: value record, quota and appeal rules, queue-time check, running-line watcher, Sri channel, model scorer.
- Any I/O, reading of records/, or wiring into dispatch or the job-finish path.
- Changing the thresholds in section 5.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Module and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for every signal; clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert which signals fire at which attempt, not that types exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only job 1 plus the decision status change; no watcher, value scoring or I/O.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design sections for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Thresholds are labelled estimates, with the basis cited as in section 5.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The T4 test must fail when the same-criterion rule is removed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run unit tests trailer_*, speedup_*, prompt_budget_*, cut_a_* that build records and assert levels per evaluation point. Run: `cargo test -p athena-core scope_guard`. Observes: All pass; speedup T4 Pause after 2nd rejection; cut A T1 Flag, no Pause; prompt-budget T5 Flag at 60 min, none at 59.
  Fails without the behavior: Without the signal functions, no T4 Pause after the 2nd speedup rejection and the asserted levels would not hold.
- Criterion 2: A different-criteria test asserts T4 not Pause; the speedup test asserts Pause on same criterion with differing numbers. Run: `cargo test -p athena-core scope_guard::tests::t4`. Observes: Both pass; always-true comparison fails the different-criteria test, always-false fails the speedup test.
  Fails without the behavior: The different-criteria test fails if the same-criterion comparison is removed or always true.
- Criterion 3: Grouping, normalisation and maybe tests, plus reading the named constants in source. Run: `cargo test -p athena-core scope_guard`. Observes: Request, retry and quoting request form one line, unrelated does not; 'Under 7 minutes' equals 'under 12 MINUTES'; Maybe flags without pausing; constants T1_ATTEMPT=3, T2_RATIO=1.5, T2_PLANNING_ROUND=3, T3_FLAG_RATIO=1.0, T3_PAUSE_RATIO=1.5, T3_LINE_DOLLARS=5.0, T4_REJECTIONS=2, T5_FLAG_MINUTES=60, T5_PAUSE_MINUTES=120 exist.
  Fails without the behavior: Grouping test fails if links are ignored; maybe test fails if a maybe paused.
- Criterion 4: Compile with the export line, and read the source for forbidden APIs. Run: `cargo check -p athena-core`. Observes: Compiles with pub mod scope_guard; in lib.rs; no std::fs, process, net, SystemTime or Instant in the file.
  Fails without the behavior: Without the export the module is unreachable from the crate and the later watcher cannot call it; an I/O call would show in the source read.
- Criterion 5: Read the D127 diff. Run: `git diff docs/decisions.md`. Observes: Status accepted, decider the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 and 2026-10-25), both changes recorded.
- Criterion 6: Read both diffs. Run: `git diff docs/design/scope-guard.md CHANGELOG.md`. Observes: Status line accepted; section 8 says whoever holds Sri's delegation; section 13 starts with jobs 1 and 5 (rules only); CHANGELOG Unreleased Added has an entry.
- Criterion 7: Run clippy, then cargo fmt --all --check and cargo test --locked across the workspace. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy and fmt report nothing; every workspace test passes, including the boundary scan and budget tests over the edited docs and the new scope_guard tests.
  Fails without the behavior: Clippy exits non-zero on any warning in the new module, fmt --check fails on unformatted code, and cargo test --locked fails if a scope_guard test, the boundary scan or a budget test fails.

Planned files:

- crates/athena-core/src/scope_guard.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/design/scope-guard.md (modify)
- CHANGELOG.md (modify)

## The change

Base 025fa6162bcc → candidate b983739ac5c5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   5 +
 crates/athena-core/src/lib.rs         |   1 +
 crates/athena-core/src/scope_guard.rs | 496 ++++++++++++++++++++++++++++++++++
 docs/decisions.md                     |  11 +-
 docs/design/scope-guard.md            |  12 +-
 5 files changed, 519 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1262 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
