# Job 20261012-185208-p1g0: Write review 2 re-checking the revised squash-chain landing design

## Request (verbatim)

> re-check the revised squash-chain landing design

## Reading (accepted by the user)

I read "re-check" as asking for a second independent review (review 2) of the revised docs/design/squash-chain-landing.md. It would confirm that each review-1 finding was answered and that the revised claims match the code and D69. This follows the review-2 precedent for the athena-v0 and study-capability-m1 designs.

Assumptions:
- 'Re-check' means write an independent review-2 document, as was done for athena-v0 and study-capability-m1, rather than edit the design.
- The design at HEAD is the revised version (it has a response appendix to review 1 and D69 mentions athena.candidate/2).
- The re-check does not run code; it reads, as review 1 did.
- CHANGELOG.md gets one line recording the review, as earlier reviews did.

## Acceptance criteria

1. [content] New file docs/design/reviews/squash-chain-landing-review-2.md exists, in the format of study-capability-m1-review-2.md: verdict, severity scale, stable finding ids (R2-...), and a 'what this review read and ran' section.
2. [content] Every review-1 finding (R1-M1..M3, R1-m1..m5, R1-n1..n2) and its three decisions for the user is checked against the revised design's response appendix (around line 271) and its stated disposition is confirmed, disputed, or marked unchecked, each with a cited section or code line.
3. [content] The revised design's new claims about code (invariant.rs, jobs.rs, history.rs, records.rs, attrs.rs) are re-checked against the current code, with line numbers, and mismatches are reported as findings.
4. [content] Check that D69 in docs/decisions.md and the pointers in docs/enforcement.md, docs/architecture.md and athena-v0.md agree with the revised design (e.g. schema athena.candidate/2, merge commits, rule-changing chains).
5. [content] The review claims only what it read; any number is measured or labelled as an estimate; no secrets or personal data appear.
6. [behavior] Run `cargo test --locked`; it passes, including the repository's wording scan over docs. Run `cargo fmt --check`; it is clean.
7. [content] No Rust code, governance file, the design itself, or anything under records/ is changed (design edits only if the review's findings are explicitly applied, which is out of scope here).

## Expected deliverables

- docs/design/reviews/squash-chain-landing-review-2.md (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/design/squash-chain-landing.md (modify)

## Out of scope

- Implementing chain landing in Rust (the build follow-up).
- Editing the design to apply review findings.
- Changing athena.toml, [history] exempt entries, or any governance file.
- Anything under records/.

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target (revised design) and the form (review 2) from the repo's review-1 file and earlier review-2 precedents.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The review cites sections and code line numbers and labels unmeasured claims.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review document plus a changelog line.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No personal data or secrets in the review.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader with stable finding ids.

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
