# Job 20261014-174937-hrzn: Redo role-prompt goal-preservation change; fix every stale 1,800 mention

## Request (verbatim)

> try again on 20261014-172342-rqgb

## Reading (accepted by the user)

Redo the rejected job from the base: edit the reader, producer and verifier prompts so the request's real goal survives to the verdict, make a non-empty request_reading_mismatch a Major finding in judge(), raise the reader budget minimally. This time grep every 1,800 mention, including socratic-review.md:176, and rewrap edited lines to file width.

Assumptions:
- The audience for this change is the maintainers and the Athena roles that read these prompts and docs; the earlier reading's criteria stand.
- The prior rejections' findings are the checked list: socratic-review.md:176 (and :219, :198 if present tense), decision-sets.md:111 and defaults-loop.md:227 over-wide lines, and the decision-sets.md:111 nit mixing current numbers into a dated review-1 measurement.
- Past-tense review and history records (docs/design/reviews/*, decisions.md:1971, defaults-loop.md:227's review-1 measurement) may keep 1,800 as dated facts; timeout_s 1800 is unrelated.
- The new budget is the measured count of the new reader set plus minimal headroom, as in the earlier attempt (about 1,910).

## Acceptance criteria

1. [content] reader.md tells the reader to name the request's audience (who will use or judge the result) in the summary or assumptions.
2. [content] reader.md requires at least one acceptance criterion written as an outcome the named audience can check; presence checks alone do not satisfy it.
3. [content] reader.md forbids putting the request's own stated goal in out_of_scope, and tells the reader to read the current output the request is about before planning a change to it.
4. [content] producer.md treats the acceptance criteria as a floor and tells the producer to serve the request's goal, while keeping the small-change and follow-conventions instructions.
5. [content] verifier.md tells the verifier to judge user-facing output as its intended audience would, and to rate a request-reading mismatch at least major, never minor because the criteria passed.
6. [behavior] In judge() in crates/athena-core/src/verify.rs, a non-empty request_reading_mismatch adds a Major finding and a reason, so the verdict is not pass. Unit tests cover mismatch present and absent or blank. Run: cargo test -p athena-core verify
7. [behavior] The BudgetSet::Reader budget is raised only as far as the new reader text needs. Run: cargo test --locked -p athena-cli --test budgets
8. [content] docs/decisions.md records the new reader budget, the measured count and why removals cannot pay for the new rules; CHANGELOG.md notes the change.
9. [content] A repo-wide Grep for 1,800 and 1800 finds no present-tense statement of the old reader budget: defaults-loop.md:47, :227, :413; decision-sets.md:107, :111; decisions.md:612; athena-v0.md:1507, :1868; socratic-review.md:176, :198 if present tense, :219 are updated. Past-tense review and history records may keep it.
10. [content] No rewritten doc line is much wider than its file's wrapping (Grep for lines of 125 or more characters on the edited lines finds none newly added), and dated review-1 measurements stay purely past figures rather than mixing in the new budget.
11. [behavior] The workspace stays clean and tested. Run: cargo fmt --check, cargo clippy, cargo test

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/decision-sets.md (modify)
- docs/design/defaults-loop.md (modify)
- docs/design/socratic-review.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- docs/records.md (modify)

## Out of scope

- Changing past-tense review and history records that cite 1,800 as a dated measurement
- Changing the job loop, a repair loop, or rejection events (a separate design change)
- Unrelated timeout_s 1800 values
- Edits to records/ or the owner's files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Grep every statement of the 1,800 budget across docs and code, then re-run the grep to show no stale present-tense hits remain.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests in verify.rs for mismatch present and absent or blank; cargo test and clippy stay clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: decisions.md records the measured reader count and why removals cannot pay for the new rules.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change, limited to the three prompts, judge(), the budget and its documents.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The mismatch-as-Major rule is enforced in judge() code, not only in prose.
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
