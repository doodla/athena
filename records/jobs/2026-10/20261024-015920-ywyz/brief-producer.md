# Job 20261024-015920-ywyz: Stop requiring producers to run the full test suite

## Request (verbatim)

> try again on 20261023-234313-h49d

## Reading (accepted by the user)

Redo job h49d from the base: remove the producer's full-suite requirement and the run.rs send-back for it, keep Athena's post-producer checks, and amend every doc that states the old rule. The old CHANGELOG D95 entry stays with a note that D113 supersedes it, and the producer runs the full suite at most once.

Assumptions:
- Goal: producers stop rerunning the 9 to 11 minute full suite; audience is Athena operators and producers, and the verifier.
- The code and prompt change is redone as the last attempt made it, since changes were discarded; only the docs differ (CHANGELOG D95 entry, D103/D95 pointers).
- D113 is the next free decision number, as in the last attempt.
- The older CHANGELOG D95 entry is kept with a supersession note, per the user's reply.
- The producer runs the full suite at most once, per the user's reply.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer tells the producer to run or report the full `cargo test --locked`. It says to run targeted tests (`cargo test -p <crate> <filter>`) while working and that Athena runs fmt, clippy and the full suite on the candidate.
2. [content] In crates/athena-core/src/run.rs the producer loop no longer sends a report back for lacking a real full-suite result. The `ran_full_tests` and `tests_missing` gating and its send-back text are removed. The send-back for unmet acceptance criteria is kept. The producer-report-gaps note no longer tells the verifier to run the full suite.
3. [content] The checks Athena runs on the candidate (fmt, clippy, full test) and their effect on the verdict are unchanged. The verifier prompt still says to cite recorded results.
4. [behavior] Run `cargo test --locked -p athena-cli --test lifecycle` and `cargo test --locked -p athena-cli --test job_plan`. A producer report with no full-suite check, or with `not-run`, goes to the verifier without a send-back. A report with unmet criteria is still sent back once. A candidate whose recorded test check fails is still rejected.
5. [behavior] Run `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean, with no unused-code warnings from the removed gating.
6. [content] docs/decisions.md has a new decision, D113, amending D95 and D103. The D95 and D103 entries each get a short pointer forward to D113. D113 states once that the earlier text stays as history, and gives the reason with the figures: job nvbb ran the suite six times and hit its 1800 s deadline, and a run takes 9 to 11 minutes.
7. [content] CHANGELOG.md [Unreleased]: the older D95 entry (near line 624) is kept, not deleted, and gets a short note that D113 supersedes it. A D113 entry gives the same reason and figures. docs/design/athena-v0.md no longer says the producer must run the full suite.
8. [content] A grep of decisions.md, athena-v0.md and CHANGELOG.md for 'full `cargo test --locked`', 'second miss' and 'ran_full_tests' finds no statement of the requirement outside quoted history or the supersession notes. The criterion evidence cites that search command and its output.

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

- Changing the post-producer fmt, clippy and full-suite checks or the verdict logic
- Adding a new pre-handover wording-search gate
- Deleting the old D95 entries

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Behavior criteria run the lifecycle and job_plan tests, clippy and fmt.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria exercise the send-back and rejection paths.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the behavior and wording, and name the grep to cite.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix every place the old rule is stated, not just the CHANGELOG line.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Docs cite job figures: six runs, 1800 s, 9 to 11 minutes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the stated files change.
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
