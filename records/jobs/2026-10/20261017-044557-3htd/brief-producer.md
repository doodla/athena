# Job 20261017-044557-3htd: Route checks by job kind per ci-routing section 3.2

## Request (verbatim)

> Build item 5 of docs/design/ci-routing.md section 10: route each job's builds and tests by job kind (the section 3.2 table), after item 4's result reuse lands.

## Reading (accepted by the user)

Add a routing function in athena-core that maps a plan's class_of_work to where each check runs (CI-reused versus local) and what the verifier is told, following the 3.2 table. Unknown or missing kind takes the config row. Audience: the Athena maintainer and reviewers of ci-routing.md.

Assumptions:
- Item 4 is only built in part (resolve exists but is not called from the job run); item 5 builds on that module and wires routing to the job-run path rather than waiting for more of item 4.
- Job kind is the plan's class_of_work, taken from the reading.
- The producer's edit loop is unchanged; routing moves Athena's checks and the verifier's test run only.
- The verifier's allowed commands stay unchanged (section 3.3).
- Measurement, cache timing and README/CHANGELOG are later items (6, 7).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core routing`: tests assert, for each kind (code, tests, docs, design, config, build, study, other), the routed checks and their skips match the section 3.2 table (docs and design skip fmt/clippy but always run test; config never skips; build adds the workflow's own run and never reuses a prior result).
2. [behavior] A test shows a missing or unrecognised job kind gets the config row (all three checks, none skipped), per section 7.
3. [behavior] A test shows the routed path calls the item 4 `resolve` with the candidate SHA: a CI result for another SHA is ignored and falls back to a local run; a red CI result fails and does not rerun locally.
4. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
5. [content] docs/design/ci-routing.md section 10 item 5 is updated to say what was built and what is not done (for example no speedup measured), without claiming a measured improvement.
6. [content] The routing is used from the job run or verifier input, not only defined: a reader can find the call site that selects checks by the plan's kind.

## Expected deliverables

- crates/athena-core/src/routing.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/design/ci-routing.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Measurement job (item 6) and the docs job (item 7)
- Changes to .github/workflows/ci.yml or the verifier's allowed commands
- Pushing the candidate commit before the read and the wait-bound value
- Claiming any speedup

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests in Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-kind routing tests; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check routing outcomes per kind, not that a module exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One routing module plus wiring and a design-doc status note.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria tie to checks running in the right place with the right fallback.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
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
