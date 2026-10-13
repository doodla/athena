# Job 20261013-204535-djek: Write second review of the revised parallel-jobs design

## Request (verbatim)

> Review the revised docs/design/parallel-jobs.md as a second review, written to docs/design/reviews/parallel-jobs-review-2.md. Check each answer to review 1's findings F1 to F8 against the source code and docs/design/native-pr-landing.md, look for new problems, and end with a verdict: accept, accept with changes (listing them), or reject. Review only; do not change the design.

## Reading (accepted by the user)

Add docs/design/reviews/parallel-jobs-review-2.md. It checks each answer to review 1's F1-F8 against the source code and native-pr-landing.md, lists new problems, and ends with a verdict. The design is untouched. CHANGELOG.md gets an entry because past reviews have one.

Assumptions:
- Review 1 is docs/design/reviews/parallel-jobs-review-1.md and its findings are labelled F1-F8.
- The CHANGELOG.md entry is allowed because existing review files have entries, for example the squash-chain review 2 entry.
- Claims in the review cite the source code and docs; any number not measured is labelled as an estimate.
- The file follows the format of existing reviews, for example native-pr-landing-review-3.md.

## Acceptance criteria

1. [content] docs/design/reviews/parallel-jobs-review-2.md exists and has one assessment per finding F1-F8 of parallel-jobs-review-1.md, each with a status: resolved, partly resolved, or unresolved.
2. [content] Each F1-F8 assessment cites the source files and functions and the native-pr-landing.md sections it was checked against, and quotes or points to the revised design's answer.
3. [content] A new-problems section lists each problem with a severity and evidence (file or section), or states that none were found.
4. [content] The review ends with a verdict that is exactly one of: accept, accept with changes (changes listed), or reject.
5. [content] CHANGELOG.md has a short entry for the new review, in the style of the existing review entries.
6. [behavior] Run git status and git diff --stat: the only changes are the new review file and CHANGELOG.md. docs/design/parallel-jobs.md and every other file are unchanged.

## Expected deliverables

- docs/design/reviews/parallel-jobs-review-2.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Any change to docs/design/parallel-jobs.md
- Any change to source code, tests, or configuration
- Fixing the problems the review finds
- Changes to review 1 or other earlier reviews

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written so an outside reader can follow it without prior context.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each status cites a file, function, or section; unmeasured numbers are labelled as estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review file plus a changelog entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verification is by git status and git diff on the file set, plus a read of the criteria.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The CHANGELOG convention and the review format were recovered from the repository.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
