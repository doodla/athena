# Job 20261023-181729-2sf0: Make land --pr judge and guard the PR's current head

## Request (verbatim)

> athena land --pr judges the checks of the job's own commit, not the pull request's current head. When a fix is pushed to the pull request (on 2026-10-18, PR #149 for job 20261018-145008-k7fq), land keeps re-running and reporting the old commit's failure even though the new head is green, so the only way to merge was around land. Make land judge the checks of the pull request's current head and guard the merge with that head, and never force-replace a newer head with the job's older commit. Until the records-history build (D111) lands, a squash of a job's landing plus any later commit is not a valid landing (that is how squash 02837e0 broke the history check), so land refuses when the pull request's current head is anything other than the job's landing commit, declared intervention or not, naming each extra commit and why. This repeats job 20261019-013306-zvmr, abandoned when the machine ran out of disk mid-run.

## Reading (accepted by the user)

Change `athena land --pr` so it judges checks of the pull request's current head and passes that head as the merge guard. It never force-replaces a newer head with the job's commit. It refuses, naming each extra commit and why, when the head is anything but the job's landing commit. Audience: the maintainer who landed PR #149 around land.

Assumptions:
- The land flow lives in crates/athena-core/src/land.rs with GitHub calls in gh.rs; I did not read them in depth.
- Refusal compares the PR head sha to the job's landing commit and lists the commits between them.
- The previous attempt's candidate touched gh.rs, main.rs, README.md and CHANGELOG.md, so those are likely again.

## Acceptance criteria

1. [behavior] A test (run with cargo test) shows that with a PR whose head differs from the job's commit and is green while the job's commit is red, land does not report the old commit's failure and judges the head's checks; and when the head equals the landing commit, land merges.
2. [behavior] A test (cargo test) shows the merge request is guarded with the PR's current head sha, and that no push or update replaces a newer head with the job's older commit.
3. [behavior] A test (cargo test) shows land refuses when the PR head is anything other than the job's landing commit, including when the extra commit is a declared intervention, and the message names each extra commit and why it is refused (squash with later commit invalid until D111 records-history build).
4. [content] README.md or CHANGELOG.md document the new land behavior and the D111 interim refusal, so a maintainer can find why land refused.
5. [behavior] cargo clippy and cargo fmt --check are clean; cargo test passes for the changed crates (the pre-existing reading_inputs_digest failure is noted if it persists).

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Building the records-history build (D111)
- Fixing the pre-existing reading_inputs_digest test failure
- Changing the squash-merge strategy beyond the guard

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each land behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check land's judging, guard and refusal by running tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix which commit land judges rather than working around it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to land.
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
