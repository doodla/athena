# Job 20261114-052309-w3zt: Scale planner timeout when a revision round escalates effort

## Request (verbatim)

> Requested by Sri (the owner). When planning escalates the planner to planner_revision_effort (xhigh) on a revision round, scale the planner's time limit with it, so a stronger revision is not cut off by the base planner_timeout_s of 600 s. Job 20261114-041123-hvyh failed this way: round 2 ran at xhigh for 606 s and hit the 600 s deadline.

## Reading (accepted by the user)

Retry of job 20261114-041123-hvyh, whose planning stopped at round 2 (xhigh, 606 s, 600 s deadline). In planning_round, when the planner revision setting raises effort above the planner's own, the planner time limit is multiplied (2x, a constant) instead of staying at planner_timeout_s. The prior job's follow-up-queue feature is not part of this job.

Assumptions:
- Code survey: crates/athena-core/src/run/planning.rs planning_round (lines ~141-165) picks model/effort via jc.config.models.planner_revision_setting when round > 1 or mechanical_passes > 0, then passes cfg.planner_timeout_s to agent_spec; crates/athena-data/src/config.rs (planner_timeout_s at line 229, planner_revision_effort at 343, planner_revision_setting near 386, effort_rank) defines the setting; tests in crates/athena-core/src/run/tests.rs (config helper near line 1328 writes planner_revision_effort and planner_timeout_s = 600). No record or data file is added, so check_record_file/AnyRecord are not involved. The timeout is measured directly on the AgentSpec built for the planner call.
- Existing code: crates/athena-core/src/run/planning.rs and crates/athena-data/src/config.rs are extended; the existing planner_revision_setting result is reused to decide whether effort was escalated, and no new config key is added.
- Design call: the scale is a fixed constant (2x, so 1200 s from the base 600 s) applied only when the revision effort differs from the planner's own effort; round 1, seeded first calls and unescalated revisions keep planner_timeout_s. A config key was avoided so athena.toml and defaults.toml stay unchanged; the 2x value is an estimate, based on 606 s observed against a 600 s limit.
- Job 20261114-041123-hvyh left no candidate changes, so there is no work to keep or drop. Its original deliverables (followups.rs, steward.rs and others) are dropped from this job and remain unbuilt follow-up; that request will need to be retried separately.
- Audience: Sri, who runs hard jobs and judges whether a stronger revision round can finish; the planner prompt is unchanged. Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/run/tests.rs configures planner_revision_effort higher than the planner's effort with planner_timeout_s = 600, runs a revision round (round 2), and asserts the planner AgentSpec timeout is greater than 600 s. Run: cargo test -p athena-core.
2. [behavior] A test asserts that round 1 and a revision without escalated effort (no planner_revision_effort set, or not stronger) still use exactly planner_timeout_s. Run: cargo test -p athena-core.
3. [content] The diff in crates/athena-core/src/run/planning.rs scales cfg.planner_timeout_s only when the revision setting raises effort, via a named constant with a doc comment saying why (job 20261114-041123-hvyh hit 600 s at xhigh).
4. [content] The deliverables are only planning.rs, run/tests.rs and CHANGELOG.md; athena.toml, defaults.toml, hestia.rs, the prompts and records/ are unchanged in the diff.
5. [behavior] cargo fmt --check, cargo clippy and cargo test pass on the final tree (recorded Athena checks count).
6. [content] CHANGELOG.md has an entry recording the planner time limit scaling and citing job 20261114-041123-hvyh.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- The follow-up queueing feature from job 20261114-041123-hvyh (followups.rs, steward.rs, manage.rs, reading.rs, lib.rs, crates/athena-cli/tests/drain.rs): dropped here, to be retried as its own request.
- Backfilling lost follow-ups from job 20261114-034034-fgwz and the hestia.rs findings; hestia.rs unchanged.
- A new athena.toml or defaults.toml key, changes to planner_timeout_s itself, critic timeout, planner prompts, workflows and records/.
- Changing which rounds escalate effort.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus cargo test and clippy criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the planning step.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Scales the limit with the escalation instead of just raising the base timeout.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code and a test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

35 more qualifying classes are left out.

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

- path: <state>/jobs/20261114-052309-w3zt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-052309-w3zt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/run/tests.rs configures planner_revision_effort higher than the planner's effort with planner_timeout_s = 600, runs a revision round (round 2), and asserts the planner AgentSpec timeout is greater than 600 s. Run: cargo test -p athena-core.
A test asserts that round 1 and a revision without escalated effort (no planner_revision_effort set, or not stronger) still use exactly planner_timeout_s. Run: cargo test -p athena-core.
The diff in crates/athena-core/src/run/planning.rs scales cfg.planner_timeout_s only when the revision setting raises effort, via a named constant with a doc comment saying why (job 20261114-041123-hvyh hit 600 s at xhigh).
The deliverables are only planning.rs, run/tests.rs and CHANGELOG.md; athena.toml, defaults.toml, hestia.rs, the prompts and records/ are unchanged in the diff.
cargo fmt --check, cargo clippy and cargo test pass on the final tree (recorded Athena checks count).
CHANGELOG.md has an entry recording the planner time limit scaling and citing job 20261114-041123-hvyh.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
