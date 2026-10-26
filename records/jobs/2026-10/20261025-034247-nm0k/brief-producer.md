# Job 20261025-034247-nm0k: Design the Artemis scope-guard role (docs only, no code)

## Request (verbatim)

> Priority: high
> Design (no code yet) the role that keeps Athena from rabbit-holing into unnecessary work (BRIEF.md addendum of 2026-10-25: 'Learnings need to go back into the system, but I want some role responsible for ensuring we don't rabbit hole into unnecessary work'), proposed as Artemis (boundaries and scope) with Demeter's view of cost; confirm or change the mapping. Before improvement work is queued or dispatched, it weighs what the work buys (which failure class, goal or product need it serves, and how often that bites) against its expected cost, and it watches running lines of work for thrash: repeated retries, growing plans, cost far past forecast. Ground it in the record: the trailer job took three rejected attempts and about 11 dollars before it was split; about 80 minutes went to a prompt-budget trap; of 49 open requests many were done or superseded. Say how it decides (rules versus a model), what it can stop or merge on its own, what goes to Sri, and how it keeps learning work flowing while cutting the unnecessary.
> Acceptance:
> - docs/design/scope-guard.md covers the value-versus-cost test, the thrash signals and their thresholds, its powers, and its relation to the steward, Hestia and the planning keep rule.
> - A proposed entry in docs/decisions.md.
> - A build plan in jobs each sized for one producer run.
> Files: docs/design/scope-guard.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Docs-only design of a role (Artemis for scope, with Demeter's cost view) that weighs improvement work's value against expected cost before queueing or dispatch and watches running work for thrash. Audience is Sri and design reviewers, who must see how it decides, what it may stop or merge, what goes to Sri, and how learnings work keeps flowing. The mapping is confirmed or changed against the BRIEF.md pantheon table.

Assumptions:
- Artemis/Demeter mapping is confirmed unless BRIEF.md pantheon table says otherwise; producer checks it.
- Decision number is D127 following D126; producer verifies the latest number.
- Figures from records/ are taken from the user's reply and labeled as such.
- Thresholds are estimates to be tuned unless the record supports them.

## Acceptance criteria

1. [content] docs/design/scope-guard.md states the value-versus-cost test: value side names failure class, goal or product need, and how often it bites, each with a source; cost side gives expected dollars and wall time labeled measured or estimate; a decision rule says when work is queued, merged, deferred or dropped.
2. [content] The design grounds itself in the record with these figures, each labeled as supplied by the request (records/ not visible to the producer) unless a repository source is cited: trailer job, three rejected attempts and about 11 dollars before the split; about 80 minutes lost to the prompt-budget trap; 49 open requests, many done or superseded; test-suite speedup, three rejected attempts (jobs 20261025-190606-eebk, 20261025-202447-xgk9, 20261025-214509-y179, about 9 dollars and 4 hours), each a real gain (test check about 12 to 7.4-7.8 minutes) but rejected for missing a fixed time target set without a measurement showing it reachable, then landed by narrowed request 20261025-230352-ph40; cut A took four tries (timing race, wait on wrong check, duplicate push-triggered runs).
3. [content] Thrash signals have numeric thresholds with stated basis or an 'estimate, to be tuned' label, covering repeated retries, growing plans, cost against forecast p90, and a line of work rejected twice for the same unmet criterion; the document shows which signals would have fired on the trailer job, the speedup and the prompt-budget trap.
4. [content] The design separates rules (code decides) from model judgment, says why, and what a model verdict may not override.
5. [content] Powers are listed: what it can stop or merge on its own, what is advisory, what goes to Sri and through which channel; learnings work (BRIEF.md addendum 2026-10-25) is not blocked, via a protected quota, default-to-proceed on doubt, and an appeal path.
6. [content] The document states its relation to the steward, Hestia and the planning keep rule (job-planning.md section 9), and confirms or changes the Artemis/Demeter mapping against the BRIEF.md pantheon table with reasons.
7. [content] The document has a numbered build plan of jobs, each sized for one producer run with scope, files touched and an acceptance check, ordered by dependency; no code is written.
8. [content] docs/decisions.md gains D127 after D126, marked proposed, in the existing format (Case, Decision, alternatives, Tradeoff, Reversal cost), linking to the design; CHANGELOG.md has an Unreleased entry naming the design and the proposed decision.
9. [behavior] Run `cargo test` and `cargo fmt --check`: any tests checking decisions numbering, docs inventory or changelog format still pass.

## Expected deliverables

- docs/design/scope-guard.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any Rust code or tests implementing the role
- Changes to athena.toml, defaults.toml or workflows
- Closing or merging the 49 open requests
- Editing BRIEF.md or records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Figures cite sources or are labeled as request-supplied or estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design written for an outside reader.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design with alternatives and a proposed decision before any build.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One document plus a decision entry and changelog line.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the document shows signals firing on the named cases.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test and fmt --check run by verifier.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No personal data or secrets written.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

docs/design/scope-guard.md states the value-versus-cost test: value side names failure class, goal or product need, and how often it bites, each with a source; cost side gives expected dollars and wall time labeled measured or estimate; a decision rule says when work is queued, merged, deferred or dropped.
The design grounds itself in the record with these figures, each labeled as supplied by the request (records/ not visible to the producer) unless a repository source is cited: trailer job, three rejected attempts and about 11 dollars before the split; about 80 minutes lost to the prompt-budget trap; 49 open requests, many done or superseded; test-suite speedup, three rejected attempts (jobs 20261025-190606-eebk, 20261025-202447-xgk9, 20261025-214509-y179, about 9 dollars and 4 hours), each a real gain (test check about 12 to 7.4-7.8 minutes) but rejected for missing a fixed time target set without a measurement showing it reachable, then landed by narrowed request 20261025-230352-ph40; cut A took four tries (timing race, wait on wrong check, duplicate push-triggered runs).
Thrash signals have numeric thresholds with stated basis or an 'estimate, to be tuned' label, covering repeated retries, growing plans, cost against forecast p90, and a line of work rejected twice for the same unmet criterion; the document shows which signals would have fired on the trailer job, the speedup and the prompt-budget trap.
The design separates rules (code decides) from model judgment, says why, and what a model verdict may not override.
Powers are listed: what it can stop or merge on its own, what is advisory, what goes to Sri and through which channel; learnings work (BRIEF.md addendum 2026-10-25) is not blocked, via a protected quota, default-to-proceed on doubt, and an appeal path.
The document states its relation to the steward, Hestia and the planning keep rule (job-planning.md section 9), and confirms or changes the Artemis/Demeter mapping against the BRIEF.md pantheon table with reasons.
The document has a numbered build plan of jobs, each sized for one producer run with scope, files touched and an acceptance check, ordered by dependency; no code is written.
docs/decisions.md gains D127 after D126, marked proposed, in the existing format (Case, Decision, alternatives, Tradeoff, Reversal cost), linking to the design; CHANGELOG.md has an Unreleased entry naming the design and the proposed decision.
Run `cargo test` and `cargo fmt --check`: any tests checking decisions numbering, docs inventory or changelog format still pass.

## Report

End with the producer report as your structured output.
