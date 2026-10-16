# Job 20261016-071202-q0xy: Run independent plan-group items in parallel in the drain loop

## Request (verbatim)

> Build parallel jobs from docs/design/parallel-jobs.md so independent items in the plan's groups run side by side, as the 2026-10-16 BRIEF.md addendum asks.

## Reading (accepted by the user)

Reading: the drain loop (crates/athena-cli/src/drain.rs) plans items into parallel groups (`plan_groups`) but runs one job at a time; build the first parallel slice so items of one plan group run concurrently, bounded by a worker limit, following docs/design/parallel-jobs.md (child-process model, per-job locks and budgets, kill switch). Audience: the Athena user running a drain, who wants shorter wall time without weakened gates.

Assumptions:
- 'The plan's groups' means Report::plan_groups / Plan::groups in drain.rs and plan.rs, whose docs call concurrent group running 'follow-up work'.
- The first build is the drain-loop slice, not the full multi-job `athena go a b c`/`--all` launcher with --pr phase from the design's section 10 item 4.
- Concurrency is built to the design's recommendation (processes or isolation without Send/Sync changes to AgentDriver/Ui) unless the drain's Steps abstraction allows a simpler safe approach; worker default 2, ceiling 4 as the design states.
- README, CHANGELOG and decisions.md entries belong to a later docs job per the design, so they are not expected to change.
- Default worker bound keeps current behavior safe; exact default is the producer's call within the design's 2/4.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs drives the drain with a fake Steps whose jobs block on a barrier; with a bound of 2 and two independent items in one plan group, both are in flight at once (the test fails if run serially). Run `cargo test -p athena-cli`.
2. [behavior] A test shows the worker bound is respected: with a bound of 1 (or more items than workers) no more than the bound run at once, and items in later groups start only after the earlier group's items finish. Run `cargo test -p athena-cli`.
3. [behavior] A test shows one item's failure or refusal does not abort siblings in its group, and that turning the kill switch (Stop) on mid-run starts no further item. Run `cargo test`.
4. [behavior] With the bound at 1 or the default, the existing drain tests still pass unchanged: serial order, report and exit codes for single-job `athena go` are unchanged. Run `cargo test`, `cargo clippy --all-targets` (clean) and `cargo fmt --check`.
5. [content] Items that name no touched path or share a path stay serialized as plan.rs groups them; landing is still through the repository lock so nothing lands unverified; any new setting (worker bound, default and ceiling) is documented where it is defined.
6. [content] The drain's module docs and the Report no longer say the loop runs one job at a time; the design doc's status or build plan notes what this slice built and what remains (multi-job `athena go`, --pr phase). Any change to athena.toml is named here and justified.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/config.rs (modify)
- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Threads-in-one-process spike
- Daemon or async runtime
- Automatic merging or re-verification of stale jobs
- Per-job work-branch pull requests (question 8)
- Run-level spend cap
- docs/decisions.md and README edits

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for concurrency, bound, isolation, kill switch; cargo test and clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check items actually overlapping, not that types exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused slice: drain-group concurrency only.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Kill switch and bound enforced in code and tested.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments written for outside readers.
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
