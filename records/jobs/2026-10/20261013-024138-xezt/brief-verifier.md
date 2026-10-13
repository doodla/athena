# Job 20261013-024138-xezt: verify "Exempt squash 5a73478 (PR #14) from the history check"

## Request (verbatim)

> Retry 20261013-024016-5s8s: exempt 5a734782e76daac9b12bd8833c97a73927c8cf94 (squash of #14; changes records/ but names no job) in [history] exempt. The producer's snapshot has no git history, so use this id.

## Reading (accepted by the user)

Add the full id 5a734782e76daac9b12bd8833c97a73927c8cf94 to `[history] exempt` in athena.toml with a reason comment, and record it as a new decision (D76) in docs/decisions.md, following D70. No code, test, or workflow changes.

Assumptions:
- The user-supplied id is a commit already in main's history, as D60 requires; the snapshot has no git history, so the producer cannot check it.
- The reason is the user's statement: the squash of #14 changes records/ but names no job.
- The next decision number is D76 (the last is D75).
- The exemption entry is appended after f2a8453 in the existing list.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` contains the full 40-character id 5a734782e76daac9b12bd8833c97a73927c8cf94, and all four existing entries remain.
2. [content] A comment above `exempt` says 5a734782e76daac9b12bd8833c97a73927c8cf94 is the squash of PR #14, which changes records/ but names no job.
3. [content] docs/decisions.md has a new D76 after D75 with Decision, Alternatives, Tradeoff, Reversal, and Provenance, stating that the id was supplied by the user and not checked against history.
4. [content] No file other than athena.toml and docs/decisions.md changes; no code, test, or workflow changes.
5. [behavior] Run `cargo test` and `cargo clippy` from the repository root: both pass, including any test that parses athena.toml's exempt list. Run `cargo fmt --check`: it passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the history check code or its rules
- Rewriting main's history
- Adding the missing job records for #14
- Changing CI workflows

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The exemption is written down as decision D76 with alternatives, tradeoff, and reversal, as D70 did.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by the existing history check and the D60/D61 validation of exempt entries; no new rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One config entry plus one decision record.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Only a commit id and prose are added; no personal data.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Format and wording recovered from D70 and the existing athena.toml comments.

## The change

Base 4042e4278935 → candidate f19d38a7a609. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md      |  2 ++
 athena.toml       |  4 +++-
 docs/decisions.md | 22 ++++++++++++++++++++++
 3 files changed, 27 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (302 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
