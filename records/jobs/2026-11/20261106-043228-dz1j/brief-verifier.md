# Job 20261106-043228-dz1j: verify "Show failed tests and panic text in introspection failed checks"

## Request (verbatim)

> In crates/athena-core/src/introspect.rs, the failed-check section of the introspection input (about line 997) passes clip_tail(output_tail, TAIL_CAP = 600). For a reused CI result, the last 600 characters are GitHub's post-job cleanup text and a Node.js 20 warning. The input also leaves out the check's failed_tests list. As a result, three introspections (jkfa, hb84, j07h) wrongly said the failing test was unknown. They then proposed building failing-test capture, which already exists: the verification.json files for vdtn, h7de and jn5t each hold failed_tests and the panic text. Fix: for each failed check, include its failed_tests names, and build the excerpt from the cargo `failures:` section (or the text before the runner's post-job cleanup lines), not from the raw tail. Acceptance: (1) a test feeds a check whose output_tail ends in CI cleanup lines and whose failed_tests is non-empty, and asserts the input names each failed test and its panic line, with no cleanup lines; (2) a check with no failed_tests still shows a tail; (3) cargo fmt --check, cargo clippy --all-targets and cargo test pass.
> 
> Acceptance:
> - The change is made as described: Introspection sees only the last 600 characters of a failed check's output, which is CI cleanup text, so it reports that no test is named when the record names it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/introspect.rs, verification.json

## Reading (accepted by the user)

In introspect.rs, each failed check in the introspection input will list its failed_tests names. Its excerpt will come from the cargo `failures:` section, or from the output before the CI post-job cleanup lines, instead of the raw last 600 characters. The audience is the introspection model, which wrongly said no failing test was named.

Assumptions:
- Code survey: read crates/athena-core/src/introspect.rs (TAIL_CAP at 949, clip_tail at 962, failed-check section at 1109-1122, test helper at ~2143 building a check with failed_tests empty) and the check type in crates/athena-core/src/ci_result.rs (failed_tests: Vec<String> at line 98, output_tail at 100). No record or data file is added, so check_record_file and AnyRecord are unaffected. The property is measured directly by rendering the input text and asserting on it.
- Cleanup markers are GitHub runner lines such as 'Post job cleanup.' and the 'Node.js 20 actions are deprecated' warning. The producer picks a small marker list. If there is no `failures:` section and no marker, fall back to the old clip_tail behaviour.
- The request lists verification.json as a file, but it is a record. It is not changed. The only deliverable is introspect.rs, plus the default CHANGELOG.md entry.
- No governance files, workflows or prompts change. The introspection system prompt is untouched.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/introspect.rs builds a failed check whose output_tail has a cargo `failures:` section with a panic line, then CI cleanup lines (post-job cleanup and a Node.js 20 warning), and whose failed_tests is non-empty. It asserts the rendered input names each failed test and its panic line and contains no cleanup lines. Run `cargo test -p athena-core introspect`.
2. [behavior] A new unit test shows a failed check with empty failed_tests and plain output still renders its tail, clipped to TAIL_CAP. Run `cargo test -p athena-core introspect`.
3. [content] The failed-check loop in introspect.rs no longer passes the raw output_tail to clip_tail alone. It prints the failed_tests names and builds the excerpt from the `failures:` section, or from the text before the cleanup lines. A diff line shows this.
4. [behavior] The existing introspect tests pass, including the one at ~3042 that checks the tail is clipped to TAIL_CAP. Run `cargo test`.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test all pass. Athena's recorded checks on the final tree count as evidence.
6. [content] CHANGELOG.md has a new entry for this change, and no other file under records/ or docs/decisions.md is edited.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how CI results are captured or stored (ci_result.rs, verification.json records)
- Changing the introspection system prompt or other sections of the input (criteria, findings)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus passing fmt, clippy and cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert on the rendered input text, not on wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bafec8c1dfc3 → candidate d09fc7b95be8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  1 +
 crates/athena-core/src/introspect.rs | 82 +++++++++++++++++++++++++++++++++++-
 2 files changed, 81 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-043228-dz1j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-043228-dz1j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43890325537 for d09fc7b95be8b293f1b1f4b678d0c3b3618cac0f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43890325537 for d09fc7b95be8b293f1b1f4b678d0c3b3618cac0f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43890325537 for d09fc7b95be8b293f1b1f4b678d0c3b3618cac0f)
- ci-result: `not a check` passed (Check results for d09fc7b95be8b293f1b1f4b678d0c3b3618cac0f come from the CI run https://github.com/doodla/athena/actions/runs/43890325537 (804s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test all pass. Athena's recorded checks on the final tree count as evidence.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
