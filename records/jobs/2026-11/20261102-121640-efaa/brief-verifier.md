# Job 20261102-121640-efaa: verify "Check ontology terms against sources with bounded, reasoned excuses"

## Request (verbatim)

> try again on 20261027-054251-33v7

## Reading (accepted by the user)

Retry of job 20261027-054251-33v7 (rejected: file-wide source-only excuses disabled the content check for 6 of 7 terms). The ontology test will compare every term with its governing sources, accept excuses only per named marker with a reason, and require every term to have checked claims.

Assumptions:
- Code survey: read crates/athena-cli/tests/ontology.rs (problems, real_text, term helpers, scratch_root, the unit tests) and docs/ontology.toml (seven terms with sources fields). The ontology file is under docs/, not crates/, so no record file or AnyRecord round-trip is added. The property is measured directly by scanning BRIEF.md and the term's own file sources for each term name.
- Job 20261027-054251-33v7 is addressed by keeping its approach (senses and excused fields, scan of BRIEF.md and file sources) and fixing each finding: (major) source-only or marker-less excuses are removed, and every excuse must name a marker and give a reason; BRIEF.md can never be excused file-wide; the real terms get senses/markers so each of the seven is genuinely checked, including Athena's BRIEF.md:11 hit and its third 'studied instance' use. (minor) markers match as whole words/phrases, not substrings, and describe the sense rather than copying a line. (nit) an excuse that matches no line is flagged as stale. Its four changed files are kept as deliverables and re-edited from base.
- Per the introspection 20261102-121613-7j4a (kdwg): the test also fails if any term has no claim checked against a source, and a deliberately altered definition for each of the seven terms is flagged.
- Goal: stop seeded term content from contradicting governing sources unnoticed. Audience: the verifier and maintainers who rely on docs/ontology.toml. Where a definition really disagrees with its source, it is corrected in docs/ontology.toml and listed in the decision/changelog.
- Source-wide excuses are dropped entirely; a per-file excuse is allowed only with a marker, a reason, and exactly one file.

## Acceptance criteria

1. [content] The change description in docs/decisions.md and CHANGELOG.md names the root cause (content checked for structure, not against sources; and an unbounded file-wide excuse let the check be met by excusing), citing jobs 20261017-173508-0wtc, 20261025-214509-y179 and 20261027-054251-33v7, with alternatives (prose-only rule, measure-first stage) and why the check lives in the ontology test.
2. [behavior] Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and each term's file sources for every term name in docs/ontology.toml and passes on the real file.
3. [behavior] Run `cargo test -p athena-cli --test ontology`: for each of the seven terms (Hermes, planning, job, Athena, product versus system, landed, Sri) a test using a deliberately altered definition or missing sense is flagged; the Hermes case (tool-only entry while the source names a role) fails, and passes with both senses listed.
4. [behavior] Run `cargo test -p athena-cli --test ontology`: tests show that an excuse with no reason, no marker, or covering more than one file is flagged, that an excuse matching no line is flagged as stale, and that a term with no checked claim is flagged.
5. [content] Grep of docs/ontology.toml shows no excuse without a marker and reason, and no file-wide excuse; the check in crates/athena-cli/tests/ontology.rs is generic over all terms, matches markers as whole words, and the rule and optional fields are described in the test header or docs/ontology.toml [about].
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`: all pass.
7. [content] `git diff --stat` shows only crates/athena-cli/tests/ontology.rs, docs/ontology.toml, docs/decisions.md and CHANGELOG.md changed; no athena.toml, defaults.toml, workflows, studies/, or speed-work files.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speed-target work from job 20261025-214509-y179 (measure-first stage, secrets.rs memo, run_check polling, crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs)
- Adding new ontology terms
- Changing any study's files under studies/
- Changes to athena.toml, defaults.toml or workflows
- Production code outside the ontology test
- Wiring the ontology into agent contexts or prompt budgets

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The test change is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests with fmt, clippy and test passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the ontology tests and check flagged altered definitions.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The excuse design and the content-vs-structure gap are fixed in the check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check every term is really compared with its sources.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Four files, one focused change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the added D152 section at the end of docs/decisions.md and the new bullet under `## [Unreleased]` → `### Changed` in CHANGELOG.md. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: Both name the root cause (content checked for structure, not against sources; an unbounded file-wide excuse met the check by excusing). Both cite 20261017-173508-0wtc, 20261025-214509-y179 and 20261027-054251-33v7. Both list the alternatives (prose-only rule, measure-first stage) and why the check lives in the ontology test. D152 also lists the corrected definitions.
- Criterion 2: The new test `the_real_file_claims_are_checked_against_the_sources` parses docs/ontology.toml and calls `claim_problems` against the repository root, which reads BRIEF.md and each term's file sources. It asserts there are no problems, that all seven terms have at least one verified sense, and that the scan found hits for each term. Run: `cargo test -p athena-cli --test ontology the_real_file_claims_are_checked_against_the_sources`. Observes: The test passes on the real file. The assertion output lists problems if it fails.
  Fails without the behavior: Without the claim scan, a definition with no sense or a source line the definition cannot back would pass. The real-file test fails if any sense marker matches no line, any `says` is missing from its definition, any term has no checked claim, or an identifying line is uncovered.
  Test seam: Production entry point: `claim_problems(&File, &Path)` in crates/athena-cli/tests/ontology.rs, added in step 2. Fixture: the real docs/ontology.toml and the real BRIEF.md and docs files, read through `repo_root()`, which exists today. No fake.
- Criterion 3: Tests `altered_definition_is_flagged_for_each_seed_term` (a loop over the seven term names, removing each sense's `says` from that term's definition in the parsed real file) and `hermes_with_only_the_tool_sense_is_flagged_until_both_senses_are_listed` (drop the Orchestration sense, then restore it). Further tests drop the Athena role and program senses. Run: `cargo test -p athena-cli --test ontology`. Observes: Each of Hermes, planning, job, Athena, product versus system, landed and Sri is flagged with a message naming the term and the missing phrase. The Hermes tool-only entry is flagged as BRIEF.md line 597 uncovered. With both senses it returns no problems.
  Fails without the behavior: Without the claim check, altered definitions pass. Hermes with only the Hermes Agent sense passes even though BRIEF.md:597 names the Orchestration role, and the test asserting it is flagged fails.
  Test seam: Entry point `claim_problems(&File, &Path)`. The fixture is the real file parsed with `toml::from_str` and mutated in memory in the test, injected by passing the mutated `File`. The function is added in step 2 and the mutation helper in step 4.
- Criterion 4: Tests `an_excuse_without_a_reason_is_flagged`, `an_excuse_without_a_marker_is_flagged`, `an_excuse_covering_two_files_is_flagged`, `a_stale_excuse_is_flagged`, `an_overbroad_excuse_is_flagged`, `a_term_without_a_checked_claim_is_flagged`, and `markers_match_whole_words_only`. They run `claim_problems` over TOML fixtures built from `GOOD`, written to a tempdir by `scratch_root()`. Run: `cargo test -p athena-cli --test ontology`. Observes: Each fixture produces a problem whose text names the fault (no reason, no marker, more than one file, matches no line, no checked claim). A well-formed excuse on an identifying line produces none. A marker `cat` does not match `concatenate`.
  Fails without the behavior: Without the bounds, an excuse with no reason, no marker or two files would pass, a stale excuse would pass, and a term with no sense would pass.
  Test seam: Entry point `claim_problems`. The fixture is the tempdir from `scratch_root()` (exists today, extended in step 3 to write a BRIEF.md with an identifying line and a docs file) plus TOML text from `GOOD` (exists today, extended with senses in step 3).
- Criterion 5: The verifier may use the Grep tool on docs/ontology.toml for `excused`, `source-only` and `files =`. The test `the_ontology_file_is_sound` also fails on any excuse without a marker or reason. Read the header of crates/athena-cli/tests/ontology.rs and the comment above `[about]` in docs/ontology.toml. Run: `grep -nE "excused|files =|marker|reason|source-only" docs/ontology.toml`. Observes: Grep finds no excuse entries and no file-wide field. The header of ontology.rs and the comment in ontology.toml describe senses, aliases, excused, identifying lines, whole-word markers and the excuse bounds. The check is a loop over `file.terms` with no term name in the code except in the tests.
- Criterion 6: Run the three commands Athena runs, as simple commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A clippy lint such as needless borrow, or a fixture left without senses, fails this.
  Test seam: Existing workspace checks.
- Criterion 7: `git status` (and `git diff --stat`) on the candidate. Run: `git status`. Observes: Only crates/athena-cli/tests/ontology.rs, docs/ontology.toml, docs/decisions.md and CHANGELOG.md are changed. There is no athena.toml, defaults.toml, workflow, studies/ or speed-work change.

Planned files:

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 66743948c35e → candidate 8eedae0c0021. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  12 +
 crates/athena-cli/tests/ontology.rs | 749 +++++++++++++++++++++++++++++++++++-
 docs/decisions.md                   |  53 +++
 docs/ontology.toml                  |  61 ++-
 4 files changed, 859 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41451170845 for 8eedae0c0021b6d4df78be6d946b1d1372a4005e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41451170845 for 8eedae0c0021b6d4df78be6d946b1d1372a4005e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41451170845 for 8eedae0c0021b6d4df78be6d946b1d1372a4005e)
- ci-result: `not a check` passed (Check results for 8eedae0c0021b6d4df78be6d946b1d1372a4005e come from the CI run https://github.com/doodla/athena/actions/runs/41451170845 (497s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
