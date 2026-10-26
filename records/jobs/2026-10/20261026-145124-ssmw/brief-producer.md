# Job 20261026-145124-ssmw: Run defaults proposals automatically after introspection and at drain

## Request (verbatim)

> try again on 20261025-033738-yz2d

## Reading (accepted by the user)

Retry of job 20261025-033738-yz2d, which failed only for low disk (environment), not for its work. Redo the original change: run `Athena::propose` automatically after each recorded introspection and once at drain start, so pending proposals appear in `athena status` and in-limit ones go through drain's existing steward path. Audience: the maintainer reading status, and the verifier running cargo test.

Assumptions:
- The typed request 'try again' takes precedence over the earlier introspection's proposed low-disk request, so the original proposing task is redone, not the disk-handling design.
- The earlier failure was environmental (1.5 GB free, 10 GB required), so the earlier accepted reading stands unchanged.
- The propose logic lives in crates/athena-core/src/proposals.rs (the request named propose.rs); the producer confirms the real path.

## Acceptance criteria

1. [behavior] A test (cargo test) records an introspection that brings a failure class to the distinct-job threshold and then finds a pending defaults proposal, without calling `athena propose` separately.
2. [behavior] A test (cargo test) shows a class that already has a pending or accepted proposal, or was rejected with fewer than N newer distinct jobs, gets no new proposal after another introspection or drain start.
3. [behavior] A test (cargo test) shows the status output lists the automatically created proposal.
4. [behavior] A drain test (cargo test) shows propose runs at drain start, and an in-limit proposal is then approved and worked under the existing steward limit while an above-limit one is listed as waiting.
5. [behavior] A failure inside automatic proposing only prints a line and never fails the introspection or the drain; cargo test, cargo clippy and cargo fmt --check pass.
6. [content] CHANGELOG.md has an entry describing automatic proposing, and the introspect and drain module docs mention it. defaults.toml, athena.toml and .github/workflows are unchanged.

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

- Low-disk cleanup or hold behavior proposed by the introspection
- Changing the proposal threshold, steward limit, or defaults.toml/athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt --check to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check proposals appearing without a separate command and showing in status.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the outcome: no manual propose needed, drain works in-limit proposals.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change around the existing propose function.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and module docs updated for outside readers.
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

A test (cargo test) records an introspection that brings a failure class to the distinct-job threshold and then finds a pending defaults proposal, without calling `athena propose` separately.
A test (cargo test) shows a class that already has a pending or accepted proposal, or was rejected with fewer than N newer distinct jobs, gets no new proposal after another introspection or drain start.
A test (cargo test) shows the status output lists the automatically created proposal.
A drain test (cargo test) shows propose runs at drain start, and an in-limit proposal is then approved and worked under the existing steward limit while an above-limit one is listed as waiting.
A failure inside automatic proposing only prints a line and never fails the introspection or the drain; cargo test, cargo clippy and cargo fmt --check pass.
CHANGELOG.md has an entry describing automatic proposing, and the introspect and drain module docs mention it. defaults.toml, athena.toml and .github/workflows are unchanged.

## Report

End with the producer report as your structured output.
