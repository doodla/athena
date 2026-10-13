# Job 20261013-042933-hzrp: Write the design doc for running independent jobs in parallel

## Request (verbatim)

> Run independent jobs in parallel: today it is one job at a time and that is the bottleneck.

## Reading (accepted by the user)

Add one design doc under docs/design for running independent jobs concurrently in `athena go`, covering per-job branch/PR landing at design level only. No code, no review file, no build: those are separate later jobs.

Assumptions:
- The doc's file name is docs/design/parallel-jobs.md, following the naming of the existing design docs.
- The independent review, the implementation, tests, README and CHANGELOG updates, and the decisions.md entry are later jobs.
- Native PR landing is designed here as an interaction (one branch/PR per job) and not built.
- The doc may cite the existing code and design docs it read, per the evidence default.

## Acceptance criteria

1. [content] A new file docs/design/parallel-jobs.md exists and states the problem: jobs run one at a time, and queued jobs must be run by hand.
2. [content] The doc specifies the concurrency model (threads vs. separate processes), the worker bound and its default, and how the existing job and repository locks are reused.
3. [content] The doc specifies how the kill switch (refuse_if_stopped) and per-job budgets apply to every parallel job, and how failure is isolated so one job's failure or lock conflict does not abort the others.
4. [content] The doc specifies the CLI surface (multi-job and --all form, per-job result lines, exit codes with single-job behavior unchanged) and how conflicting jobs end stale or blocked, never merged silently.
5. [content] The doc has a section on landing each parallel job as its own branch/PR, designed and explicitly marked not built, and consistent with docs/design/native-pr-landing.md and docs/design/squash-chain-landing.md.
6. [content] The doc lists alternatives considered with reasons for rejection, open questions for the reviewer, and a build plan splitting later review and implementation jobs. It states that the independent review is pending.
7. [content] No Rust source, Cargo, config, CI, README, CHANGELOG, or docs/design/reviews files change. A decisions.md entry is deferred to the build job.
8. [behavior] Run `git status` and `git diff --stat`: only docs/design/parallel-jobs.md is added or modified. Run `cargo fmt --check`: it passes.

## Expected deliverables

- docs/design/parallel-jobs.md (create)
- docs/design/athena-v0.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- The independent review file under docs/design/reviews
- Any implementation in crates/ or xtask/
- README, CHANGELOG, and docs/decisions.md edits
- Building native PR landing
- Changes to docs/design/athena-v0.md

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The user asked for the design first, with the review and build as later jobs.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The doc cites the code and docs it relied on.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc, no code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The doc is written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A later independent job reviews the doc; this job is checked by diff inspection.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Hidden requirements were recovered from the earlier readings and the user's replies.

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
