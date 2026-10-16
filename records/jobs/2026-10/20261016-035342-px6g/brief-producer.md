# Job 20261016-035342-px6g: Plan first: coordinated plan orders the drain; stop at job boundary

## Request (verbatim)

> As the BRIEF.md addendum '2026-10-16: Speed, planning first, and initiatives in parallel' asks: make planning come first. Every request is fleshed out in planning (goal, which rung of the ladder it serves, acceptance, open questions, dependencies, what it touches) before design or code, and open initiatives are planned together into one coordinated plan that says which can run in parallel and in what order, instead of working the queue one at a time oldest first. This covers the queued item 20261016-005949-n2qb (gaps audit and refine the planning pipeline) and 20261016-035330-vs76 (queue order and stopping a drain). Design work, starting with moving compute to CI with intelligent routing, comes after this lands.

## Reading (accepted by the user)

Add a planning stage before design/code. Each request is fleshed out (goal, ladder rung, acceptance, open questions, dependencies, touched paths), and all open items are planned together into one ordered plan with parallel groups. The drain follows that plan instead of strictly oldest first, can be stopped at a job boundary, and the plan-stage audit from n2qb and the order/stop work from vs76 are covered. Audience: the owner and the verifier, who judge by running drain tests.

Assumptions:
- Goal: make planning the first stage for every request and replace one-at-a-time oldest-first draining with a coordinated plan, serving the owner who wrote the 2026-10-16 addendum.
- Parallel running itself is not implemented; the plan only states which items can run in parallel and in what order, and the drain still runs one job at a time in plan order (execution parallelism is follow-up).
- Stop at a job boundary is a stop-file or flag checked between jobs, with a new Stop reason.
- Plan order follows ladder priority, then dependencies, then age as tiebreak.
- The n2qb audit is delivered as a first slice (stage tracing plus a planning-completeness gate), not a full autonomous role.
- I only skimmed the code: drain.rs has Stop with five reasons and QueueView items; no planning module exists. Exact seams may differ.

## Acceptance criteria

1. [behavior] Run cargo test: a drain test with three queued items where a newer item is planned first shows the drain works items in plan order, not oldest first.
2. [behavior] Run cargo test: a drain test that requests a stop mid-run shows the loop finishes the current job, starts no further job, and reports a distinct stop reason (not a killed job).
3. [behavior] Run cargo test: a plan built from several open items names, for each, goal, ladder rung, acceptance, open questions, dependencies, and touched paths, and groups items with disjoint touches and no dependency as parallel while ordering dependent items after what they need.
4. [behavior] Run cargo test: an item whose plan lacks goal, ladder rung, or acceptance is refused for design or code and reported, so planning cannot be skipped.
5. [content] The plan-stage audit from n2qb is described: each logged gap (intervention, introspection, rejection, rating) is traced to a stage (reading, decomposition, assumptions, criteria, outcome check), with a check of whether later gaps of that kind fell; if full automation is too large, the remainder is stated as follow-up.
6. [content] Docs (README, design doc, CHANGELOG) state the planning-first order and how to stop a drain at a job boundary, for an outside reader.
7. [behavior] Run cargo test, cargo clippy and cargo fmt --check: all pass, and no new unused or untested module.
8. [content] No design work for moving compute to CI is included; it is left for after this lands.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/plan.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Moving compute to CI and intelligent routing (design after this lands)
- Running jobs concurrently
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New plan and stop logic have passing tests; cargo test and clippy clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on plan and drain order; parallel execution deferred.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check order and stop behavior by running tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs written for outside readers.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Alternatives noted in docs/decisions.md or design doc.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
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
