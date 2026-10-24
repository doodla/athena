# Job 20261024-025358-kb23: verify "Build docs/spec.toml for Olympus sections 1-6 with an xtask test"

## Request (verbatim)

> Build the first layer of the cohesive spec again (decision 3C, BRIEF.md addendum '2026-10-17: Olympus'): docs/spec.toml covering BRIEF.md 'Project Olympus' sections 1 to 6 (vision, pantheon, feedback, the example, the two loops, R1 to R8), with an xtask test. Each requirement has an id, a plain cohesive statement, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), and one or more sources, each a verbatim quote with its heading; duplicates merge into one requirement with several sources; a conflict or open choice in Sri's words is recorded as an open question quoting both, never resolved. The test fails on a quote that is not a verbatim substring of BRIEF.md (whitespace collapsed), a duplicate id, a supersedes target that does not exist, or a requirement without a source, with in-memory negative cases, and finds at least one requirement for each of R1 to R8 and sections 1 to 5. No statement about who runs Athena or where (xtask/tests/policy.rs). Record the format and the choice of TOML in docs/decisions.md; sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job. Job 20261018-145008-k7fq built this once, but it reached main only through a hand merge that was reverted (2a86df5), so it is built again here.

## Reading (accepted by the user)

Rebuild the first layer of the cohesive spec (decision 3C): docs/spec.toml turns BRIEF.md 'Project Olympus' sections 1–6 (R1–R8 included) into sourced requirements, and xtask/tests/spec.rs checks it against BRIEF.md. The audience is Sri and the requirements-engineer role, who need a spec whose every line can be traced to his words by a test.

Assumptions:
- The earlier job's one real gap, a quote checked against the whole brief rather than under its heading, is fixed here.
- toml and serde are already xtask dev-dependencies, so no Cargo change is needed.
- The failing athena-data test (reading_inputs_digest_is_unchanged_by_the_job_plan_section) predates this work and is out of scope.
- The test reads BRIEF.md from the repository root through CARGO_MANIFEST_DIR, as policy.rs does.
- The Olympus brief text is quoted word for word in BRIEF.md after line 550. Quotes come from there, and headings are those listed there (## 1. Vision through ### R8).

## Acceptance criteria

1. [content] docs/spec.toml has, for each requirement, an id, a plain cohesive statement, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), and one or more sources, each a verbatim quote plus its BRIEF.md heading. It covers Olympus sections 1 to 6 and has at least one requirement for each of R1 to R8 and for sections 1 to 5.
2. [content] Duplicates across the brief are merged into one requirement with several sources. Any conflict or open choice in Sri's words is recorded as an open question quoting both sides, with no resolution.
3. [content] docs/decisions.md records the spec format and the choice of TOML, with alternatives. It notes that sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job.
4. [content] No statement in the spec or the new test says who runs Athena or where, so xtask/tests/policy.rs still passes. No governance files change.
5. [behavior] Run `cargo test -p xtask --test spec`. It passes on the real spec. Each in-memory negative case makes the checker fail: a quote that is not a verbatim substring of BRIEF.md after whitespace collapse, a duplicate id, a supersedes target that does not exist, and a requirement with no source. A further test fails if any of R1 to R8 or sections 1 to 5 has no requirement.
6. [behavior] The checker also confirms each quote sits under its stated heading, not just somewhere in BRIEF.md. A negative case pairs a real quote with the wrong heading and fails. Run `cargo test -p xtask`.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. `cargo test -p xtask --test policy` still passes.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)

## Out of scope

- Sections 7 to 10, the governing philosophy and the addenda, which follow in a second job
- Fixing the unrelated failing athena-data digest test
- Any change to BRIEF.md
- A shared vocabulary or ontology
- A requirements-engineer role

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The xtask test has negative cases and a coverage check, and cargo test and clippy must stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the checker against the real spec and against bad specs built in memory.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and decision text are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim quotes from BRIEF.md.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check is a Rust test, and the data is TOML.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that covers only sections 1 to 6.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The spec carries no personal data or secrets.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Verbatim sourcing is enforced by the test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the spec tests on the real spec and read docs/spec.toml for the fields. Run: `cargo test -p xtask --test spec`. Observes: the_real_spec_passes and the coverage test pass; serde enums reject any kind outside goal/requirement/constraint/acceptance and any status outside active/superseded; each requirement has id, statement, sources with heading and quote; R1 to R8 and sections 1 to 5 each have a requirement.
  Fails without the behavior: If coverage or the field checks were missing, a spec lacking a field or an R/section would still pass; the coverage negative and deny_unknown_fields/enum parsing would fail.
- Criterion 2: Read docs/spec.toml for merged duplicates and open questions; run the open-question side negative test. Run: `cargo test -p xtask --test spec`. Observes: Duplicates (e.g. independent evaluation in Stage E and R4; roles measured by value in section 2 and R8) are single requirements with several sources; any open question quotes both sides with no resolution field; open_question_sides_are_checked_like_sources fails a non-verbatim side.
  Fails without the behavior: If open-question sides were not checked, open_question_sides_are_checked_like_sources would see no error and fail.
- Criterion 3: Read the D114 diff. Run: `git diff docs/decisions.md`. Observes: D114 names the spec format, TOML, alternatives (YAML, JSON, Markdown) and notes sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job.
- Criterion 4: Run the policy test and list changed files. Run: `cargo test -p xtask --test policy`. Observes: Policy tests pass; git status shows only docs/spec.toml, xtask/tests/spec.rs, docs/decisions.md, docs/enforcement.md, CHANGELOG.md; no governance file.
  Fails without the behavior: A full Hermes-row quote or a banned word in a statement or the test source makes athena_refers_to_no_one_who_runs_it_or_where fail.
- Criterion 5: Run the spec tests; negatives mutate a clone of the parsed real spec in memory. Run: `cargo test -p xtask --test spec`. Observes: Real spec yields no errors; each mutation (changed quote, repeated id, unknown supersedes, emptied sources, dropped R3/section 2 requirements) yields its specific error; whitespace-different line breaks still pass.
  Fails without the behavior: Without quote, id, supersedes, source or coverage checks, a_quote_that_is_not_in_the_brief_fails, a_duplicate_id_fails, a_dangling_supersedes_fails, a_requirement_without_a_source_fails and a_missing_requirement_for_a_section_or_r_fails would find no error and fail.
- Criterion 6: Run the whole xtask suite; the wrong-heading negative pairs a real quote with another real heading, and a subsection quote with its parent. Run: `cargo test -p xtask`. Observes: Checker returns a heading error for each; real spec still passes; other xtask tests pass.
  Fails without the behavior: If the checker searched the whole brief, a_real_quote_under_the_wrong_heading_fails would get no error and fail.
- Criterion 7: Run clippy with warnings denied, then fmt check, then the policy test. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy exits clean on xtask/tests/spec.rs; cargo fmt --all --check exits clean; cargo test -p xtask --test policy passes including M24 (every test named in enforcement.md exists as a fn).
  Fails without the behavior: An unformatted or lint-failing spec.rs makes fmt --check or clippy exit non-zero; an M33 row naming a test that does not exist makes policy M24 fail.

Planned files:

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)

## The change

Base 6a5f9bba3b1b → candidate 4e3cfb114842. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md        |   6 +
 docs/decisions.md   |  26 ++
 docs/enforcement.md |   1 +
 docs/spec.toml      | 859 ++++++++++++++++++++++++++++++++++++++++++++++++++++
 xtask/tests/spec.rs | 406 +++++++++++++++++++++++++
 5 files changed, 1298 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1009 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
