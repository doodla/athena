# Job 20261018-070117-qmxv: Baseline the checks before the producer; mark failures pre-existing

## Request (verbatim)

> Before the producer starts, run the job's checks on the base commit and record which ones already fail. At verification, mark those as pre-existing instead of rejecting the job. Judge each acceptance criterion on its own instead of leaving them all unjudged when one unrelated check fails.

## Reading (accepted by the user)

Athena will run the capability's checks on the base commit before the producer starts and record which fail. At verification, a check that failed on the base and still fails is marked pre-existing and no longer blocks the job. The verifier is then run and judges every acceptance criterion even when an unrelated check fails. Audience: Athena's maintainer, and verifiers reading the check results.

Assumptions:
- The baseline runs the same routed checks (crates/athena-core/src/routing.rs) in a snapshot of the base commit, before the first producer invocation in run.rs; a nested run skips it, as it already skips checks.
- A check is pre-existing only if it failed on the base and fails again on the candidate, matched by name. A check that passed on the base and fails on the candidate is a real failure and still blocks.
- Today verification runs the verifier only when every check passed (`if all_passed` in run.rs). That gate changes. When only non-pre-existing failures remain, the job still ends rejected, but the verifier runs and gives a verdict on each criterion. The verdict reaches the findings.
- The baseline result is stored with the job's records, so it survives a restart and appears in the verifier brief as a labelled note on each check.
- The user wants no new config key unless one is needed; the baseline always runs.
- A baseline run that cannot start is recorded as 'baseline unavailable'. No check is then treated as pre-existing.

## Acceptance criteria

1. [behavior] A test using the fake agent and a check that fails on the base shows the base failure recorded before the producer's first invocation. Run `cargo test` and read the test.
2. [behavior] A test shows a check failing on both base and candidate is marked pre-existing, in the verifier brief and in the check record. The job is not rejected for that check alone. Run `cargo test`.
3. [behavior] A test shows a check that passes on the base and fails on the candidate is not marked pre-existing and still blocks the job. Run `cargo test`.
4. [behavior] A test shows the verifier runs and returns a verdict for every acceptance criterion when an unrelated non-pre-existing check fails. The failing check is still reported and the job is rejected. Run `cargo test`.
5. [behavior] A job whose base has no failing checks behaves as before: all checks run, and nothing is marked pre-existing. Run `cargo test`.
6. [content] The check record or verification record carries a pre-existing marker and a note saying the check also failed on the base commit. The records are documented in docs/ and CHANGELOG.md.
7. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass. Each new or changed module has tests.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/design.md (modify)

## Out of scope

- Fixing the checks that already fail on the base.
- Changing which checks a capability defines, or the skip_when_only rules.
- Changing the verifier's budget or the model's grants.
- Re-running the baseline after a send-back or retry.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy and fmt passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check pre-existing marking and per-criterion judging by running tests, not by the existence of fields.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the `all_passed` gate that withholds judgment, not one failing check.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the run and check path, with no new config.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in code and tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are in Rust.
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
