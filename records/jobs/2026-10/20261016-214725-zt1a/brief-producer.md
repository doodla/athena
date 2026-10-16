# Job 20261016-214725-zt1a: Retry bounded parallel drain with config and schema landing together

## Request (verbatim)

> try again on 20261016-200605-xh54

## Reading (accepted by the user)

Retry job 20261016-200605-xh54: the drain runs independent items of one plan group concurrently, up to a new [parallel] max_concurrency (default 1). The Config field is added in config.rs in the same change as the athena.toml section, and a test parses the real athena.toml. The audience is the Athena user running a drain, who wants a shorter wall time and unchanged serial behavior by default.

Assumptions:
- The typed request, 'try again', takes precedence over the introspection's proposed repair/guard job. This retry rebuilds the parallel drain; it does not build the commit-time config check.
- The repository I see (the base) has no [parallel] in athena.toml and no parallel field in Config (grep found neither), so it parses today. Nothing needs repairing first.
- The earlier failure was the athena.toml edit landing without the schema field. Here the schema field and the athena.toml section go in one change, and a test parses the real athena.toml to prove they match.
- Send+Sync bounds on traits in gh.rs and jev.rs, plus admission.rs test fixes (the test used [parallel] as an unknown section), are likely needed and should be listed this time, as should README and roles-and-ladder wording.
- A panic in a worker is documented as not isolated (as in the earlier attempt), unless it is cheap to isolate.

## Acceptance criteria

1. [behavior] A test loads the repository's real athena.toml (with its [parallel] section) through Config and passes. A config without [parallel] yields max_concurrency 1. A config with an unknown top-level section is rejected. Run `cargo test`.
2. [behavior] A drain test with bound 1 shows items run strictly in the existing order, and all existing drain tests pass unchanged. Run `cargo test -p athena-cli`.
3. [behavior] A drain test with bound 2 and two independent items in one group has both in flight at once (barrier; fails if serial). An item that depends on another starts only after it finishes. Run `cargo test -p athena-cli`.
4. [behavior] Tests show one item's failure or refusal does not stop its siblings, the worker bound is never exceeded, and Stop starts no further item. Run `cargo test`.
5. [behavior] Ordering and retry rules are unchanged; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
6. [content] The [parallel] section in athena.toml and its schema field in crates/athena-data/src/config.rs are in the same change. The field, its default (1) and its ceiling are documented where defined. The final report names and justifies the athena.toml change.
7. [content] The Drain and Report docs no longer say the loop runs one job at a time. docs/design/parallel-jobs.md and CHANGELOG.md state what this slice built and what remains (multi-job `athena go`, the --pr phase), and disclose the Send+Sync trait changes.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- README.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- A commit-time or submit-time config/schema consistency check (the introspection's proposal).
- Multi-job `athena go`.
- The --pr phase.
- Isolating worker panics per item, unless it is trivial.
- Rewording the '(could run in parallel)' plan-groups label in display.rs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and TOML.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain and config tests; cargo test, clippy --all-targets and fmt --check pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check concurrency with a barrier test, not just the presence of a field.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Schema and config land in one change, with a test that loads the real athena.toml.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Multi-job `athena go` and the --pr phase stay out of scope.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that the real athena.toml parses under Config.
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
