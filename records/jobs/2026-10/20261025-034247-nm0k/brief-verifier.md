# Job 20261025-034247-nm0k: verify "Design the Artemis scope-guard role (docs only, no code)"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of docs/design/scope-guard.md section 'Value versus cost'. Run: `git diff`. Observes: Value inputs (failure class, goal/need, frequency) each with a source field; cost inputs (dollars, wall time) each labeled measured or estimate; a decision table mapping outcomes to queue, merge, defer, drop.
- Criterion 2: Read the 'Grounding in the record' section. Run: `git diff`. Observes: All listed figures present (trailer ~$11 and 3 rejections; ~80 min prompt-budget trap; 49 open requests; speedup jobs eebk, xgk9, y179, ~$9 and 4 h, 12 to 7.4-7.8 min, ph40; cut A four tries) each labeled request-supplied or citing D126/CHANGELOG.
- Criterion 3: Read the thrash signals table and the replay table. Run: `git diff`. Observes: Each signal (retries, plan growth, cost vs forecast p90, same-criterion rejected twice) has a number and a basis or 'estimate, to be tuned'; the replay lists which signals fire on trailer, speedup and prompt-budget trap, and at which attempt.
  Fails without the behavior: Without the same-criterion signal the speedup replay would show no firing before the third rejection; the table must show it firing at the second.
- Criterion 4: Read the 'Rules versus model' section. Run: `git diff`. Observes: A list of what code decides, what the model scores, why, and an explicit list of what a model verdict may not override (rule stops, Sri-only items, quota, protected governance files).
- Criterion 5: Read the 'Powers' section and the learnings protection subsection. Run: `git diff`. Observes: Three lists (own authority, advisory, to Sri with the channel), plus quota, default-to-proceed and appeal path tied to the BRIEF.md addendum of 2026-10-25.
  Fails without the behavior: If the quota, default-to-proceed or appeal were absent, the section would let the guard block learnings work; each must be present.
- Criterion 6: Read the 'Relations' and 'Mapping' sections. Run: `git diff`. Observes: Steward (docs/design/roles-and-ladder.md), Hestia, and job-planning.md section 9 keep rule each treated; mapping confirmed with reasons citing the BRIEF.md pantheon table rows.
- Criterion 7: Read the 'Build plan' section. Run: `git diff`. Observes: Numbered jobs ordered by dependency (e.g. record schema and signals as pure functions; replay test on named cases; queue-time value check; running-line watcher; model scorer; Sri channel and quota; measurement), each with scope, files and an acceptance check; no code in the diff.
- Criterion 8: Read the added D127 block after D126 and the CHANGELOG Unreleased entry. Run: `git diff`. Observes: D127 titled and marked proposed with Case, Decision, Alternatives, Tradeoff, Reversal cost, Provenance, linking docs/design/scope-guard.md; CHANGELOG Unreleased has an Added line naming the design and D127.
- Criterion 9: Run cargo test --locked and cargo fmt --all --check; both pass (docs-only change). Run: `cargo test --locked`. Observes: Exit 0 for both, including policy.rs boundary and personal-data scans over the new file.
  Fails without the behavior: A banned word or email-like text in the new doc would fail the policy scan test.

Planned files:

- docs/design/scope-guard.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base cd15f918b675 → candidate 429ae86f382f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md               |   4 +
 docs/decisions.md          |  30 +++++
 docs/design/scope-guard.md | 299 +++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 333 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1251 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
