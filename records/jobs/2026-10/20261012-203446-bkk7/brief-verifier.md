# Job 20261012-203446-bkk7: verify "List the boundary-scan banned words in the producer brief"

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

## The change

Base 0ed3073dedfb → candidate a0a40343b67d. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  3 ++
 Cargo.lock                                |  1 +
 crates/athena-context/prompts/producer.md |  1 +
 crates/athena-context/src/briefs.rs       | 83 +++++++++++++++++++++++++++++++
 docs/decisions.md                         | 19 +++++++
 docs/enforcement.md                       |  2 +-
 xtask/Cargo.toml                          |  1 +
 xtask/tests/policy.rs                     | 16 ++++++
 8 files changed, 125 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (293 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
