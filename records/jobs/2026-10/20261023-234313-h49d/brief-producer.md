# Job 20261023-234313-h49d: Stop requiring a full cargo test run from the producer

## Request (verbatim)

> Producers rerun the full cargo test suite (9 to 11 minutes) many times: job 20261019-013309-nvbb's producer ran it six times and hit its 1800 s deadline, and the producer brief sends a report back without 'a real full cargo test --locked result' (job 20261023-165947-e7bd). Athena runs fmt, clippy and the full suite on the candidate itself after the producer, so the producer should run targeted tests while working and need not report a full-suite run; a send-back should not demand one. Keep any check that would let a failing candidate through.

## Reading (accepted by the user)

The producer prompt tells producers to run the full `cargo test --locked`, and run.rs sends a report back (once) without a real result for it. Remove both, tell producers to run targeted tests, and keep the post-producer fmt, clippy and full-suite checks that Athena runs on the candidate. The audience is Athena operators and producers, who lose 9 to 11 minutes per full run.

Assumptions:
- Athena already runs fmt, clippy and the full test suite on the candidate and a failure fails it, as the request states; I did not trace that code path in full.
- The send-back mechanism for unmet acceptance criteria stays; only the full-test demand is removed.
- Pinned study jobs were already exempt, so their behavior is unchanged.
- A new decision entry (next free number after the highest in docs/decisions.md) is appropriate because this reverses part of D95; D95 itself is not rewritten.
- The producer may still report a full-suite run if it chooses to; the report schema keeps checks_run.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer tells the producer to run the full `cargo test --locked` or to report it. It says to run targeted tests (`cargo test -p <crate> <filter>`) while working and that Athena runs fmt, clippy and the full suite on the candidate.
2. [content] In crates/athena-core/src/run.rs the producer loop no longer sends a report back for lacking a real full `cargo test --locked` result. The `ran_full_tests` and `tests_missing` gating and the sent-back test text are removed. The send-back for unmet acceptance criteria is kept.
3. [content] The checks Athena runs on the candidate (fmt, clippy, full test) and their effect on the final verdict are unchanged. A failing check still fails the candidate. The producer-report-gaps note to the verifier no longer tells it to run the full suite, and the verifier prompt still says to cite recorded results.
4. [behavior] Run `cargo test --locked -p athena-cli --test lifecycle`. The tests show that a producer report with no full-suite check, or with `not-run`, goes to the verifier without a send-back. A report with unmet criteria is still sent back once. A candidate whose recorded test check fails is still rejected.
5. [behavior] Run `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean, with no unused-code warnings left by the removed gating.
6. [content] docs/decisions.md records the change as a new decision that amends D95 and its D95 follow-up wording, and docs/design/athena-v0.md and CHANGELOG.md [Unreleased] no longer say the producer must run the full suite. The documents state the reason with the figures from the two jobs (nvbb: six runs, 1800 s deadline; 9 to 11 minutes per run).

## Expected deliverables

- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-context/src/outputs.rs (modify)

## Out of scope

- Changing verifier behavior or its prompt beyond the gaps note
- Changing athena.toml timeouts or budgets
- Changing which checks Athena runs on the candidate
- Changing the send-back for unmet acceptance criteria

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Lifecycle and job_plan tests are updated for the new behavior; cargo test and clippy must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The demand is removed in both the prompt and the run.rs enforcement, not only the wording of one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A criterion runs the lifecycle tests and checks that no send-back happens and that failing candidates are still rejected.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The full-suite gate stays enforced by Athena's own post-producer check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and CHANGELOG text states the reason and the measured run times.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the producer test requirement.
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
