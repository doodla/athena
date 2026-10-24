# Job 20261024-025358-kb23: Build docs/spec.toml for Olympus sections 1-6 with an xtask test

## Request (verbatim)

> Build the first layer of the cohesive spec again (decision 3C, BRIEF.md addendum '2026-10-17: Olympus'): docs/spec.toml covering BRIEF.md 'Project Olympus' sections 1 to 6 (vision, pantheon, feedback, the example, the two loops, R1 to R8), with an xtask test. Each requirement has an id, a plain cohesive statement, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), and one or more sources, each a verbatim quote with its heading; duplicates merge into one requirement with several sources; a conflict or open choice in Sri's words is recorded as an open question quoting both, never resolved. The test fails on a quote that is not a verbatim substring of BRIEF.md (whitespace collapsed), a duplicate id, a supersedes target that does not exist, or a requirement without a source, with in-memory negative cases, and finds at least one requirement for each of R1 to R8 and sections 1 to 5. No statement about who runs Athena or where (xtask/tests/policy.rs). Record the format and the choice of TOML in docs/decisions.md; sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job. Job 20261018-145008-k7fq built this once, but it reached main only through a hand merge that was reverted (2a86df5), so it is built again here.

## Reading (accepted by the user)

Rebuild the first layer of the cohesive spec (decision 3C): docs/spec.toml turns BRIEF.md 'Project Olympus' sections 1–6 (R1–R8 included) into sourced requirements, and xtask/tests/spec.rs checks it against BRIEF.md. The audience is Sri and the requirements-engineer role, who need a spec whose every line can be traced to his words by a test.

Assumptions:
- The earlier job's one real gap, a quote checked against the whole brief rather than under its heading, is fixed here.
- toml and serde are already xtask dev-dependencies, so no Cargo change is needed.
- The failing athena-data test (reading_inputs_digest_is_unchanged_by_the_job_plan_section) predates this work and is out of scope.
- The test reads BRIEF.md from the repository root through CARGO_MANIFEST_DIR, as policy.rs does.
- The Olympus brief text is quoted word for word in BRIEF.md after line 550. Quotes come from there, and headings are those listed there (## 1. Vision through ### R8).

## Acceptance criteria

1. [content] docs/spec.toml has, for each requirement, an id, a plain cohesive statement, a kind (goal, requirement, constraint or acceptance), a status (active or superseded), and one or more sources, each a verbatim quote plus its BRIEF.md heading. It covers Olympus sections 1 to 6 and has at least one requirement for each of R1 to R8 and for sections 1 to 5.
2. [content] Duplicates across the brief are merged into one requirement with several sources. Any conflict or open choice in Sri's words is recorded as an open question quoting both sides, with no resolution.
3. [content] docs/decisions.md records the spec format and the choice of TOML, with alternatives. It notes that sections 7 to 10, the governing philosophy and the 2026-10-17 addenda follow in a second job.
4. [content] No statement in the spec or the new test says who runs Athena or where, so xtask/tests/policy.rs still passes. No governance files change.
5. [behavior] Run `cargo test -p xtask --test spec`. It passes on the real spec. Each in-memory negative case makes the checker fail: a quote that is not a verbatim substring of BRIEF.md after whitespace collapse, a duplicate id, a supersedes target that does not exist, and a requirement with no source. A further test fails if any of R1 to R8 or sections 1 to 5 has no requirement.
6. [behavior] The checker also confirms each quote sits under its stated heading, not just somewhere in BRIEF.md. A negative case pairs a real quote with the wrong heading and fails. Run `cargo test -p xtask`.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. `cargo test -p xtask --test policy` still passes.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)

## Out of scope

- Sections 7 to 10, the governing philosophy and the addenda, which follow in a second job
- Fixing the unrelated failing athena-data digest test
- Any change to BRIEF.md
- A shared vocabulary or ontology
- A requirements-engineer role

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The xtask test has negative cases and a coverage check, and cargo test and clippy must stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the checker against the real spec and against bad specs built in memory.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and decision text are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim quotes from BRIEF.md.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check is a Rust test, and the data is TOML.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that covers only sections 1 to 6.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The spec carries no personal data or secrets.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Verbatim sourcing is enforced by the test.
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
