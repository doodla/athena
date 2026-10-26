# Job 20261026-194248-xw3g: verify "Turn job planning off per the D110 keep rule and record the decision"

## Request (verbatim)

> Priority: urgent
> Apply the job-planning keep rule (docs/design/job-planning.md section 9, D110) now that it has decided. athena status reports 58 of 105 planned jobs landed (55%), at or below the 64% baseline at the 100-job point, with cost per landed job $3.43 above the $2.50 guard: the rule says turn planning off. Set `enabled = false` under [job_plan] in athena.toml. Add a decision entry in docs/decisions.md recording the result with the numbers from athena status, that the rule (not a person) decided it, that the decision was taken by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and that the earlier keep-planning-on override (intervention 20261025-033755-b01h) was taken at 51 jobs, below the decision point. Note what is kept: the planning code, the resume-from-kept-plan path and the status measures stay, so planning can be turned back on for a new measured trial. Update section 9 of job-planning.md with the outcome.
> Acceptance:
> - athena.toml has enabled = false under [job_plan] and no other change.
> - docs/decisions.md has the new entry with the measured numbers and the decider; job-planning.md section 9 states the outcome.
> - A test or the existing config tests show the config parses with planning off and a change job runs with no planner or critic invocation.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, docs/decisions.md, docs/design/job-planning.md, CHANGELOG.md

## Reading (accepted by the user)

Turn job planning off per the decided keep rule: set enabled = false in [job_plan], record the decision with its numbers and decider in decisions.md, update section 9 of job-planning.md, and add a CHANGELOG entry. The existing repo-config test asserts enabled is true, so it is updated to expect planning off.

Assumptions:
- Goal: apply the keep rule's outcome (planning off); audience is Sri and later maintainers reading the decision log and design doc.
- The existing test the_repository_config_turns_job_planning_on_with_the_designs_caps asserts enabled is true and will fail, so it must be updated and renamed; this makes crates/athena-data/src/config.rs a needed deliverable beyond the listed files.
- A no-planner-invocation test for the disabled path probably already exists in crates/athena-cli/tests/job_plan.rs; if not, one is added there.
- Numbers are taken from the request as given, since athena status cannot be run by the verifier.

## Acceptance criteria

1. [content] athena.toml has `enabled = false` under [job_plan]; git diff shows that one line as the only change to the file (caps and timeouts untouched).
2. [content] docs/decisions.md has a new entry with the athena status numbers (58 of 105 landed, 55%, 64% baseline, $3.43 per landed job vs $2.50 guard), states the rule decided and not a person, the brain acted under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26), the override 20261025-033755-b01h was taken at 51 jobs below the decision point, and what is kept (planning code, resume-from-kept-plan path, status measures) for a new measured trial.
3. [content] docs/design/job-planning.md section 9 states the outcome: planning turned off by the keep rule, with the numbers, and what remains.
4. [content] CHANGELOG.md has an entry for turning job planning off.
5. [behavior] Run `cargo test`: the repo config test in athena-data/src/config.rs is updated to assert the repository athena.toml parses with job_plan present and enabled == false (caps kept), and passes; an existing or added test shows a change job with planning off makes no planner or critic invocation (e.g. athena-cli tests/job_plan.rs disabled-path test), and passes.
6. [behavior] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean and the full `cargo test` passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Removing or changing planning code, the resume-from-kept-plan path, or the status measures
- Changing planner/critic caps or timeouts in [job_plan]
- Starting a new planning trial
- Re-measuring or re-deriving the athena status numbers

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Config test updated; fmt, clippy and test suite must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites the measured athena status numbers and the intervention id.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check parsed config value and no planner/critic invocation when run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The keep rule is applied via the config flag, enforced by code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and doc text written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change with the minimal test edit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of the file. Run: `git diff -- athena.toml`. Observes: A single changed line, `enabled = true` to `enabled = false`, under [job_plan]. The caps, timeouts and max_rounds lines are unchanged.
- Criterion 2: Read the added entry. Run: `git diff -- docs/decisions.md`. Observes: A D135 entry with 58 of 105 landed (55%), the 64% baseline, $3.43 against the $2.50 guard, the rule (not a person) as decider, the brain acting under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26), override 20261025-033755-b01h taken at 51 jobs below the decision point, and what is kept for a new measured trial.
- Criterion 3: Read the diff of section 9. Run: `git diff -- docs/design/job-planning.md`. Observes: An outcome paragraph in section 9: planning turned off by the keep rule with the numbers, and the code, resume path and status measures kept.
- Criterion 4: Read the diff. Run: `git diff -- CHANGELOG.md`. Observes: A new entry under [Unreleased] saying job planning is turned off by the keep rule (D135).
- Criterion 5: Run the updated config test and the existing disabled-path test. Run: `cargo test --locked -p athena-data the_repository_config_turns_job_planning_off && cargo test --locked -p athena-cli --test job_plan with_planning_absent_or_off`. Observes: Both pass. The config test shows the repository athena.toml parses with job_plan present, enabled false and caps 0.75/0.75/600/600/3. The CLI test shows a change job with planning off invokes only Reader, Producer and Verifier and writes no job-plan or critique file.
  Fails without the behavior: If athena.toml still had enabled = true, the config test would fail on the enabled assertion. If the off path invoked a planner or critic, the invocation-list assertion would fail.
- Criterion 6: Run the three checks Athena runs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A stale test that still asserts enabled == true would fail cargo test.

Planned files:

- athena.toml (modify)
- crates/athena-data/src/config.rs (modify)
- docs/decisions.md (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)

## The change

Base ea05b5fdd4ed → candidate 953a763ac0cb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  5 +++++
 athena.toml                      |  2 +-
 crates/athena-data/src/config.rs |  4 ++--
 docs/decisions.md                | 26 ++++++++++++++++++++++++++
 docs/design/job-planning.md      |  7 +++++++
 5 files changed, 41 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1359 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean and the full `cargo test` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
