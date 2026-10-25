# Job 20261025-150509-dmwc: Raise producer_timeout_s to 3000 and record the decision

## Request (verbatim)

> Priority: urgent
> Raise the producer deadline in athena.toml [budget] producer_timeout_s from 1800 to 3000 seconds. Six producer runs since 2026-10-24 hit the 1800 s deadline and lost all their work (jobs 20261019-013309-nvbb, 20261023-234149-jcd2, 20261024-140624-ewbs, 20261025-034731-2am0, 20261017-044405-4zcq, and the one before nvbb), several doing legitimate work whose targeted test builds take 5 to 10 minutes. Record the decision with its alternatives (splitting jobs, a faster suite, keeping work on timeout, each also queued) and the reversal.
> Acceptance:
> - athena.toml has producer_timeout_s = 3000 and parses (the repository config test passes).
> - docs/decisions.md has the decision entry; CHANGELOG.md has an entry.
> Files: athena.toml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Change athena.toml [budget] producer_timeout_s from 1800 to 3000, add a decision entry in docs/decisions.md (alternatives: splitting jobs, faster suite, keeping work on timeout; plus reversal), and add a CHANGELOG Unreleased entry. The audience is the Athena operator, who needs producers to stop losing work at the deadline.

Assumptions:
- The new decision is appended to docs/decisions.md as the next numbered entry after the last existing one (D15 is seen, later ones may exist), in the same heading format.
- The three alternatives are recorded as queued follow-ups, not built here.
- Reversal means the steps and the signal for setting the value back to 1800 (for example, if no timeouts occur in a stated number of jobs, or cost rises).
- The six job count comes from the request; only five job ids are named, so the entry names those five and says the sixth is unnamed.
- No code changes; verifier_timeout_s and other budget values are untouched. Other docs that may quote 1800 are checked and updated only if they state the current value.

## Acceptance criteria

1. [content] athena.toml [budget] has producer_timeout_s = 3000 and no other budget value changes.
2. [content] docs/decisions.md has a new entry naming the 1800 to 3000 change, the evidence (the named failed jobs), the alternatives (splitting jobs, a faster suite, keeping work on timeout, each marked queued), and how to reverse it.
3. [content] CHANGELOG.md has an entry under [Unreleased] describing the producer deadline raise from 1800 to 3000 seconds and why.
4. [behavior] Run `cargo test`: the repository config test that parses athena.toml passes with producer_timeout_s = 3000, and the whole suite passes; `cargo clippy` is clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Splitting jobs, speeding up the test suite, or preserving work on timeout (recorded as queued alternatives only)
- Changing verifier_timeout_s or other budget settings
- Code changes in crates/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision entry cites the named job ids and the 5 to 10 minute test build figure as the request's claim, labeled as reported.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The entry is written so an outside reader understands the context, options, and reversal.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Three files only: one config value, one decision entry, one changelog line.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test (including the config parse test) passes and clippy is clean.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that athena.toml parses with 3000 through the repo config test, not that the file was edited.
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

athena.toml [budget] has producer_timeout_s = 3000 and no other budget value changes.
docs/decisions.md has a new entry naming the 1800 to 3000 change, the evidence (the named failed jobs), the alternatives (splitting jobs, a faster suite, keeping work on timeout, each marked queued), and how to reverse it.
CHANGELOG.md has an entry under [Unreleased] describing the producer deadline raise from 1800 to 3000 seconds and why.
Run `cargo test`: the repository config test that parses athena.toml passes with producer_timeout_s = 3000, and the whole suite passes; `cargo clippy` is clean.

## Report

End with the producer report as your structured output.
