# Job 20261030-024655-bwng: verify "Give requirement OLY-B-A1-03 a pushback mechanism and measure"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A1-03 a mechanism
> 
> Requirement OLY-B-A1-03 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena pushes back with evidence when a request is wrong, costly or in tension with another, still meets it unless the change is agreed, and proposes its next big task when the study is done.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A1-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A1-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small, tested Rust module in athena-cli that turns evidence about a request (wrong, costly, in tension) into a pushback note while still meeting the request unless a change is agreed, and proposes a next big task when a study is done. Record it as the mechanism of OLY-B-A1-03 in docs/spec.toml with a measure ending `status: not shown` naming the test. Audience: Sri and the verifier reading the spec.

Assumptions:
- Code survey: read crates/athena-cli/src/strategy.rs (the OLY-B-A1-02 precedent: a module plus a test that parses docs/spec.toml and checks mechanism, owner and measure) and grepped crates/ for existing pushback code (none) and for `status: ` measures (crates/athena-core/src/status.rs, tests/buy_first.rs, which require a measure to end in a `athena status` label or `status: not shown`). No new record or data file is added under crates/, so check_record_file/AnyRecord round-trip is not touched.
- Existing code: extends crates/athena-cli/src/strategy.rs conventions and docs/spec.toml requirement OLY-B-A1-02; the new module follows the same test pattern.
- The mechanism is a pure function in a new crates/athena-cli/src/pushback.rs: given a request and evidence findings (wrong, costly, tension), it returns a Pushback with the evidence and an alternative, and always keeps the request in effect (meets it) until an agreement flag is set; a second function proposes a next big task with reasons when a study is done. It is not wired to refuse anything, so no refusal-rate criterion applies.
- The measure ends `status: not shown` and names the test, as OLY-B-A1-02 does.
- Goal and audience: Sri and the verifier check that the requirement has a real mechanism, owner and measure.
- No governance files (athena.toml, defaults.toml, workflows) change.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-A1-03 has mechanism naming crates/athena-cli/src/pushback.rs (a file that exists), owner "Athena", and a measure that names the test and ends with `status: not shown`.
2. [behavior] `cargo test -p athena-cli pushback` passes: tests show evidence yields a pushback carrying the evidence and an alternative, the request is still met unless the change is agreed, and a finished study yields a next-big-task proposal with reasons; the tests fail if the module's logic is removed.
3. [behavior] A test in pushback.rs parses docs/spec.toml and asserts OLY-B-A1-03's mechanism, owner and measure (as strategy_view_is_the_mechanism_of_oly_b_a1_02 does); `cargo test -p athena-cli` passes.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass across the workspace.
5. [content] CHANGELOG.md gains an entry for the change, and no other requirement in docs/spec.toml is modified.

## Expected deliverables

- crates/athena-cli/src/pushback.rs (create)
- docs/spec.toml (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Wiring pushback into the reader/planner prompts or changing any prompt file
- Changing other requirements in docs/spec.toml
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; fmt, clippy, test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the pushback tests rather than checking file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module plus the spec edit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bb831c416e42 → candidate 1063ebce621a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   1 +
 crates/athena-cli/src/lib.rs      |   1 +
 crates/athena-cli/src/pushback.rs | 229 ++++++++++++++++++++++++++++++++++++++
 docs/spec.toml                    |   4 +-
 4 files changed, 233 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-024655-bwng/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-024655-bwng/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43303983517 for 1063ebce621ab049e4fef88f1278e906e1f1f105)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43303983517 for 1063ebce621ab049e4fef88f1278e906e1f1f105)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43303983517 for 1063ebce621ab049e4fef88f1278e906e1f1f105)
- ci-result: `not a check` passed (Check results for 1063ebce621ab049e4fef88f1278e906e1f1f105 come from the CI run https://github.com/doodla/athena/actions/runs/43303983517 (890s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass across the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
