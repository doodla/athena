# Job 20261013-205753-m1bs: verify "Revise parallel-jobs design per review 2 changes N1–N6"

## Request (verbatim)

> Revise docs/design/parallel-jobs.md to make every change listed in section 4 of docs/design/reviews/parallel-jobs-review-2.md (N1 to N6), so the design reaches 'accept'. N1 is High and must be settled first. Design only; the build is the next job.

## Reading (accepted by the user)

Edit docs/design/parallel-jobs.md to apply review 2 section 4: N1 (local landing stays in the child, ordered by the repository lock; say who runs land --pr), then N2, N3, and the low items N4–N6 and closing question 9. Add a CHANGELOG.md entry in the usual style. No code.

Assumptions:
- Review 2 section 4 lists N1–N3 as required and N4–N6 plus question 9 closure as optional in text; the user asks for all, so all are done.
- N4–N6 detail is in review 2 section 3; the producer reads it there.
- The CHANGELOG entry goes under [Unreleased].

## Acceptance criteria

1. [content] Section 7 point 2 and section 8 of parallel-jobs.md state that local landing stays in the child, ordered by the repository lock, and say who runs land --pr and when; the claim that the rebuild precedes the next landing check is removed (N1).
2. [content] The doc states section 9 rules 1 and 2 take effect with the build and rule 3 is the working rule before then (N2).
3. [content] The doc says how the launcher distinguishes locked, refused, error and reread, without relying on message text, and the --json field list for reread is corrected (N3).
4. [content] N4 (open-PR exit code matches single-job command), N5 (--all default not buying work expected to go stale) and N6 (declared-paths field named as in the repository) are applied, and question 9 is marked closed.
5. [content] Only docs/design/parallel-jobs.md and CHANGELOG.md change; the CHANGELOG entry is under [Unreleased] in the existing style.
6. [behavior] Run git diff --stat and git status: only those two files are changed; cargo fmt --check still passes.

## Expected deliverables

- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)
- docs/design/reviews/parallel-jobs-review-2.md (modify)

## Out of scope

- Any code or tests
- Governance files
- records/
- Writing review 3

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered N1–N6 details from review 2 rather than asking.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design doc plus a changelog line.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims about current behavior cite the source file or section.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.

## The change

Base e549bc719bf6 → candidate 84545456684b. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  |   6 +
 docs/design/parallel-jobs.md                  | 162 ++++++++++++++++++--------
 docs/design/reviews/parallel-jobs-review-2.md |   5 +
 3 files changed, 125 insertions(+), 48 deletions(-)
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
