# Job 20261018-073510-m3yh: Mark base-failing checks pre-existing and judge each criterion anyway

## Request (verbatim)

> At verification, a check that already failed on the base commit (recorded before the producer started, part 1) is marked pre-existing instead of rejecting the job, and each acceptance criterion is judged on its own instead of leaving them all unjudged when one unrelated check fails (part 2 of 2; run after part 1 lands).

## Reading (accepted by the user)

At verification, a check that failed in the job's baseline record (same name, failed on the base commit) is marked pre-existing and no longer fails the verdict. The verifier also runs when some other check fails, so every acceptance criterion gets its own result instead of none. Audience: the Athena operator reading verification records and verdicts.

Assumptions:
- Part 1 landed: the Baseline record is taken before the producer and held in closing.baseline in run.rs.
- A check is pre-existing only if the baseline has an entry with the same name and passed=false.
- The verifier now runs even when a non-pre-existing check fails, so criteria are judged; the final verdict still fails on that check.
- Prefer using the check note over a new CheckRun field, to avoid a schema change under deny_unknown_fields.
- No athena.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] A unit test in verify.rs shows judge() passes when a check failed in the baseline and fails again at verification, with the failure recorded as pre-existing and not listed in the verdict's failure reasons. Run `cargo test -p athena-core verify`.
2. [behavior] A unit test shows a check that passed in the baseline (or has no baseline entry) and fails at verification still makes the final verdict fail, with a reason naming it. Run `cargo test -p athena-core verify`.
3. [behavior] A lifecycle test shows that when one check fails, the verifier still runs and every acceptance criterion in the verification record has its own result, while the final verdict still fails on that check. Run `cargo test -p athena-cli --test lifecycle`.
4. [behavior] A lifecycle test shows a job whose base commit already fails a check, and whose change leaves it failing, can pass verification when everything else passes. Run `cargo test -p athena-cli --test lifecycle`.
5. [content] The verification record marks pre-existing failures distinctly (for example in the check note), so a reader sees which failures were already on the base.
6. [content] A missing baseline, or one with a note saying it could not run, treats no failure as pre-existing. An empty baseline is never read as success.
7. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass with no new warnings.
8. [content] CHANGELOG.md records the new behavior in plain language for an outside reader.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how the baseline is taken (part 1).
- Re-running checks or retrying flaky ones.
- Changing landing, CI routing or budget logic.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and lifecycle tests; cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the verdict outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the verdict computation and the verifier-skip condition.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A missing or empty baseline marks nothing pre-existing.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across verify.rs and run.rs.
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
