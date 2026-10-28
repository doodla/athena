# Job 20261028-033455-94t4: verify "Narrow the governance reading rule and retry a refused reading once"

## Request (verbatim)

> Priority: urgent
> The governance-file rule in crates/athena-core/src/reading.rs (the GOVERNANCE_BY_KEYWORD table and check_governance_coverage) refuses any reading whose request merely contains one of its common words, and the reader never learns why, so the next read fails the same way. Since it landed it has stopped three drains in a row and refused the reading of its own fix three times. Fix both parts. (1) Narrow the rule: a request word alone no longer requires a governance path; require one only when the request names the path itself or a file under it, or when a deliverable implies it (the GOVERNANCE_BY_DELIVERABLE table stays). (2) When any reading rule refuses a reading, read once more at once with the refusal text given to the reader as a correction, and count only a second refusal as a failure; list each reading rule in one short line in the reader's brief.
> Acceptance:
> - A test shows a request that only mentions common words from the old table (with no governance path named) is accepted, and one that names a governance path but leaves it uncovered is still refused.
> - A test with a fake reader whose first reading is refused shows the refusal text reaches the second read and the second reading is accepted.
> - The reader brief names each reading rule in one line and stays within its size limit; the size test passes.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, crates/athena-context/src/briefs.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Reading acceptance stops requiring a governance path for bare common words (deploy, config, ci, gate…); it requires one only when the request names the governance path or a file under it, or a deliverable implies it (GOVERNANCE_BY_DELIVERABLE stays). When any reading rule refuses a reading, the reader is run once more at once with the refusal text as a correction, only a second refusal fails, and the reader brief lists each reading rule in one line. Audience: the drain operator and the reader agent.

Assumptions:
- The in-run retry belongs in Athena::read_for (reading.rs), which every read path calls, so first reads, rereads after correction and config rereads all get it; drain.rs changes only as far as its failure counting or tests need.
- The refusal text is passed to the reader through a new ReaderInputs field rendered as a correction section in reader_brief (briefs.rs); no prompt file under the token budget changes, since the reader prompt set has only 3 tokens left.
- Rule list in the brief is one short line per rule: prior result and change scope, criteria evidence, governance coverage, and the reading-build validity checks.
- The keyword table is replaced by path-naming detection: the request text contains the governance path (e.g. athena.toml, defaults.toml, setup.sh, rust-toolchain.toml) or a path under .github/workflows/.
- Both invocations of the reader are recorded, or at minimum the second reading and its cost are, so budget accounting stays correct.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/reading.rs shows a request that only mentions words from the old keyword table (e.g. 'deploy', 'config', 'ci', 'gate') with no governance path named is accepted by check_governance_coverage; run `cargo test -p athena-core`.
2. [behavior] A unit test shows a request naming a governance path (e.g. athena.toml or .github/workflows/ci.yml) whose reading leaves it uncovered is still refused, and is accepted once it is listed and named in a criterion or named in out_of_scope; run `cargo test -p athena-core`.
3. [behavior] A test with a fake reader whose first reading is refused shows the refusal text appears in the second read's brief and the second reading is accepted and recorded; a second refusal is reported as a failure. Run `cargo test`.
4. [behavior] reader_brief names each reading rule in one line and the existing brief size test passes; run `cargo test -p athena-context`.
5. [content] GOVERNANCE_BY_DELIVERABLE is unchanged and a deliverable that is a governance file with no criterion naming it is still refused.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry describing the narrowed governance rule and the one-time re-read with the refusal text.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- athena.toml, defaults.toml and other governance files
- Changing GOVERNANCE_BY_DELIVERABLE contents
- Prompt files under the token budget (frame.reader, contract)
- Changing admission's is_governance or governance_change_is_listed

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the narrowed rule and the retry; fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing acceptance, refusal and the refusal text reaching the second read.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the over-broad keyword rule and the missing refusal feedback to the reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change across the named files only.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The retry and rule list are enforced in code and tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 648de0d98d7c → candidate 0bf69bc1bf53. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  17 ++
 crates/athena-context/src/briefs.rs     |  73 +++++++-
 crates/athena-core/src/reading.rs       | 321 +++++++++++++++++++++++---------
 crates/athena-core/src/run/mod.rs       |   9 +-
 crates/athena-core/src/run_pin_tests.rs | 133 +++++++++++++
 5 files changed, 462 insertions(+), 91 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1517 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
