# Job 20261014-043917-63g6: Add defaults proposals: record, validation, status listing, apply check

## Request (verbatim)

> Build step 3 of the build order in docs/design/defaults-loop.md section 12: the proposal record and its validation, 'athena status' surfacing, and the apply-as-a-job check (section 9). When introspection records of one class reach the repeat threshold across distinct jobs, Athena writes a proposed default (id, statement, applies_to, version bump, sources citing those records), validated with DefaultsModel::parse and the evidence-after-decision rule. 'athena status' lists open proposals with their sources. Accepting one runs a normal job that edits defaults.toml, so a governance change is always a verified landing and never a direct write. Rejecting one is recorded and suppresses that class until new evidence arrives. No Jev in this step. Verify with fixture records in a temporary directory.

## Reading (accepted by the user)

Build step 3 of defaults-loop.md §12: from introspection records (cases) it clusters by failure class, and at N distinct jobs writes an athena.defaults-proposal/1 record validated with DefaultsModel::parse and the evidence-after-decision rule. `athena status` lists open proposals with sources. Accepting one makes an ordinary job with defaults.toml as a deliverable, checked against the proposal. Rejecting records a reason and suppresses the class until new evidence arrives. No Jev; tests use fixture records in a temp dir.

Assumptions:
- The repeat threshold N is a named constant in code (assumed 3 distinct jobs, per the §8.2 example), not a new athena.toml key, so no governance file changes.
- Clustering uses the existing failure_class id on recorded introspections (Records.cases) with distinct job ids; no Jev matching.
- The statement and applies_to are drafted by code from the class registry entry and the cases' cause text (no language model, no Jev); the user may edit them in the apply job.
- Proposals are stored as a new schema athena.defaults-proposal/1 in athena-data records.rs under records/, written the way introspections are; status transitions (rejected, accepted with job) append a new record rather than mutate.
- Accept is a new CLI path (athena accept-proposal <id> or similar) that submits an ordinary job request 'apply defaults proposal X' through the normal reading and acceptance flow; it never writes defaults.toml. The apply-as-a-job check is code that verifies the candidate defaults.toml: id and statement equal the proposal, version equals base plus one, and DefaultsModel::parse accepts it.
- Reject is a CLI command taking a reason; suppression holds until a class has at least N more distinct jobs of cases newer than the rejection (design §9).
- The §8 fields assumptions, signals and evidence (Jev probabilities) are omitted or empty in this step, since no Jev exists; the record notes this.

## Acceptance criteria

1. [content] A new athena.defaults-proposal/1 record type exists in athena-data with fields id, kind (add/amend), statement, applies_to, version {from,to}, sources (intervention/introspection ids with job ids), status (pending/accepted with job/rejected with reason/withdrawn) and candidate_sha256, and is registered in the record validation and schema listing.
2. [content] Proposal generation code reads introspection cases, groups them by failure class, and proposes only when cases span at least N distinct jobs; N is a named constant.
3. [content] Every proposal is validated by building a candidate defaults.toml and calling DefaultsModel::parse; the id must be unique among the model and pending proposals, the version bump is base plus one, and sources are checked with the existing evidence-after-decision rule.
4. [content] No code in the proposal path opens defaults.toml for writing; a test asserts this on the module source.
5. [behavior] Run `cargo test`: fixture-record tests in a temp dir show that (a) cases of one class across fewer than N jobs yield no proposal, (b) N distinct jobs yield one pending proposal citing those records, (c) repeats within the same job do not count, (d) a candidate that fails DefaultsModel::parse or cites evidence that postdates the decision is refused, (e) rejection suppresses the class until N newer distinct-job cases arrive, after which a proposal reappears.
6. [behavior] Run `cargo test`: a test shows `athena status` (text and --json) lists open proposals with id, kind, statement, version bump and source records, and omits rejected and accepted ones.
7. [behavior] Run `cargo test`: apply-check tests accept a candidate defaults.toml that matches the proposal and reject one with a wrong statement, wrong id, version not base+1, or a parse failure; accepting a proposal produces a job request listing defaults.toml as a deliverable instead of editing the file.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean. The design doc §12 step 3 is marked built with a short description, and CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-core/src/proposals.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/proposals.rs (create)
- crates/athena-data/src/lib.rs (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- README.md (modify)

## Out of scope

- Jev client, evaluate calls, matching or clustering by Jev (steps 5, 6)
- Measuring render() with the live counter (step 4)
- Writing athena.default-expectations/1 records or the assumptions/signals/evidence fields beyond what exists without Jev
- Any direct write to defaults.toml by Athena; editing defaults.toml itself
- Running over the real records/ directory
- Changes to athena.toml, CI, or other governance files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-record tests in a temporary directory; cargo test passes and clippy is clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt --check as the verifier's commands.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Follows the written design (§8, §9, §12) and updates §12 to mark step 3 built.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The 'never write defaults.toml' rule is enforced by a test on the module source, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change limited to step 3; Jev, render measurement and expectations records are excluded.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments, design doc update and changelog entry written for an outside reader.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Threshold, record names and CLI shape recovered from the design doc and existing introspection code.
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
