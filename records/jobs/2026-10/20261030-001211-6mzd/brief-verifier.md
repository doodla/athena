# Job 20261030-001211-6mzd: verify "Give OLY-B-SET-02 a model-roster mechanism and test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-SET-02 a mechanism
> 
> Requirement OLY-B-SET-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Work uses Opus, Sonnet and Haiku, plus Jev through the Vercel AI Gateway for fast, cheap structured decisions.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-SET-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-SET-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a checked model roster (Opus, Sonnet, Haiku via configured roles, plus Jev via the Vercel AI Gateway constants in jev.rs) with a test that fails without it, then record it as the mechanism and measure for OLY-B-SET-02 in docs/spec.toml. Audience: Athena's owner and the spec gap check.

Assumptions:
- Code survey: read crates/athena-core/src/jev.rs (ENDPOINT, MODEL, KEY_VAR, shadow-only), crates/athena-core/src/status.rs (status entry point), and model names in crates/athena-core/src/reading.rs and run tests; no new record or data file under crates/ is added, so check_record_file/AnyRecord are unaffected.
- Existing code: crates/athena-core/src/jev.rs and docs/spec.toml (other Jev requirements already use `athena score-jev`); the change extends jev.rs constants with a roster.
- The measure is `status: not shown` plus the test, since no `athena status` line shows the roster.
- Jev's existing shadow-only rule stays; the requirement is met by the roster existing, not by Jev deciding work.
- Consumers: none found for config; athena.toml and defaults.toml are not changed.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-SET-02 has a mechanism other than "none" naming the roster code path and test, owner Athena, and a measure ending with `status: not shown` and the test command.
2. [behavior] Run `cargo test -p athena-core` (the new roster test): it asserts the roster holds Opus, Sonnet and Haiku model names and Jev with the ai-gateway.vercel.sh endpoint; it fails when the roster entry for any of the four is removed and passes with the change.
3. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass; the xtask spec test (`cargo test -p xtask --test spec`) still passes with the edited spec.toml.
4. [content] Existing Jev constants (ENDPOINT, MODEL in crates/athena-core/src/jev.rs) are reused, not duplicated; no secrets are written.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/model_roster.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Making Jev decide real work (it stays shadow-only)
- Changing athena.toml, defaults.toml or workflows
- Other requirements such as OLY-B-SET-04

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criterion
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion runs the roster test that fails without the mechanism
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One roster module and one spec edit
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 10450d9f2758 → candidate ab9185fcc5e0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  3 +++
 crates/athena-core/src/jev.rs            | 41 ++++++++++++++++++++++++++++++++
 crates/athena-core/tests/model_roster.rs | 21 ++++++++++++++++
 docs/spec.toml                           |  4 ++--
 xtask/shard-timings.txt                  |  1 +
 5 files changed, 68 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-001211-6mzd/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001211-6mzd/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43141115910 for ab9185fcc5e00e85ff9a0b452199ee9a87dc74fe)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43141115910 for ab9185fcc5e00e85ff9a0b452199ee9a87dc74fe)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43141115910 for ab9185fcc5e00e85ff9a0b452199ee9a87dc74fe)
- ci-result: `not a check` passed (Check results for ab9185fcc5e00e85ff9a0b452199ee9a87dc74fe come from the CI run https://github.com/doodla/athena/actions/runs/43141115910 (696s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: planned files left unchanged without a reason: crates/athena-core/src/lib.rs)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
