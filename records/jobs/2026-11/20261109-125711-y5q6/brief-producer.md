# Job 20261109-125711-y5q6: Make the reader state which agent roles a "Athena knows X" request

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: "Athena knows X" is ambiguous between documentation for readers and behavior built into the agents.
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: knowledge-request-read-as-docs-for-humans-not-agent-behavior (20261109-091813-hhqq)
> Cause: "Athena knows X" is ambiguous between documentation for readers and behavior built into the agents. Nothing makes the reader ask which one is meant. It also does not check which existing rules would stop the agents acting on the stated fact...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261109-091753-tt8c (job 20261109-080749-p0fs); prevention it proposed: Add a reader default: when a request says Athena or its agents should know, understand or act on something, the reading must state which roles and prompts must change, and must name any governing rule (such as the ladder rule in roles-and-l...
> Introspection 20261109-125651-wz8f (job 20261109-080220-zkhy); prevention it proposed: When a request changes what a role should decide or do, the reading step must list the rules that bound that role's actions (ladder rungs, budgets, role limits) and put any blocking rule into scope as a deliverable. Where a rule is owner-go...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (class knowledge-request-read-as-docs-for-humans-not-agent-behavior, 20261109-091813-hhqq): the reader prompt never makes the reader decide whether "Athena should know/act on X" means docs or agent behavior, nor list the rules that bound the affected role. Fix it in crates/athena-context/prompts/reader.md, within the 71-token reader budget, with a test on the composed prompt. The audience is the reader role, plus the verifiers who judge its readings. Retries 20261109-080749-p0fs and 20261109-080220-zkhy.

Assumptions:
- Code survey: I read crates/athena-context/prompts/reader.md, where the reader rules live. crates/athena-core/src/reading.rs and crates/athena-cli/tests/budgets.rs compose it as READER_FRAME. Prompt edits are mirror-checked by crates/athena-core/src/mirrors.rs. The change adds no record or data file, so check_record_file, AnyRecord and the round-trip test are not involved. The budget is measured directly by composing the real prompt.
- Term ladder: the ordered rungs of change size or authority that cap what a role may decide, per docs/design/roles-and-ladder.md; a rung limit is one of the rules the reader must list.
- Fixing at the reader prompt matches both introspections' proposals. The steward-side failures in job 20261109-080220-zkhy and the README failure in 20261109-080749-p0fs are instances; their code and doc candidates are not kept.
- The prompt edit may need a docs/decisions.md entry, so it is listed with moderate probability.
- Placing the rule in defaults.toml was rejected: it is a governance file, and the reader budget counts defaults too.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md gains one rule in 'Reading the request': when a request says Athena or its agents should know, understand or act on something, the reading must state whether it means docs or agent behavior, name the roles and prompts that change, list the rules that bound those roles (ladder rungs, budgets, role limits, admission rules) and list as deliverables any blocking rule that is not owner-only. Verify with git diff.
2. [behavior] A new unit test in crates/athena-context/src/lib.rs asserts the composed reader prompt contains the docs-versus-agent-behavior rule and the rule-listing requirement. It fails on the pre-change reader.md and passes after. Run: cargo test -p athena-context.
3. [behavior] The budget check in crates/athena-cli/tests/budgets.rs still passes: the reader set stays at or under 2000 tokens (71 left), so the producer must trim existing reader.md words if needed. Run: cargo test -p athena-cli --test budgets.
4. [content] The change description (CHANGELOG.md [Unreleased]) names the root cause, the ambiguity between docs for readers and built-in agent behavior, and says the fix is in the reader prompt, not in jobs 20261109-080749-p0fs or 20261109-080220-zkhy. Their candidates' README.md, CHANGELOG.md and steward changes are dropped, so the verifier can check the prior findings do not recur in the diff.
5. [behavior] Mirror and policy checks pass for the prompt edit: cargo test (whole workspace), including athena-core mirrors tests and the xtask policy test, plus cargo fmt --check and cargo clippy, all pass.
6. [content] No new refusal rule is added; the change is prose-only, marked as such in CHANGELOG.md. It does not edit defaults.toml, athena.toml or workflows, which Athena changes only via its proposal path, so no past readings are newly refused (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- README.md, docs/architecture.md and the steward prompt or code (crates/athena-core/src/steward.rs, crates/athena-data/src/roles.rs, crates/athena-data/src/records.rs): the earlier candidates are dropped
- defaults.toml, athena.toml, workflows, BRIEF.md and records/
- A code check that refuses readings
- Changing role budgets

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the reader prompt that produced both misreadings, not in either job's output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A test on the composed reader prompt, plus fmt, clippy and the whole test suite.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test checks the composed prompt text the reader actually receives.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule in one prompt file, within the token budget.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 13 cases in 13 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a reading's deliverable states a limit, exception or "only N" claim about Athena's behavior, require the reading and plan to derive the list from the enforcing source (the admission path classification code and config) and cite it. Add a criterion that each listed item maps to that source and t...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-125711-y5q6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-125711-y5q6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/reader.md gains one rule in 'Reading the request': when a request says Athena or its agents should know, understand or act on something, the reading must state whether it means docs or agent behavior, name the roles and prompts that change, list the rules that bound those roles (ladder rungs, budgets, role limits, admission rules) and list as deliverables any blocking rule that is not owner-only. Verify with git diff.
A new unit test in crates/athena-context/src/lib.rs asserts the composed reader prompt contains the docs-versus-agent-behavior rule and the rule-listing requirement. It fails on the pre-change reader.md and passes after. Run: cargo test -p athena-context.
The budget check in crates/athena-cli/tests/budgets.rs still passes: the reader set stays at or under 2000 tokens (71 left), so the producer must trim existing reader.md words if needed. Run: cargo test -p athena-cli --test budgets.
The change description (CHANGELOG.md [Unreleased]) names the root cause, the ambiguity between docs for readers and built-in agent behavior, and says the fix is in the reader prompt, not in jobs 20261109-080749-p0fs or 20261109-080220-zkhy. Their candidates' README.md, CHANGELOG.md and steward changes are dropped, so the verifier can check the prior findings do not recur in the diff.
Mirror and policy checks pass for the prompt edit: cargo test (whole workspace), including athena-core mirrors tests and the xtask policy test, plus cargo fmt --check and cargo clippy, all pass.
No new refusal rule is added; the change is prose-only, marked as such in CHANGELOG.md. It does not edit defaults.toml, athena.toml or workflows, which Athena changes only via its proposal path, so no past readings are newly refused (expected 0% of newly refused readings).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
