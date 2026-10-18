# Job 20261018-045437-7z81: verify "Turn job planning on in athena.toml and record the decision"

## Request (verbatim)

> try again on 20261018-040912-rd0j

## Reading (accepted by the user)

Retry of job 5b: enable [job_plan] in athena.toml with the design's caps, and record the decision in docs/decisions.md. Per the user's reply, test-only edits are allowed in athena-cli's test helper, athena-data config tests and run_pin_tests so they strip the section; every test edit is noted in the design doc.

Assumptions:
- Goal: turn job planning on in this repo's config; audience is the owner and the verifier, who check the config parses and nothing regresses.
- The user's reply to the earlier question takes precedence: test-only edits in config.rs and run_pin_tests.rs are allowed.
- Planning stage code and config schema stay unchanged (out of scope per the original request).
- Decision entry number follows the last in docs/decisions.md (earlier attempt used D110).

## Acceptance criteria

1. [content] athena.toml has a [job_plan] section with enabled = true, planner_usd = 0.75, critic_usd = 0.75, planner_timeout_s = 600, critic_timeout_s = 600, max_rounds = 3, and no unknown fields.
2. [behavior] Run `cargo test --locked`: all tests pass, including the repo-config test parsing athena.toml with planning on, crates/athena-cli/tests, run_pin_tests, the athena-data config tests, and xtask's boundary test.
3. [content] Only test code changed under crates/: crates/athena-cli/tests/common/mod.rs, test code in crates/athena-data/src/config.rs, and test code in crates/athena-core/src/run_pin_tests.rs. No non-test code (planning stage, config schema) changed. The producer's final output lists the changed files and confirms each is allowed.
4. [content] Tests that need the unplanned path strip the [job_plan] section wherever it sits in athena.toml, sharing one stripping helper where crate boundaries allow. Tests appending their own [job_plan] (job_plan.rs, config.rs unknown-key and defaults tests) do not end up with the section twice.
5. [content] docs/decisions.md has a new numbered entry turning planning on, citing docs/design/job-planning.md: alternatives from section 7; keep rule from section 9 (landed share of dispatched change jobs, baseline 64%, hold through 30 planned jobs unless it falls, decide at 100 jobs on 72% and a $2.50 cost ceiling per landed job); reversal (set enabled = false).
6. [content] CHANGELOG.md has an entry for turning job planning on naming athena.toml; the 'Built as' note in docs/design/job-planning.md mentions every test edit.
7. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; both pass.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Changing the planning stage or the [job_plan] schema
- Non-test code changes under crates/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests, clippy, fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria run cargo test/clippy/fmt to check behavior.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites the design and gives measured baseline numbers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused config change plus record.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check tests actually pass with planning on.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base dde59d6598db → candidate 7e9e257a80e9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  6 +++++
 athena.toml                             | 10 ++++++++
 crates/athena-cli/tests/common/mod.rs   | 21 +++++++++++++++--
 crates/athena-core/src/run_pin_tests.rs |  8 ++++++-
 crates/athena-data/src/config.rs        | 41 +++++++++++++++++++++++++++++----
 docs/decisions.md                       | 27 ++++++++++++++++++++++
 docs/design/job-planning.md             | 11 +++++++++
 7 files changed, 117 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (930 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
