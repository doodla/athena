# Job 20261025-033743-npr5: verify "Design Hestia, the continuity role, with Hades cleanup folded in"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the check table in docs/design/hestia.md and confirm eight rows each with check, code or model, cadence, fix or report. Run: `Grep -n for 'queued request', 'branch', 'worktree', '2026-10-23', 'checkpoint', 'installed binary', 'budget', 'keep rule', 'athena propose', 'docs' in docs/design/hestia.md; Read the table`. Observes: Eight rows, each with all four fields; the leftover-cache row cites the 2026-10-23 disk-full cache; the doc-drift and queue rows are marked model-assisted, the rest deterministic.
  Fails without the behavior: A missing row or empty field would leave one of the eight failures with no check, so it would go unnoticed again.
- Criterion 2: Read the budget, measures and Hades sections. Run: `Grep -n -i 'budget\|stop rule\|estimate\|measured\|Hades\|split' docs/design/hestia.md`. Observes: Dollars and wall time per run and in total, a stop rule, each number labeled estimate or sourced, measures (findings per run, spot-check true-positive rate, time to resolution), the fold-in and the split trigger, and the empty-findings rule.
  Fails without the behavior: Without the stop rule or labels a reviewer could not tell a measured figure from a guess.
- Criterion 3: Read the alternatives and boundary sections and check each cited source exists (BRIEF.md pantheon table at line 603-606, D127). Run: `Grep -n 'Alternatives\|Artemis\|D127\|BRIEF.md' docs/design/hestia.md`. Observes: At least model-only, separate Hades now, and fold into the D127 guard are listed with reasons; the Artemis boundary says Hestia reports and does not gate; citations resolve.
  Fails without the behavior: No alternatives or boundary would leave the role open to creeping into gating.
- Criterion 4: Check that D129 is the last top-level heading, status proposed, with Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance bullets, and that the pinned-prefix test passes. Run: `Grep -n '^## D12[89]' docs/decisions.md; Read the tail of docs/decisions.md; cargo test --locked -p athena-cli decisions`. Observes: One new '## D129 · ' heading after D128, 'proposed' in its Status, all fields present; the log guard test the_decision_log_is_well_formed_and_only_appended_to passes.
  Fails without the behavior: An entry placed inside another or without Provenance fails the log guard test.
- Criterion 5: Read the build plan section. Run: `Grep -n 'Job [0-9]\|Depends' docs/design/hestia.md`. Observes: Numbered jobs, each with deliverables, a test or behavior check, dependencies and a one-producer size, with deterministic checks (branches, worktrees, caches, unpushed checkpoints, binary age, budgets, keep rule, propose) before model-assisted ones.
  Fails without the behavior: Unordered or unsized jobs could not be queued as single producer runs.
- Criterion 6: Compare changed files to the allowed list and read the CHANGELOG diff against neighbouring entries. Run: `git diff --stat; git status`. Observes: Changes only in docs/design/hestia.md, docs/decisions.md, CHANGELOG.md, docs/design/roadmap.md; a bullet under [Unreleased] in the existing style; no .rs, athena.toml or defaults.toml in the diff.
  Fails without the behavior: A code or governance file in the diff would break the no-code scope.
- Criterion 7: Run both from the repository root. Run: `cargo fmt --all --check && cargo test --locked`. Observes: Both pass, including the decision-log guard, backtest link check, budgets test and the boundary-word scan over the new files.
  Fails without the behavior: A modified old entry, a stray supersession phrase without a link, or a banned word or email in the new text would fail these tests.

Planned files:

- docs/design/hestia.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)
- docs/decision-links.toml (modify)

## The change

Base 39aec96b7f33 → candidate f470ce0d3b8f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |   2 +
 docs/decisions.md      |  30 +++++++
 docs/design/hestia.md  | 208 +++++++++++++++++++++++++++++++++++++++++++++++++
 docs/design/roadmap.md |   3 +
 4 files changed, 243 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1279 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo fmt --check` from the repository root; both pass, which shows that existing doc-linking or decision-numbering tests accept the new entry and design file.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
