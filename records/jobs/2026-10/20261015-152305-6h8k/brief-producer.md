# Job 20261015-152305-6h8k: Count only outcome-changing landings; make drain's source check semantic

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

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
