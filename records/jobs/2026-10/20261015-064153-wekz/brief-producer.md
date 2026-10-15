# Job 20261015-064153-wekz: Add a drain command that works the queue within a budget

## Request (verbatim)

> Athena's steward and audience queue work, but every queued item waits for me to run it, land it and rate the result again. Work through your own queue yourself, within a budget I set, until the outcome you are working toward passes its check or the budget is spent, and tell me where it stopped.

## Reading (accepted by the user)

Add a CLI command (`drain`) that runs queued steward- and audience-queued requests itself: run, land and rate each item. It stops when the work's own check passes (for the explainer, the audience rates the live page 4 or more), when the user-set budget is spent, or when the same item fails twice (thrashing). It then reports where it stopped. The audience is the repository owner, who now runs go, land and rate by hand.

Assumptions:
- An empty queue is not success: the loop reports 'queue empty, target not met' as a distinct stop reason.
- The target is the audience rating threshold of 4 or more on the live page, read from the existing audience rating records; the threshold is a flag with default 4.
- Thrashing means the same item fails (rejected or not landed) twice; the loop stops and names it.
- Budget is a USD cap flag (and optional job-count limit); a job starts only if recorded spend plus its cap fits.
- Items that need a person (above-ladder proposals) are listed as waiting and never accepted or landed by the loop.
- Reuse existing run, land and rating functions in athena-core; the CLI only orchestrates.
- No new config key; the choice is recorded in decisions.

## Acceptance criteria

1. [behavior] `cargo test` passes, with new fake-agent tests: (a) queue of two items, both run and landed, rating reaches 4+, stop reason 'target met'; (b) budget too small for the next job gives 'budget spent' with the remaining items still queued; (c) an empty queue with the target unmet stops at once with no spend and says the queue is empty, not success; (d) the same item failing twice gives 'repeated failure' naming the item.
2. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean.
3. [behavior] `cargo run -p athena-cli -- drain --help` documents the budget, rating-threshold and job-limit flags and lists the stop reasons.
4. [content] A unit test pins that no job starts when recorded spend plus the next job's cap would exceed the budget, including when an earlier job reported a cost above its estimate.
5. [content] The final output (text and --json) names the stop reason, the jobs run with outcomes and costs, total spent, the latest rating against the target, and the items still queued or waiting for a person, so the owner can see where it stopped without opening records.
6. [content] A test shows an item needing a person (above-ladder proposal) is listed as waiting and left untouched, never accepted, approved or landed by the loop.
7. [content] README/docs and CHANGELOG describe the command, budget semantics, stop reasons, and what it does not decide; a decisions entry records the choice not to add a config key.

## Expected deliverables

- crates/athena-cli/src/drain.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Accepting, approving or landing items on a person's behalf
- Changing the audience rating mechanism or the steward's proposal ladder
- Adding a configuration key to athena.toml
- Changing CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust in the athena-cli crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-agent tests for each stop reason; clippy and fmt clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the tests and the --help output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command reusing existing run, land and rate paths.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: No new crates; reuse existing orchestration.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The budget guard and the person-needed exclusion are enforced in code and pinned by tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for an outside reader.
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
