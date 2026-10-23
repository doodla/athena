# Job 20261019-013306-zvmr: Make land --pr judge the PR's current head and refuse extra commits

## Request (verbatim)

> athena land --pr judges the checks of the job's own commit, not the pull request's current head. When a fix is pushed to the pull request (on 2026-10-18, PR #149 for job 20261018-145008-k7fq), land keeps re-running and reporting the old commit's failure even though the new head is green, so the only way to merge was around land. Make land judge the checks of the pull request's current head, and refuse to merge when that head contains changes beyond the verified content and records unless each extra commit is a declared intervention.

## Reading (accepted by the user)

`athena land --pr` will ask the host for the pull request's current head and judge that head's checks. It will merge guarded by that head, and it will never force-replace a newer head with the job's commit. Until the records-history build (D111) lands, it refuses whenever the head is anything other than the job's landing commit, declared intervention or not, and names the extra commits. The audience is the user running land after pushing a fix to the PR.

Assumptions:
- Goal: land --pr must merge a PR whose head was fixed after landing, without the user bypassing land. It must refuse any head that is not the job's landing commit.
- The user's reply to the earlier reading overrides the original 'declared intervention' exception: the refusal covers all extra commits until D111 lands.
- The code is in crates/athena-core/src/gh.rs (land_pr, wait_for_checks, PrLink, replace_head); the exact host trait method for reading the PR head may need adding.
- The request text that mentions PR #149 describes a past failure; it needs no change outside land.
- The refusal message may mention D111 by name, since the user named it.

## Acceptance criteria

1. [behavior] placeholder
2. [behavior] A test shows that when the PR's current head is not the job's landing commit, land_pr refuses to merge. The error names every extra commit, covers declared interventions as well as undeclared ones, and tells the user why (a squash of the landing plus a later commit is not a valid landing until D111). The host's merge endpoint is not called. Run `cargo test --locked -p athena-core gh::`.
3. [behavior] A test shows that when the PR head is the job's landing commit and its checks are green, land_pr merges guarded by that head. A second test shows that the PR's checks, not the stale commit's, are what gets reported after the PR is fixed and its head is back at the landing commit. Run `cargo test --locked -p athena-core gh::`.
4. [behavior] A test shows land_pr never calls the remote's replace/force-push when the remote head differs from, or descends from, the job's commit. Run `cargo test --locked -p athena-core gh::`.
5. [behavior] The existing external_calls tests (retries, persistent 500s) and the display tests for the land --pr report still pass. Run `cargo test --locked -p athena-cli`. `cargo clippy --all-targets --locked` is clean and `cargo fmt --check` passes.
6. [content] The module and function docs state that checks and the merge guard are keyed to the PR's current head. The refusal message names each extra commit and points to the D111 limitation. The CHANGELOG entry, if added, describes the fix for an outside reader.
7. [content] No governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)

## Out of scope

- Building the records-history check (D111)
- Fixing the pre-existing failing test reading_inputs_digest_is_unchanged_by_the_job_plan_section
- Changing spec checker behavior in xtask
- Changing how land works without --pr

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added in gh.rs; cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run land_pr against a fake host and check the merge guard, the refusal and the absence of a force-replace.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check lookup is keyed to the PR's current head, not patched for one case.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and refusal messages are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in gh.rs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the named test commands.
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
