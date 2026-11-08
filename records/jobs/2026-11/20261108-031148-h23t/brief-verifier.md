# Job 20261108-031148-h23t: verify "Retry firstmate M2 study; fix spec coverage so cargo test passes"

## Request (verbatim)

> try again on 20261108-023653-70wy

## Reading (accepted by the user)

Retry of job 20261108-023653-70wy: redo the firstmate long-running-sessions (M2) study at the same pin for Sri and outside readers. The last attempt's study content was sound. It was rejected because cargo test failed on spec coverage of the BRIEF addenda from 2026-11-04 on, so this retry also adds the missing requirements to docs/spec.toml.

Assumptions:
- Job 20261108-023653-70wy is addressed as follows. The red cargo test (6 xtask spec tests: 'no requirement for' the BRIEF addenda 2026-11-04 to 2026-11-08) is fixed by adding requirements to docs/spec.toml. The c9 claim is fixed by citing only that a dead pid is reclaimed. The c17 and intro limits[3] nit is fixed by naming Pi, omp and OpenCode. The intro what[4] nit is fixed by adding the only-nudged exception. The rules.rs change is kept as a deliverable (refactor plus M2 tests), and the CI-skip limit is an existing M1 pattern, left as is.
- The typed request 'try again' takes precedence over the introspection's proposed baseline-check feature. That feature and the 'separate job' framing are out of scope here; the spec fix is included only so that cargo test goes green.
- Code survey: read xtask/tests/spec.rs (coverage test at line 323) and docs/spec.toml (format header lines 1-33; each BRIEF heading with text must be the first source heading of some requirement, with mechanism, owner, measure ending 'status: <label>', and verbatim quotes). The earlier candidate's crates/athena-cap-study/src/rules.rs changes are the study rule tests. No record or data file is added under crates/, so no check_record_file or AnyRecord change applies.
- Pinned commit is 6407052f4c521e39401ce2305dfa45d201249827, same as the M1 study; the M2 topics come from BRIEF.md milestone M2.
- Neither athena.toml, defaults.toml nor any workflow needs to change.

## Acceptance criteria

1. [content] studies/firstmate-long-running-sessions/ holds pin.json (sha 6407052f4c521e39401ce2305dfa45d201249827), claims.json, model.json, findings.json, rationales.json, intro.json and validation.json in the same shape as studies/firstmate-work-handling/.
2. [content] Every claim cites file and line sources at the pin with quotes. Claim c9 states only that a dead pid is reclaimed. The bounded-retry claim and the intro name Pi, omp and OpenCode. The intro sentence about the update restarting secondmates includes the only-nudged exception. Every intro sentence maps to a claim id.
3. [content] The model covers state, resumption, crash survival, unattended operation, self-update, drift and context rot as states, transitions, guards and counters, and records gaps in the source as findings.
4. [behavior] Run `cargo test --locked` and `cargo clippy` from the repo root. Both pass with no failures, including the study rule tests for the new directory and the six xtask/tests/spec.rs coverage tests.
5. [content] docs/spec.toml gains requirements whose first source heading is each BRIEF.md addendum from 2026-11-04 to 2026-11-08 that lacked one, each with verbatim quotes, mechanism, owner and a measure ending in a valid status label.
6. [content] git diff shows no change under studies/firstmate-work-handling/, and CHANGELOG.md has an entry for the M2 study.

## Expected deliverables

- studies/firstmate-long-running-sessions/pin.json (create)
- studies/firstmate-long-running-sessions/claims.json (create)
- studies/firstmate-long-running-sessions/model.json (create)
- studies/firstmate-long-running-sessions/findings.json (create)
- studies/firstmate-long-running-sessions/rationales.json (create)
- studies/firstmate-long-running-sessions/intro.json (create)
- studies/firstmate-long-running-sessions/validation.json (create)
- CHANGELOG.md (modify)
- crates/athena-cap-study/src/rules.rs (modify)
- docs/spec.toml (modify)

## Out of scope

- Changes under studies/firstmate-work-handling/
- Studying Hermes Agent or Athena itself
- Changes to athena.toml, defaults.toml or workflows
- M3 and later milestones
- Building the baseline test check proposed by the introspection
- Changes to xtask code or BRIEF.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass including the study rule tests and xtask spec tests.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: every claim cites file, lines and a verbatim quote at the pin; gaps are recorded as findings.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: the intro is written for outside readers and each sentence traces to claim ids.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: the spec coverage gap that blocked the last attempt is fixed rather than worked around.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: the verifier runs cargo test and clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check that tests exercise the new study and the spec coverage.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e5e3bf3e79d1 → candidate 6217ca65b774. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md   |  4 +++
 docs/spec.toml | 93 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 97 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-031148-h23t/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-031148-h23t/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42442830371 for 6217ca65b7744b6824849c000968f6e9af25b952)
- ci-result: `not a check` passed (Check results for 6217ca65b7744b6824849c000968f6e9af25b952 come from the CI run https://github.com/doodla/athena/actions/runs/42442830371 (565s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "studies/firstmate-long-running-sessions/ holds pin.json (sha 6407052f4c521e39401ce2305dfa45d201249827), claims.json, model.json, findings.json, rationales.json, intro.json and validation.json in the same shape as studies/firstmate-work-handling/."; "Every claim cites file and line sources at the pin with quotes. Claim c9 states only that a dead pid is reclaimed. The bounded-retry claim and the intro name Pi, omp and OpenCode. The intro sentence about the update restarting secondmates includes the only-nudged exception. Every intro sentence maps to a claim id."; "The model covers state, resumption, crash survival, unattended operation, self-update, drift and context rot as states, transitions, guards and counters, and records gaps in the source as findings."; "Run `cargo test --locked` and `cargo clippy` from the repo root. Both pass with no failures, including the study rule tests for the new directory and the six xtask/tests/spec.rs coverage tests."; "git diff shows no change under studies/firstmate-work-handling/, and CHANGELOG.md has an entry for the M2 study.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
