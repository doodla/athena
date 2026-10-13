# Job 20261013-210357-vwx5: Revise parallel-jobs design for review 2 N1-N6; changelog only

## Request (verbatim)

> Retry 20261013-205753-m1bs: same revision of docs/design/parallel-jobs.md for review 2's N1-N6, with a CHANGELOG.md entry; do not edit the review file or any other file. Re-wrap edited paragraphs to the doc's width.

## Reading (accepted by the user)

Redo the previous job's revision of docs/design/parallel-jobs.md to settle N1 (High, first) then N2-N6 from section 4 of review 2, and add a CHANGELOG.md entry. Edit only those two files, not the review file, and re-wrap every edited paragraph to the doc's width.

Assumptions:
- The doc's wrap width is about 110 columns, matching CHANGELOG.md; the producer should confirm from existing lines.
- The previous attempt's content was otherwise sound, so only the file scope and wrapping need to change.

## Acceptance criteria

1. [content] Section 7 point 2 and section 8 of parallel-jobs.md keep local landing in the child, ordered as review 2 N1 requires, and N1 is settled before the other items.
2. [content] N2 to N6 of review 2 section 4 are each addressed in parallel-jobs.md: section 9 rules 1 and 2 take effect with the build and rule 3 is the working rule; how the launcher tells locked, refused, error and reread apart, with the --json text corrected; the exit code of an open pull request; the --all default; and the field holding declared paths is named.
3. [content] CHANGELOG.md has an entry under [Unreleased] describing the revision, and the existing wording that says the design is not edited is kept accurate.
4. [content] Edited paragraphs in parallel-jobs.md are re-wrapped to the doc's existing width, with no overlong lines, including the kill-switch paragraph in section 4.2.
5. [behavior] Run `git status` and `git diff --stat`: only docs/design/parallel-jobs.md and CHANGELOG.md are changed; docs/design/reviews/parallel-jobs-review-2.md is untouched.

## Expected deliverables

- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing docs/design/reviews/parallel-jobs-review-2.md or any other file
- Any code or build work for parallel jobs
- Governance files

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only two files change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design text and changelog are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The scope violation and the wrap slip are both fixed; all edited paragraphs are re-wrapped, not just one line.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier checks the diff scope and the wrapping.

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
