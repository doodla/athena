# Job 20261016-185517-qcrz: Run independent plan-group items in parallel in the drain loop

## Request (verbatim)

> try again on 20261016-071202-q0xy

## Reading (accepted by the user)

Retry of the parallel-jobs build: the drain runs items of one plan group concurrently up to a new `[parallel] max_concurrency` setting (default 1), with the schema field added in athena-data/config.rs together with the athena.toml section so athena.toml parses at every commit. Audience: the Athena user running a drain, who wants shorter wall time with serial behavior unchanged by default.

Assumptions:
- The Config struct lives in crates/athena-data/src/config.rs (deny_unknown_fields), not athena-core; the earlier failure came from adding [parallel] to athena.toml with no schema field.
- The new section is optional via #[serde(default)] like [landing], with max_concurrency defaulting to 1, so older athena.toml files still parse.
- Drain Limits gains a worker bound taken from config (and clamped to a documented ceiling); Steps is shared across threads, so parallel runs use scoped threads over a Sync Steps or equivalent.
- Landing stays serialized through the repository lock; dependent or path-overlapping items stay in later groups as plan.rs already groups them.
- Retry (ATTEMPTS) and ordering rules are unchanged; the kill switch stops starting further items.
- Multi-job `athena go` and the --pr phase remain out of this slice.

## Acceptance criteria

1. [behavior] A test parses the repository's athena.toml (containing a [parallel] section with max_concurrency) through Config and passes; a config without [parallel] also parses and yields max_concurrency 1. Run `cargo test`.
2. [behavior] A drain test with a bound of 1 shows items run strictly in the existing order, and all existing drain tests pass unchanged. Run `cargo test -p athena-cli`.
3. [behavior] A drain test with a bound of 2 and two independent items in one group has both in flight at once (barrier; fails if serial), and an item that depends on another starts only after it finishes. Run `cargo test -p athena-cli`.
4. [behavior] A test shows one item's failure or refusal does not stop its siblings, the worker bound is never exceeded, and Stop starts no further item. Run `cargo test`.
5. [behavior] Ordering and retry rules are unchanged, and `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
6. [content] athena.toml gains a [parallel] section only together with its schema field in crates/athena-data/src/config.rs; the field, default (1) and ceiling are documented where defined, and the athena.toml change is named and justified in the final report.
7. [content] Drain module docs and Report docs no longer say the loop runs one job at a time; docs/design/parallel-jobs.md and CHANGELOG.md say what this slice built and what remains (multi-job `athena go`, --pr phase). The final report maps each criterion to its evidence.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)

## Out of scope

- Multi-job `athena go`
- The --pr landing phase
- Changing retry count or ordering rules
- Changes to defaults.toml or .github/workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy --all-targets and fmt --check pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check concurrency by a barrier test, not by the presence of a field.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Schema field, default, and athena.toml section delivered together, with a load test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One slice: bounded parallel groups only.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test loads the real athena.toml so a section without a schema field fails the build.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New setting documented where defined; design doc and CHANGELOG updated.
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
