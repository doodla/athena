# Job 20261026-043431-kgrm: verify "Design the context and knowledge engineer role (Hermes with Hestia)"

## Request (verbatim)

> Priority: urgent
> Design (no code yet) the context and knowledge engineer role, which owns the craft of context efficiency in everything Athena writes and gives to agents (BRIEF.md addenda of 2026-10-25, 'make sure that the context provided to agents is sufficient and good. What could've been planned better', and 2026-10-26, 'Everything is for agents. So Athena should architect to be context efficient in what it writes, and some role should own the craft there'). Proposed as Hermes (context and handoffs) with Hestia (knowledge); confirm or change the mapping. Scope, in two parts. (1) What Athena writes, judged by how little context an agent needs to find, understand and safely change it: code structure (module and file size, function length, one concern per module, so a change touches few files and callers are easy to find; gh.rs at 5,276 lines and a 724-line produce_and_verify were where critics kept finding missed callers), docs (short, indexed, one fact in one place, with a map from question to file; the decision log is 3,231 lines), records and prompts. It sets the conventions, measures against them (with the code-health check, request 20261026-042558-5p65, and the Jev code-health checks, 20261026-043137-zc0v), and queues restructuring when they slip. (2) What agents are given: briefs, the system-facts file, recurring failure classes, the vocabulary and the spec; after each failed, rejected or planning-stopped job it asks what context would have prevented it and changes what agents read. Ground it in the record: planning stops from facts no file stated, retries that lost the earlier verifier's findings (7 jobs), prompt sets at their budgets (reader 1997 of 2000). Say what it measures and how it keeps context within budgets.
> Acceptance:
> - docs/design/context-engineer.md covers both parts with conventions, measures, cadence, and relations to Hestia, the code-health check, Jev checks, the introspector and the requirements engineer (20261017-173506-k67x).
> - A proposed entry in docs/decisions.md and a build plan in jobs each sized for one producer run.
> Files: docs/design/context-engineer.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Write docs/design/context-engineer.md, a design with no code, for a role that owns context efficiency: (1) how little context an agent needs to find, understand and safely change what Athena writes, and (2) what agents are given. The audience is Sri, who judges the design. Also add a proposed decisions.md entry and a CHANGELOG line.

Assumptions:
- Capability is change: the job edits docs only. Class is design.
- Hermes (context and handoffs) and Hestia (knowledge) is confirmed unless the roles docs show a conflict. The designer checks docs/design/hestia.md and roles-and-ladder.md and records the call.
- The code-health check and the Jev checks may be unbuilt or only designed. The doc states which parts exist and does not rebuild them.
- Priority urgent is kept as stated, but urgent_reason is null because this is a design request and does not cut wasted time.

## Acceptance criteria

1. [content] docs/design/context-engineer.md covers part 1 (what Athena writes: code structure, docs, records, prompts) with named conventions (file and function size, one concern per module, one fact in one place, a question-to-file map, an index) and, for each, a measure and a threshold.
2. [content] The design covers part 2 (what agents are given: briefs, system-facts file, failure classes, vocabulary, spec). It describes a post-mortem step after each failed, rejected or planning-stopped job that asks what context would have prevented it, and says what changes as a result.
3. [content] The design is grounded in cited record evidence: gh.rs at 5,276 lines, the 724-line produce_and_verify, the 3,231-line decision log, the 7 retries that lost earlier verifier findings, planning stops from unstated facts, and the reader prompt at 1997 of 2000. Each number cites its source or is labeled as an estimate.
4. [content] The design confirms or changes the Hermes and Hestia mapping with a reason, and states the role's relations to Hestia, the code-health check (20261026-042558-5p65), the Jev code-health checks (20261026-043137-zc0v), the introspector and the requirements engineer (20261017-173506-k67x). It checks them against the existing docs (hestia.md, requirements-map.md, jev-code-health.md if present) and does not assume unbuilt things exist.
5. [content] The design says what the role measures, its cadence, how it keeps context within budgets, and how it queues restructuring when conventions slip. An outside reader can check these against the stated numbers.
6. [content] docs/decisions.md has a proposed entry in the file's existing format. The design contains a build plan split into jobs, each sized for one producer run, with its own acceptance bullets that check behavior, not just that files exist.
7. [content] CHANGELOG.md has an entry for the design. No source code, athena.toml or defaults.toml changes.

## Expected deliverables

- docs/design/context-engineer.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)

## Out of scope

- Writing any code, including the code-health check or Jev client changes
- Splitting gh.rs or produce_and_verify now
- Editing prompts or the system-facts file
- Rewriting the 3,231-line decision log

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design is written with alternatives, and the decision is proposed before anything is built.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite sources or are labeled as estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The doc is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The build plan splits into jobs each sized for one producer run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The build-plan jobs carry acceptance bullets that check behavior.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Each convention is either enforced by a named measure or marked prose-only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Grep the part 1 table: every convention row (file size, function size, one concern per module, one fact in one place, question-to-file map, index) carries a measure, an estimate-labeled threshold and 'mechanism:' or 'prose-only'. Compare the row count with the label count. Run: `grep -c 'mechanism:\|prose-only' docs/design/context-engineer.md`. Observes: Each named convention row has all three fields and an enforcement label; a threshold spot-checked against gh.rs size.
  Fails without the behavior: A row without a measure, threshold or label shows as a row with no match in the grep.
- Criterion 2: Grep for the headings of part 2 and the post-mortem: triggers (failed, rejected, planning-stopped), the question, the outcome class list and the displacement rule; read them. Run: `git diff --stat`. Observes: All five inputs (briefs, system-facts, failure classes, vocabulary, spec) are covered; the post-mortem names its three triggers, the question and a fixed outcome list.
  Fails without the behavior: Without the post-mortem section no outcome list exists to find.
- Criterion 3: Grep each figure (5,276; 724; 3,231; 1997; 7 retries; planning stops) and check a source path, budget line or 'stated by the request' beside it; re-measure gh.rs and decisions.md line counts via Read/Grep count. Run: `git diff`. Observes: Every figure has a citation or label; records-only figures are marked not measured here; base decisions.md count is stated.
  Fails without the behavior: An uncited figure is a defect.
- Criterion 4: Read the mapping and relations sections against hestia.md section 8, roles-and-ladder.md, requirements-map.md and BRIEF.md 4A; grep the doc for introspect.rs, athena.introspection/1, 5p65, zc0v and k67x. Run: `git diff docs/design/hestia.md`. Observes: Mapping confirmed with reason; five relations stated; introspector cited as built, code-health check, Jev checks and requirements engineer marked as not built with their sources; ownership split with the requirements engineer stated; hestia.md has a cross-reference.
  Fails without the behavior: A relation claimed without a source file, or the introspector called absent, shows in the grep.
- Criterion 5: Read the measures, cadence, budget and queue sections; recompute 1997/2000 = 99.85% and check the 95% rule and the cap of 3 each have a number and a cadence. Run: `git diff`. Observes: Each rule has a number or period an outside reader can test against the budget lines and file sizes.
  Fails without the behavior: A rule without a number fails the outside-reader check.
- Criterion 6: Compare the D130 headings and bullets with D129's; grep the build plan for 'Acceptance' bullets per job and check each names a failing case before the passing one; check D130 lists 'separate post-mortem record' as a rejected alternative. Run: `git diff docs/decisions.md`. Observes: D130 is marked proposed in D129's format and appended after D129 only; six jobs each have behavior bullets; job 4's bullet exercises the existing introspection path.
  Fails without the behavior: A bullet that only says a file exists is rejected on reading.
- Criterion 7: git diff --stat to list changed paths; grep CHANGELOG.md for the new line under Unreleased; run cargo test --locked for the boundary scan and budget test. Run: `cargo test --locked`. Observes: Only the four doc files change; no .rs, athena.toml or defaults.toml; tests pass.
  Fails without the behavior: A code or config path in the stat, or banned or email-like text, fails the criterion or the scan test.

Planned files:

- docs/design/context-engineer.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)

## The change

Base df7270827c31 → candidate 544eaf52743c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |   3 +
 docs/decisions.md               |  31 ++++
 docs/design/context-engineer.md | 378 ++++++++++++++++++++++++++++++++++++++++
 docs/design/hestia.md           |   3 +
 4 files changed, 415 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1279 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
