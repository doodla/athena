# Job 20261104-153450-y72x: verify "Amend D08 so docs match read-only commit history access"

## Request (verbatim)

> try again on 20261010-225741-yvk0

## Reading (accepted by the user)

Retry of job 20261010-225741-yvk0: its only major finding was that docs/decisions.md D08 (line 233, "agents lose `git log` of the real history. That is the point.") still contradicts the new read-only commit view. Audience is the verifier and later readers of the decision log. This retry is docs-only: amend D08, mark it "Amended by D155", and ensure D155 lists alternatives and tradeoffs.

Assumptions:
- Job 20261010-225741-yvk0 passed fmt, clippy and test and failed only on one unmet content criterion (D08), so I assume the producer starts from its candidate (code, tests, CHANGELOG, enforcement.md and the D155 hunk already present). The repository copy I read has no D155 and D08 line 233 is unchanged, so if the producer instead starts from base, it must also write D155 (the new decision) in docs/decisions.md; the criteria cover both cases.
- Findings of 20261010-225741-yvk0: (major) D08 not amended: fixed by rewording line 233 and adding 'Amended by D155'. (minor) run/mod.rs and run_pin_tests.rs changed outside deliverables: no longer applies, they are kept as-is and excluded in out_of_scope as needed wiring. (nit) hashes taken from request text only: no longer applies, it is a documented limit in D155 and CHANGELOG and stays.
- Other docs saying agents never see real history: Grep found D08 line 233 as the clear one; docs/design/athena-v0.md:87 and :449 refer to D08 and should be read and reworded only if they claim no history access. docs/enforcement.md and docs/design/*.md are touched only if such a sentence exists.

## Acceptance criteria

1. [content] docs/decisions.md D08 Tradeoff line (around line 233) no longer says agents lose the real history; it says agents can read the commits a job names through a filtered read-only view and the rest of history stays hidden, and D08 contains 'Amended by D155' (as at 'Amended by D63 and D115'). Check with Grep for 'That is the point' and 'Amended by D155' in docs/decisions.md.
2. [content] docs/decisions.md has a D155 entry that lists alternatives (no history access, full history access, generated file only) with the tradeoff of each, and states the request-text-only hash limit. Check by reading the entry.
3. [content] Grep of docs/ for sentences saying agents never see or lose the real history finds none left unreworded or unmarked; a reader checking D08 can follow it to D155.
4. [content] Only files under docs/ differ in this retry's diff versus the candidate; no file under crates/, athena.toml, defaults.toml or workflows changes.
5. [behavior] Run `cargo test` and `cargo clippy` and `cargo fmt --check`; all still pass, showing the docs edit broke nothing (docs-lint or doc-reference tests included).

## Expected deliverables

- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any change to the candidate's code: crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-context/src/briefs.rs are kept as the earlier job left them, not edited here
- Extracting hashes from brief text as well as request text
- Making full history available to every agent, or any git write access
- Edits to athena.toml, defaults.toml, or workflows
- Building the proposed decision-to-claim consistency check (a separate job)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: D155 and the D08 amendment cite the exact hidden-path rules and tests in the candidate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision text is written for an outside reader, with alternatives and tradeoffs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change, code left untouched.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the sentence is gone and the pointer exists via Grep, not just that a file changed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Searches all docs for the same stale claim, not only D08.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base d63d8e8e3d17 → candidate b4965b3b0f6e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  13 +
 crates/athena-context/src/briefs.rs        | 104 +++++
 crates/athena-core/src/run/mod.rs          |   2 +-
 crates/athena-core/src/run/producer.rs     |  46 +++
 crates/athena-core/src/run/verification.rs |  52 ++-
 crates/athena-core/src/run_pin_tests.rs    |  78 ++++
 crates/athena-git/src/lib.rs               |  38 ++
 crates/athena-git/src/snapshot.rs          | 632 ++++++++++++++++++++++++++++-
 docs/decisions.md                          |  52 ++-
 docs/enforcement.md                        |   1 +
 10 files changed, 1012 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (ran with --no-fail-fast added to the configured command)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo clippy` and `cargo fmt --check`; all still pass, showing the docs edit broke nothing (docs-lint or doc-reference tests included).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
