# Job 20261018-133651-1ncn: Build docs/spec.toml for Olympus sections 1-6 with a checking test

## Request (verbatim)

> try again on 20261018-120736-90w3

## Reading (accepted by the user)

Retry of job 20261018-120736-90w3, halved per the user: build docs/spec.toml from BRIEF.md "Project Olympus" sections 1 to 6 only (vision, pantheon, feedback, example, lifecycle/two loops, R1 to R8), each requirement citing verbatim quotes and headings. An xtask test checks the file. Audience: agents that read the spec, and Sri, who approves it. The verifier-timeout fix is not part of this job.

Assumptions:
- The user's reply to the earlier reading takes precedence: this job is the Olympus spec layer limited to sections 1 to 6, not the verifier-timeout mechanism.
- The test lives in xtask/tests/spec.rs and uses a TOML parser dependency in xtask (toml crate, bought not built) and serde.
- Section headings cited are the BRIEF.md headings such as '1. Vision' and 'R1 — Causal investigation'.
- The test's coverage count is restricted to sections 1 to 6 so it will pass now; the second job extends it to phases, section 10 and decisions 3C, 4A, 5A, 7A.
- The earlier deliverable docs/ontology.toml and docs/enforcement.md changes are kept only if the test needs registering there.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test parses docs/spec.toml and passes, meaning every source quote is a verbatim substring of BRIEF.md after collapsing whitespace, ids are unique, supersedes targets exist, and every requirement has at least one source.
2. [behavior] Run `cargo test -p xtask`: in-memory negative cases each make the checker fail for (a) a quote not in BRIEF.md, (b) a duplicate id, (c) a supersedes target that does not exist, (d) a requirement with no source.
3. [behavior] Run `cargo test -p xtask`: a test counts requirements by their sources' headings and finds at least one each for R1 to R8 (section 6) and for sections 1 to 5 (Vision, Pantheon, Feedback, Example, lifecycle with the two loops).
4. [content] Each requirement in docs/spec.toml has an id, a plain cohesive statement, a kind (goal, requirement, constraint, acceptance), a status (active or superseded), and one or more sources (verbatim quote plus heading). Duplicates are merged into one requirement with several sources. Requirement-bearing sentences in Olympus sections 1 to 6 are covered.
5. [content] docs/spec.toml records at least one open question with both verbatim quotes where Sri's words conflict or leave a choice open, recorded and not resolved. If sections 1 to 6 contain none, the verifier can see the question field is supported by the format and the test, and the spec says so.
6. [content] docs/spec.toml has no statement about who runs Athena or where; xtask/tests/policy.rs still passes under `cargo test`.
7. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all clean and passing.
8. [content] CHANGELOG.md has an entry for the spec; docs/decisions.md records the spec format and the choice of TOML with alternatives (Markdown with ids, JSON, YAML), and notes that sections 7 to 10, governing philosophy and 2026-10-17 addenda follow in a second job.

## Expected deliverables

- docs/spec.toml (create)
- xtask/tests/spec.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- docs/ontology.toml (modify)

## Out of scope

- Olympus sections 7 to 10, the governing philosophy and the 2026-10-17 addenda (second job)
- The Athena brief's earlier parts
- The requirements-engineer role
- Changing any reader or prompt to use the spec
- The verifier timeout, unjudged outcome and retry mechanism
- Changes to athena.toml, defaults.toml, or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Checker test is Rust in xtask.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the toml and serde crates instead of a hand-written parser; record in decisions.md.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Positive and negative tests; cargo test, clippy, fmt clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Halved to sections 1 to 6 per the user.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain statements and docs for an outside reader.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test and check the coverage by headings.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verification by running cargo test, clippy and fmt.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim BRIEF.md quotes.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data, and no owner-side details in the spec.
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
