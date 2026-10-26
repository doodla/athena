# Job 20261025-033743-npr5: Design Hestia, the continuity role, with Hades cleanup folded in

## Request (verbatim)

> Priority: high
> Design (no code yet) Hestia, the Olympus role for continuity (BRIEF.md 'Project Olympus' section 2: knowledge, documentation, operational health and long-term quality; and the addendum of 2026-10-25: something that sanity checks the system and keeps everything neat and clean), with Hades's cleanup folded in for now. Ground it in what has gone unnoticed until now: queued requests already done or superseded; leftover branches, worktrees and build caches (a cache filled the disk on 2026-10-23); checkpoints never pushed; an installed binary older than its source; prompt budgets nearly full; a keep rule nobody evaluated; athena propose never run; docs that no longer match the code. Say which checks are deterministic code and which need a model, how often each runs, what Hestia fixes itself as small jobs versus reports as work items, its budget, and how its findings are measured.
> Acceptance:
> - docs/design/hestia.md covers each item above with a check, a cadence, and fix-or-report.
> - A proposed (not accepted) entry in docs/decisions.md.
> - A build plan in jobs each sized for one producer run.
> Files: docs/design/hestia.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Write docs/design/hestia.md for Sri and the brain: one design for a continuity role (sanity checks and tidiness, per the BRIEF.md 2026-10-25 addendum), grounded in the eight named failures. Each gets a check, a code-or-model call, a cadence and fix-or-report. Add a proposed decision to docs/decisions.md and a CHANGELOG line. No code.

Assumptions:
- Goal: a reviewable design Sri and the brain can accept or amend before any Hestia code is built; audience is Sri, the brain and the later producers of the build jobs.
- The next decision number is D129 (D128 is the last heading found); the producer should confirm by reading the end of docs/decisions.md.
- Hestia reports and makes small fixes only; it does not gate work, per D127. Governance files are not changed.
- Facts about the eight failures (disk fill, stale binary, never-pushed checkpoints, budgets) come from the request and are cited as stated unless the producer can confirm them in the repository; records/ is not available to check them.
- Fixes Hestia does itself are small jobs through the normal change pipeline, not direct edits; destructive cleanup (deleting branches, worktrees, caches) is limited to what is provably Athena's own and merged, otherwise reported.
- docs/decision-links.toml may need an entry if a test requires every decision to be linked; otherwise unchanged.

## Acceptance criteria

1. [content] docs/design/hestia.md has a section or table row for each of the eight items: queued requests done or superseded; leftover branches, worktrees and build caches (citing the 2026-10-23 disk-full cache); checkpoints never pushed; installed binary older than its source; prompt budgets nearly full; the unevaluated keep rule; athena propose never run; docs that no longer match the code. Each row states the check, whether it is deterministic code or needs a model, its cadence, and whether Hestia fixes it as a small job or reports a work item.
2. [content] The design states Hestia's budget (dollars and wall time per run, in total, with a stop rule) and how findings are measured (for example findings per run, true-positive rate on spot check, time from appearance to resolution). It labels every number as an estimate or says where it was measured. It also says how Hades's cleanup is folded in and when it would split out.
3. [content] The design lists alternatives (for example: model-only sweeps, separate Hades now, folding into the scope guard of D127) and states the boundary with Artemis (D127 says Hestia does not gate work), with sources cited from BRIEF.md and the repository.
4. [content] docs/decisions.md gains one new entry, numbered after D128, with status proposed (not accepted), in the same fields as neighbouring entries (Case, Decision, Alternatives, Tradeoff, Reversal cost, Provenance), inserted as a top-level entry and not inside another.
5. [content] The design has a build plan of numbered jobs, each sized for one producer run, with deliverables, a test or behavior check, and dependencies, with the deterministic checks first.
6. [content] CHANGELOG.md has an entry for the design and the proposed decision, matching the file's existing format. No code, athena.toml or defaults.toml file changes.
7. [behavior] Run `cargo test` and `cargo fmt --check` from the repository root; both pass, which shows that existing doc-linking or decision-numbering tests accept the new entry and design file.

## Expected deliverables

- docs/design/hestia.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Any Rust code, CLI commands or crates for Hestia
- Changes to athena.toml, defaults.toml or workflows
- Accepting the decision or queueing the build jobs
- Actually cleaning branches, caches or the queue
- Changing the Artemis scope guard design

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design written for an outside reader, defining Hestia, Hades and the eight failures in plain terms.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite BRIEF.md, D127 and the repository; unmeasured numbers are labeled estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Alternatives are written down and the decision stays proposed, for review before any build.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design file, one proposed decision, one changelog line; the build is split into one-producer jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check each item's check, cadence and fix-or-report, not just the file's existence.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The design says which checks are enforced by code and marks prose-only rules.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Design requires that an empty findings list is reported with the checks that ran, never counted as success alone.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets, personal data or email addresses in the docs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Findings are measured by resolved problems and time to resolution, not by sweeps run.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/design/hestia.md has a section or table row for each of the eight items: queued requests done or superseded; leftover branches, worktrees and build caches (citing the 2026-10-23 disk-full cache); checkpoints never pushed; installed binary older than its source; prompt budgets nearly full; the unevaluated keep rule; athena propose never run; docs that no longer match the code. Each row states the check, whether it is deterministic code or needs a model, its cadence, and whether Hestia fixes it as a small job or reports a work item.
The design states Hestia's budget (dollars and wall time per run, in total, with a stop rule) and how findings are measured (for example findings per run, true-positive rate on spot check, time from appearance to resolution). It labels every number as an estimate or says where it was measured. It also says how Hades's cleanup is folded in and when it would split out.
The design lists alternatives (for example: model-only sweeps, separate Hades now, folding into the scope guard of D127) and states the boundary with Artemis (D127 says Hestia does not gate work), with sources cited from BRIEF.md and the repository.
docs/decisions.md gains one new entry, numbered after D128, with status proposed (not accepted), in the same fields as neighbouring entries (Case, Decision, Alternatives, Tradeoff, Reversal cost, Provenance), inserted as a top-level entry and not inside another.
The design has a build plan of numbered jobs, each sized for one producer run, with deliverables, a test or behavior check, and dependencies, with the deterministic checks first.
CHANGELOG.md has an entry for the design and the proposed decision, matching the file's existing format. No code, athena.toml or defaults.toml file changes.
Run `cargo test` and `cargo fmt --check` from the repository root; both pass, which shows that existing doc-linking or decision-numbering tests accept the new entry and design file.

## Report

End with the producer report as your structured output.
