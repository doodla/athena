# Job 20261012-194212-fmwd: verify "Exempt PR #9 squash f2a8453 from the history check"

## Request (verbatim)

> PR #9 was squash-merged as f2a8453380196b95f2e2cc629ffa969af57e6c85 (OP-11); exempt it from the history check like ee4b0dd

## Reading (accepted by the user)

Add the PR #9 squash commit f2a8453380196b95f2e2cc629ffa969af57e6c85 to `[history] exempt` in athena.toml, as D67 did for ee4b0dd. Add a comment, a new decision D70 and a CHANGELOG line. All three give the user's reason: most of PR #9's jobs predate athena.candidate/2, so the chain rule cannot link them. OP-11 is not cited anywhere.

Assumptions:
- The new decision is D70, since D69 is the last entry in docs/decisions.md.
- The user's stated reason replaces any other reason; D69's chain rule requires athena.candidate/2 records, which PR #9's older jobs lack.
- The full id supplied is correct and in history; it cannot be checked from this snapshot.
- The change follows the ee4b0dd precedent in D67 and D68: config, a decision and a changelog line, with no code.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` contains the full id f2a8453380196b95f2e2cc629ffa969af57e6c85 and keeps the three existing entries unchanged.
2. [content] The comment above `exempt` in athena.toml says f2a8453 is the squash of PR #9 and gives the reason: most of PR #9's jobs predate athena.candidate/2, so the chain rule (D69) cannot link them. It does not mention OP-11.
3. [content] docs/decisions.md gains D70 after D69 with decision, alternatives, tradeoff, reversal (delete the entry and comment) and provenance (user request). It cites D60, D67, D68 and D69. It does not mention OP-11.
4. [content] CHANGELOG.md has an entry saying the history check no longer judges f2a8453, with the same reason. It does not mention OP-11.
5. [content] The string OP-11 appears in no changed file.
6. [behavior] Run `cargo test`: all tests pass, including any that parse athena.toml and validate the exempt list.
7. [behavior] Run `git diff --stat` or `git status`: only athena.toml, docs/decisions.md and CHANGELOG.md changed.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code, test or workflow change
- Restricting merge methods (D66 option c)
- Teaching the gate to link pre-candidate/2 jobs
- Citing OP-11 anywhere

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Took the format from the ee4b0dd precedent (D67, D68, the athena.toml comment, CHANGELOG) instead of asking.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a single exempt entry plus its comment, decision and changelog line.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The change contains only the commit id and the reason, and no OP-11 reference, as the user asked.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing history check and the D60/D61 exempt validation enforce the entry.

## The change

Base cc49ea4233b0 → candidate 73d00067a3a2. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md      |  4 ++++
 athena.toml       |  4 +++-
 docs/decisions.md | 22 ++++++++++++++++++++++
 3 files changed, 29 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (289 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
