# Job 20261010-154001-beq1: verify "List uncited interventions as open work in athena status"

## Request (verbatim)

> List interventions that call for follow-up as open work in athena status, each with a next command, until a later job cites them (m6, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

`athena status` counts interventions but never lists them as open work. The fix lists each intervention that no later job cites under open work, with a next command in `next`. The audience is a fresh-context user or agent who should not have to read the records to find follow-ups (m6 in the v0 acceptance review).

Assumptions:
- An intervention is 'cited' when a request job created after it names its record id in the request text. Records have no dedicated citing field, so the producer should check for a better existing link, such as the job or introspection fields, and use it if one exists.
- Every intervention without such a citation counts as needing follow-up. The producer may exclude cases the repository shows are already handled.
- The next command is `athena ask "<request that addresses intervention ID>"`. `athena ask` exists in the CLI, but the producer should confirm its argument form.
- The change is in `StatusView` and `status()` in crates/athena-core/src/status.rs, and in the text rendering in crates/athena-cli/src/display.rs. Interventions are listed in the status output, not added to the job `open` list.
- The goal is the acceptance review's m6 fix. The JSON status shape gains an additive field.
- The `tested` default needs a unit test for a cited intervention and an uncited one, plus a CLI integration test.

## Acceptance criteria

1. [behavior] Run `cargo test`. A new test records an intervention against a landed job and shows `athena status` listing it as open work, with its id, category and a next command. A second new test shows it disappears from open work once a later job's request cites its id.
2. [content] The status view has a distinct open-interventions list (id, what, category, next command). Its entries also appear in `next`, so a reader sees what to run without opening records.
3. [content] An intervention cited by a later job is not listed. The rule for 'cited' is documented in a doc comment and in docs where status is described.
4. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
5. [content] The existing counts under 'interventions' (by category and class) are unchanged. Status for a repository with no interventions shows no new open work.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- docs/records.md (modify)

## Out of scope

- Fixing the other findings in the acceptance review (m1–m5, m7 and later).
- Changing the intervention record schema or adding new record types.
- Automatically creating follow-up jobs.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that status lists and then drops the intervention, not that fields exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to status computation and rendering.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The status view derives open interventions from the records, not from a single instance.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the citation meaning and the next command from the repository and the review.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base cbd00c195c28 → candidate 2b96e997206a. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                             |  4 ++
 crates/athena-cli/src/display.rs         |  8 ++++
 crates/athena-cli/tests/introspection.rs | 35 ++++++++++++++
 crates/athena-cli/tests/lifecycle.rs     | 81 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs         | 55 +++++++++++++++++++++-
 crates/athena-data/src/records.rs        |  3 ++
 docs/records.md                          |  8 ++++
 7 files changed, 193 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (721 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
