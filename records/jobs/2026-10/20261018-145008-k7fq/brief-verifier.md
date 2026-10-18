# Job 20261018-145008-k7fq: verify "Build docs/spec.toml from Olympus sections 1-6 with a checking test"

## Request (verbatim)

> try again on 20261018-133651-1ncn

## Reading (accepted by the user)

Retry of job 20261018-133651-1ncn with the accepted halved scope: docs/spec.toml covers only BRIEF.md "Project Olympus" sections 1 to 6 (vision, pantheon, feedback, example, two loops, R1 to R8), each requirement citing verbatim quotes and headings, checked by an xtask test. Audience is the agents that read the spec and Sri, who approves it. The verifier-timeout mechanism change is a separate job, so the spec is kept compact to be cheap to verify.

Assumptions:
- The new request, as typed, is to retry the spec job; the verifier-timeout redesign suggested by the introspection is not part of it.
- Scope stays halved as the user directed: Olympus sections 1 to 6 only; sections 7 to 10, governing philosophy and 2026-10-17 addenda follow in a second job.
- To keep verification inside the deadline, the spec is concise: one requirement per distinct idea, short quotes, merged duplicates, and no padding.
- The test lives in xtask/tests/spec.rs and uses the toml crate (bought, not built) added to xtask/Cargo.toml.
- Statements about who runs Athena or where are omitted, since xtask/tests/policy.rs forbids Athena files naming them.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes, meaning every source quote is a verbatim substring of BRIEF.md after collapsing whitespace, ids are unique, supersedes targets exist, and every requirement has at least one source.
2. [behavior] Run `cargo test -p xtask`: in-memory negative cases each make the checker fail for (a) a quote not in BRIEF.md, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
3. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one each for R1 to R8 and for Olympus sections 1 to 5 (Vision, Pantheon, Feedback, Example, lifecycle with the two loops).
4. [content] Each requirement in docs/spec.toml has an id, a plain cohesive statement, a kind (goal, requirement, constraint, acceptance), a status (active or superseded), and one or more sources (verbatim quote plus heading); duplicates are merged into one requirement with several sources.
5. [content] docs/spec.toml records at least one open question with both verbatim quotes where Sri's words conflict or leave a choice open, unresolved; if sections 1 to 6 contain none, the spec says so and the format and test still support the field.
6. [content] docs/spec.toml has no statement about who runs Athena or where; xtask/tests/policy.rs still passes.
7. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.
8. [content] CHANGELOG.md has an entry for the spec; docs/decisions.md records the spec format and the choice of TOML with alternatives (Markdown with ids, JSON, YAML), and notes that sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- docs/ontology.toml (modify)

## Out of scope

- Olympus sections 7 to 10, the governing philosophy and the 2026-10-17 addenda (second job)
- The Athena brief's earlier parts
- The requirements-engineer role
- Changing any reader or prompt to use the spec
- Changing the verifier's timeout or outcome mechanism

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Checker and tests are Rust in xtask.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the toml and serde crates for parsing; record the TOML choice in docs/decisions.md.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy -D warnings and fmt --check must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim quotes from BRIEF.md with headings.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only sections 1 to 6; no role or prompt changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test and negative cases rather than checking that files exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Spec and decision text are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the xtask spec test against the real docs/spec.toml and BRIEF.md. Run: `cargo test -p xtask --test spec`. Observes: `the_spec_matches_the_brief` passes with an empty list of problems.
  Fails without the behavior: If the checker did not verify quotes, ids, supersedes and sources, or the spec had a misquoted or sourceless requirement, the test `the_spec_matches_the_brief` fails.
- Criterion 2: Four in-memory tests mutate the real spec text: altered quote, duplicated id, bad supersedes, removed sources; each asserts the checker reports that specific problem. One further asserts the unmodified text yields none. Run: `cargo test -p xtask --test spec`. Observes: `a_quote_not_in_the_brief_is_flagged`, `a_duplicate_id_is_flagged`, `a_missing_supersedes_target_is_flagged`, `a_requirement_without_a_source_is_flagged` all pass.
  Fails without the behavior: Without each check, the matching negative test would find no problem and its assertion that the problem list names the fault would fail.
- Criterion 3: Group requirements by their sources' headings and assert at least one each for R1..R8 and sections 1-5 (Vision, Pantheon, Feedback, Example, lifecycle). Run: `cargo test -p xtask --test spec`. Observes: `every_requirement_area_has_a_requirement` passes; the counts are asserted nonzero.
  Fails without the behavior: If R1-R8 or any of sections 1-5 lacked a requirement, `every_requirement_area_has_a_requirement` fails naming the heading.
- Criterion 4: Typed deserialization with deny_unknown_fields plus enum checks; verifier also reads docs/spec.toml for plain statements and merged duplicates. Run: `cargo test -p xtask --test spec`. Observes: The test passes; the file's requirements each show id, statement, kind, status and one or more sources.
  Fails without the behavior: A requirement missing a field, with a kind or status outside the enums, would fail `the_spec_matches_the_brief` through serde errors and enum checks.
- Criterion 5: Read docs/spec.toml for [[open_question]] entries with two verbatim quotes, or the stated none-found reason; the test verifies quotes and that either one or the reason exists. Run: `cargo test -p xtask --test spec`. Observes: Test passes; the file shows the open question(s) unresolved, or the explicit none statement.
  Fails without the behavior: If the open-question field were not checked, `an_open_question_quote_not_in_the_brief_is_flagged` fails.
- Criterion 6: Run policy tests, which scan tracked files; the verifier also reads docs/spec.toml for such statements. Run: `cargo test -p xtask --test policy`. Observes: All policy tests pass.
  Fails without the behavior: If spec.toml named who runs Athena or where, `athena_refers_to_no_one_who_runs_it_or_where` fails.
- Criterion 7: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All three exit 0; this includes M24, so the new enforcement row's test names exist, and the ontology test accepts the new term.
  Fails without the behavior: A lint, format or test failure would show as a non-zero exit.
- Criterion 8: Read the diff for CHANGELOG.md under [Unreleased] / Added and docs/decisions.md D111 (case, decision, alternatives Markdown with ids, JSON, YAML, tradeoff, reversal cost, provenance) including the note that sections 7-10, the governing philosophy and the 2026-10-17 addenda follow in a second job. Run: `git diff`. Observes: Both entries are present with those contents.

Planned files:

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- docs/ontology.toml (modify)

## The change

Base ae8eb535e611 → candidate 7614b401f9d8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md        |   5 +
 docs/decisions.md   |  24 ++
 docs/enforcement.md |   1 +
 docs/ontology.toml  |   6 +-
 docs/spec.toml      | 680 ++++++++++++++++++++++++++++++++++++++++++++++++++++
 xtask/tests/spec.rs | 283 ++++++++++++++++++++++
 6 files changed, 997 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (pre-existing: this check already failed on the base commit)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
