# Job 20261028-035117-c5qb: verify "Plan hard jobs only; count later-caused failures in keep rule"

## Request (verbatim)

> Priority: urgent
> Turn job planning back on for hard jobs only, and fix the keep rule's measure (Sri, 2026-10-28). Planning was turned off on 2026-10-26 because the keep rule measured landed share and cost; the next day a job with no planner or critic (20261027-054211-a2n5) landed a crude word-matching refusal rule that stopped three drains, which landed share counts as a success. Change: (1) in athena.toml [job_plan], planning runs when the reading marks a job hard (the same marking that already escalates the producer), and is skipped otherwise; a job is hard when it adds or changes a rule that can refuse or hold work, or changes landing, the history check, records, the run pipeline or drain; (2) the keep rule (docs/design/job-planning.md section 9) also counts, per planned and unplanned job, landings that later caused a failure, a revert or a fix job citing them, and reports them in athena status beside the landed share.
> Acceptance:
> - A test shows a reading marked hard runs the planner and critic and an ordinary reading does not, with athena.toml set to planning for hard jobs only.
> - A test shows status reports, for planned and unplanned landings, how many were later cited as the cause of a failure, revert or fix.
> - docs/design/job-planning.md section 9 and docs/decisions.md record the change and why.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, crates/athena-data/src/config.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, docs/design/job-planning.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Replace the boolean job_plan switch with a hard-jobs-only mode in athena.toml, so the run pipeline plans and critiques only when the reading is marked hard. Extend the keep rule (job-planning.md section 9) and athena status to count, per planned and unplanned landing, later failures, reverts or fix jobs citing them. This follows job 20261027-054211-a2n5, whose landed refusal rule showed landed share is the wrong measure. Audience: Sri and the operators who read athena status.

Assumptions:
- Job 20261027-054211-a2n5 is cited as the motivating case. Its two findings (broad governance keywords, separate keyword test table) concern reading.rs governance matching and are not part of this request. They are out of scope and remain unaddressed here.
- The config gains a mode setting (for example mode = "hard") in [job_plan] in place of or alongside enabled; the existing enabled=false stays a full off switch.
- Hardness comes from the existing hard marking in the reading, which already escalates the producer. reading.rs only gets a hardness definition (rule that can refuse or hold work, landing, history check, records, run pipeline, drain) if the marking lacks it.
- A later cause is read from existing records: failure introspections, revert records and fix jobs citing a landed job id. No new record kind is added unless none exists.
- athena.toml is a governance file and is a named deliverable here.

## Acceptance criteria

1. [behavior] Run cargo test: a test loads athena.toml (planning for hard jobs only), and shows a reading marked hard runs the planner and critic while an ordinary reading runs neither.
2. [behavior] Run cargo test: a test shows athena status reports, separately for planned and unplanned landings, how many were later cited as the cause of a failure, revert or fix job, beside the landed share.
3. [content] athena.toml [job_plan] is set so planning runs for hard jobs only (git diff shows the change), and crates/athena-data/src/config.rs parses and validates the setting.
4. [content] docs/design/job-planning.md section 9 and docs/decisions.md record the new keep-rule measure, the hard-only planning, and why, citing job 20261027-054211-a2n5.
5. [content] The hard definition in code or docs covers rules that refuse or hold work, landing, the history check, records, the run pipeline and drain.
6. [behavior] cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.
7. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- docs/design/job-planning.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Fixing the two findings of job 20261027-054211-a2n5 (broad governance keywords, KEYWORD_REQUESTS table)
- Changing prompt files
- Changing defaults.toml, workflows or other governance files besides athena.toml
- Changing the planner or critic prompts, caps or max_rounds
- Reverting or reworking the refusal rule job 20261027-054211-a2n5 landed

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for planning dispatch and status counts; cargo test, clippy -D warnings and fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise hard versus ordinary readings and the status output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The keep rule now measures later harm, not only landed share.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Hard-only planning is enforced by config and run code, tested.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decisions explain the change and why for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b0771a4903b9 → candidate 7eea4454caab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  18 +++
 athena.toml                          |   8 +-
 crates/athena-cli/src/display.rs     |  57 +++++++-
 crates/athena-cli/tests/job_plan.rs  | 184 ++++++++++++++++++++++++
 crates/athena-context/src/outputs.rs |   9 +-
 crates/athena-core/src/hestia.rs     |   7 +-
 crates/athena-core/src/reading.rs    |  28 ++++
 crates/athena-core/src/run/mod.rs    |  10 +-
 crates/athena-core/src/status.rs     | 272 ++++++++++++++++++++++++++++++++++-
 crates/athena-data/src/config.rs     |  62 +++++++-
 crates/athena-data/src/records.rs    |  47 +++++-
 docs/decisions.md                    |  41 ++++++
 docs/design/job-planning.md          |  34 ++++-
 13 files changed, 757 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1526 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy with -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
