# Job 20261017-044408-wcwy: Add the auditor role, judged against BRIEF.md and owner goals

## Request (verbatim)

> Add the auditor role from the BRIEF.md addendum '2026-10-16: An auditor and a warden' (the auditor only; the warden is a separate request). Replaces the too-large job 20261016-052605-qw38.

## Reading (accepted by the user)

Add an auditor role modelled on the existing audience role: a fixed recorded definition, a typed output, a core module that runs it, and an `athena audit` CLI command. It judges real outcomes against BRIEF.md and the owner's goals, not a job's own criteria. The warden is excluded. The audience is the owner, who wants a grader whose yardstick jobs cannot change.

Assumptions:
- The audience role is the template: Role::Audience, AUDIENCE_PROMPT, core/src/audience.rs, tests/audience.rs, and the `athena audience` command.
- The auditor is run on request through a CLI command. Cadence-based scheduling is a field only.
- Enforcing the protected prompt and checks is the warden's job and is deferred.
- The auditor changes nothing and its highest rung is Implementation.

## Acceptance criteria

1. [behavior] Run `cargo test`. It passes, including new tests that run the auditor against a fake driver and check that it records a typed finding.
2. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
3. [content] roles.rs defines an auditor role with a fixed prompt constant and a valid RoleDefinition. The prompt says to judge outcomes against BRIEF.md and the owner's goals rather than a job's own criteria, to look at real outcomes rather than records, and to cite evidence for each finding.
4. [behavior] Run `cargo test`. A test shows the auditor's brief contains BRIEF.md and not the job's acceptance criteria. Another shows a finding with no evidence is rejected.
5. [behavior] Run `cargo run -- audit --help`, or the equivalent test. The command exists and, with the fake driver, produces a recorded auditor invocation visible to the owner in CLI output.
6. [content] Role::Auditor has a name that round-trips through the parse and as_str functions in records.rs, and every exhaustive match on Role compiles.
7. [content] No warden code is added. The change does not edit BRIEF.md, athena.toml, defaults.toml, or any workflow file. CHANGELOG.md has an entry for the auditor.
8. [content] The change records that the auditor's prompt is protected from jobs only by being a fixed constant. The warden request will enforce protection. This is marked as prose-only in the docs or code comments.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/auditor.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/auditor.rs (create)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- The warden role and its holds on landings
- Automatic scheduling of the auditor
- Changes to BRIEF.md, athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module tests; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Auditor only; the warden is left out.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the auditor through the fake driver.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Protection is marked prose-only until the warden lands.
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
