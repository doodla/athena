# Job 20261014-170031-2r9b: Make role prompts carry a request's real goal through to the verdict

## Request (verbatim)

> Rework the role prompts so a request's real goal survives to the verdict; job 20261014-154909-f4bt shows how it is lost. Its reader turned 'useful to someone who has never seen firstmate' into presence checks (a section exists, a glossary exists), never looked at the page, and put the real gaps out of scope; the producer is told to change only what the criteria need; the verifier saw that the page never says what firstmate is but filed it minor because the criteria passed, and request_reading_mismatch is not used in any verdict. Change it so: the reader names the audience and writes at least one criterion as an outcome that audience can check, never puts the request's own goal out of scope, and reads the current output before planning a change to it; the producer treats criteria as a floor and serves the request; a request-reading mismatch from the verifier counts as major in code; the verifier judges user-facing output as its audience would. The reader's fixed prompt is at 1,798 of 1,800 tokens; raise that budget as far as these rules need, and record why.

## Reading (accepted by the user)

Edit the reader, producer and verifier prompts so the reader names the audience, writes an outcome criterion and reads current output; the producer treats criteria as a floor; and the verifier judges output as its audience would. In judge(), a non-empty request_reading_mismatch becomes a Major finding that blocks a pass. Raise the reader budget just enough for the new text and record why.

Assumptions:
- 'Counts as major in code' means a mechanism in judge() (athena-core verify.rs) as well as prompt text, per the mechanism default. The prompt line alone would be prose-only.
- The reader's budget rises only as far as the added rules need. I estimate roughly 1,950 to 2,100 tokens, to be measured. Other role budgets stay unchanged unless the producer or verifier additions overflow them, in which case they rise too, with reasons recorded.
- The contract stays untouched, and no schema change is needed because request_reading_mismatch already exists in the verifier output.
- The study capability's prompt files, if separate, are out of scope. Only the three role prompts in crates/athena-context/prompts change.

## Acceptance criteria

1. [content] reader.md tells the reader to name the request's audience (who will use or judge the result) in the summary or assumptions.
2. [content] reader.md requires at least one acceptance criterion written as an outcome that the named audience can check. Presence checks (a section or file exists) alone do not satisfy it.
3. [content] reader.md forbids putting the request's own stated goal in out_of_scope, and tells the reader to read the current output (page, file, behavior) the request is about before planning a change to it.
4. [content] producer.md treats the acceptance criteria as a floor: it tells the producer to serve the request's goal, and not stop at the literal criteria or change only what they need. It keeps the small-change discipline and the instruction to follow conventions.
5. [content] verifier.md tells the verifier to judge user-facing output as its intended audience would, and to rate a request-reading mismatch as at least major. It must not downgrade a gap in the request's goal to minor because the criteria passed.
6. [behavior] In crates/athena-core/src/verify.rs, judge() adds a Major finding and a reason when the verifier output has a non-empty request_reading_mismatch, so the verdict is not pass. Unit tests cover a mismatch present (not pass) and absent or blank (unchanged). Run: cargo test -p athena-core verify
7. [behavior] The reader budget in BudgetSet::Reader is raised only as far as the new reader text needs, and no further. The budgets test passes at the new value. Run: cargo test --locked -p athena-cli --test budgets
8. [content] docs/decisions.md records the new reader budget, the measured token count and the reason: the new rules cannot be paid for by removals. CHANGELOG.md notes the change. Other documents that state 1,800 are updated.
9. [behavior] The workspace stays clean and tested. Run: cargo fmt --check, cargo clippy, cargo test

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/decision-sets.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- docs/design/defaults-loop.md (modify)
- docs/records.md (modify)

## Out of scope

- Fixing the firstmate explainer page from job 20261014-154909-f4bt.
- Changing the contract prompt or the defaults list.
- Changing the verifier output schema or record schemas.
- Raising the producer or verifier budgets beyond what the new text needs.

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A reading mismatch counts as major through code in judge(), not through prompt prose alone.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the new judge() behavior. The budgets test passes at the new budget. Clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix covers all three roles where the goal was lost (reader, producer, verifier), not only the reading.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new budget is a measured token count, and the recorded reason cites the budgets test output.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision record and changelog explain the change for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the prompts, the verdict logic and the budget, with no broader refactor.
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
