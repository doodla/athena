# Job 20261013-105334-78fs: verify "Review docs/design/native-pr-landing.md independently"

## Request (verbatim)

> Next on RM-0: review docs/design/native-pr-landing.md independently (its first review was rejected). The parallel-jobs review (F1) says it must come before the parallel pool. Fold in today's evidence: GitHub's squash appends a Co-authored-by paragraph that hides Athena-Job trailers, and a squash may name only one landed job.

## Reading (accepted by the user)

Write only a new independent review of native-pr-landing.md under docs/design/reviews/, with numbered, severity-rated findings and a verdict. Check it against today's evidence (squash Co-authored-by paragraph burying Athena-Job trailers; one landed job per squash) and the parallel-jobs F1 ordering. The design is not revised; that is a later job.

Assumptions:
- Review file is named native-pr-landing-review-N.md following existing naming; no review file for this design exists in the tree, so N is probably 2 (the rejected first review may be absent) or 1.
- Today's GitHub evidence is recorded in the review, not asserted as measured by the reviewer unless the reviewer can cite how it was observed.
- Existing docs tests may check review files or the requirements map; the new file should not break them.

## Acceptance criteria

1. [content] A new review file under docs/design/reviews/ for native-pr-landing has numbered findings, each citing the design section or line and a severity, and ends with a verdict.
2. [content] The review examines how Athena-Job trailers are found when GitHub's squash appends a Co-authored-by paragraph after them. It labels this as observed on 2026-10-13 and says how it was observed.
3. [content] The review examines the one-landed-job-per-squash limit: its effect on the chain rule (M30) and on pull requests holding several jobs.
4. [content] The review checks that the design is placed before the parallel pool, citing F1 in docs/design/reviews/parallel-jobs-review-1.md.
5. [content] Claims keep (read)/(design) labels. Numbers are cited or marked as estimates. No secrets or personal data, including email addresses, appear.
6. [content] native-pr-landing.md is not revised; the revision is left to a separate job. Only Markdown files are added, and no Rust, governance or records/ path changes.
7. [behavior] Run `git status` and `git diff --stat`: only .md files under docs/ are added or changed. Run `cargo test` and `cargo clippy`: both pass, so any docs-consistency tests accept the new file.

## Expected deliverables

- docs/design/reviews/native-pr-landing-review-2.md (create)
- docs/design/reviews/native-pr-landing-review-1.md (create)
- docs/design/native-pr-landing.md (modify)

## Out of scope

- Revising native-pr-landing.md
- Changing parallel-jobs.md
- Any Rust code, governance file or records/ change
- Implementing native PR landing

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Review only; one new Markdown file.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Findings cite design lines and label observations with date and method.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email addresses or personal data.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Run cargo test and clippy to confirm docs-consistency checks still pass.

## The change

Base 1fdefe3e4d41 → candidate a08e2950587a. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/design/reviews/native-pr-landing-review-2.md | 241 ++++++++++++++++++++++
 1 file changed, 241 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (327 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
