# Job 20261015-152305-6h8k: verify "Count only outcome-changing landings; make drain's source check semantic"

## Request (verbatim)

> Drain reported 'target met' after landing 20261015-141831-hyb4, a defaults change that did not touch the explainer, and its check against the pinned source passed a page that an independent reader found wrong on three facts (teardown can reopen instead of close, yolo-on self-merge, which handoff frees a place). Only a landing that changed the outcome may count, and the source check must catch statements the source contradicts or leaves out.

## Reading (accepted by the user)

Two faults in `athena drain` (crates/athena-cli/src/drain.rs). (1) A landing that did not change the explainer's content still allows "target met"; only a landing whose diff touches what the page is built from may count. (2) The pinned-source check passes pages with wrong facts because it only tests the study models for self-consistency; it must compare the page's statements with the pinned source and fail on any statement the source contradicts or omits. The audience is the person running drain who trusts "target met".

Assumptions:
- "Changed the outcome" means the landed job's diff touches a path the live explainer is generated from (studies/**, xtask/src/site.rs, the pages workflow, explainer docs); the path list is one named constant, so it is easy to adjust.
- A landing that did not change the outcome still counts as landed for the report, but a rating after only such landings cannot satisfy the target.
- Catching contradictions and omissions needs reading the pinned source (the commit in studies/*/pin.json), so the source check becomes a capped model call (budget-guarded, paid from the verifier cap) over the page's statements and the source, in addition to the existing deterministic model_contradictions floor. This reverses alternative (a) rejected in D99, and the new decision says so.
- The pinned source is obtained the way studies mount it (core pin machinery); tests use stubs only.

## Acceptance criteria

1. [behavior] With a stubbed Steps, a drain whose only landing changed nothing the explainer is built from (the job's changed paths are outside studies/, the site generator and the page sources) and whose rating reaches the target does not stop as "target met"; it keeps running or stops for another reason, and the report says the landing did not change the outcome. Run `cargo test -p athena-cli --test drain`.
2. [behavior] With a stub, a landing whose changed paths include a study file or the site generator, followed by a rating at the target and a passing source check, still stops as "target met" (the happy path is not broken). Run `cargo test -p athena-cli`.
3. [behavior] A regression test feeds the source check a page statement that the pinned source contradicts (teardown can reopen instead of close; yolo-on allows self-merge; the wrong handoff frees a place) and another that leaves out a fact the source states; the check fails for each and names the statement, so the rating cannot end the run. Run `cargo test -p athena-cli`.
4. [behavior] The model-backed source check is guarded by the budget like other model calls: when its cap does not fit, drain stops as "budget spent" instead of counting the target as met, and its cost is added to spent_usd. Run `cargo test -p athena-cli`.
5. [content] `Stop::TargetMet` meaning, the module docs, `athena drain --help`, and a new decision in docs/decisions.md state both rules and the alternatives (the old self-consistency check is kept as a floor), and CHANGELOG.md records the change.
6. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass for the workspace.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/pin.rs (modify)

## Out of scope

- Fixing the three wrong facts on the live explainer page itself or re-running the study.
- Changing the audience rating prompt or the target rating default.
- Reverting or reworking job 20261015-141831-hyb4 (its nit about §3.7 wording and the reader budget raise).

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the two gaps in the stop rule (what counts as a landing, what the source check can catch), not just the three facts that surfaced.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and integration tests in drain.rs and tests/drain.rs cover both rules; fmt, clippy, test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run drain against stubs and check the stop reason and report, not that functions exist.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A decision entry with alternatives is added to docs/decisions.md because the source check changes shape.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Both rules are enforced in the drain loop, not by prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing pin/mount machinery and model-call plumbing instead of a new fetcher.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 168829ad7060 → candidate ddf03b7df2f4. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  14 ++
 README.md                        |  10 +-
 crates/athena-cli/src/display.rs |  12 ++
 crates/athena-cli/src/drain.rs   | 340 +++++++++++++++++++++++++++++++++++----
 crates/athena-cli/src/main.rs    |  24 ++-
 crates/athena-cli/tests/drain.rs | 211 +++++++++++++++++++++++-
 crates/athena-core/src/pin.rs    | 241 +++++++++++++++++++++++++++
 docs/decisions.md                |  46 ++++++
 8 files changed, 854 insertions(+), 44 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (693 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
