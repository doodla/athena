# Job 20261012-203446-bkk7: List the boundary-scan banned words in the producer brief

## Request (verbatim)

> producers keep failing the boundary-word test (4 times today, latest 20261012-202739-h77m); put the banned words in the producer brief up front

## Reading (accepted by the user)

Producers keep writing words the M19 boundary scan rejects (latest: "handoff" in a design review). I read this as adding a banned-words section to the generated producer brief. Rust builds the list from fragments, so no tracked file contains the words.

Assumptions:
- The brief is generated at runtime and stored under records/, which the scan skips, so the list can appear there. Tracked prompt files are scanned, so the list must be assembled in Rust from fragments.
- Briefs for all capabilities go through producer_brief, so one change covers them.
- The list is duplicated from xtask/tests/policy.rs unless xtask may depend on athena-context. The dependency-direction test probably forbids that dependency.

## Acceptance criteria

1. [content] producer_brief in crates/athena-context/src/briefs.rs emits a section, placed before the Checks section, that tells the producer not to write the boundary-scan words and fragments (singular, plural and hyphenated forms included) in any file, and explains that the scan is whole-word and case-insensitive.
2. [content] No tracked file contains a banned term literally. The Rust source builds each term from fragments, as xtask/tests/policy.rs does, and crates/athena-context/prompts/producer.md stays free of them.
3. [behavior] A new unit test in athena-context renders a producer brief and asserts it names every word in the boundary list. Run `cargo test -p athena-context`.
4. [behavior] The full suite passes, including athena_refers_to_no_one_who_runs_it_or_where, and clippy is clean. Run `cargo test --locked` and `cargo clippy --all-targets`.
5. [content] The word list has one stated owner, or a test fails when the brief's list and the xtask scanner's list differ. The list cannot drift silently.
6. [content] The M19 row in docs/enforcement.md mentions that the producer brief lists the banned words up front. The row stays accurate and still passes the enforcement-map test.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- xtask/tests/policy.rs (modify)
- docs/enforcement.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the scanner's word list or the scan's behavior
- Changing the reader or verifier briefs
- Fixing the existing review document that failed
- Editing records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A unit test checks that the brief names every banned word.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the producer brief.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test ties the brief's list to the scanner's list.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The brief text explains why the words are banned, without naming who runs Athena.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I found the scanner and the brief builder in the repository instead of asking.

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
