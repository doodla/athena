# Job 20261108-023649-67f6: verify "Add a diagnose-failure skill and load it into introspection"

## Request (verbatim)

> I saw the diagnose job die. Dunno what the problem is, don't care. Just that it wasn't figured out. Who knows what the method was like. We need better skills

## Reading (accepted by the user)

Reads "better skills" as: Athena has no skills yet and its failure diagnosis (introspection) has no stated method, so a failed diagnosis can't be traced. Add one versioned skill file describing a stepwise method for diagnosing a failed job, and have the introspection brief load it. Audience: Sri and the later verifier judging whether failures get diagnosed.

Assumptions:
- Goal: failed or rejected jobs get diagnosed with a stated, versioned method; audience is Sri, who judges whether failures are figured out, and Athena's introspection step.
- Code survey: read crates/athena-core/src/introspect.rs (module docs: introspect_job and introspect_intervention run after a rejected or failed job; Jev is asked in shadow; a failed introspection is recorded as failed and not itself introspected), crates/athena-context/src/briefs.rs (where briefs are composed), crates/athena-context/prompts/*.md, and crates/athena-data/src/records/introspection.rs (schema athena.introspection/1). The skill is a markdown file, not a record under crates/ data; no new record or data file is added, so check_record_file/AnyRecord/round-trip are unaffected unless the producer adds one. The property (skill text reaches the introspection brief) is measured directly by a test on the composed brief text.
- Existing code: crates/athena-context/src/briefs.rs, crates/athena-core/src/introspect.rs, docs/design/defaults-loop.md, BRIEF.md section A12 (skills versioned in this repo).
- The 'diagnose job' that died is an introspection of a failed job; its exact failure was not investigated (user does not care), so the change adds method, not a fix for one instance.
- Skill content is generic (no system-specific instructions outside labeled examples), per BRIEF.md, and short because prompt token budgets are tight.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.
- Term ladder: not used.

## Acceptance criteria

1. [content] A new skill markdown file exists in a skills directory (e.g. skills/diagnose-failure.md) giving ordered steps: read the stored failures and check tails, state the cause as something other than a restatement of the event, compare with earlier cases, name the evidence cited, and say what to do when evidence is missing.
2. [behavior] A test in crates/athena-context or crates/athena-core builds the introspection brief and asserts it contains the skill text; run `cargo test` and it passes.
3. [behavior] `cargo test` passes and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
4. [content] The skill file contains no secrets or personal data and no system-specific instructions outside clearly labeled examples; it is written for an outside reader.
5. [content] No existing prompt set exceeds its token budget; reader and verifier prompt files are untouched.
6. [content] CHANGELOG.md has an entry naming the skill and the outcome: introspection of a failed job now follows the stated method.

## Expected deliverables

- skills/diagnose-failure.md (create)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)

## Out of scope

- Investigating why the specific diagnose job died
- Parallel sessions or an outside-knowledge loop
- Skills for other capabilities
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Loading and test code is Rust; the skill is markdown content.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test covers the brief containing the skill; cargo test and clippy criteria.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Skill written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One skill, one wiring point.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion checks the brief contains the skill text via a test.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion ties to failures being diagnosed by a stated method.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered meaning from BRIEF.md 2026-11-08 relay note and A12.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets in the skill.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base e34b12ae5a33 → candidate cfe028465b6d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |  3 +++
 crates/athena-cli/tests/introspection.rs         | 29 ++++++++++++++++++++++++
 crates/athena-context/skills/diagnose-failure.md | 13 +++++++++++
 crates/athena-core/src/introspect.rs             | 10 +++++++-
 4 files changed, 54 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-023649-67f6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-023649-67f6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42445353500 for cfe028465b6d076d8b5b56a1f3b57063bc6d9607)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42445353500 for cfe028465b6d076d8b5b56a1f3b57063bc6d9607)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42445353500 for cfe028465b6d076d8b5b56a1f3b57063bc6d9607)
- ci-result: `not a check` passed (Check results for cfe028465b6d076d8b5b56a1f3b57063bc6d9607 come from the CI run https://github.com/doodla/athena/actions/runs/42445353500 (744s): fmt failure, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
