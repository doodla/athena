# Job 20261027-012938-yckz: verify "Move Transport row from gh/mod.rs to gh/transport.rs"

## Request (verbatim)

> Priority: high
> Structure pass job G2 (docs/design/structure-pass.md, section 4). Move the Transport row to gh/transport.rs. No behavior change. Depends on 20261026-232356-ds51.
> Acceptance:
> - The proof of section 5 holds, shown only by what the verifier can run itself: existing tests pass with only import lines edited; the `cargo test -- --list` output is the same before and after; moved text is the same text; public paths are unchanged through re-exports; a Grep shows no file outside gh/ names a moved item by its old module path.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/transport.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G2: move the Transport items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) verbatim into gh/transport.rs, re-exported from gh/mod.rs so public paths are unchanged. No behavior change; audience is the verifier checking the section 5 proof with commands it can run itself.

Assumptions:
- G1 (gh.rs renamed to gh/mod.rs) has landed.
- The exact item list is taken from the Transport row of docs/design/structure-pass.md section 4; the producer follows that row.
- Moved items may need pub(super)/pub(crate) visibility or use-line adjustments; this is the only permitted text change.
- Other docs citing gh.rs are not edited (section 5 rule 7).

## Acceptance criteria

1. [content] gh/transport.rs holds the Transport-row items; their text matches the text removed from gh/mod.rs apart from visibility and use lines, visible in git diff.
2. [content] gh/mod.rs declares the transport module and re-exports every moved item, so existing paths such as crate::gh::Runner remain valid.
3. [content] A Grep the verifier runs shows no file outside crates/athena-core/src/gh/ names a moved item by an old module path (e.g. gh::transport::X where it was gh::X).
4. [content] Existing test code is unchanged except for import lines, visible in git diff.
5. [behavior] Run cargo test --locked -- --list on the base commit and on the change: the outputs are identical.
6. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
7. [content] CHANGELOG.md has an entry for G2 stating no behavior change.
8. [content] No criterion rests on the producer's report; all checks are runnable by the verifier.

## Expected deliverables

- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/transport.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Moving other rows of the structure pass
- Updating docs citing gh.rs
- Any behavior change
- Editing introspect.rs or the external_calls_guard test

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check test list parity and passing tests, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy, and cargo test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the commands itself.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test list diff and Grep are run by the verifier.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 352560ffe8d6 → candidate 353fc4f54a96. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   5 +
 crates/athena-core/src/gh/mod.rs       | 342 +--------------------------------
 crates/athena-core/src/gh/transport.rs | 340 ++++++++++++++++++++++++++++++++
 3 files changed, 355 insertions(+), 332 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1396 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
