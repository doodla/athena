# Job 20261029-130605-mbpm: verify "Cover every brief heading and addendum in docs/spec.toml and test it"

## Request (verbatim)

> Priority: urgent
> Finish moving the owner's brief (the brief file at the repository root) into docs/spec.toml, so the spec holds every requirement and the brief can be retired. After spec layer 2 the spec has 135 requirements but still nothing for the brief's opening goals (including 'gets measurably faster, cheaper, and more correct'), the Part 2 requirements A1 to A13 where not yet covered, and the dated addenda before 2026-10-17 and after 2026-10-24 (among them 2026-10-12 'Chunking and parallel work' and 2026-10-13 'Athena owns acceleration'). Add them in the same format, each with verbatim quotes under their headings, and extend the spec test so it fails when any heading or dated addendum in the brief has no requirement citing it. Record conflicts between the owner's words as open questions quoting both.
> Acceptance:
> - The spec test fails if a heading or dated addendum of the brief has no citing requirement, and passes on the updated spec.
> - The spec has requirements citing the 2026-10-12 and 2026-10-13 addenda and the opening goal about getting measurably faster.
> - The boundary scan and fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: docs/spec.toml, xtask/tests/spec.rs, CHANGELOG.md

## Reading (accepted by the user)

Reads the request as: add sourced requirements to docs/spec.toml for all brief headings not yet cited (opening goals, A1–A13 which sit under "Part 1" in BRIEF.md, the operating model and other pre-Olympus headings, and the addenda dated before 2026-10-17 or after 2026-10-24). Extend xtask/tests/spec.rs so it fails when any brief heading or dated addendum has no citing requirement; record owner conflicts as open questions quoting both sides. The audience is the owner and the verifier who retires the brief.

Assumptions:
- Goal: let the owner retire BRIEF.md because docs/spec.toml cites every heading; audience is the owner and the verifier running the spec test.
- The request's 'Part 2 requirements A1 to A13' are the headings '### A1.' to '### A13.' under '## Part 1: Athena' in BRIEF.md; I take the headings, not the Part number, as authoritative.
- 'Every heading' means every heading in BRIEF.md, including the pre-Olympus ones (Goal, O1–O5, Standards, Boundaries, S1–S7, D1–D3 and so on), plus Olympus subheadings and all dated addenda. Pure structural headings such as the title and 'Addenda' may be exempted by a short explicit list in the test; the producer decides which and writes the list down.
- The sections() helper in xtask/tests/spec.rs currently limits the citable text to Olympus through 2026-10-24 plus two pinned addenda. It must be widened to the whole brief so earlier and later headings can be cited, while keeping the existing negative tests (the 2026-10-25 'Evidence labels Jev' out-of-range test needs rewriting).
- BRIEF.md itself is not deleted in this job; retiring it is a later step, and the spec test still reads it.
- Existing requirement ids OLY-A-22 (2026-10-13 acceleration) and OLY-A-23 exist already. The 2026-10-13 addendum may already be cited, so the producer checks first and adds only what is missing. New ids avoid collisions with existing ones.
- Priority urgent is noted; I found no reason in the request that makes this cut waste.
- Code survey: not needed; this is a docs and test change that touches no crates/ code. The hestia spec review in crates/athena-core/src/hestia.rs reads docs/spec.toml, so the producer runs the workspace tests to catch any count or label assumption.

## Acceptance criteria

1. [behavior] `cargo test -p xtask --test spec` passes on the updated spec, and the new coverage test lists no gaps for any heading or dated addendum of BRIEF.md.
2. [behavior] A negative test in xtask/tests/spec.rs removes the requirements citing one heading (e.g. A6 or the 2026-10-12 'Chunking and parallel work' addendum) and asserts coverage reports exactly that heading as a gap; `cargo test -p xtask --test spec` runs it and passes.
3. [content] docs/spec.toml has requirements whose first source heading is '2026-10-12: Chunking and parallel work', '2026-10-13: Athena owns acceleration', and a heading under 'Goal' whose verbatim quote contains 'measurably faster, cheaper, and more correct'.
4. [content] docs/spec.toml has requirements citing each of A1 to A13 in BRIEF.md and each pre-2026-10-17 and post-2026-10-24 addendum, each with verbatim quotes under its heading and mechanism, owner and measure fields in the existing format.
5. [content] Where BRIEF.md has conflicting owner statements, docs/spec.toml has [[open_question]] entries with two or more sides quoting both; none silently resolves a conflict.
6. [content] CHANGELOG.md has a new entry describing the spec extension and the stricter coverage test.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (whole workspace, including any test of the spec review in crates/athena-core/src/hestia.rs) and the boundary scan all pass.
8. [content] The checked-in spec contains no secrets or personal data (public default); the 'at least 135 active requirements' floor in the spec test is raised to match the new count.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Deleting or editing BRIEF.md
- Changing athena.toml, defaults.toml or workflow files
- Implementing any mechanism the new requirements name; they record mechanism and status only
- Resolving conflicts between the owner's words

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the test failing on a missing heading and passing on the updated spec, not that files exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and the full test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A negative in-memory coverage test proves the new rule fires.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Requirements are written plainly with verbatim cited quotes.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the spec or changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the spec test and checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b424e639b623 → candidate 3049237795cd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   15 +
 crates/athena-core/src/hestia.rs |    2 +-
 docs/enforcement.md              |    2 +-
 docs/spec.toml                   | 1978 +++++++++++++++++++++++++++++++++++++-
 xtask/tests/spec.rs              |  290 +++---
 5 files changed, 2169 insertions(+), 118 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1665 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (whole workspace, including any test of the spec review in crates/athena-core/src/hestia.rs) and the boundary scan all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
