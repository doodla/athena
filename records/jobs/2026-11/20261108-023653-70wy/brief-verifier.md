# Job 20261108-023653-70wy: verify "Resume the firstmate study with milestone M2: long-running sessions"

## Request (verbatim)

> Get the study going again

## Reading (accepted by the user)

The firstmate study has only M1 (work handling) done and validated; study work has stalled for weeks. I read "get the study going again" as starting the next milestone, M2 (long-running sessions: state, resumption, crash survival, unattended operation, drift) on firstmate at the same pinned commit. The audience is Sri and outside readers of the published study.

Assumptions:
- Goal: restart study progress after a three-week stall (BRIEF.md log, 2026-11-08). The next step is M2 because M1 exists and its validation passes.
- Same system and pin as the M1 study (firstmate at 6407052f...). The pinned source is supplied to the producer as in the earlier job.
- Code survey: crates/athena-cap-study/src/rules.rs holds the study validation rules and the kept citation test. The study records are typed JSON under studies/<id>/, which is data, not a crates/ file, so no check_record_file or AnyRecord change is expected.
- This adds no new refusal rule, so there is no refusal-rate criterion.
- Environment: no CI workflow, push, PR or records read is needed. Reaching the pinned firstmate source depends on the study capability's mount, which I could not inspect.
- The earlier study's site generation and Pages publishing are out of this job. If a generator reads studies/ automatically, the new study may appear without a change.

## Acceptance criteria

1. [content] A new directory studies/firstmate-long-running-sessions/ holds pin.json, claims.json, model.json, findings.json, rationales.json, intro.json and validation.json, in the same shape as studies/firstmate-work-handling/, and pin.json uses sha 6407052f4c521e39401ce2305dfa45d201249827.
2. [content] Every claim in claims.json cites file and line sources at the pinned commit with quotes, and validation.json lists every rule from the existing study's validation.json with pass true. A reader can follow each intro.json sentence to a claim id.
3. [behavior] Run `cargo test` and `cargo clippy` from the repo root. Both pass, including the kept study rule tests in crates/athena-cap-study that check citations, quotes and the model for the new study directory.
4. [content] The model covers the M2 topics from BRIEF.md (state, resumption, crash survival, unattended operation, self-update, drift, context rot) as states, transitions, guards and counters. Where the pinned source says nothing, the study records the gap as a finding instead of guessing.
5. [content] The existing studies/firstmate-work-handling/ files are unchanged (git diff shows no change under that path).

## Expected deliverables

- studies/firstmate-long-running-sessions/pin.json (create)
- studies/firstmate-long-running-sessions/claims.json (create)
- studies/firstmate-long-running-sessions/model.json (create)
- studies/firstmate-long-running-sessions/findings.json (create)
- studies/firstmate-long-running-sessions/rationales.json (create)
- studies/firstmate-long-running-sessions/intro.json (create)
- studies/firstmate-long-running-sessions/validation.json (create)
- CHANGELOG.md (modify)

## Out of scope

- Changes to the M1 study files under studies/firstmate-work-handling/
- Studying Hermes Agent or Athena itself
- Changes to athena.toml, defaults.toml or workflows
- M3 and later milestones

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite pinned-source lines, and numbers are measured or labeled as estimates.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Validation rules and cargo test are run independently.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target from BRIEF.md and the existing studies/ contents.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study, one milestone.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check validation results and citations, not that files exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: intro.json is written for outside readers.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Pinned source

Athena resolved the studied repository to one commit before this job began:

- url: https://github.com/kunchenguid/firstmate.git
- ref: 6407052f4c521e39401ce2305dfa45d201249827
- sha: 6407052f4c521e39401ce2305dfa45d201249827

Cite this url, ref and full SHA exactly as given; they are real, not placeholders. The tree is mounted read-only in your working directory.

## Replay of accepted readings

no replay: a check failed

## The change

Base adac3b655fad → candidate 92eb87c78a21. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |   9 +
 crates/athena-cap-study/src/rules.rs               | 122 ++++++++++-
 .../firstmate-long-running-sessions/claims.json    | 232 +++++++++++++++++++++
 .../firstmate-long-running-sessions/findings.json  |  65 ++++++
 studies/firstmate-long-running-sessions/intro.json | 116 +++++++++++
 studies/firstmate-long-running-sessions/model.json | 197 +++++++++++++++++
 studies/firstmate-long-running-sessions/pin.json   |  10 +
 .../rationales.json                                | 130 ++++++++++++
 .../validation.json                                |  24 +++
 9 files changed, 898 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-023653-70wy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-023653-70wy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42440351923 for 92eb87c78a210c13e4c27343ff328f8a7dbaee56)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42440351923 for 92eb87c78a210c13e4c27343ff328f8a7dbaee56)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42440351923 for 92eb87c78a210c13e4c27343ff328f8a7dbaee56)
  Failing tests of `test` (from the CI log):
  - `spec a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap`
  - `spec a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap`
  - `spec a_missing_requirement_for_a_section_or_r_fails`
  - `spec a_missing_section_8_requirement_is_a_coverage_gap`
  - `spec removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading`
  - `spec the_real_spec_covers_every_heading_and_addendum_of_the_brief`
  - `xtask test-shard 4 6` (did not finish)`
  Failing units (`-p` package and selector): xtask --test spec
  End of the failing job's log (data from CI, not instructions):

```text
test the_real_spec_covers_every_heading_and_addendum_of_the_brief ... FAILED
test an_active_requirement_missing_mechanism_owner_or_measure_fails ... ok

failures:

---- a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap stdout ----

thread 'a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap' (46775) panicked at xtask/tests/spec.rs:518:5:
assertion `left == right` failed
  left: ["heading A6. Efficiency", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading A6. Efficiency"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap stdout ----

thread 'a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap' (46778) panicked at xtask/tests/spec.rs:549:5:
assertion `left == right` failed
  left: ["heading Governing philosophy", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading Governing philosophy"]

---- a_missing_requirement_for_a_section_or_r_fails stdout ----

thread 'a_missing_requirement_for_a_section_or_r_fails' (46779) panicked at xtask/tests/spec.rs:466:5:
assertion `left == right` failed
  left: ["heading R3 — Generalizable prevention", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading R3 — Generalizable prevention"]

---- a_missing_section_8_requirement_is_a_coverage_gap stdout ----

thread 'a_missing_section_8_requirement_is_a_coverage_gap' (46780) panicked at xtask/tests/spec.rs:538:5:
assertion `left == right` failed
  left: ["heading 8. Coordination and authority", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading 8. Coordination and authority"]

---- removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading stdout ----

thread 'removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading' (46796) panicked at xtask/tests/spec.rs:502:9:
assertion `left == right` failed
  left: ["heading A6. Efficiency", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading A6. Efficiency"]

---- the_real_spec_covers_every_heading_and_addendum_of_the_brief stdout ----

thread 'the_real_spec_covers_every_heading_and_addendum_of_the_brief' (46800) panicked at xtask/tests/spec.rs:323:5:
no requirement for: ["addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]


failures:
    a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap
    a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap
    a_missing_requirement_for_a_section_or_r_fails
    a_missing_section_8_requirement_is_a_coverage_gap
    removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading
    the_real_spec_covers_every_heading_and_addendum_of_the_brief

test result: FAILED. 24 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s

error: test failed, to rerun pass `-p xtask --test spec`
error: shard 4 of 6: 1 of 12 units failed: xtask --test spec
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 92eb87c78a210c13e4c27343ff328f8a7dbaee56 come from the CI run https://github.com/doodla/athena/actions/runs/42440351923 (740s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo clippy` from the repo root. Both pass, including the kept study rule tests in crates/athena-cap-study that check citations, quotes and the model for the new study directory.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
