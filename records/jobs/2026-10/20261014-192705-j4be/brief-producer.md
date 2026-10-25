# Job 20261014-192705-j4be: Make verifier check cited documents; flag self-covered map rows in code

## Request (verbatim)

> Close the gaps that let job 20261014-174815-7bf8 land a steward that never calls a model. Its request cited a BRIEF.md addendum, but the verifier judged only the reading's criteria and never compared the change with that addendum; and the job moved its own requirement rows A9-1 and A10-1 to covered, which nobody checked independently. When a request cites a document, the verifier must check the change against it, and falling short of it counts as a request-reading mismatch. A job may not mark its own requirement rows covered. Put A9-1 and A10-1 back to partial with the reason.

## Reading (accepted by the user)

Closes two verification gaps seen in job 20261014-174815-7bf8. The verifier prompt now requires checking the change against any document the request cites, and falling short is a request-reading mismatch. Athena's own verdict code flags as major any candidate that moves a requirements-map row to Covered. Rows A9-1 and A10-1 go back to Partial with the reason. The audience is the user, who has to trust Athena's verdicts.

Assumptions:
- Core cannot call the CLI's strategy::parse_map, because athena-cli depends on core. So the check uses a small row parser in athena-core. A shared one could move into athena-data, but I assume the small duplicate is acceptable.
- The check reads the map text at the base and at the candidate. run.rs already has git access to both, and the verifier's snapshot is one commit past the base. The producer picks the exact plumbing.
- The finding is Athena-sourced, like the other Athena findings, so the verifier cannot omit it.
- The user's reply fixes the scope to a major finding. It does not ask for a hard block beyond the normal fail verdict.
- The document-comparison rule stays prose-only, because the repository cannot tell mechanically which documents a request cites. The change records that.
- A9-1's missing piece is that the steward never calls a model. Nothing in this change builds that.

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md says that when the request cites a document (a file or section), the verifier reads it and compares the change with it, and that falling short is a request_reading_mismatch.
2. [content] Athena's verdict code (crates/athena-core/src/verify.rs, fed by crates/athena-core/src/run.rs) compares docs/design/requirements-map.md at the base with the candidate. For every row whose status changed to Covered from Partial or Not covered, or that was added already Covered, it adds an Athena-sourced major finding naming the row ID. So the final verdict cannot be pass. Rows that stay Covered, or move away from Covered, add no finding.
3. [content] The producer prompt says a job may not mark its own requirement rows Covered. docs/enforcement.md or the decision entry records that this is enforced by code and that the document-comparison rule is prose-only.
4. [content] In docs/design/requirements-map.md, A10-1 reads Partial: cost and yield tracking, reshaping and retiring are not built. A9-1 reads Partial: the steward never calls a model. Each cell says why.
5. [content] The G5 counts in docs/design/strategy-view.md match the map, so A9-1 and A10-1 appear as Partial.
6. [content] docs/decisions.md has a new entry with alternatives and the tradeoff, citing job 20261014-174815-7bf8 as the case. CHANGELOG.md notes the change. The roles-and-ladder doc stops describing A9-1 and A10-1 as covered, if it does.
7. [behavior] Unit tests in verify.rs cover a row moved to Covered (major finding, verdict fail), a row left Covered (no finding), and a row moved away from Covered (no finding). Run: cargo test --locked -p athena-core verify
8. [behavior] The token budgets still hold with the longer prompts. Run: cargo test --locked -p athena-cli --test budgets
9. [behavior] The whole workspace passes, including the strategy-view-against-map check. Run: cargo test --locked --workspace
10. [behavior] The workspace is lint- and format-clean. Run: cargo clippy --locked --workspace --all-targets -- -D warnings, and cargo fmt --check

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)
- docs/design/roles-and-ladder.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-cli/src/strategy.rs (modify)

## Out of scope

- Making the steward call a model, which is the cause of the gap in A9-1.
- Building cost and yield tracking, reshaping or retiring of roles (rest of A10-1).
- Mechanically detecting which documents a request cites.
- Changing the stored verification record schema beyond the new finding.
- Re-reviewing or reverting the other changes from job 20261014-174815-7bf8.

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The self-grading rule is enforced in verdict code, as the user asked. The document-comparison rule is explicitly marked prose-only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover the code check, and the workspace test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the two process gaps (unchecked cited documents, self-graded rows) rather than only resetting the two rows.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A decision entry records the alternatives and tradeoff.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision entry cites job 20261014-174815-7bf8 and its verifier findings.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: two rules, one code check and two row fixes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, prompts and the changelog are written for an outside reader.
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
