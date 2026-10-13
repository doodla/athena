# Job 20261013-210357-vwx5: verify "Revise parallel-jobs design for review 2 N1-N6; changelog only"

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

## The change

Base 79affe1a7fec → candidate 8f35f4ec011d. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                 |  12 ++-
 docs/design/parallel-jobs.md | 177 ++++++++++++++++++++++++++++---------------
 2 files changed, 129 insertions(+), 60 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (376 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
