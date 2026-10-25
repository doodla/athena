# Job 20261012-084057-0eaq: Write independent review 2 of the revised study capability M1 design

## Request (verbatim)

> re-check the revised study design

## Reading (accepted by the user)

I read 'the revised study design' as docs/design/study-capability-m1.md, which was revised after review 1 and says it needs a re-check before building. The job writes an independent review 2 in docs/design/reviews/. It checks the review-1 dispositions and the revision for new defects against the §9 checks. It does not edit the design.

Assumptions:
- 'The revised study design' is docs/design/study-capability-m1.md, which its own header says was revised after review 1 and needs a re-check (review 2) before building.
- 'Re-check' means write an independent review 2 in docs/design/reviews/, not edit the design. Findings are reported to the user; the design is left for its author to answer.
- No model call or run of a studied system is needed; the review is by reading and by checks that only read the repository, as in review 1.
- The review does not pass or fail the design for the user; it gives a verdict and lists decisions for the user.

## Acceptance criteria

1. [content] A new file docs/design/reviews/study-capability-m1-review-2.md exists, in the format of review 1: verdict, severity scale, stable finding ids, a section per check, decisions for the user, and an appendix of experiments.
2. [content] The review follows the eight checks of §9 of docs/design/study-capability-m1.md and records a result (pass or findings) for each.
3. [content] The review verifies each of the 15 dispositions and the 8 decisions in Appendix A against the revised design text, citing section and line, and says where a claimed fix is absent or only partial.
4. [content] The review checks the revision for new defects. Candidates: the 'Quote mismatch' rule is classed as evidence but Appendix A names only 'six' evidence rules; the rule counts and classes are inconsistent; §5.4 'Unsupported number' cannot test a missing label if the type refuses it; whether §2.1 containment and the provider-side spend limit are feasible; and whether the A4-A 'chosen for review' wording is stale.
5. [content] Each BRIEF.md line number cited in the review matches BRIEF.md in the tree, and each in-document anchor the review relies on resolves.
6. [content] The review makes no claim about firstmate and states its verdict (approve, approve with changes, or return) and the consequence under §9 for scheduling a build.
7. [content] The review file is the only reviewed document changed: docs/design/study-capability-m1.md is not edited by this job, and the existing review-1 file is unchanged.
8. [behavior] Run `git status` and `git diff --stat`: only the new review file (and optionally a CHANGELOG.md entry) appears. Run `cargo test` and confirm it passes, including the xtask policy tests.

## Expected deliverables

- docs/design/reviews/study-capability-m1-review-2.md (create)
- CHANGELOG.md (modify)
- docs/design/study-capability-m1.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Editing docs/design/study-capability-m1.md to fix findings
- Creating the athena-cap-study crate or any Rust code
- Studying, reading, or running firstmate
- Writing the Pages publishing design
- Changing governance files such as athena.toml, defaults.toml, CI workflows, or Cargo.toml lints

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The request names no file. The repository shows one design that asks for a re-check, so I read it that way and did not ask.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every finding in the review must cite a section or a BRIEF.md line, and counts are marked as counted or estimated.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written to stand alone, in the format of review 1.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review file; the design itself is not edited.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The review's own citations and anchors are checked against the tree, and cargo test must still pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Re-checking is the §9 review gate for a significant design before it is built.

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
