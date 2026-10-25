# Job 20261013-152648-1ds0: Push the landed commit for land --pr and repair open PRs on rerun

## Request (verbatim)

> 'athena land --pr' opened PR #39 for job 20261013-150709-b67z, but CI's 'athena check --history' failed on that branch with 2 violations: the pushed candidate commit dd50c2b changes governed content with no Athena-Job trailer, and the records commit de4ef6a changes records with no trailer naming its job. Every commit pushed to athena/<job> must pass 'athena check --history' on its own, as the commits on main do. One way to get that is to push the job's locally landed commit, which already carries the trailers and the records, instead of the bare candidate. Pick the approach and record why. Running 'athena land --pr' again on a job whose PR is already open must replace that branch's head with the fixed commit (athena/<job> is Athena's own branch) and reuse the open PR, so rerunning on job b67z repairs PR #39. Add tests.

## Reading (accepted by the user)

Choose and record an approach so every commit on athena/<job> passes `athena check --history`; the proposed one is pushing the job's locally landed commit (from the outcome record) instead of the bare candidate or a synthetic records commit. Rerunning `land --pr` on a job with an open PR replaces the branch head with that commit (forced, safely, since the branch is Athena's own) and reuses the PR, which repairs PR #39 on job b67z.

Assumptions:
- The push uses the landed commit in the job's outcome record (landed_commit), or the commit on local HEAD carrying that job's Athena-Job trailer, so each pushed commit has trailers and records.
- Replacing the head uses a lease-protected force push (--force-with-lease against the observed remote head), limited to athena/<job> by the existing push guard.
- The PR stays reused: the stored link is kept and its head updated; no second PR is opened.
- The decision and its alternatives (bare candidate plus trailer rewrite, synthetic records commit, landed commit) are recorded in docs/design/native-pr-landing.md and/or docs/decisions.md.
- Merging stays behind the off-by-default merge_via_rest opt-in.

## Acceptance criteria

1. [content] The pushed head for athena/<job> is the job's locally landed commit (with its Athena-Job trailer and records), not the bare candidate nor the records-only commit made by work_commit/commit_with_records; the unused code is removed or justified.
2. [content] The chosen approach, rejected alternatives, and the reason (every pushed commit must pass athena check --history alone) are recorded in the design doc or decisions.md, and CHANGELOG.md is updated.
3. [content] When a stored PR link is open and the remote branch head differs from the fixed commit, land_pr replaces the branch head (guarded to athena/<job> only, with a lease) and reuses the same PR number, updating the stored link head; it does not open a second PR.
4. [content] The push guard still refuses any branch other than athena/<job> and any force outside the replace path.
5. [behavior] Run `cargo test`: new unit tests in gh.rs (fake remote and host) cover the pushed commit being the landed one, rerun with an open PR replacing the head and not calling open_pr, rerun when the head already matches (no-op), and refusal for non-work branches; an athena-git test covers the lease force push.
6. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass with no warnings.
7. [behavior] A test using a real temporary git repo with a bare remote shows the pushed branch's commits each pass the history check (trailer on governed and records changes), run via `cargo test`.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- docs/design/native-pr-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Actually repairing PR #39 on GitHub (the user reruns athena land --pr afterward).
- Changing CI workflows or athena check --history itself.
- Enabling merging by default.
- Rewriting history on main.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and Markdown docs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the pushed commit and rerun repair; cargo test and clippy must pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The approach choice and alternatives are written down in the design doc.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the PR publish path.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by running the tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The push guard enforces the branch restriction in code, tested.

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
