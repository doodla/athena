# Job 20261015-051600-458x: Build the explainer page outside the verifier's work tree

## Request (verbatim)

> Two explainer jobs (20261015-031159-xx7n, 20261015-033631-x3a1) were rejected only because 'the verifier changed its snapshot': building the site in the verifier's working copy writes into it. Building the page must not invalidate verification.

## Reading (accepted by the user)

`cargo run -p xtask -- site` must not write into the repository work tree. The default output moves to the cargo target directory (`$CARGO_TARGET_DIR/site`, else `target/site`), which Athena already points outside the snapshot. The Pages workflow and docs follow. Audience: the Athena owner, whose explainer jobs were rejected on the blocker "the verifier changed its snapshot".

Assumptions:
- The failing write was the verifier building the page with the default `site/` or an `--out` path inside its work tree. I could not confirm which path changed the hash, because the verifier's transcripts are not in the snapshot.
- The producer should first reproduce the problem. Snapshot::work_tree_tree in crates/athena-git/src/snapshot.rs honours .gitignore, and /site is already ignored, so the offending path may be something else, such as an explicit --out.
- Athena's verifier environment sets CARGO_TARGET_DIR to {job_dir}/target/{side}, outside the snapshot (crates/athena-agent/src/env.rs). Writing the page there leaves the verifier's work tree untouched.
- Athena's own snapshot/verify code (crates/athena-core/src/run.rs, snapshot.rs) is not changed. The fix is in the page builder, not in loosening the snapshot-changed check.
- The two rejected explainer jobs' content changes (glossary, CHANGELOG notes) are not part of this request.

## Acceptance criteria

1. [behavior] In a verifier-like setting, run `cargo run -p xtask -- site`, then `git status --short`. The status output is empty, and index.html exists under the cargo target directory's site/ folder rather than in the repository root.
2. [behavior] `cargo test -p xtask` passes. It includes a test that the default output directory is not inside the repository work tree. It also has a test that `--out` pointing inside the repo root, apart from the ignored target directory, is refused with a clear error. The existing build_writes_index_html test still passes.
3. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean.
4. [content] .github/workflows/pages.yml uploads the new output directory, and its path matches the xtask default (for example `target/site`). The workflow stays a thin call into `cargo run --locked -p xtask -- site`.
5. [content] docs/design/explainer-site.md and the xtask usage text state where the page is written and why. The reason is that building must not change the verifier's snapshot. The old `/site` location is no longer described as the default.
6. [content] CHANGELOG.md [Unreleased] has an entry saying building the explainer page no longer writes into the work tree, so a verifier's snapshot stays unchanged.

## Expected deliverables

- xtask/src/site.rs (modify)
- xtask/src/main.rs (modify)
- .github/workflows/pages.yml (modify) (a governance file: allowed because the reading lists it)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- .gitignore (modify)
- README.md (modify)

## Out of scope

- Changing the explainer's content (glossary, claims, CHANGELOG notes from the rejected jobs).
- Loosening or changing the verifier's snapshot-changed check in crates/athena-core or crates/athena-git.
- Re-running the rejected jobs 20261015-031159-xx7n and 20261015-033631-x3a1.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Remove the write into the work tree at its source (the default output location) instead of weakening the verifier's snapshot check.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests in xtask for the default output path and for refusing an --out inside the repository root. cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule that building the page never writes into the tracked repository tree is enforced in code (the xtask refuses it) and by a test, not only stated in prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the xtask, the Pages workflow, and the docs that describe them.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A behavior criterion has the verifier run the site build and then check git status.
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
