# Job 20261113-205906-0m6r: Give reader and planner the field-meaning fact before reusing a field

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Planning checks that named files and seams exist, but nothing checks that a field the reading reuses
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: reading-maps-requirement-to-existing-field-with-different-meaning-fails-planning-rounds (20261112-090746-vvvp)
> Cause: Planning checks that named files and seams exist, but nothing checks that a field the reading reuses actually means what the requirement needs. The reading matched the words "fixed in Athena" to a field by name and plausibility without read...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261112-090730-g560 (job 20261030-013421-tfwf); prevention it proposed: Add a reading-stage check: when the reading maps a requirement onto an existing struct field, CLI argument or record key, it must quote that field's documented meaning from the code and state why it matches. A mismatch, or a requirement wit...
> Introspection 20261113-205845-yyfk (job 20261113-194336-scfc); prevention it proposed: Before a reading is accepted, run a step that resolves each config key or field it reuses against the live athena.toml and the code that reads it. It should report the effective value for each role and whether the change would alter behavio...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: neither the reader's brief nor the planner prompt told the stage that a reused struct field, CLI argument or record key must be read for its documented meaning, so jobs 20261030-013421-tfwf (restart `via` read as "fix in Athena") and 20261113-194336-scfc (hard_producer_* read as a planner upgrade) matched by name. The fix supplies that rule as information in the reader brief's Reading rules and the planner prompt, not as a refusal check. Both earlier candidates changed no files, so none is kept; the earlier findings are addressed as stated in the assumptions.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/reader.rs (the READING_RULES list around lines 290-300 lists rules with check names or `(prose only)`), crates/athena-context/prompts/planner.md and critic.md, crates/athena-core/src/reading/surveys.rs (check_code_survey, which this change does not touch), and crates/athena-core/src/run/planning.rs. Entry point is brief rendering for the reader and planner; no record or data file is added, so AnyRecord and check_record_file are unaffected. The property is measured directly by asserting the rendered brief text.
- Existing code: crates/athena-context/src/briefs/reader.rs and crates/athena-context/prompts/planner.md are extended; the new criteria reuse them.
- Job 20261030-013421-tfwf: its candidate changed no files, so nothing is kept. Its major finding (via is documented as what the new context read to resume, crates/athena-cli/src/main.rs:450, yet was counted as a fix) is addressed at the origin by the new rule, which would have made the reader quote that doc and add a new `fixed` field. Its OLY-B-O5-02 feature work (docs/spec.toml, status.rs, display.rs, manage.rs) is out of scope here and stays queued by the drain.
- Job 20261113-194336-scfc: its candidate changed no files, so nothing is kept. Its finding (hard_producer_model/effort do not upgrade the planner under mode = hard; the reading did not check the live athena.toml) is addressed by the same rule, which also tells the stage to check the live config value. Its planning.rs, tests.rs, config.rs, drain.rs and spec.toml work is out of scope.
- Goal and audience: the reader and planner roles are the audience; the goal is that they receive the information before mapping a requirement to an existing field. This is preferred over a refusal rule, since the reading's meaning cannot be judged by mechanism.
- Athena's reader prompt has 2 tokens of headroom, so the rule goes in the reader brief's Reading rules, which is code, not in prompts/reader.md. Whether the brief has its own token budget test is unconfirmed; the producer must run the tests.
- Consumers: none found (no athena.toml or defaults.toml change).

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-context (run `cargo test -p athena-context`) renders the reader brief and asserts its Reading rules section contains a rule requiring, for each reused existing struct field, CLI argument, record key or config key, a `Field meaning:` assumption quoting the documented meaning with a crates/ path:line and why it matches, or a new field when it does not. The test fails on the code before the change.
2. [behavior] A new unit test in crates/athena-context (`cargo test -p athena-context`) asserts the planner prompt/brief carries the same instruction, so a plan that reuses a field must quote its documented meaning and check the live athena.toml value; it fails before the change.
3. [content] The new rule is generic (any field, CLI argument, record key or config key), not specific to restart `via` or `hard_producer_*`; the diff of crates/athena-context/src/briefs/reader.rs shows it added to the Reading rules list marked `(prose only)`.
4. [content] The diff adds no new refusing check in crates/athena-core; the information is given in the brief and prompt, so expected 0% of past accepted readings are newly refused.
5. [content] The CHANGELOG.md entry names the root cause (the reading and plan stages were never told to verify a reused field's documented meaning) and cites jobs 20261030-013421-tfwf and 20261113-194336-scfc; any decision is recorded as a new appended docs/decisions.md entry with a Provenance bullet.
6. [content] Token budgets hold: `cargo test` (including the prompt-budget tests) passes with planner.md growth within the 95 tokens left; no edit to the reader prompt, which has 2 tokens left.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (Athena's recorded checks on that tree count).

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/prompts/critic.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Any refusal check in crates/athena-core that rejects readings
- Implementing OLY-B-O5-02 or the restart `fixed` field
- Planner effort or model escalation (job 20261113-194336-scfc work)
- Editing athena.toml, defaults.toml, workflows or docs/spec.toml
- Editing the reader prompt prompts/reader.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the rendered brief text, not file existence
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to brief and prompt text
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the stage lacking the information
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

- path: <state>/jobs/20261113-205906-0m6r/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-205906-0m6r/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-context (run `cargo test -p athena-context`) renders the reader brief and asserts its Reading rules section contains a rule requiring, for each reused existing struct field, CLI argument, record key or config key, a `Field meaning:` assumption quoting the documented meaning with a crates/ path:line and why it matches, or a new field when it does not. The test fails on the code before the change.
A new unit test in crates/athena-context (`cargo test -p athena-context`) asserts the planner prompt/brief carries the same instruction, so a plan that reuses a field must quote its documented meaning and check the live athena.toml value; it fails before the change.
The new rule is generic (any field, CLI argument, record key or config key), not specific to restart `via` or `hard_producer_*`; the diff of crates/athena-context/src/briefs/reader.rs shows it added to the Reading rules list marked `(prose only)`.
The diff adds no new refusing check in crates/athena-core; the information is given in the brief and prompt, so expected 0% of past accepted readings are newly refused.
The CHANGELOG.md entry names the root cause (the reading and plan stages were never told to verify a reused field's documented meaning) and cites jobs 20261030-013421-tfwf and 20261113-194336-scfc; any decision is recorded as a new appended docs/decisions.md entry with a Provenance bullet.
Token budgets hold: `cargo test` (including the prompt-budget tests) passes with planner.md growth within the 95 tokens left; no edit to the reader prompt, which has 2 tokens left.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (Athena's recorded checks on that tree count).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
