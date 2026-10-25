# Job 20261024-114125-7z2t: Add reconcile step for missing landing records at land and status

## Request (verbatim)

> Build job 3, part 2, of the records-history plan (docs/design/records-history.md section 8.1, decision D111): a reconcile step that finds main's first-parent commits since the last landing record that have no landing record, matches each to a landing-intent record by head tree and expected base, and writes the missing landing record; with no matching intent it asks the host which pull request merged, and if the host cannot answer it reports the commit as unattributed and writes nothing. Run it at the start of athena land and athena status. Tests: a crash between the host's merge and the landing record is repaired from the intent alone; a host that is unavailable gives 'unattributed' and no record; a commit with no intent and no pull request is reported, never guessed.

## Reading (accepted by the user)

Add a bounded reconcile step in athena-core (records-history §8.1, D111) that repairs missing landing records on main's first-parent commits from landing intents, or from the host, else reports 'unattributed'. It runs at the start of `athena land` and `athena status`. The audience is the maintainer and verifier.

Assumptions:
- The walk starts after the later of the last landing record and the first landing-intent record on athena/records.
- Matching requires both head tree and expected base (parent) to equal the intent's.
- Unattributed commits are reported in output, not treated as errors that abort land or status.
- Exact module placement in athena-core is my guess from file names; I did not read the code.

## Acceptance criteria

1. [behavior] Crash repair: a test has a main first-parent commit whose tree and parent match a landing-intent and that has no landing record. Reconcile writes the landing record from the intent alone, with the host making no call. Check with `cargo test`.
2. [behavior] Host unavailable: a commit with no matching intent and an erroring host gives an 'unattributed' finding and writes no landing record. Check with `cargo test`.
3. [behavior] No guessing: a commit with no intent and no merged PR is reported unattributed and nothing is written. It is never attributed to a nearby intent or PR. Check with `cargo test`.
4. [behavior] Host fallback: with no matching intent and the host naming the merged PR, the landing record is written for that PR. Commits that already have a landing record are skipped. Check with `cargo test`.
5. [behavior] Bound: commits before the first landing-intent record on athena/records, and before the last landing record, are never reconciled. When the records branch is absent or holds no intent, reconcile does nothing and reports that. A test covers both cases. Check with `cargo test`.
6. [behavior] `athena land` and `athena status` run reconcile first, and an unattributed commit appears in their output. A test shows this. `cargo test`, `cargo clippy` and `cargo fmt --check` are clean.
7. [content] CHANGELOG.md has an entry citing docs/design/records-history.md §8.1, D111, and job 3 part 2.
8. [content] The change reuses the existing host and git helpers and record types, adds no dependency, and changes no governance file.

## Expected deliverables

- crates/athena-core/src/reconcile.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-data/src/history.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the existing history check or the cutover
- Writing records for unattributed commits
- Changes to landing-intent creation

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus clippy and fmt criteria.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing helpers; no new dependency.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check behavior via cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused reconcile step.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check repaired or reported outcomes.
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

Crash repair: a test has a main first-parent commit whose tree and parent match a landing-intent and that has no landing record. Reconcile writes the landing record from the intent alone, with the host making no call. Check with `cargo test`.
Host unavailable: a commit with no matching intent and an erroring host gives an 'unattributed' finding and writes no landing record. Check with `cargo test`.
No guessing: a commit with no intent and no merged PR is reported unattributed and nothing is written. It is never attributed to a nearby intent or PR. Check with `cargo test`.
Host fallback: with no matching intent and the host naming the merged PR, the landing record is written for that PR. Commits that already have a landing record are skipped. Check with `cargo test`.
Bound: commits before the first landing-intent record on athena/records, and before the last landing record, are never reconciled. When the records branch is absent or holds no intent, reconcile does nothing and reports that. A test covers both cases. Check with `cargo test`.
`athena land` and `athena status` run reconcile first, and an unattributed commit appears in their output. A test shows this. `cargo test`, `cargo clippy` and `cargo fmt --check` are clean.
CHANGELOG.md has an entry citing docs/design/records-history.md §8.1, D111, and job 3 part 2.
The change reuses the existing host and git helpers and record types, adds no dependency, and changes no governance file.

## Report

End with the producer report as your structured output.
