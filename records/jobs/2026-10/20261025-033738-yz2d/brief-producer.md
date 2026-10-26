# Job 20261025-033738-yz2d: Run defaults proposals automatically after introspection and at drain

## Request (verbatim)

> Priority: high
> Run athena propose automatically (BRIEF.md addendum of 2026-10-25): it turns a failure class seen across enough jobs into a defaults proposal, but it only runs by hand, nobody runs it, and no proposal exists although classes have recurred 5 to 27 times. Run it after each introspection is recorded (and at the start of athena drain), so proposals appear in athena status, and within the steward's limit are worked like other approved items.
> Acceptance:
> - A test shows an introspection that brings a class to the threshold produces a pending proposal without a separate command.
> - A class already proposed or rejected is not proposed again (existing rule kept).
> - athena status lists the new proposal.
> Files: crates/athena-core/src/introspect.rs, crates/athena-core/src/propose.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Make `Athena::propose` run without a separate command: after each introspection is recorded, and once at the start of `athena drain`, so pending proposals show in `athena status` and in-limit ones are approved and worked by drain's existing steward path. Audience: the maintainer who reads status and the verifier running cargo test.

Assumptions:
- crates/athena-core/src/propose.rs does not exist; the logic is in proposals.rs (Athena::propose, generate), so that file is the one modified.
- Automatic proposing is hooked in introspect_job/introspect_intervention (never-failing wrappers) and a new Steps method called at drain start; existing generate rules (threshold, no repeat for pending/rejected classes) are kept as is.
- Existing `athena propose` command stays.

## Acceptance criteria

1. [behavior] A test (run with cargo test) records an introspection that brings a failure class to the distinct-job threshold and then finds a pending defaults proposal, without calling `athena propose` separately.
2. [behavior] A test (cargo test) shows a class that already has a pending/accepted proposal, or was rejected with no N newer distinct jobs, gets no new proposal after another introspection or drain start.
3. [behavior] A test (cargo test) shows the status output lists the automatically created proposal.
4. [behavior] A drain test (cargo test) shows propose runs at drain start, and an in-limit proposal is then approved and worked under the existing steward limit; an above-limit one is listed as waiting.
5. [behavior] A failure inside automatic proposing only prints a line and never fails the introspection or the drain; cargo test, cargo clippy and cargo fmt --check pass.
6. [content] CHANGELOG.md has an entry describing automatic proposing; docs of the introspect/drain modules mention it. defaults.toml, athena.toml and workflows are unchanged.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/proposals.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Changing the proposal threshold or the generation/rejection rules
- Editing defaults.toml or accepting proposals automatically beyond the existing steward ladder
- Removing the manual `athena propose` command

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for threshold, no re-proposal, status listing; clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check proposals appearing from introspection/drain, not wiring.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria runnable via cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused hook change reusing Athena::propose.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All Rust.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test (run with cargo test) records an introspection that brings a failure class to the distinct-job threshold and then finds a pending defaults proposal, without calling `athena propose` separately.
A test (cargo test) shows a class that already has a pending/accepted proposal, or was rejected with no N newer distinct jobs, gets no new proposal after another introspection or drain start.
A test (cargo test) shows the status output lists the automatically created proposal.
A drain test (cargo test) shows propose runs at drain start, and an in-limit proposal is then approved and worked under the existing steward limit; an above-limit one is listed as waiting.
A failure inside automatic proposing only prints a line and never fails the introspection or the drain; cargo test, cargo clippy and cargo fmt --check pass.
CHANGELOG.md has an entry describing automatic proposing; docs of the introspect/drain modules mention it. defaults.toml, athena.toml and workflows are unchanged.

## Report

End with the producer report as your structured output.
