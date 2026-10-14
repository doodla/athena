# Job 20261014-072452-c42z: verify "Design a Socratic plan review in docs/design/socratic-review.md"

## Request (verbatim)

> Design (no code) a Socratic plan review, written to docs/design/socratic-review.md. It serves the BRIEF.md addendum '2026-10-14: Socratic questions', the 2026-10-13 addendum's 'good questions that generate good defaults', A1 (pushes back, open questions), A2-4 (mispredictions update defaults, estimates and routing), A3 (lessons from corrections turned into context, checks and examples) and A6-4 (Jev for rubric checks and gates). After a reading and before dispatch, Athena asks the plan questions of fixed kinds: assumptions, evidence the verifier can observe, file boundaries, decomposition, consistency of scope with deliverables, and root cause versus symptom. Answer each from the cheapest source first: facts from code, records or the host (for example 'does this repository allow merge commits?'), then Jev as a typed question with a probability, and only the residue goes to the user, ranked by how likely the answer changes the plan times the cost of being wrong, capped at one or two questions. Record every question and answer; repeated identical answers feed the defaults loop as proposals; a failure that introspection traces to an unasked or wrongly answered question raises that question's weight. Ground it in today's records, citing for each failure the question that would have caught it: D87 (merge commits assumed), 9gwv (criterion the verifier could not observe), 16zf (owner-only BRIEF.md planned), nem8 (design bundled with build), 8qkc and m1bs (scope versus deliverables), the four reader-budget retries, and the 'real repo' claim in PR #59. Also record whether a failure came from the plan, the execution, the verification or the environment, and make acceptance honest ('athena go' without a prompt is not the user's acceptance). Include alternatives, the cost per job, and the reader's token budget.

## Reading (accepted by the user)

Write one design document (no code) specifying a plan review between reading and dispatch: fixed question kinds, a cheapest-source-first answer ladder (facts, then Jev, then at most two user questions), recording, defaults-loop feedback, and failure attribution. It is grounded in the cited failures, with alternatives, per-job cost and the reader token budget.

Assumptions:
- The cited ids (D87, 9gwv, 16zf, nem8, 8qkc, m1bs, PR #59) are in records/ or docs/decisions.md, and the producer can read them from docs and BRIEF.md; where a record is unavailable the doc cites what is available and says so.
- Jev is the rubric/gate component named in A6-4 and BRIEF.md; the design treats it as an existing typed-question interface and does not specify its internals.
- Numbers for cost per job and reader budget are taken from athena.toml and records, or labeled as estimates.
- Design only: no Rust, no config change, no new defaults.toml entries.

## Acceptance criteria

1. [content] docs/design/socratic-review.md exists, contains no code changes elsewhere, and states the six fixed question kinds: assumptions, evidence the verifier can observe, file boundaries, decomposition, scope versus deliverables consistency, root cause versus symptom.
2. [content] It defines the answer ladder: facts from code, records or the host first; then Jev as a typed question with a probability; then only the residue to the user, ranked by likelihood of changing the plan times cost of being wrong, capped at two questions.
3. [content] It maps each failure to the question that would have caught it: D87, 9gwv, 16zf, nem8, 8qkc, m1bs, the four reader-budget retries, and the PR #59 'real repo' claim, citing the record or file each came from.
4. [content] It specifies recording every question and answer, repeated identical answers becoming defaults-loop proposals, and weight increases when introspection traces a failure to an unasked or wrongly answered question.
5. [content] It classifies each failure's origin as plan, execution, verification or environment, and defines honest acceptance (a bare 'athena go' without a prompt is not user acceptance).
6. [content] It links to the BRIEF.md addendum 2026-10-14 and the 2026-10-13 addendum, A1, A2-4, A3 and A6-4, and includes an alternatives section, a cost-per-job estimate labeled as estimate or measured with its source, and the reader's token budget.
7. [behavior] Run cargo test and cargo fmt --check; both still pass, since only docs changed (git diff --stat shows only docs/ and optionally CHANGELOG.md).

## Expected deliverables

- docs/design/socratic-review.md (create)
- docs/design/roadmap.md (modify)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Any Rust code, tests, or xtask changes
- Editing athena.toml, defaults.toml, or BRIEF.md
- Implementing Jev or the defaults-loop changes
- Reviewing the design (a separate review file under docs/design/reviews/)

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The deliverable is a written design with alternatives, to be reviewed before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each failure and number cites its source; unmeasured numbers are labeled estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader with terms defined.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new document; no code.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The design includes a root-cause-versus-symptom question kind and failure-origin attribution.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 7510a46b97d7 → candidate 5b4579dcb872. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                    |   5 +
 docs/design/requirements-map.md |   2 +-
 docs/design/roadmap.md          |   4 +-
 docs/design/socratic-review.md  | 246 ++++++++++++++++++++++++++++++++++++++++
 4 files changed, 255 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (536 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
