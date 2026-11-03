# Job 20261103-123447-c3fa: Add required plan 'dependents' section with admission refusal and critic

## Request (verbatim)

> try again on 20261102-063419-0beg

## Reading (accepted by the user)

Retry of job 20261102-063419-0beg (dispatched, never verified, no findings recorded): add a required 'dependents' section to plans (PlanOutput schema, JobPlan record, planner brief), refused in plan_findings when missing or when a dependent lacks a test and a reason; the critic gains a "what else does this break" question; a CI-reuse replay test shows verify.rs's seen-commands rule must be named. Audience: Athena's planner, critic and the owner reviewing plans.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 166, the admission entry point for plans, run after the planner and before the critic), crates/athena-context/src/outputs.rs (PlanOutput, line 367, with its JSON schema), crates/athena-data/src/records.rs (JobPlan, line 2415). The new field must be serde-default so stored JobPlan records without it still deserialize; this is covered by the existing records round-trip test and AnyRecord validation. No new record file or data file is added. The property (refusal of a plan) is measured directly by calling plan_findings on plan values.
- Job 20261102-063419-0beg was never verified and recorded no findings, so there is nothing to fix. Its approach is kept: its 23 changed files are all kept as deliverables below, and the work is redone from the base. That includes the producer's design: a dependents entry has a name, a dependent, a kind (code, rule or host setting), and a test or a reason; the refusal is a Major finding so the plan goes back to the planner without a critic call.
- The earlier candidate also edited docs/system-facts.md and crates/athena-context/src/lib.rs, run/mod.rs and several cli tests; these are kept as likely deliverables with lower probability, only where the new field requires them.
- Prompt growth stays within the headroom: planner 477 tokens left, critic 483 left. The contract, reader and verifier prompts are not touched.
- No governance file (athena.toml, defaults.toml, workflows) needs to change; the new rule is code in admission.rs.
- Expected 15% of newly refused readings: the rule refuses only stored or replayed plans that lack a dependents section, and replay of past plans is not migrated; the higher refusal rate is intended because the planner brief now asks for dependents.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core admission: plan_findings on a plan with no dependents (empty list) returns a Major finding naming the missing section; an entry with a blank name, or with neither test nor reason, returns a Major finding naming that dependent; a complete section adds no finding.
2. [behavior] Run cargo test -p athena-core: a replay test for the CI-reuse case searches the real sources for the changed behaviour (exit_code) and shows the seen-commands filter in crates/athena-core/src/verify.rs turns up, and the dependents requirement is unmet unless the section names that rule; the test fails if it is not named.
3. [behavior] Run cargo test -p athena-context and cargo test -p athena-data: PlanOutput's JSON schema includes dependents, and a stored JobPlan record without the field still deserializes (records round-trip test passes).
4. [content] crates/athena-context/prompts/planner.md requires a 'dependents' section: for each behaviour, record, config key, check name or host setting the plan changes, list the code, rules (decisions, verdict rules, admission and gate checks) and host settings that read or rely on it, found by searching the repository and docs/system-facts.md, each with a test or a reason it is unaffected.
5. [content] crates/athena-context/prompts/critic.md asks, as its own question, what else this change breaks, and tells the critic to check the dependents list against a repository search.
6. [behavior] Run cargo test -p athena-context: the prompt token budget test passes with planner and critic each within 1300 tokens.
7. [content] CHANGELOG.md has an entry describing the dependents section, its admission refusal and the critic question; docs/design/job-planning.md describes the new field.
8. [behavior] The change refuses more plans than before (plans without dependents); expected 15% of newly refused readings, and the higher refusal rate is intended. Check by running cargo test -p athena-core refusal_corpus.

## Expected deliverables

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/system-facts.md (modify)

## Out of scope

- Automatically computing dependents by static analysis; the planner finds them by search
- Re-refusing or migrating plans already stored in records
- Changes to athena.toml, defaults.toml or workflows
- Fixing the four earlier breakages themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New admission and replay tests; cargo test and clippy are standing, not listed as their own criterion per the request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings and the replay test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, CHANGELOG and design doc updated for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by admission refusal, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Adds the missing dependents question to planning rather than fixing single instances.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

Run cargo test -p athena-core admission: plan_findings on a plan with no dependents (empty list) returns a Major finding naming the missing section; an entry with a blank name, or with neither test nor reason, returns a Major finding naming that dependent; a complete section adds no finding.
Run cargo test -p athena-core: a replay test for the CI-reuse case searches the real sources for the changed behaviour (exit_code) and shows the seen-commands filter in crates/athena-core/src/verify.rs turns up, and the dependents requirement is unmet unless the section names that rule; the test fails if it is not named.
Run cargo test -p athena-context and cargo test -p athena-data: PlanOutput's JSON schema includes dependents, and a stored JobPlan record without the field still deserializes (records round-trip test passes).
crates/athena-context/prompts/planner.md requires a 'dependents' section: for each behaviour, record, config key, check name or host setting the plan changes, list the code, rules (decisions, verdict rules, admission and gate checks) and host settings that read or rely on it, found by searching the repository and docs/system-facts.md, each with a test or a reason it is unaffected.
crates/athena-context/prompts/critic.md asks, as its own question, what else this change breaks, and tells the critic to check the dependents list against a repository search.
Run cargo test -p athena-context: the prompt token budget test passes with planner and critic each within 1300 tokens.
CHANGELOG.md has an entry describing the dependents section, its admission refusal and the critic question; docs/design/job-planning.md describes the new field.
The change refuses more plans than before (plans without dependents); expected 15% of newly refused readings, and the higher refusal rate is intended. Check by running cargo test -p athena-core refusal_corpus.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
