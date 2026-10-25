# Job 20261016-182922-w314: Build CI result reuse, routing, and after-measurement (ci-routing 4-7)

## Request (verbatim)

> Finish the CI routing build: items 4-7 of docs/design/ci-routing.md section 10 (result reuse keyed by commit, per-job routing of what runs locally vs in CI, and the after measurement with xtask walltime against the baseline from #117).

## Reading (accepted by the user)

Implement build-plan items 4–7 of docs/design/ci-routing.md: per-SHA CI result reuse with local fallback, routing by job kind per the 3.2 table, a before/after comparison in `xtask walltime`, and the closing docs. Audience is the owner and the verifier, who judge by tests that fail without the mechanism and by measured, labeled numbers.

Assumptions:
- Item 6 needs N≈20 real post-build jobs, which cannot exist inside this change. I assume the job builds the comparison tooling (xtask walltime after/compare mode against the #117 baseline) and records the after numbers only if records exist, otherwise states plainly that they are not yet measured.
- 'Baseline from #117' means the output/doc of the baseline job (docs/design/ci-routing-baseline.md); I did not check what #117 contains.
- Result reuse lives in athena-core (checks.rs, verify.rs, gh.rs) with wiring in athena-cli; Athena, not the verifier, fetches the CI record, and the verifier command set stays unchanged (design 3.3).
- The design review is still open; the owner's request overrides it, as it did for items 2 and 3.
- Not checked in detail: the exact current code structure of checks, verify and gh; paths are the likely ones.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: tests show a CI result whose SHA differs from the candidate's is ignored, a missing/cancelled/timed-out result triggers the local fallback run, a red result fails the check without fallback, and a stale record never marks a job verified.
2. [behavior] Run `cargo test --locked`: tests show one run of fmt/clippy/test per SHA is reused by the verifier input and by Athena's check step, so the same SHA is not checked twice.
3. [behavior] Run `cargo test --locked`: routing tests cover each job kind in design 3.2 (code, tests, docs, design, config, build, study); docs-only skips fmt and clippy but not test; a workflow-file change in the candidate forces local run; a job with no kind takes the config all-checks path.
4. [behavior] Run `cargo run --locked -p xtask -- walltime` on fixture records: it outputs before/after medians and p90 by job kind, flags fewer than 20 jobs, and flags a regression when the after median exceeds the baseline.
5. [behavior] `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` pass.
6. [content] The verifier's allowed command list in athena.toml is unchanged, and the verifier input carries the CI record (SHA, run address, per-step conclusions).
7. [content] docs/design/ci-routing.md section 10 marks items 4–7 built or partly built, naming what is not done; any after-measurement number is either measured from records with the command shown or stated as not yet measured.
8. [content] README and CHANGELOG describe the routing and result reuse in a way a new reader can follow, and a decisions entry records the call.

## Expected deliverables

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/ci_result.rs (create)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- xtask/src/walltime.rs (modify)
- xtask/tests/walltime.rs (modify)
- docs/design/ci-routing.md (modify)
- docs/design/ci-routing-baseline.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Widening the verifier's command set or giving it network/gh access
- Recording the design review verdict or marking the design accepted
- Self-hosted builders or local compiler caches (alternatives F, G)
- Fabricating after-measurement numbers when too few post-build jobs exist
- Changing defaults.toml or records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new logic and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New modules have tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check SHA mismatch, missing, and red behavior by running tests, not file existence.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Missing or stale CI results never yield verified; the job blocks if local fallback cannot run.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Docs label unmeasured numbers as estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Four items kept as separable steps in one change; item 7 docs last.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Measurement compares wall time with equal checks run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Reuse rules are enforced by tests that fail without them.
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
