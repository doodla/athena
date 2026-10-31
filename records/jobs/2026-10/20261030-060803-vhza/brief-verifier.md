# Job 20261030-060803-vhza: verify "Find why the steward's decisions source is empty and fix or document it"

## Request (verbatim)

> Priority: normal
> 
> Make the decisions source readable to the steward
> 
> The steward could not read decisions: Source 'decisions' is empty in this copy: records/decision-states and records/default-expectations contain no files, though introspection 20261029-135005-vb9w cites recorded decision D84. Find out why, and make it readable, or record why it cannot be.
> 
> Acceptance:
> - The cause of the failure to read decisions is found and written down
> - decisions is readable by the steward, or the notes say why it cannot be
> - fmt, clippy and test pass
> 
> Files: records/decision-states, records/default-expectations

## Reading (accepted by the user)

The steward's `decisions` source (defined in crates/athena-core/src/steward.rs) reported empty. The job finds the cause, writes it down, and makes decisions readable to the steward, or records why it cannot be. Audience: the steward and the maintainer reading its introspections.

Assumptions:
- Code survey: crates/athena-core/src/steward.rs (SOURCES lists `decisions` with two record directories; GitView::workspace writes every file from RecordSet::load, then creates any missing source directory as empty), crates/athena-gate/src/records.rs (RecordSet::load copies every blob under records/ at HEAD after check_record_file), crates/athena-data/src/records.rs (AnyRecord paths for decision-state and default-expectations records; round-trip tests around lines 3372 and 3405), crates/athena-core/src/store.rs (Records::load). Order: load records at HEAD, write files, then mkdir missing source dirs, so an empty source means nothing is committed under those paths at HEAD, not a copy bug.
- Those two record directories do not exist in the snapshot I read (records/ is hidden from readers by design), so I could not confirm directly that no decision records are committed. Working hypothesis: no decision-state or default-expectations records are committed at the base commit (they are written but not landed, or D84 lives in docs/design only). The producer must verify this with git log/show on the record paths and with Grep over crates/ for the writers (Athena::record_default_expectations, decision-state writer) before choosing a fix.
- If the cause is a missing writer or an unlanded record, the fix is in code (steward workspace or the writer). If decisions are legitimately empty, the fix is to make the steward's empty-source message say so and to record that in docs. No new record file is added under crates/, so no AnyRecord change is expected.
- The user's stated outcome is the goal: reading decisions works or the notes say why not. The cause note goes in CHANGELOG.md and, if useful, docs/records.md.

## Acceptance criteria

1. [content] The change writes down the cause of the empty `decisions` source in a committed file (CHANGELOG.md or docs/records.md), citing the code path (crates/athena-core/src/steward.rs and crates/athena-gate/src/records.rs) that produces the empty directory. A reader can check this with git diff.
2. [content] Either the diff changes crates/athena-core/src/steward.rs (or the record writer) so the steward's workspace holds decision records when they exist at HEAD, with a test in the same crate that builds a view containing a decision-state record and asserts the file appears under the decisions source paths, or the committed notes state why decisions cannot be read and what would change that.
3. [behavior] A test that the decisions source is populated when a decision-state record exists in the view and stays an empty directory when none exists; run `cargo test -p athena-core steward` and it passes.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the change.
5. [content] No file under records/ is edited, and no governance file (athena.toml, defaults.toml) is changed.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)
- docs/records.md (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-gate/src/records.rs (modify)

## Out of scope

- Editing or fabricating files under records/
- Changing athena.toml or defaults.toml
- Redesigning the other steward sources (jobs, events, introspections, class-registry, git-log, code)
- Changing the decision-state or default-expectations record schemas

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Any code change is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A test covers the decisions source behavior; cargo test and clippy must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The job finds why the source is empty rather than only silencing the message.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs a test that shows the decisions source populated, not just a wiring check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the steward being able to read decisions, or a recorded reason it cannot.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier runs fmt, clippy and test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the decisions source.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ddbae007827f → candidate 35d3ed48ff7f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  8 +++++
 crates/athena-core/src/steward.rs | 70 ++++++++++++++++++++++++++++++++++++++-
 docs/records.md                   | 13 ++++++++
 3 files changed, 90 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1858 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the change.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
