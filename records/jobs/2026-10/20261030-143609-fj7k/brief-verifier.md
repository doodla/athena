# Job 20261030-143609-fj7k: verify "Count only 'Depends on' ids and retry links as plan dependencies"

## Request (verbatim)

> try again on 20261030-125332-sz3z

## Reading (accepted by the user)

Retry of job 20261030-125332-sz3z (failed on low disk, never verified): make plan_entry take dependencies only from 'Depends on' lines and the retry link of a 'try again on' request, so cited job ids no longer hold items. The earlier candidate is dropped and the change is redone from base.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (plan_entry at ~line 2125 builds entry.depends_on from job_ids_in(text), the cause of the bug) and grepped crates/athena-cli/tests/drain.rs and crates/athena-cli/src/drain.rs for plan_entry; plan.rs holds the hold logic and reason text, which stay unchanged. No record or data file is added, so check_record_file/AnyRecord are not touched.
- Retry of job 20261030-125332-sz3z: the user wants the same fix redone. The failure was environmental (low disk), not a verifier finding, so the original request's goal and acceptance stand. The introspection's proposal for disk handling is a separate change and is out of scope here.
- Goal: stop evidence-citing requests (including the steward's root-cause requests) from being held forever; the audience is the people waiting on the 29 held items.
- A 'Depends on' line is matched case-insensitively with optional '>', list marker or numbering; only full job ids count. The retry link of a 'try again on <id>' request remains a dependency.
- This tightens dependency detection, so it newly holds nothing and releases held items; no refusal rate increase.

## Acceptance criteria

1. [behavior] A new test through Plan::build_with (plan_entry built from request text) shows a request quoting a rejected job id in its text with no 'Depends on' line is planned and not in Plan::refused. Run: cargo test -p athena-core.
2. [behavior] A new test shows a 'Depends on <id>' line naming a rejected job with no landed retry is still held; its Refusal reason contains 'depends on <id>' and 'rejected'. Run: cargo test -p athena-core.
3. [behavior] A new test shows a 'try again on <id>' request still depends on that id, and that an id on a 'Depends on' line is a dependency while the same id elsewhere in the text is not. Run: cargo test -p athena-core.
4. [behavior] A drain test in crates/athena-cli/tests/drain.rs shows an item that only cites a rejected job id is planned and run, not held. Run: cargo test -p athena-cli.
5. [behavior] Existing reading tests about job ids (reading.rs near line 4164) and drain tests that used 'after <id>' are updated to the new rule; cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [content] CHANGELOG.md has an entry saying only 'Depends on' ids and retry links are dependencies and that cited job ids no longer hold an item.
7. [content] The doc comment of plan_entry in crates/athena-core/src/reading.rs states the new rule in place of 'depends on every other job whose full id it names'.
8. [content] Job 20261030-125332-sz3z failed only on low disk (10.0 GB free, 10.0 GB required) before verification; it had no code findings. Its candidate (reading.rs, plan.rs, drain.rs tests, CHANGELOG.md) is dropped; this retry starts from base and redoes the same change.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing how rejected or failed dependencies are held, or the held reason text
- Releasing or re-reading the held items by hand
- Changing athena.toml, defaults.toml or workflow files
- Changing the 'referenced job not among the records' open question beyond what the new rule needs
- Low-disk handling (cleanup, waiting-for-disk hold) proposed by the introspection; a separate request

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for planned, held and retry cases; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix dependency extraction in plan_entry, not the held items one by one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing items planned or held.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is cited-id requests no longer held, checked by tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to dependency extraction.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7849c30d1cf2 → candidate 3e1410d9bf50. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   9 ++
 crates/athena-cli/tests/drain.rs  |  41 +++++++-
 crates/athena-core/src/plan.rs    |   8 +-
 crates/athena-core/src/reading.rs | 211 +++++++++++++++++++++++++++++++++++++-
 4 files changed, 260 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1741 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass, including the existing reading test about job ids at reading.rs around line 4164, updated to the new rule.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
