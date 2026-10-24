# Job 20261024-045548-7qht: Count Athena-recorded passing same-tree checks as seen under D43

## Request (verbatim)

> Since 88562f1, change verifiers cite Athena's recorded fmt, clippy and test results instead of rerunning them, but D43 (crates/athena-core/src/verify.rs, around line 335) still marks a behavior criterion unmet unless its evidence cites a command the verifier was seen running, so job 20261024-033933-ta60 was rejected on 'cargo test passes for the workspace' while Athena's own recorded test run on the same candidate passed. Count a command as run when Athena itself ran and recorded it on the same candidate tree and it passed (the checks shown under 'Checks Athena ran on this tree'), so citing it satisfies D43; a recorded check that failed, or one recorded for a different tree, still does not. Test both cases and update D43's documentation.

## Reading (accepted by the user)

Change `judge` in verify.rs so a behavior criterion's cited command counts as run if the verifier ran it, or Athena ran and recorded it on the same candidate tree and it passed. Failed or other-tree records still fail D43. Both cases are tested and D43 is documented as amended. The audience is the verifier and the maintainers who read D43.

Assumptions:
- The user's reply applies to this job: the workspace test/clippy/fmt criterion is a content criterion, judged by Athena's recorded checks; everything else from the earlier reading stays.
- D43 is amended through a new decision rather than rewriting D43's history.
- Recorded checks are matched to cited commands by command text, and same-tree means the recorded tree equals the candidate tree.

## Acceptance criteria

1. [behavior] A unit test shows a behavior criterion citing `cargo test` is met, with commands_verified true and no D43 finding, when the verifier ran nothing but a passing `cargo test` check is recorded for the same candidate tree. Run `cargo test -p athena-core verify`.
2. [behavior] A unit test shows the criterion is still unmet, with the D43 'cited but not seen running' finding, when the recorded `cargo test` check failed. Run `cargo test -p athena-core verify`.
3. [behavior] A unit test shows the criterion is still unmet when the passing recorded check is for a different tree than the candidate. Run `cargo test -p athena-core verify`.
4. [behavior] Existing D43/D63 tests still pass: a command never run by the verifier and not recorded is unseen, and a behavior criterion with no commands is still unmet. Run `cargo test -p athena-core verify`.
5. [content] Athena's own recorded fmt, clippy and test checks on the candidate tree passed ('Checks Athena ran on this tree'); this satisfies the workspace-wide test, clippy and fmt criterion, which is a content criterion rather than a behavior one.
6. [content] The module doc of crates/athena-core/src/verify.rs and the docs of `commands_run` and the new input field state the rule: a cited command is seen if the verifier ran it, or Athena ran and recorded it on the same candidate tree and it passed; failed or other-tree records do not count.
7. [content] docs/decisions.md has a new decision amending D43 with the rule, alternatives, tradeoff, and provenance citing job 20261024-033933-ta60 and commit 88562f1; D43's section points to it; docs/decision-links.toml links it to D43 as 'amended' with correct line numbers.
8. [content] crates/athena-core/src/run.rs passes the recorded checks and same-tree indication into `judge`, excluding nested-run 'not run' placeholders, so the live pipeline gets the new behavior.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the records-branch writer in athena-git or the two-threads test nit from job ta60.
- Changing how the verifier's own commands are observed.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the passing, failed and other-tree cases; workspace tests and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria are tests of the verdict that D43 produces.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion that job ta60 failed on is made checkable from a recorded check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision are written for outside readers.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision cites the job and commit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to judge, its caller and docs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the rule in D43 rather than the one rejected job.
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
