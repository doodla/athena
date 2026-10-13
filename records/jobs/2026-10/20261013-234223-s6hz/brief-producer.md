# Job 20261013-234223-s6hz: Design decision sets with assumptions and failure signals

## Request (verbatim)

> Design (no code) how Athena classifies, organizes and applies its decisions, from the BRIEF.md addendum '2026-10-13: Decision sets, and knowing when a decision is not working'. Write docs/design/decision-sets.md. Today decisions live apart and only defaults are applied: BRIEF.md addenda, defaults.toml (12), docs/decisions.md (D01-D90, a flat log whose supersessions live only in prose), and corrections inside job records. Give every decision typed fields (kind: requirement, expectation, design or process; scope: classes of work, paths and phase such as reading, producing, verifying, landing; status: active, proposed or superseded by a named decision; source; enforcement: code check, brief, or prose only). Group decisions into sets by scope, and apply a set's active decisions to a role's brief when a job matches its scope, the way defaults are applied now; defaults become one kind of decision. Also, so Athena cannot be confidently wrong: each decision states its assumptions and the observable signals that would show it is failing; Athena watches those signals in its records (rejections, interventions, corrections, check failures, host errors), and a firing signal puts the decision under review instead of leaving it applied. Example: D87 chose merge commits without stating that it assumed the repository allows them, and GitHub refused with HTTP 405 within the hour. Fit this with docs/design/defaults-loop.md, its introspection step and Jev (/v1/evaluate) for set membership and signal matching. Cover how the existing decisions are migrated, and keep within the reader's token budget.

## Reading (accepted by the user)

Write docs/design/decision-sets.md (no code): typed decision fields, sets by scope applied to role briefs, per-decision assumptions and failure signals that put a decision under review, fit with defaults-loop and Jev, and a migration plan for defaults.toml, D01-D90, addenda and job-record corrections.

Assumptions:
- Design only: no Rust, no schema files, no changes to defaults.toml or athena.toml.
- Jev /v1/evaluate behavior is taken from what existing docs say; the producer reads defaults-loop.md and decisions.md first.
- Decision numbering for any new decision recorded follows the next free D-number in docs/decisions.md.

## Acceptance criteria

1. [content] docs/design/decision-sets.md exists and contains no code changes elsewhere; only docs files change.
2. [content] Defines typed fields: kind (requirement, expectation, design, process), scope (classes of work, paths, phase: reading, producing, verifying, landing), status (active, proposed, superseded by a named decision), source, enforcement (code check, brief, prose only).
3. [content] Explains grouping decisions into sets by scope and how a matching job's active decisions are applied to a role's brief like defaults are now, with defaults as one kind of decision, and how the token budget of the brief is respected.
4. [content] Specifies per-decision assumptions and observable signals (rejections, interventions, corrections, check failures, host errors), how Athena watches them in records, and that a firing signal puts the decision under review rather than leaving it applied; uses D87 and the HTTP 405 merge refusal as the worked example.
5. [content] Explains the fit with docs/design/defaults-loop.md (introspection step) and Jev /v1/evaluate for set membership and signal matching, citing what those files say.
6. [content] Includes a migration section covering BRIEF.md addenda, defaults.toml's 12 defaults, docs/decisions.md D01-D90 including prose-only supersessions, and corrections in job records; lists alternatives considered and is concise.
7. [behavior] Run cargo test and cargo fmt --check; both pass unchanged, confirming no code was altered.

## Expected deliverables

- docs/design/decision-sets.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Implementing the decision store, matcher, or signal watcher
- Actually migrating D01-D90 into typed records
- Changing defaults.toml or any governance file

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design written with alternatives before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite the repository files and decision numbers they rest on.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused document.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Hidden requirements recovered from BRIEF.md, defaults-loop.md, decisions.md.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Each proposed rule states its enforcement or is marked prose-only.

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
