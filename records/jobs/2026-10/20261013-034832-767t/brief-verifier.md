# Job 20261013-034832-767t: verify "Study firstmate's work handling (M1) pinned at 6407052f"

## Request (verbatim)

> Retry 20261013-033155-kmh0.

## Reading (accepted by the user)

Retry the M1 study of firstmate's work handling, pinned to the full commit SHA 6407052f4c521e39401ce2305dfa45d201249827 that the user gave (pinned-source/ itself has no SHA). The study must carry that SHA in pin.json and every citation, and its validation must be reproducible by a kept test. It must also fix the earlier minor flaws.

Assumptions:
- The pin SHA is 6407052f4c521e39401ce2305dfa45d201249827, as the user stated; pinned-source/ contents are read as that commit.
- The studies/ tree from the failed attempt is absent, so the study is created from scratch.
- The study follows the typed records in crates/athena-cap-study (pin, claims, rationales, model, validation).
- The earlier tree's docs/non-rust-files.md studies/** row is still needed for the new JSON files.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/pin.json holds the full SHA 6407052f4c521e39401ce2305dfa45d201249827; no placeholder such as UNRESOLVED-PIN-SHA appears anywhere in the study.
2. [content] Every citation in claims.json and rationales.json carries that SHA, an existing path and line range, and a verbatim quote from the pinned tree.
3. [behavior] Run cargo test: a kept test in the repository validates studies/firstmate-work-handling, including the wrong-SHA and quote-match rules, and passes.
4. [content] Every claim is tagged mechanism or prose-only, never 'not examined'. Model transitions with guard 'none' do not say code evaluates a guard.
5. [content] docs/non-rust-files.md lists the studies/** JSON files with their reason, and CHANGELOG.md names the change.
6. [behavior] Run cargo fmt --check and cargo clippy; both report no problems.

## Expected deliverables

- studies/firstmate-work-handling/pin.json (create)
- studies/firstmate-work-handling/claims.json (create)
- studies/firstmate-work-handling/rationales.json (create)
- studies/firstmate-work-handling/model.json (create)
- studies/firstmate-work-handling/validation.json (create)
- crates/athena-cap-study/src/rules.rs (modify)
- docs/non-rust-files.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the study capability's design or rules beyond what a kept validation test needs.
- Studying systems other than firstmate's work handling.
- Editing records/, athena.toml, defaults.toml, or CI configuration.

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites the pinned SHA, path, line range and a verbatim quote.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A kept test validates the study against real pinned-source text; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Validation must be reproducible by the verifier running cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The JSON study files are non-Rust, so each is listed with its reason in docs/non-rust-files.md.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Every claim's enforcement is tagged mechanism or prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study plus the minimum test and doc changes.

## The change

Base eae57fee143e → candidate c050407604c4. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                    |   5 +
 crates/athena-cap-study/src/rules.rs            | 231 ++++++++++++++++++++++++
 docs/decisions.md                               |  26 +++
 docs/non-rust-files.md                          |   1 +
 studies/firstmate-work-handling/claims.json     | 165 +++++++++++++++++
 studies/firstmate-work-handling/findings.json   |  20 ++
 studies/firstmate-work-handling/model.json      | 108 +++++++++++
 studies/firstmate-work-handling/pin.json        |  10 +
 studies/firstmate-work-handling/rationales.json |  65 +++++++
 studies/firstmate-work-handling/validation.json |  24 +++
 10 files changed, 655 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (306 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
