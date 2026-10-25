# Job 20261013-215435-em5g: Fix stdin/stdout pipe deadlock in git helpers and gitleaks runner

## Request (verbatim)

> Fix a pipe deadlock that now hangs CI (PR #47's 'fmt, clippy, tests' job ran 20+ minutes twice; reproduced locally). Four tests in xtask/tests/policy.rs hang: its git() helper (lines 20-38) writes all input to 'git check-attr --stdin' before reading any output, so once the output passes the 64 KB pipe buffer, git blocks writing to the test while the test blocks writing to git. The repository's tracked files grew past that today. Root cause: every place that pipes input to a child writes it all before draining stdout. The same pattern is in Git::exit (crates/athena-git/src/lib.rs, about line 194, used by Athena's own git calls) and the gitleaks runner (crates/athena-gate/src/secrets.rs, about line 160). Fix all three by writing stdin on its own thread (or an equivalent that drains output concurrently), ideally through one shared helper, and add a test that pipes more than 1 MB through a child that echoes it back.

## Reading (accepted by the user)

Writing child stdin before draining stdout deadlocks once output exceeds the pipe buffer. I add a shared helper in athena-git that writes stdin on a separate thread while collecting output, use it in Git::exit and the gitleaks runner, and fix the xtask policy.rs git() the same way. A regression test pipes over 1 MB through a child that echoes it.

Assumptions:
- The shared helper lives in athena-git (athena-gate already depends on it); xtask may not depend on athena-git because of the architecture dependency-graph policy test, so it may get a small local fix.
- No new crate is needed; std::thread::scope suffices.
- Broken-pipe errors on stdin write are tolerated when the child exits, as the child's status is the real signal.

## Acceptance criteria

1. [content] Git::exit, the gitleaks runner's run, and the xtask policy.rs git() helper no longer write all stdin before reading stdout; stdin is written on a separate thread (or equivalent concurrent drain).
2. [content] The athena-git and athena-gate call sites share one helper; xtask uses it or a documented local equivalent if using it would alter the crate dependency graph.
3. [content] A write error from a child that exits early (broken pipe) is handled without panic or hang.
4. [behavior] Run `cargo test` from the repo root: a new test piping more than 1 MB through an echoing child (e.g. cat) passes, and xtask policy tests complete rather than hang.
5. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-git/src/lib.rs (modify)
- crates/athena-gate/src/secrets.rs (modify)
- xtask/tests/policy.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Other process-spawning code that does not pipe stdin
- CI workflow changes
- Reducing the number of tracked files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared helper fixes the pattern at every call site rather than only the failing test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Adds a >1 MB echo regression test; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the three call sites and the helper.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Uses std threads; no new crate.

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
