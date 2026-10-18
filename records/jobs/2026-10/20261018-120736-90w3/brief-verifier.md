# Job 20261018-120736-90w3: verify "Build the Olympus spec layer in docs/spec.toml with a checking test"

## Request (verbatim)

> try again on 20261017-205600-a37m

## Reading (accepted by the user)

Retry: build docs/spec.toml, the cohesive requirements layer from the Project Olympus part of BRIEF.md and the 2026-10-17 addenda, with verbatim cited sources and open questions, plus an xtask test that checks it. The audience is the agents that will read the spec and Sri, who approves it. The reading is unchanged from the accepted one.

Assumptions:
- Retry of rejected job 20261017-205600-a37m with the same scope; the earlier accepted reading stands.
- Audience: agents that will read the spec, and Sri who approves it.
- The earlier rejection reason is unrecorded; likely causes are quote mismatches against BRIEF.md, incomplete coverage, or a policy.rs leak, so the producer should verify each quote and run the whole workspace tests.
- The checker lives in xtask/tests/spec.rs and may need a toml dependency (buy-first) in xtask/Cargo.toml.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes; in-memory negative cases each fail the checker for (a) a quote not a verbatim substring of BRIEF.md after collapsing whitespace, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
2. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one for each of R1–R8, Phases 1–6 with their acceptance, section 10 (Final system acceptance), and 2026-10-17 decisions 3C, 4A, 5A and 7A.
3. [content] Each requirement in docs/spec.toml has an id, a plain statement, a kind (goal, requirement, constraint, acceptance), a status (active or superseded), and one or more sources (verbatim quote plus heading). Duplicates across sections are merged into one requirement with several sources. Requirement-bearing sentences in Olympus sections 1–10, the governing philosophy and the 2026-10-17 addenda are covered.
4. [content] docs/spec.toml records at least one open question with both verbatim quotes where Sri's words conflict or leave a choice open; it is recorded, not resolved.
5. [content] docs/spec.toml has no statement about who runs Athena or where; `cargo test -p xtask` shows xtask/tests/policy.rs still passes.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.
7. [content] CHANGELOG.md has an entry for the spec; docs/decisions.md records the spec format and the choice of TOML with alternatives (Markdown with ids, JSON, YAML). No earlier Athena brief parts are specified; no reader or prompt is changed.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/ontology.toml (modify)

## Out of scope

- The Athena brief's earlier parts (a later layer)
- The requirements-engineer role
- Changing any reader or prompt to use the spec
- Owner/operator-side statements about who runs Athena or where

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: xtask test with negative cases; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check quotes, counts and negative cases, not just file existence.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use a maintained toml crate for parsing rather than hand-rolling.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Spec and decision record written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No owner-side or personal data in the spec.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the Olympus layer; no reader or prompt changes.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration tests in xtask/tests/spec.rs: the real spec yields no problems; four in-memory mutations each assert a specific problem string naming the planted cause/id, and the same test asserts the unmutated spec is clean. Run: `cargo test -p xtask`. Observes: All pass; mutations yield 'quote not found', 'duplicate id <id>', 'supersedes <id> not found', 'no sources' respectively.
  Fails without the behavior: If the checker skipped whitespace-collapsed containment, id uniqueness, supersedes existence, or non-empty sources, the matching negative test would find no such string and fail.
- Criterion 2: Coverage test groups requirements by source heading (with item label for addenda) and requires ≥1 for R1–R8, Phases 1–6 and their acceptance, '10. Final system acceptance', 3C, 4A, 5A, 7A; a second case removes one group in memory and expects the missing key named. Run: `cargo test -p xtask`. Observes: Passes on the real spec; the removal case reports the missing heading.
  Fails without the behavior: A missing requirement for any required heading makes the coverage test fail naming it.
- Criterion 3: Schema test deserialises with serde deny_unknown_fields, validates kind/status enums and non-empty id/statement/sources/heading/quote; coverage also requires headings for vision, governing philosophy, sections 1–9 and the addenda. Producer's diff review checks duplicates merged into multi-source requirements. Run: `cargo test -p xtask`. Observes: All requirements validate; every section heading appears among sources.
  Fails without the behavior: A missing field, bad enum value or missing source fails the schema test; an uncovered section fails coverage.
- Criterion 4: Open-question test requires ≥1 [[open_question]], each with ≥2 sources verbatim in BRIEF.md after the Olympus line, and no resolution field. Run: `cargo test -p xtask`. Observes: Passes; the question carries both quotes.
  Fails without the behavior: No open question, fewer than two sources, or a quote not in BRIEF.md fails the test.
- Criterion 5: The new spec.rs boundary test reads docs/spec.toml directly (independent of git tracking) and checks boundary_words()/boundary_fragments(); its negative plants a banned word in an in-memory copy and expects a flag. The unchanged policy.rs tests also run. Run: `cargo test -p xtask`. Observes: Boundary test and policy.rs tests pass; the planted-word case is flagged.
  Fails without the behavior: A banned word or fragment in docs/spec.toml fails the direct boundary test even when untracked.
- Criterion 6: Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test; includes enforcement-map and ontology checks. Run: `cargo test`. Observes: All exit zero.
  Fails without the behavior: A format, lint or test failure gives a non-zero exit.
- Criterion 7: Read the diff for CHANGELOG.md entry, D111 in docs/decisions.md (format, TOML choice, alternatives Markdown with ids/JSON/YAML, exclusion of 2A and brain addendum), enforcement row, ontology term; confirm nothing under crates/, no prompt or reader changed. Run: `git diff`. Observes: Only deliverable files changed; spec covers Olympus layer only.

Planned files:

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- docs/ontology.toml (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)

## The change

Base a0421c951aa9 → candidate 02608a7e3723. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   4 +
 crates/athena-cli/tests/ontology.rs |   3 +-
 docs/decisions.md                   |  32 ++
 docs/enforcement.md                 |   1 +
 docs/ontology.toml                  |   9 +
 docs/spec.toml                      | 912 ++++++++++++++++++++++++++++++++++++
 xtask/tests/spec.rs                 | 549 ++++++++++++++++++++++
 7 files changed, 1509 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (979 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "docs/spec.toml has no statement about who runs Athena or where; `cargo test -p xtask` shows xtask/tests/policy.rs still passes."; "Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
