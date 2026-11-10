# Job 20261029-180923-wx3a: verify "Give OLY-2-15 (Dionysus exploration) a bounded-experiment mechanism"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-2-15 a mechanism
> 
> Requirement OLY-2-15 in docs/spec.toml is active and has no mechanism.
> Requirement: Dionysus handles exploration: proposing unconventional solutions and bounded experiments.
> Owner: Dionysus
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-2-15
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-2-15 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core module, exploration.rs, that models a Dionysus bounded experiment (hypothesis, unconventional approach, hard bound) and refuses unbounded or empty ones. Then record it as OLY-2-15's mechanism and measure in docs/spec.toml, with a test that fails without it. Audience: the Hestia spec review and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, the allowed `status:` labels), crates/athena-core/src/hestia.rs (spec review, parse_spec, SPEC_FILE, and the test that parses include_str!(docs/spec.toml)), and crates/athena-core/src/proposals.rs (an existing record-backed proposal flow). No code mentions experiments yet. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: extends crates/athena-core/src/lib.rs (module list) and follows the style of crates/athena-core/src/manage.rs and crates/athena-core/src/proposals.rs; docs/spec.toml holds the requirement entry.
- Design call: no printed status label fits exploration, so the measure ends with `status: not shown` and names the test. Adding a status line is out of scope to keep the change small.
- Design call: a pure-code module, with no language-model role and no new crate (buy-first: nothing off the shelf models this). Not a new refusal rule on readings, so no refusal-rate criterion applies.
- Goal: Dionysus's exploration duty gets a checkable mechanism; the audience is the Hestia spec review and the verifier.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-2-15 has a mechanism naming crates/athena-core/src/exploration.rs (not "none"), keeps owner "Dionysus", and a measure ending with `status: not shown` that names the test (cargo test -p athena-core exploration). Check with Grep on docs/spec.toml.
2. [content] crates/athena-core/src/exploration.rs exists, is declared in crates/athena-core/src/lib.rs, and defines a bounded-experiment type (hypothesis, unconventional approach, bound, outcome) with a validate function.
3. [behavior] Run `cargo test -p athena-core exploration`: tests show that an experiment with no bound or an empty hypothesis is refused, that a bounded one is accepted, and that an experiment which exceeds its bound is reported as stopped. Each test fails if validate is stubbed to accept everything.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass. This includes the existing spec tests in hestia.rs and xtask that parse docs/spec.toml.

## Expected deliverables

- crates/athena-core/src/exploration.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- A new language-model Dionysus role or prompt
- A new `athena status` line
- Changes to other requirements in docs/spec.toml
- Changes to athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs the tests that exercise refusal and acceptance.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: No existing crate fits; plain Rust, recorded as a design call.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 23e19e7de9b4 → candidate a0237ca2cbdd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   4 ++
 crates/athena-core/src/exploration.rs | 107 ++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs         |   1 +
 docs/spec.toml                        |   4 +-
 4 files changed, 114 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-180923-wx3a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-180923-wx3a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42872638024 for a0237ca2cbddc04a9e50a36498d5e1db26653cca)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42872638024 for a0237ca2cbddc04a9e50a36498d5e1db26653cca)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42872638024 for a0237ca2cbddc04a9e50a36498d5e1db26653cca)
- ci-result: `not a check` passed (Check results for a0237ca2cbddc04a9e50a36498d5e1db26653cca come from the CI run https://github.com/doodla/athena/actions/runs/42872638024 (574s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass. This includes the existing spec tests in hestia.rs and xtask that parse docs/spec.toml.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
