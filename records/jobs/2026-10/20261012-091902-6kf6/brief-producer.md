# Job 20261012-091902-6kf6: Build athena-cap-study crate: study records and validation rules

## Request (verbatim)

> build the study capability again; 20261012-090901-9vwq failed admission

## Reading (accepted by the user)

I read this as building the reusable core of the study capability: a new athena-cap-study crate with the design's typed records, the §5.4 validation rules, and a Capability impl with prompts. The run step, containment, and Pages publishing are left out. The file set is chosen to avoid the admission rules (governance files, size, personal data) that the earlier attempt likely tripped.

Assumptions:
- Job 9vwq failed admission, which checks governance files, file size, secrets or emails, records/, and classification changes; its log is not available to me. I assume the cause was an unlisted governance file, an oversize file, or personal data, and I avoid all three.
- The design (docs/design/study-capability-m1.md) passed review 2 with named changes (R2-m1, R2-m2, R2-m3). I assume the builder applies those: spend-rule wording, a field on study.finding/1 naming what it covers, and added finding kinds.
- One job cannot build everything in the design, so I read 'build the study capability' as the reusable validation core: the crate, records, the model, the §5.4 rules, and the Capability impl. Containment, the run step, replay, and Pages publishing are left for later jobs.
- The Capability trait in athena-core may not fit a multi-step study job. I assume the crate implements it as-is and any needed trait change is deferred and noted.
- The workspace Cargo.toml may list members explicitly. If it only uses a glob, it will not change.

## Acceptance criteria

1. [content] A new crate crates/athena-cap-study exists with its own Cargo.toml (workspace package/lint inheritance, no edits to any [lints] table) and src/lib.rs, and is a member of the workspace.
2. [content] The crate defines typed records from design §4 with a schema field: study.pin/1, study.source/1, study.claim/1, study.run/1, study.rationale/1, study.finding/1, study.gap/1, study.validation/1, plus study.model/1 with the §5.1 elements. Numbers are Quantity values (measured or labeled estimate), not bare numbers.
3. [content] The crate implements the §5.4 validation rules as pure functions over records, each classed evidence (non-waivable) or model (waivable); a waiver clears only a model rule and only when it names the rule and ids and cites a user intervention record.
4. [content] Rules that need git lookups (Wrong SHA, Bad citation, Quote mismatch) go through a small trait or fixture repository, not the live network.
5. [content] The crate implements the core Capability trait (id "study", card, producer and verifier prompt sections under crates/athena-cap-study/prompts/) but is NOT registered in athena-cli wiring, and does not run any studied system.
6. [content] docs/architecture.md gains an athena-cap-study row listing its dependency edges, matching the actual Cargo.toml dependencies.
7. [content] CHANGELOG.md records the new crate; a decisions.md entry records the scope call (validation core only; containment, run step, and Pages publishing deferred).
8. [behavior] Run `cargo test` from the repo root: every §5.4 rule has a test with one planted fault and one clean case, and all tests pass, including existing architecture/dependency tests.
9. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both clean.
10. [behavior] Run `git status` and `git diff --stat`: no change under records/, no governance file (athena.toml, defaults.toml, setup.sh, rust-toolchain.toml, workflows, .gitattributes, rustfmt/clippy config), no file over the size limit, no email addresses.

## Expected deliverables

- crates/athena-cap-study/Cargo.toml (create)
- crates/athena-cap-study/src/lib.rs (create)
- crates/athena-cap-study/src/records.rs (create)
- crates/athena-cap-study/src/rules.rs (create)
- crates/athena-cap-study/src/model.rs (create)
- crates/athena-cap-study/prompts/producer.md (create)
- crates/athena-cap-study/prompts/verifier.md (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/capability.rs (modify)

## Out of scope

- Containment, the run step, replay, and any running of a studied system
- The Pages explainer design and publish step
- Studying firstmate or any real system; pinning a SHA
- Registering the capability in athena-cli wiring or athena.toml
- Changes to governance files, [lints] tables, or CI workflows
- Changing the Capability trait in athena-core, unless unavoidable and noted

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The new crate is Rust; its prompts are non-Rust markdown, as in athena-cap-change/prompts.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each validation rule gets a planted-fault test and a clean-case test; cargo test and clippy must be clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Narrowed to the validation core and the crate skeleton instead of the whole design.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Follows the reviewed design docs/design/study-capability-m1.md and its review 2 changes.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Evidence rules are enforced in code and are non-waivable; the verifier's judgments (§5.5) stay marked as prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing workspace crates (athena-data Quantity, athena-git, serde) and do not build a formal-language checker, per alternative A2-D.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I recovered the scope from the design doc, D64, and the architecture table.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Examples stay synthetic; no secrets or email addresses.

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
