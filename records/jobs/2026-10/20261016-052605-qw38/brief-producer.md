# Job 20261016-052605-qw38: Add auditor and warden roles per the 2026-10-16 BRIEF addendum

## Request (verbatim)

> Add the auditor and warden roles from the BRIEF.md addendum '2026-10-16: An auditor and a warden'.

## Reading (accepted by the user)

Implement the 2026-10-16 BRIEF addendum: an auditor role that judges real outcomes against BRIEF.md and owner goals, and a warden role plus landing check that holds any landing touching protected files or weakening a check. Follows the audience-role pattern (role, core module, CLI, tests, design doc), for the owner.

Assumptions:
- Goal: the owner (and anyone judging Athena's trustworthiness) gets in-house grading with a protected yardstick, per the 2026-10-16 addendum in BRIEF.md.
- Modeled on how the audience role was added: role definition in roles.rs, core module, CLI command, tests, design doc.
- Warden is a mechanical check on landings (protected paths, loosened checks) plus a role definition; it holds rather than rejects.
- No athena.toml/defaults.toml/workflow change is needed; protected list lives in code. If the list must live in athena.toml, that would need owner action.

## Acceptance criteria

1. [behavior] Run `cargo test`: tests pass showing a landing that modifies BRIEF.md, an auditor/warden prompt, a grader prompt or threshold, or a permission/budget/kill-switch setting is held with a recorded reason, while an ordinary landing still lands.
2. [behavior] Run `cargo test`: a test shows a landing that deletes a test or lowers a threshold (loosens a check) is held.
3. [behavior] Run `cargo test`: a test shows a job's change to the auditor's or warden's prompts or checks is held.
4. [behavior] Run `cargo test`: an auditor run through the fake agent records a verdict citing evidence of a real outcome, judged against BRIEF.md and owner goals.
5. [behavior] Run `cargo clippy` and `cargo fmt --check`: both clean; `cargo test` passes overall.
6. [content] crates/athena-data/src/roles.rs defines valid `auditor` and `warden` RoleDefinitions with short fixed prompts and tests; the auditor prompt says to judge against BRIEF.md and owner goals, not the job's criteria, and to look at real outcomes rather than records.
7. [content] The protected list is defined in exactly one place in code, and a design doc plus CHANGELOG entry explain both roles, the list, alternatives considered, and that only the owner changes these rules.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/warden.rs (create)
- crates/athena-core/src/auditor.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/warden.rs (create)
- crates/athena-cli/tests/auditor.rs (create)
- docs/design/auditor-and-warden.md (create)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- docs/decisions.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Changing BRIEF.md, athena.toml, defaults.toml, or CI workflows
- Changing existing graders' prompts or thresholds
- Scheduled or automatic auditor runs beyond a CLI command

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests, clippy and fmt criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check holds happening on real landing scenarios.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Protection enforced by a warden check in code, not prose.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
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
