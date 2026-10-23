# Job 20261019-013309-nvbb: Tell verifiers to reuse Athena's recorded fmt/clippy/test results

## Request (verbatim)

> Let the verifier reuse the check results Athena already recorded for the candidate (fmt, clippy, test) instead of rerunning the full test suite, which now takes about 11 minutes and pushed two verifiers past their time limit on 2026-10-18.

## Reading (accepted by the user)

The verifier brief already lists the checks Athena ran, but nothing stops verifiers rerunning the full `cargo test` (often because a "cargo test passes" behavior criterion cites it), which took ~11 min and hit the time limit on 2026-10-18. I read the goal as making the verifier rely on the recorded fmt, clippy and test results for the candidate's SHA and run only targeted tests for new behavior. The audience is Athena's verifiers and the operator who reads their verdicts.

Assumptions:
- Athena already records fmt, clippy and test results for the candidate (the brief section 'Checks Athena ran on this tree'; ci_result::CiRecord and verify::reuse_statement, which only yields text when the SHA matches); the gap is that verifiers are not told to rely on them and criteria still cite `cargo test`.
- I could not find a caller of reuse_statement outside verify.rs, so wiring it into the brief is possible but is kept optional (low probability) to keep the change small.
- The verifier command allow-list keeps `cargo test` so targeted tests stay possible.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [content] The verifier brief's 'Checks Athena ran on this tree' section tells the verifier that fmt, clippy and test results there are recorded for the candidate and must not be rerun, and that the full `cargo test` suite is not to be rerun.
2. [content] The verifier prompt (crates/athena-context/prompts/verifier.md, and the change prompt if it conflicts) says to cite the recorded check result for a criterion such as 'cargo test passes', and to run only targeted tests (for example `cargo test -p <crate> <filter>`) to exercise new behavior.
3. [content] The reader prompt tells readers not to write a behavior criterion that requires rerunning the full suite, fmt or clippy; such outcomes are content criteria satisfied by the recorded checks. Nothing in the change makes a failed or missing recorded check count as passed.
4. [behavior] A briefs.rs test renders a verifier brief and asserts the no-rerun instruction appears next to the checks section; running `cargo test -p athena-context` passes.
5. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean, and `cargo test -p athena-context` passes with existing brief snapshot tests updated deliberately.
6. [content] The change is a focused edit with no secrets or personal data. CHANGELOG.md notes the change and names the 2026-10-18 timeouts as the reason.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/verify.rs (modify)

## Out of scope

- Changing athena.toml timeouts or budgets
- Speeding up or splitting the test suite itself
- Changing how or when Athena runs the checks
- Letting a verifier reuse results recorded for a different SHA
- Changes to the study capability's verifier

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that verifiers actually stop rerunning the suite and that a failed recorded check still fails the verdict, not just that text exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A briefs.rs test covers the new instruction; cargo test and clippy must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the cause (verifiers and readers pushing full-suite reruns) rather than raising the timeout.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Prompt and brief text edits only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The behavior criterion runs the brief-rendering test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier checks the rendered brief text via tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Wording and changelog explain the reason for an outside reader.
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
