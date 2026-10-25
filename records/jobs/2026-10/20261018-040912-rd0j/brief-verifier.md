# Job 20261018-040912-rd0j: verify "Turn job planning on in athena.toml and record the decision"

## Request (verbatim)

> Build job 5b of docs/design/job-planning.md (revision 2, section 8): turn job planning on. This changes athena.toml, a governance file: list it as a deliverable and name it in a criterion. Athena has been rebuilt from a main that holds jobs 1 to 5a, so the installed binary knows the [job_plan] schema.
> Plan: (1) In athena.toml add [job_plan] with enabled = true and the caps and timeouts from the design (planner_usd 0.75, critic_usd 0.75, 600 second timeouts, max_rounds 3). (2) In crates/athena-cli/tests/common/mod.rs, the test config drops [job_plan], so existing tests keep today's path. (3) In docs/decisions.md, record the decision to turn planning on: the alternatives in the design's section 7, the measurement and keep rule in section 9 (landed share of dispatched change jobs, baseline 64%; hold through 30 planned jobs unless it falls; decide at 100 jobs on 72% and a $2.50 cost ceiling per landed job), and the reversal (set enabled = false).
> Acceptance:
> - athena.toml has [job_plan] enabled = true with those values, and cargo test's repo-config test parses it.
> - cargo test: every existing test passes with the test config's [job_plan] dropped.
> - docs/decisions.md has the entry with alternatives, the keep rule with its numbers, and the reversal.
> - cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: changing the stage itself.

## Reading (accepted by the user)

Job 5b of docs/design/job-planning.md: add [job_plan] (enabled, 0.75 caps, 600 s timeouts, max_rounds 3) to athena.toml, make the CLI test config strip that section so existing tests keep the unplanned path, and log the decision (alternatives, keep rule, reversal) in docs/decisions.md. The audience is the owner and the verifier, who check the config parses and the tests still pass.

Assumptions:
- Section numbers and figures are taken from the request; I did not read docs/design/job-planning.md, and the producer must check it.
- athena.toml has no [job_plan] now; common/mod.rs test_config builds from it by slicing, so the producer must strip the new section there.
- The next decision number follows D109 in docs/decisions.md.
- The installed binary reads [job_plan], so it accepts the new section.

## Acceptance criteria

1. [content] athena.toml has a [job_plan] section with enabled = true, planner_usd = 0.75, critic_usd = 0.75, planner_timeout_s = 600, critic_timeout_s = 600 and max_rounds = 3 (no unknown fields; the schema is deny_unknown_fields).
2. [behavior] Run `cargo test --locked`: the repo-config test in athena-data parses athena.toml with [job_plan] enabled, and every existing test passes, including crates/athena-cli/tests, with the test config's [job_plan] dropped.
3. [content] crates/athena-cli/tests/common/mod.rs test_config() removes the [job_plan] section from the athena.toml text it includes, so tests that append their own [job_plan] (job_plan.rs) still work and others take the unplanned path. It works wherever the section sits in athena.toml.
4. [content] docs/decisions.md has a new numbered entry that turns planning on: the alternatives from section 7 of docs/design/job-planning.md, the keep rule from section 9 (metric: landed share of dispatched change jobs, baseline 64%; hold through 30 planned jobs unless it falls; decide at 100 jobs on 72% and a $2.50 cost ceiling per landed job), and the reversal (set enabled = false). It cites the design.
5. [content] CHANGELOG.md has an entry for turning job planning on, naming athena.toml.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both pass. Run `cargo test --locked` including xtask's boundary test; it passes.
7. [content] The planning stage's code is unchanged: no file under crates/ other than the test helper in athena-cli/tests/common/mod.rs differs.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/common/mod.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Changing the planner, critic or planning stage code
- Changing the thresholds or measurement design in the design doc
- Evaluating the keep rule: no jobs are measured in this change

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test criterion checks that the config parses and that existing tests pass without the section.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision entry must cite design sections and give the numbers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config change with its test helper and docs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The repo-config test enforces that athena.toml parses.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 8c9b190ec3ab → candidate 93804b9efc3c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  4 +++
 athena.toml                             | 10 +++++++
 crates/athena-cli/tests/common/mod.rs   | 20 ++++++++++++-
 crates/athena-core/src/run_pin_tests.rs |  7 ++++-
 crates/athena-data/src/config.rs        | 53 ++++++++++++++++++++++++++++++---
 docs/decisions.md                       | 27 +++++++++++++++++
 docs/design/job-planning.md             |  2 ++
 7 files changed, 117 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (930 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: acceptance criteria not met or not reported: "The planning stage's code is unchanged: no file under crates/ other than the test helper in athena-cli/tests/common/mod.rs differs.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
