# Job 20261030-033229-hb3a: Give OLY-B-A9-01 a mechanism: automation register with retire rule

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A9-01 a mechanism
> 
> Requirement OLY-B-A9-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The automation engineer builds v0 and stays on call, takes toil and repeat causes, delivers reusable capabilities, and registers each automation with its savings and retires those that cost more than they save.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A9-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A9-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core module holding an automation register (each entry with its cost and measured savings) and a rule that flags automations costing more than they save for retirement. Record it as the mechanism of OLY-B-A9-01 in docs/spec.toml, with a measure ending `status: not shown` that names the test, following the OLY-B-A3-02 pattern.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, 'status: not shown' rule), crates/athena-core/tests/learnings_recorded.rs (the OLY-B-A3-02 pattern: parse_spec, measure_problem, spec_gaps tests), crates/athena-data/src/records.rs (only an 'automation engineer' role string, no register). No record or data file under crates/ is added, so check_record_file/AnyRecord/round-trip are not touched; the register is Rust code with in-test data.
- Existing code: extend the pattern in crates/athena-core/tests/learnings_recorded.rs and use athena_core::hestia::{parse_spec, measure_problem, spec_gaps}; docs/spec.toml is modified, not created.
- Goal: make the spec gap for OLY-B-A9-01 close with a real, tested rule; audience is the Hestia spec pass and the verifier. The full engineer role (on-call, building v0) is prose-level; the enforceable core is savings registration and retirement.
- The measure uses `status: not shown` because no athena status label shows automations; wiring a status line is out of scope.
- No new refusal rule applies to past readings; the retire rule is a standalone function not wired into drain or gates.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] In docs/spec.toml, OLY-B-A9-01 has mechanism no longer "none", naming crates/athena-core/src/automation.rs and the test file crates/athena-core/tests/automation_register.rs; owner stays Athena.
2. [content] The OLY-B-A9-01 measure in docs/spec.toml says how the requirement is shown, names `cargo test -p athena-core --test automation_register`, and ends with `status: not shown` (no label from STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs is claimed that athena status does not print).
3. [content] crates/athena-core/src/automation.rs defines an automation register entry (name, owner, build/run cost, measured savings, source toil or repeat cause) and a pure function returning the entries whose cost exceeds their savings, plus entries lacking savings registration; it is exported from crates/athena-core/src/lib.rs.
4. [behavior] Run `cargo test -p athena-core --test automation_register`: tests show an automation costing more than it saves is listed for retirement, one saving more is kept, an entry with no savings is refused, and the real docs/spec.toml entry for OLY-B-A9-01 has a non-none mechanism, owner Athena, a passing measure_problem, and is absent from spec_gaps. The spec test fails if the mechanism reverts to "none".
5. [behavior] Run `cargo fmt --check`, `cargo clippy`, and `cargo test` (or the Athena-recorded passing checks on the same tree): all pass.
6. [content] CHANGELOG.md records the change; no existing entry of docs/decisions.md or records/ is edited.

## Expected deliverables

- crates/athena-core/src/automation.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/automation_register.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding an `athena status` line or label for automations
- Creating an automation engineer pipeline role or prompt
- Wiring the retire rule into the drain or any gate
- Editing docs/decisions.md or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the named cargo test and check retire/keep outcomes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module, one test file, one spec edit.
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

- path: <state>/jobs/20261030-033229-hb3a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033229-hb3a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In docs/spec.toml, OLY-B-A9-01 has mechanism no longer "none", naming crates/athena-core/src/automation.rs and the test file crates/athena-core/tests/automation_register.rs; owner stays Athena.
The OLY-B-A9-01 measure in docs/spec.toml says how the requirement is shown, names `cargo test -p athena-core --test automation_register`, and ends with `status: not shown` (no label from STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs is claimed that athena status does not print).
crates/athena-core/src/automation.rs defines an automation register entry (name, owner, build/run cost, measured savings, source toil or repeat cause) and a pure function returning the entries whose cost exceeds their savings, plus entries lacking savings registration; it is exported from crates/athena-core/src/lib.rs.
Run `cargo test -p athena-core --test automation_register`: tests show an automation costing more than it saves is listed for retirement, one saving more is kept, an entry with no savings is refused, and the real docs/spec.toml entry for OLY-B-A9-01 has a non-none mechanism, owner Athena, a passing measure_problem, and is absent from spec_gaps. The spec test fails if the mechanism reverts to "none".
Run `cargo fmt --check`, `cargo clippy`, and `cargo test` (or the Athena-recorded passing checks on the same tree): all pass.
CHANGELOG.md records the change; no existing entry of docs/decisions.md or records/ is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
