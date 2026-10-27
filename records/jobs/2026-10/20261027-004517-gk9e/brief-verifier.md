# Job 20261027-004517-gk9e: verify "Move the Transport row from gh/mod.rs to gh/transport.rs"

## Request (verbatim)

> try again on 20261026-232359-msh3

## Reading (accepted by the user)

Retry of structure-pass job G2: move the Transport items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog; about 340 lines) verbatim into gh/transport.rs. Re-export them from gh/mod.rs so public paths stay unchanged, and add a CHANGELOG entry. The audience is the verifier applying section 5 of structure-pass.md.

Assumptions:
- The earlier rejection recorded no findings, so the same reading is reused with extra care on the section 5 proof (moved text identical, only import lines edited in tests).
- The G1 dependency has landed or is independent of these lines; Transport is a leaf.
- Line ranges in structure-pass.md (36-374) may have shifted; the producer locates items by name.
- Visibility is preserved; re-exports keep sibling code compiling without changing item text.

## Acceptance criteria

1. [content] gh/transport.rs holds the Transport row items (Runner, GhCli, GhFailure, parse_status, is_transient, transient_text, Retry, retry_transient, clock_entropy, HttpReply, HttpPost, CurlHttp, HostErrorLog) with the same text as before; git diff -M --color-moved shows only moved lines.
2. [content] gh/mod.rs declares `mod transport;` and re-exports each moved item, so no file outside gh/ changes its import path. Test modules have only import lines edited and the test list is unchanged.
3. [content] The report lists the caller-check Grep for each moved name's old path, as section 5 requires.
4. [content] Only gh/mod.rs, gh/transport.rs and CHANGELOG.md change, and CHANGELOG.md has an entry for the move. No config, workflow or other docs file changes.
5. [behavior] Run `cargo test --locked -- --list` and check that the test names match the list before the move. Run `cargo test --locked` and check that it passes, including the retry tests that use the moved code.
6. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both must be clean.

## Expected deliverables

- crates/athena-core/src/gh/transport.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of structure-pass.md section 2.1 (other G jobs)
- Any behavior change, renaming or refactor of the moved code
- Edits to docs/design/structure-pass.md or any config or workflow file

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt checks are listed as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test list is compared before and after, and the full suite is run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the three named files change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 0a7aea763929 → candidate 4ad89d7eab16. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   9 +
 crates/athena-core/src/gh/mod.rs       | 342 +--------------------------------
 crates/athena-core/src/gh/transport.rs | 340 ++++++++++++++++++++++++++++++++
 3 files changed, 359 insertions(+), 332 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1396 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked -- --list` and check that the test names match the list before the move. Run `cargo test --locked` and check that it passes, including the retry tests that use the moved code.; Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both must be clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
