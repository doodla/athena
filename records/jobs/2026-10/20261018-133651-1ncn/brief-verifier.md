# Job 20261018-133651-1ncn: verify "Build docs/spec.toml for Olympus sections 1-6 with a checking test"

## Request (verbatim)

> try again on 20261018-120736-90w3

## Reading (accepted by the user)

Retry of job 20261018-120736-90w3, halved per the user: build docs/spec.toml from BRIEF.md "Project Olympus" sections 1 to 6 only (vision, pantheon, feedback, example, lifecycle/two loops, R1 to R8), each requirement citing verbatim quotes and headings. An xtask test checks the file. Audience: agents that read the spec, and Sri, who approves it. The verifier-timeout fix is not part of this job.

Assumptions:
- The user's reply to the earlier reading takes precedence: this job is the Olympus spec layer limited to sections 1 to 6, not the verifier-timeout mechanism.
- The test lives in xtask/tests/spec.rs and uses a TOML parser dependency in xtask (toml crate, bought not built) and serde.
- Section headings cited are the BRIEF.md headings such as '1. Vision' and 'R1 — Causal investigation'.
- The test's coverage count is restricted to sections 1 to 6 so it will pass now; the second job extends it to phases, section 10 and decisions 3C, 4A, 5A, 7A.
- The earlier deliverable docs/ontology.toml and docs/enforcement.md changes are kept only if the test needs registering there.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes, meaning every source quote is a verbatim substring of BRIEF.md after collapsing whitespace, ids are unique, supersedes targets exist, and every requirement has at least one source.
2. [behavior] Run `cargo test -p xtask`: in-memory negative cases each make the checker fail for (a) a quote not in BRIEF.md, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
3. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one each for R1 to R8 (section 6) and for sections 1 to 5 (Vision, Pantheon, Feedback, Example, lifecycle with the two loops).
4. [content] Each requirement in docs/spec.toml has an id, a plain cohesive statement, a kind (goal, requirement, constraint, acceptance), a status (active or superseded), and one or more sources (verbatim quote plus heading). Duplicates are merged into one requirement with several sources. Requirement-bearing sentences in Olympus sections 1 to 6 are covered.
5. [content] docs/spec.toml records at least one open question with both verbatim quotes where Sri's words conflict or leave a choice open, recorded and not resolved. If sections 1 to 6 contain none, the verifier can see the question field is supported by the format and the test, and the spec says so.
6. [content] docs/spec.toml has no statement about who runs Athena or where; xtask/tests/policy.rs still passes under `cargo test`.
7. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.
8. [content] CHANGELOG.md has an entry for the spec; docs/decisions.md records the spec format and the choice of TOML with alternatives (Markdown with ids, JSON, YAML), and notes that sections 7 to 10, governing philosophy and 2026-10-17 addenda follow in a second job.

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
- The verifier timeout, unjudged outcome and retry mechanism
- Changes to athena.toml, defaults.toml, or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Checker test is Rust in xtask.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the toml and serde crates instead of a hand-written parser; record in decisions.md.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Positive and negative tests; cargo test, clippy, fmt clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Halved to sections 1 to 6 per the user.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain statements and docs for an outside reader.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test and check the coverage by headings.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verification by running cargo test, clippy and fmt.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim BRIEF.md quotes.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data, and no owner-side details in the spec.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Real-file test runs check on docs/spec.toml and BRIEF.md. Run: `cargo test -p xtask --test spec`. Observes: the_spec_is_valid_against_the_brief passes: quotes are whitespace-collapsed substrings, ids unique, supersedes targets exist, each requirement has a source.
  Fails without the behavior: A bad quote, duplicate id, dangling supersedes, unsourced requirement or missing file fails it.
- Criterion 2: Four in-memory mutation tests of a valid baseline spec, each asserting a specific error from check; a baseline test shows the unmutated spec passes. Run: `cargo test -p xtask --test spec`. Observes: a_quote_not_in_the_brief_is_flagged, a_duplicate_id_is_flagged, a_missing_supersedes_target_is_flagged, a_requirement_without_a_source_is_flagged pass.
  Fails without the behavior: If check skipped any of the four checks, the matching test would see Ok and fail.
- Criterion 3: sections_one_to_six_are_covered groups requirements by source headings (validated against BRIEF.md Olympus range) and requires at least one for each of 13 groups: sections 1-5 and R1-R8; a negative test with a missing group asserts it is reported by name. Run: `cargo test -p xtask --test spec`. Observes: Test passes; on failure lists uncovered groups.
  Fails without the behavior: A missing group such as R6 fails and is named.
- Criterion 4: Schema check enforces kind in goal/requirement/constraint/acceptance, status in active/superseded, non-empty statement, and quote+heading per source; reviewer reads docs/spec.toml for coverage of the Vision list, Pantheon table, feedback sub-sections, Stages A-F, 5.1 and R1-R8 with duplicates merged. Run: `cargo test -p xtask --test spec`. Observes: Passes with schema enforced; file read shows merged multi-source requirements.
  Fails without the behavior: An invalid kind, status or empty statement fails the schema test.
- Criterion 5: If sections 1-6 contain a conflict or open choice, spec.toml has a [[question]] with both verbatim quotes; otherwise a header comment says none was found. In-memory tests show a question with two quotes parses and one with a bad quote is flagged. Run: `cargo test -p xtask --test spec`. Observes: Question tests pass; question quotes validated like requirement quotes.
  Fails without the behavior: A question quote absent from BRIEF.md fails check.
- Criterion 6: Two checks: spec.rs scans docs/spec.toml text for the copied M19 word list (independent of git tracking), with an in-memory test that a planted banned word is flagged; and the policy test runs. Reader confirms no statement on who runs Athena or where. Note policy.rs only sees the file once tracked, so the final Athena run on the committed tree confirms it. Run: `cargo test -p xtask --test spec`. Observes: The banned-word scan test and a_banned_word_in_a_quote_is_flagged pass; then cargo test -p xtask --test policy also passes.
  Fails without the behavior: A quote like the full Hermes row with 'handoffs' would fail the spec.rs scan even uncommitted.
- Criterion 7: Run the three checks. M24 reads only tracked .rs files, so the M33 row's test names are found only once xtask/tests/spec.rs is tracked; Athena tracks it, the producer does not run git add. The producer keeps the M33 row and does not drop it to make an untracked run pass; the final criterion 7 run is on the committed tree. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy clean; cargo fmt --check and cargo test --locked also pass on the committed tree, including M24.
  Fails without the behavior: A lint, format difference or misnamed test in the enforcement map fails.
- Criterion 8: Inspect the diff of CHANGELOG.md and docs/decisions.md. Run: `git diff`. Observes: CHANGELOG.md has a spec entry; docs/decisions.md has D111 with format, TOML choice, rejected Markdown with ids, JSON and YAML, and note that sections 7-10, governing philosophy and 2026-10-17 addenda follow in a second job.

Planned files:

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)

## The change

Base 42da96d97e91 → candidate 2cfceb0a5cf5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md        |   5 +
 docs/decisions.md   |  26 ++
 docs/enforcement.md |   1 +
 docs/spec.toml      | 794 ++++++++++++++++++++++++++++++++++++++++++++++++++++
 xtask/tests/spec.rs | 477 +++++++++++++++++++++++++++++++
 5 files changed, 1303 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (980 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "Run `cargo test -p xtask`: a test parses docs/spec.toml and passes, meaning every source quote is a verbatim substring of BRIEF.md after collapsing whitespace, ids are unique, supersedes targets exist, and every requirement has at least one source."; "Run `cargo test -p xtask`: in-memory negative cases each make the checker fail for (a) a quote not in BRIEF.md, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source."; "docs/spec.toml has no statement about who runs Athena or where; xtask/tests/policy.rs still passes under `cargo test`."; "Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
