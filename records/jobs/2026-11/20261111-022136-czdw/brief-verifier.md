# Job 20261111-022136-czdw: verify "Give OLY-8-05 a critic-challenge mechanism, test and spec entry (retry)"

## Request (verbatim)

> try again on 20261029-204942-2zqw

## Reading (accepted by the user)

Retry of job 20261029-204942-2zqw: put the "search for ways the conclusions might be wrong" rule in the versioned critic role prompt (crates/athena-context/prompts/critic.md, frame.critic version bump), test it, and record it as OLY-8-05's mechanism in docs/spec.toml with measure ending `status: planning`. The earlier wrong path prompts/critic.md is replaced by the real path.

Assumptions:
- Goal and audience: the spec reader and verifier can see that Ares (the critic stage) is tasked to search for ways plan conclusions might be wrong, via a versioned prompt rule, a test and a spec mechanism.
- Code survey: read crates/athena-context/src/lib.rs (CRITIC_FRAME = frame.critic v1 from crates/athena-context/prompts/critic.md; budget tests at ~lines 402-541 that check frame wording), crates/athena-core/src/run/planning.rs (critic_round composes CONTRACT + CRITIC_FRAME at line 338), crates/athena-context/src/briefs/planner.rs (critic_brief) and briefs.rs tests. No new record or data file is added, so check_record_file/AnyRecord/round-trip tests are unaffected. The property is measured directly: the composed critic prompt text contains the challenge rule, and frame version is bumped.
- Job 20261029-204942-2zqw findings addressed: (1) challenge rule was in the per-job brief, not the versioned frame: fixed by moving it to crates/athena-context/prompts/critic.md and bumping frame.critic to version 2, removing the brief-only section from planner.rs; (2) behavior evidence not tied to commands: producer must run and cite cargo fmt --check, cargo clippy --all-targets, cargo test; (3) test check failed on drain::parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn, which passed locally and does not touch this change: producer first runs cargo test -p athena-cli --test drain on base and change and reports; if it fails on base too, it is flaky and not fixed here; (4) `status: planning` is an indirect, shared measure: kept, as the accepted reading chose it, and stated as indirect in the measure text.
- Its candidate files are handled: CHANGELOG.md kept (entry updated); docs/spec.toml kept; crates/athena-context/src/briefs.rs kept with the test retargeted at the frame; crates/athena-context/src/briefs/planner.rs modified to drop the brief-only challenge section (or kept as a pointer if the test needs it).
- Prompt headroom: critic set has 345 tokens left; the added rule must be under ~100 tokens. Update any budget figure in docs/system-facts.md only if it states critic size.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-8-05 has mechanism naming existing paths (crates/athena-context/prompts/critic.md and critic_round in crates/athena-core/src/run/planning.rs) instead of "none", keeps owner = "Ares", and its measure ends with `status: planning`; Grep confirms the paths exist and that `athena status` prints a planning line.
2. [content] crates/athena-context/prompts/critic.md contains a rule telling the critic to search for ways the plan's conclusions might be wrong (counterexamples, fragile assumptions, failure modes), and CRITIC_FRAME's version in crates/athena-context/src/lib.rs is bumped from 1 to 2; git diff shows both.
3. [behavior] A new test in crates/athena-context (briefs.rs or lib.rs tests) asserts the composed critic prompt contains the challenge rule; run `cargo test -p athena-context` to see it pass, and it fails when the rule is removed from critic.md.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` all pass, including the prompt token budget tests; run each command. `cargo test -p athena-cli --test drain` is run on the change and any failure is checked against the base.
5. [content] CHANGELOG.md has an entry for the change; no governance file (athena.toml, defaults.toml, workflows) changes; docs/system-facts.md changes only if it states a critic prompt budget figure that moved.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- Creating a new Ares role or prompt
- Changing other requirements' mechanisms
- Changes to athena.toml, defaults.toml or workflows
- Fixing the flaky drain test unless it fails on the base and blocks this job

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the critic prompt rule and test, not just file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test required.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 50df4967e5a6 → candidate 92366cd01788. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  2 ++
 crates/athena-context/prompts/critic.md |  1 +
 crates/athena-context/src/briefs.rs     | 13 +++++++++++++
 crates/athena-context/src/lib.rs        |  2 +-
 docs/spec.toml                          |  4 ++--
 docs/system-facts.md                    |  2 +-
 6 files changed, 20 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-022136-czdw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-022136-czdw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42948424550 for 92366cd01788cdde3dc04a28dba227fd873611ea)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42948424550 for 92366cd01788cdde3dc04a28dba227fd873611ea)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42948424550 for 92366cd01788cdde3dc04a28dba227fd873611ea)
- ci-result: `not a check` passed (Check results for 92366cd01788cdde3dc04a28dba227fd873611ea come from the CI run https://github.com/doodla/athena/actions/runs/42948424550 (694s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
