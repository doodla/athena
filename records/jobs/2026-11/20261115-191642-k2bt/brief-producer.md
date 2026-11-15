# Job 20261115-191642-k2bt: Drop deadline/pacing text from briefs; check queued fixes still apply

## Request (verbatim)

> From Sri (the owner), word for word: "the planner is now told its time limit and asked to finish within it. That's pointless" and "It's not human, plus it'll take however long it takes. Yeah, also do that"
> 
> Background (not Sri's own words; added so the request can be read on its own). Job 6ee6c79d844c "Tell the planner its deadline and to answer within it" landed on 15 Nov after the time limits were raised to 24 hours; it was queued when the planner limit was 600 s. "Also do that" agrees to two things:
> 1. Remove the deadline and pacing text from the planner brief (and from the producer brief added by "Tell the producer its deadline and pacing in its brief", 64a92cc76d29), since a model does not pace itself like a person and the work takes as long as it takes.
> 2. Before a queued fix runs, check whether the problem it was filed for still exists under current configuration and code, and close it with the reason if not. Example still queued: "Scale planner time limit when revision escalates effort" (20261115-113508-66w1, retry of 20261114-052309-w3zt), which no longer applies now that limits are 24 hours.

## Reading (accepted by the user)

Per Sri: remove the "## Deadline" section from the planner brief (job 6ee6c79d844c) and the "## Time budget" section from the producer brief (64a92cc76d29), keeping the enforced limits themselves. Also add a planner-prompt rule to first check whether a queued fix's problem still exists under current config and code, and say so instead of building if it does not. Job 20261115-113508-66w1 is addressed as dropped-from-scope: see assumptions.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/planner.rs (DEADLINE_RULE, deadline_section at lines 94-114, call at 181, PlannerInputs.deadline_s at 44), crates/athena-context/src/briefs.rs (time_budget_section 763, call 696, producer inputs deadline_s 373, tests ~1865-2072), crates/athena-core/src/run/planning.rs (deadline_s: timeout_s at 156, escalated factor 145/527), run/producer.rs:51 producer_deadline_s, run/mod.rs:1244, run/tests.rs:1438. No records or data files are added, so no record round-trip applies.
- Goal and audience: Sri, who judges whether the planner and producer briefs stop telling models a clock to pace against; models do not pace like humans and work takes as long as it takes. The deadline is still enforced, only no longer announced.
- Job 20261115-113508-66w1 (landed, pass; findings: minor fails-without check for criterion 2 could not show it, nit about string-inequality escalation detection): the scaling it added is kept as is, and its two findings concern test demonstration and style of code this job does not touch, so they no longer need fixing here. Its files planning.rs, tests.rs and CHANGELOG.md are kept as deliverables of this job only for the deadline_s plumbing removal and the changelog entry. Job 20261114-052309-w3zt (dispatched, never verified) is superseded by 66w1; its candidate files are the same three. The queued retry of that scaling is moot now that limits are 24 hours; it is not reworked here.
- Part 2 is read as an agent-behavior change (planner prompt rule), not a queue-closing mechanism in Rust; closing already-queued jobs such as 66w1's siblings is an owner action outside this repository change. The reader prompt has 2 tokens spare so it is not edited.
- Existing code: extends crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs.rs and crates/athena-context/prompts/planner.md; no new module.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs no longer defines DEADLINE_RULE or deadline_section, and the planner brief has no '## Deadline' section (git diff shows removal; Grep for 'Deadline' in briefs/planner.rs finds none).
2. [content] crates/athena-context/src/briefs.rs no longer defines time_budget_section and the producer brief has no '## Time budget' section; Grep for 'Time budget' in crates/ finds none; the deadline_s input fields that only fed these sections are removed from the planner and producer brief inputs and their callers (planning.rs, producer.rs, run/mod.rs), while the enforced agent-spec deadline is unchanged.
3. [behavior] Brief tests assert the rendered planner brief and producer brief contain neither 'deadline of' / 'seconds of wall-clock' nor 'Time budget', and the old tests pinning those sections (briefs.rs ~1865-1880, run/tests.rs:1438) are removed or inverted; run `cargo test -p athena-context` and `cargo test -p athena-core`; the new absence assertions fail if either section is restored.
4. [content] crates/athena-context/prompts/planner.md gains one short rule (within its 95-token headroom): when the request is a queued fix or retry, first check from the code and config whether its problem still exists, and if not, report that with the reason instead of planning a change. The prompt token-budget test still passes (`cargo test -p athena-context`).
5. [content] CHANGELOG.md has a new Unreleased entry recording this change and citing Sri's request and jobs 6ee6c79d844c and 64a92cc76d29. Because the change is prompt-only, recorded fmt, clippy and test checks on the final tree pass.
6. [content] The 66w1 scaling (PLANNER_ESCALATED_TIMEOUT_FACTOR in run/planning.rs) is left in place unless removing the dead deadline plumbing requires editing adjacent lines; planner_timeout_s, producer limits and athena.toml/defaults.toml values are unchanged (git diff shows no change to them).

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/iris.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing enforced time limits (planner_timeout_s, producer limit, critic timeout) or athena.toml/defaults.toml.
- The 66w1 escalation scaling itself, the follow-up queueing feature from 20261114-041123-hvyh, and hestia.rs.
- A Rust mechanism that auto-closes stale queued jobs; and closing specific queued jobs, which is an owner action.
- docs/decisions.md, crates/athena-data/src/config.rs (listed by w3zt, never changed) and records/.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that rendered briefs lack the deadline and pacing text, via tests that fail if restored.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt recorded passes required.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Kept to brief text, its plumbing and one prompt rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261115-191642-k2bt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-191642-k2bt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs/planner.rs no longer defines DEADLINE_RULE or deadline_section, and the planner brief has no '## Deadline' section (git diff shows removal; Grep for 'Deadline' in briefs/planner.rs finds none).
crates/athena-context/src/briefs.rs no longer defines time_budget_section and the producer brief has no '## Time budget' section; Grep for 'Time budget' in crates/ finds none; the deadline_s input fields that only fed these sections are removed from the planner and producer brief inputs and their callers (planning.rs, producer.rs, run/mod.rs), while the enforced agent-spec deadline is unchanged.
Brief tests assert the rendered planner brief and producer brief contain neither 'deadline of' / 'seconds of wall-clock' nor 'Time budget', and the old tests pinning those sections (briefs.rs ~1865-1880, run/tests.rs:1438) are removed or inverted; run `cargo test -p athena-context` and `cargo test -p athena-core`; the new absence assertions fail if either section is restored.
crates/athena-context/prompts/planner.md gains one short rule (within its 95-token headroom): when the request is a queued fix or retry, first check from the code and config whether its problem still exists, and if not, report that with the reason instead of planning a change. The prompt token-budget test still passes (`cargo test -p athena-context`).
CHANGELOG.md has a new Unreleased entry recording this change and citing Sri's request and jobs 6ee6c79d844c and 64a92cc76d29. Because the change is prompt-only, recorded fmt, clippy and test checks on the final tree pass.
The 66w1 scaling (PLANNER_ESCALATED_TIMEOUT_FACTOR in run/planning.rs) is left in place unless removing the dead deadline plumbing requires editing adjacent lines; planner_timeout_s, producer limits and athena.toml/defaults.toml values are unchanged (git diff shows no change to them).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
