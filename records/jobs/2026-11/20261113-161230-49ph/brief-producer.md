# Job 20261113-161230-49ph: Raise planner effort on each plan revision instead of thrashing

## Request (verbatim)

> when the plan thrashing, increase intelligence
> Improve the model instead

## Reading (accepted by the user)

When planning thrashes (Athena's own checks or the critic send the plan back to the planner), the next planner call runs at a higher effort level, one step per revision up to "max", instead of repeating at the same setting. Audience: Athena operators reading invocation records and the drain output.

Assumptions:
- Reading of the request: 'plan thrashing' means planner/critic rounds that revise repeatedly (revise verdicts, or the mechanical revision passes that end in stop_on_mechanical_limit); 'increase intelligence / improve the model' means raise the planner's effort level (Claude Code --effort), not change the model family. Goal: fewer planning stops; audience: Athena operators.
- Code survey: crates/athena-core/src/run/planning.rs (plan_round loops rounds 1..=max_rounds with an inner mechanical loop; planning_round calls agent_spec with Role::Planner and the producer model; a revision brief is Some on every re-call), crates/athena-core/src/reading.rs (agent_spec line ~443 sets effort via config.effort_for(role)), crates/athena-data/src/config.rs (effort_for maps Planner and Critic to verifier_effort 'high'; EFFORT_LEVELS low..max; Models::producer_setting is the existing escalation pattern for hard jobs). athena.toml [models] and [job_plan] were read. No new record or data file is added, so check_record_file/AnyRecord are not touched; effort is already recorded in each invocation record, which is the direct measure.
- Design choice: the escalation is a pure function in athena-data (next effort level above a base, capped at the last of EFFORT_LEVELS) used by planning_round and passed to agent_spec as an effort override; the first planner call and the critic keep the configured effort. No new athena.toml key is added, so athena.toml is not a deliverable.
- Consumers: none found for a new config key because none is added; agent_spec callers in reading.rs and run/*.rs pass an effort override of None to stay unchanged.
- Term thrash: the planner is called again for the same job because the critic revised the plan or Athena's own plan checks found a blocker or major finding, per the existing thrash signals in docs/design/job-planning.md.
- Environment: none; no CI, push or records read is needed. Tests use the fake agent in crates/athena-agent/src/fake.rs and crates/athena-core/src/run/tests.rs.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-data/src/config.rs shows the escalation helper returns the next EFFORT_LEVELS entry per step, stays at 'max' when capped, and keeps an unknown base unchanged; `cargo test -p athena-data` passes.
2. [behavior] A test in crates/athena-core/src/run/tests.rs drives a planning job with a revise verdict then an accept and asserts, from the recorded planner invocations, that the first planner call ran at the configured effort and the second at a strictly higher effort, while the critic's effort is unchanged; `cargo test -p athena-core` passes.
3. [behavior] A test covers the mechanical-revision path: a plan that breaks Athena's plan rules is re-planned at a higher effort on the next pass (asserted from invocation records), and a plan that still fails stops the job as before.
4. [content] crates/athena-core/src/run/planning.rs passes an escalated effort only when `revision` is Some (a re-call), and the first call of a job uses effort_for(Role::Planner) unchanged; the diff of agent_spec in crates/athena-core/src/reading.rs adds an optional effort override and every other caller passes None.
5. [content] docs/design/job-planning.md describes the escalation (when it applies, step size, the cap at max), and a new appended entry in docs/decisions.md records the decision with a Provenance bullet; no existing decisions.md entry or records/ file is edited.
6. [content] CHANGELOG.md has an entry for the change; no change to athena.toml, defaults.toml or workflows.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the final tree (Athena's recorded check output counts).

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/job-planning.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/spec.toml (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Changing the planner or critic model family or the [models] settings in athena.toml
- Changing max_rounds, planner_usd or critic_usd caps
- Adding thrash detection for other loops (producer, verifier)
- Editing existing entries of docs/decisions.md or anything under records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new unit and run tests plus cargo test, clippy and fmt passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the recorded planner effort rises across revisions, not that wiring exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The escalation is enforced in code in planning_round, not by prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entry written for an outside reader.
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

- path: <state>/jobs/20261113-161230-49ph/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-161230-49ph/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A unit test in crates/athena-data/src/config.rs shows the escalation helper returns the next EFFORT_LEVELS entry per step, stays at 'max' when capped, and keeps an unknown base unchanged; `cargo test -p athena-data` passes.
A test in crates/athena-core/src/run/tests.rs drives a planning job with a revise verdict then an accept and asserts, from the recorded planner invocations, that the first planner call ran at the configured effort and the second at a strictly higher effort, while the critic's effort is unchanged; `cargo test -p athena-core` passes.
A test covers the mechanical-revision path: a plan that breaks Athena's plan rules is re-planned at a higher effort on the next pass (asserted from invocation records), and a plan that still fails stops the job as before.
crates/athena-core/src/run/planning.rs passes an escalated effort only when `revision` is Some (a re-call), and the first call of a job uses effort_for(Role::Planner) unchanged; the diff of agent_spec in crates/athena-core/src/reading.rs adds an optional effort override and every other caller passes None.
docs/design/job-planning.md describes the escalation (when it applies, step size, the cap at max), and a new appended entry in docs/decisions.md records the decision with a Provenance bullet; no existing decisions.md entry or records/ file is edited.
CHANGELOG.md has an entry for the change; no change to athena.toml, defaults.toml or workflows.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass on the final tree (Athena's recorded check output counts).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
