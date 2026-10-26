# Job 20261026-043431-kgrm: Design the context and knowledge engineer role (Hermes with Hestia)

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

docs/design/context-engineer.md covers part 1 (what Athena writes: code structure, docs, records, prompts) with named conventions (file and function size, one concern per module, one fact in one place, a question-to-file map, an index) and, for each, a measure and a threshold.
The design covers part 2 (what agents are given: briefs, system-facts file, failure classes, vocabulary, spec). It describes a post-mortem step after each failed, rejected or planning-stopped job that asks what context would have prevented it, and says what changes as a result.
The design is grounded in cited record evidence: gh.rs at 5,276 lines, the 724-line produce_and_verify, the 3,231-line decision log, the 7 retries that lost earlier verifier findings, planning stops from unstated facts, and the reader prompt at 1997 of 2000. Each number cites its source or is labeled as an estimate.
The design confirms or changes the Hermes and Hestia mapping with a reason, and states the role's relations to Hestia, the code-health check (20261026-042558-5p65), the Jev code-health checks (20261026-043137-zc0v), the introspector and the requirements engineer (20261017-173506-k67x). It checks them against the existing docs (hestia.md, requirements-map.md, jev-code-health.md if present) and does not assume unbuilt things exist.
The design says what the role measures, its cadence, how it keeps context within budgets, and how it queues restructuring when conventions slip. An outside reader can check these against the stated numbers.
docs/decisions.md has a proposed entry in the file's existing format. The design contains a build plan split into jobs, each sized for one producer run, with its own acceptance bullets that check behavior, not just that files exist.
CHANGELOG.md has an entry for the design. No source code, athena.toml or defaults.toml changes.

## Report

End with the producer report as your structured output.
