# Job 20261024-232059-mymj: verify "Extend docs/spec.toml to BRIEF sections 7-10, philosophy and addenda"

## Request (verbatim)

> Build the second layer of the cohesive spec (decision 3C): extend docs/spec.toml, which covers BRIEF.md 'Project Olympus' sections 1 to 6, to sections 7 to 10, the governing philosophy and the addenda of 2026-10-17 to 2026-10-24, in the same format and checked by the same test.
> Acceptance:
> - Every new requirement cites verbatim quotes under their headings, and the xtask spec test passes.
> - Conflicts between Sri's words are recorded as open questions quoting both.
> - The boundary scan (xtask/tests/policy.rs) still passes.
> Files: docs/spec.toml, xtask/tests/spec.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Add sourced requirements (and open questions for conflicting words) to docs/spec.toml for Olympus sections 7 to 10, "Governing philosophy", and the addenda dated 2026-10-17 to 2026-10-24. Widen xtask/tests/spec.rs so its section cutter no longer stops at "## 7." and coverage covers the new parts; record the decision and a changelog line. Audience: Sri and the verifier checking quotes against BRIEF.md.

Assumptions:
- Goal: a cohesive, sourced spec of the whole Olympus brief; audience is Sri and the verifier, who check quotes against BRIEF.md.
- Addenda 2026-10-25 and 2026-10-26 are out of scope because the request names 2026-10-17 to 2026-10-24.
- The existing test's sections() function ends at `## 7.` and coverage_gaps only checks sections 1 to 5 and R1 to R8; both need extending in the same test file, and this is the required change to 'the same test'.
- Headings used must be unique within the Olympus-onward text; the brief has two `## Addenda` headings but the new spec cites only dated addendum headings.
- Quote placement: text under a heading runs only to the next heading of any level, so quotes must be cited under the nearest subheading (e.g. Phase N, Metric integrity).
- Decision numbering in docs/decisions.md follows the existing D-number format; the request's 'decision 3C' is cited by name in the new entry.
- Boundary scan in xtask/tests/policy.rs is unaffected as no governance files change; not read in detail.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask --test spec`: it passes, including the_real_spec_passes, so every new requirement and open-question side quotes BRIEF.md verbatim under a heading from sections 7 to 10, Governing philosophy, or the 2026-10-17 to 2026-10-24 addenda.
2. [behavior] Run `cargo test -p xtask --test policy`: the boundary scan still passes.
3. [content] xtask/tests/spec.rs no longer stops cutting sections at the `## 7.` line; it reads through the 2026-10-24 addenda and stops before the 2026-10-25 addendum, so no new requirement cites 2026-10-25 or later text. Coverage tests now also require at least one requirement under each of sections 7, 8, 9, 10, Governing philosophy, and each addendum dated 2026-10-17 to 2026-10-24 (checked by Grep of the test).
4. [behavior] The test file has in-memory negative cases for the new reach, run by `cargo test -p xtask --test spec`: a non-verbatim quote under a section 7+ heading fails, a missing section 8 requirement is reported as a coverage gap, and a heading dated after 2026-10-24 is rejected.
5. [content] Where Sri's words conflict across sections or addenda (for example the 2026-10-17 to 2026-10-24 addenda against earlier text, such as the history exemptions and trailer rules), docs/spec.toml has an [[open_question]] with two or more sides, each quoting both statements verbatim; the conflicting requirements are not silently resolved. Existing superseded/active status is used only where the brief says so.
6. [content] New requirement ids follow the existing OLY-N-NN pattern and existing entries for sections 1 to 6 are unchanged (git diff shows only additions there).
7. [content] docs/decisions.md gets an entry recording that the second layer (decision 3C) extends the spec, with the alternatives considered; CHANGELOG.md gets a matching line. No secrets or personal data are added.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Addenda dated 2026-10-25 and later
- Changes to sections 1 to 6 entries in docs/spec.toml
- Changes to BRIEF.md, athena.toml, defaults.toml or xtask/tests/policy.rs
- Resolving open questions; they are only recorded

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and the decision entry are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim quotes checked by the spec test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the spec and policy tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the quote-check test and negative cases rather than checking file existence.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data added.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the spec, its test, and two log files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Quote rules are enforced by the spec test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e26f9d393229 → candidate 4440e81a37da. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md        |   7 +
 docs/decisions.md   |  27 +++
 docs/spec.toml      | 666 +++++++++++++++++++++++++++++++++++++++++++++++++++-
 xtask/tests/spec.rs | 167 +++++++++++--
 4 files changed, 842 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1634 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
