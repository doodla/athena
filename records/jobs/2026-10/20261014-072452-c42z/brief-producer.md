# Job 20261014-072452-c42z: Design a Socratic plan review in docs/design/socratic-review.md

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
