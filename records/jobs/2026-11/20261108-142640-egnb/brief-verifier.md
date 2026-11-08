# Job 20261108-142640-egnb: verify "Add pinned-versions and explicit-tool-paths defaults for the reader"

## Request (verbatim)

> try again on 20261027-044511-kktq

## Reading (accepted by the user)

Retry of job 20261027-044511-kktq. Add two [[default]] entries to defaults.toml, each with its whole rule in the first sentence (the only part reader render() shows), and raise the reader budget by the smallest round amount instead of splitting or trimming existing defaults. The audience is the reader step, which should stop asking about version pins and tool paths or silent fallback.

Assumptions:
- Retry of job 20261027-044511-kktq. Its candidate changed CHANGELOG.md, crates/athena-cli/tests/budgets.rs, defaults.toml and docs/system-facts.md. Every file is dropped and redone from the base, and each is a deliverable below except none are kept as-is. Findings addressed: (1) the five split defaults (buy-first, tested, public, verified, lazy-requests) are NOT split or shortened, and a test pins their rendered lines; (2) the new first sentences carry the bump command and the no-silent-fallback halves, and the test asserts the rendered context, not the model; (3) docs/system-facts.md is now a listed deliverable because the budgets test requires its reader line to match the computed figure; (4) the misread of render() is fixed: render shows only the first sentence (crates/athena-data/src/defaults.rs:58-83).
- Code survey: crates/athena-data/src/defaults.rs (parse, render, first_sentence: a '. ' ends the rendered text); crates/athena-cli/tests/budgets.rs (computed_rows, system_facts_budgets_match_computed, every_role_set_is_within_its_budget use defaults().render()); crates/athena-context/src/lib.rs:175 (BudgetSet::Reader => 2_000); crates/athena-core/src/run_pin_tests.rs:1625 (hard-codes 'of 2000 tokens used' for the reader line). No record or data file is added under crates/, so check_record_file and AnyRecord are not involved. The property is measured on defaults().render() itself.
- The reader set has 3 tokens of headroom (1997 of 2000), so two new rendered lines (about 40-50 tokens each, estimated at ceil(bytes/3)) will not fit. The reader budget in crates/athena-context/src/lib.rs is raised to the smallest multiple of 50 that fits, and the changelog states the old and new figure and why.
- The bump command is the existing `cargo run -p xtask -- repin` (xtask/src/main.rs:16, README.md:252-256), so the pinned-versions first sentence names it. Both entries are marked prose-only, with no enforcement code, matching the existing prose-only entries. Enforcing them in the runner is out of scope.
- Statements are one sentence each so the whole rule renders. Any '. ' inside a statement ends the render, so a command like 'cargo run -p xtask -- repin' must contain no period-space. applies_to is ['config','build','code'] for both. source cites this request and job 20261027-044511-kktq.
- Governance: defaults.toml is the requested deliverable and is named in criteria. athena.toml and .github/workflows are not needed, because the budget lives in crates/athena-context/src/lib.rs, not athena.toml.
- Docs that quote the reader figure change with it: docs/system-facts.md line 22 (checked by system_facts_budgets_match_computed) and docs/design/context-engineer.md line 54 (prose table, not test-checked).

## Acceptance criteria

1. [content] defaults.toml gains a [[default]] with id "pinned-versions" whose statement is ONE sentence (no '. ' inside) saying config version pins such as claude_version stay exact and never float, are bumped only by `cargo run -p xtask -- repin`, and a refusal from a mismatch names that command, marked prose-only. Check with Grep on defaults.toml.
2. [content] defaults.toml gains a [[default]] with id "explicit-tool-paths" whose statement is ONE sentence saying a check never relies on inherited environment or ambient tools, the runner passes every tool path explicitly, output records whether the preferred tool or the fallback ran, and a silent fallback is a defect, marked prose-only. Both entries have applies_to and a source citing this request or job 20261027-044511-kktq. Check with Grep on defaults.toml.
3. [content] git diff shows no change to the statement, applies_to or source of any existing default in defaults.toml: no sentence splitting or shortening of buy-first, tested, public, verified, lazy-requests, root-cause or behavior-evidence. Only two entries are appended.
4. [behavior] A new test in crates/athena-cli/tests/budgets.rs builds defaults().render() from the real defaults.toml and asserts it contains both new ids and the key phrases of every half: 'claude_version', 'xtask -- repin', 'never float', 'explicit', 'fallback', 'defect', and the recorded tool used. Run `cargo test -p athena-cli --test budgets` and see it pass.
5. [behavior] A second new test asserts the rendered line of every pre-existing default equals its pre-change line (listed verbatim in the test), so any change to an existing default's rendered text fails. Run `cargo test -p athena-cli --test budgets` and see it pass.
6. [behavior] The reader budget in crates/athena-context/src/lib.rs is raised only by the smallest round amount that fits, and no other budget changes. docs/system-facts.md, docs/design/context-engineer.md and the hard-coded figure in crates/athena-core/src/run_pin_tests.rs match the new figures. Run `cargo test` and see every_role_set_is_within_its_budget, system_facts_budgets_match_computed and the run_pin test pass.
7. [content] CHANGELOG.md has an entry for the two defaults that states the old and new reader budget and why it was raised. No other files change beyond the listed deliverables.
8. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/system-facts.md (modify)
- docs/design/context-engineer.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing mechanisms in the runner that enforce the version-pin or tool-path policies.
- Adding a bump command or changing claude_version handling; the existing `cargo run -p xtask -- repin` is only named.
- Changing athena.toml, CI workflows, or any budget other than the reader's.
- Splitting, shortening or dropping any existing default's rule or rendered text.
- Edits to docs/decisions.md history.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New render tests plus fmt, clippy and cargo test as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the rendered reader context contains the behavior phrases, not just the ids.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The new entries are marked prose-only, and the render and budget tests enforce their presence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: two entries, one budget raise and the figures that follow from it.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the root cause of the earlier failure: the render shows only the first sentence and the budget had no room.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and the changelog are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the entries or the changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e08b7bd52c48 → candidate 2a26ba018def. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            | 10 +++++
 crates/athena-cli/tests/budgets.rs      | 65 +++++++++++++++++++++++++++++++++
 crates/athena-context/src/lib.rs        |  2 +-
 crates/athena-core/src/run_pin_tests.rs |  2 +-
 defaults.toml                           | 12 ++++++
 docs/design/context-engineer.md         |  2 +-
 docs/system-facts.md                    |  2 +-
 7 files changed, 91 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-142640-egnb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-142640-egnb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42492699289 for 2a26ba018deff3124353785331044f45397a20fa)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42492699289 for 2a26ba018deff3124353785331044f45397a20fa)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42492699289 for 2a26ba018deff3124353785331044f45397a20fa)
- ci-result: `not a check` passed (Check results for 2a26ba018deff3124353785331044f45397a20fa come from the CI run https://github.com/doodla/athena/actions/runs/42492699289 (766s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
