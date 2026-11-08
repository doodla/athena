# Job 20261108-004822-e16j: verify "Retry: trim change/study verifier and study producer prompts safely"

## Request (verbatim)

> try again on 20261024-232057-r4pj

## Reading (accepted by the user)

Retry of job 20261024-232057-r4pj: trim brief-repeating text from the change verifier, study verifier and study producer prompt sets so each is at least 100 tokens under 1300 (≤1200), keeping every phrase tests pin (e.g. the five verifier-frame snapshot phrases at crates/athena-context/src/lib.rs:531-541), and record only verified claims in decisions/changelog/system-facts.

Assumptions:
- Job 20261024-232057-r4pj is addressed finding by finding: (1) the blocker/test failure from removing snapshot wording is fixed by keeping verbatim all five phrases asserted by the_verifier_frame_limits_snapshot_runs_to_the_snapshot ("a snapshot, not the real repository", "no records/, no owner files, no history", "a result about the snapshot only", "say so explicitly in evidence and findings", "never report it as a result on the real repository") and cutting elsewhere, e.g. the change and study verifier/producer capability prompts, which already free most of the room; (2) the D166 misstatement is fixed by writing the new decisions entry only from what was run and by not claiming the producer brief states that Athena reruns checks; (3) the behavior criteria name the commands to be run (cargo test --locked, clippy) so evidence cites commands that ran.
- Code survey: read crates/athena-context/src/lib.rs (prompt constants and frame tests such as the verifier-frame snapshot test at line 531 and the critic frame test), and the prompts under crates/athena-context/prompts, crates/athena-cap-change/prompts and crates/athena-cap-study/prompts; prompt sizes are measured by crates/athena-cli budgets test; no records or data files are added, so check_record_file/AnyRecord are not involved. The producer should grep crates/ for string literals in tests that assert prompt text before each cut.
- The earlier candidate's files are kept as a starting point only in intent: CHANGELOG.md, both verifier.md files, athena-context producer.md and verifier.md, decisions.md, system-facts.md are all deliverables again; the retry starts from base, so those edits are redone, with the athena-context verifier.md snapshot bullet restored to its pinned wording.
- The study producer measured 1257 tokens, not the 1276 in the request; the target is at most 1200 for the change verifier, study verifier and study producer, budgets unchanged at 1300.
- Audience: the repo maintainers and the verifier who check that prompts stay lean and tests pass.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes and the printed sizes for the change verifier, study verifier and study producer are each at most 1200, with budget values unchanged.
2. [behavior] Run `cargo test --locked -p athena-context --lib` then `cargo test --locked` for the whole workspace: all pass, including the_verifier_frame_limits_snapshot_runs_to_the_snapshot, and `cargo clippy --locked --all-targets` is clean.
3. [content] `git diff` shows no test file (*.rs under tests/ or #[cfg(test)] code) modified; the diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md.
4. [content] crates/athena-context/prompts/verifier.md still contains verbatim the five phrases asserted at crates/athena-context/src/lib.rs:534-538, and every other phrase asserted by tests for the edited prompts is still present (checkable with Grep against test literals).
5. [content] docs/decisions.md has a new entry listing each removed passage or group with its file and why it was removable, making no claim about tests or brief contents beyond what the diff and the test run show; a reader can match each bullet to the diff.
6. [content] CHANGELOG.md has an entry describing the prompt trim and the new headroom; docs/system-facts.md budget lines equal the computed sizes (the system_facts_budgets_match_computed test passes).
7. [content] No prompt rule is added or changed in meaning; removed text only repeats brief content or backs no current rule or test; nothing secret or personal is committed (standing defaults: outcome-check, behavior-evidence, small, public, strangers).

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/contract.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Changing any budget value, any test file, athena.toml or defaults.toml
- Adding new rules or instructions to the prompts
- Trimming the reader, planner or critic prompts beyond what shared frames imply
- Adding new checks or tooling for prompt-edit jobs (a prior prevention already tried)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require running budgets, workspace tests and clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check sizes and test passes, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt trim.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Named commands for verifier.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e46b94671da1 → candidate ef8941e5ff02. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  4 ++++
 crates/athena-cap-change/prompts/verifier.md |  4 ++--
 crates/athena-cap-study/prompts/producer.md  | 12 +++++-----
 crates/athena-cap-study/prompts/verifier.md  |  6 ++---
 crates/athena-context/prompts/producer.md    |  3 +--
 crates/athena-context/prompts/verifier.md    | 10 ++++-----
 docs/decisions.md                            | 33 ++++++++++++++++++++++++++++
 docs/system-facts.md                         |  4 ++--
 8 files changed, 55 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-004822-e16j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-004822-e16j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431788507 for ef8941e5ff0279500e055acc92a924b1df95b7f9)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431788507 for ef8941e5ff0279500e055acc92a924b1df95b7f9)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431788507 for ef8941e5ff0279500e055acc92a924b1df95b7f9)
- ci-result: `not a check` passed (Check results for ef8941e5ff0279500e055acc92a924b1df95b7f9 come from the CI run https://github.com/doodla/athena/actions/runs/42431788507 (626s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
