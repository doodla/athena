# Job 20261030-033234-vm1m: Measure tokens loaded per class of work in athena status

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A13-01 a mechanism
> 
> Requirement OLY-B-A13-01 in docs/spec.toml is active and has no mechanism.
> Requirement: Context compression is an explicit goal: Athena measures tokens loaded per class of work and whether outcomes held.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A13-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A13-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Give OLY-B-A13-01 a real mechanism: a new athena-core module computes tokens loaded per landed job by class of work, beside whether outcomes held (landed share and mistakes per class). `athena status` prints it as a `context` line, and docs/spec.toml names the module and ends the measure with `status: context`. Goal is context compression; the audience is the Athena owner reading `athena status` and the spec gap list.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView, mistake_report, landed_jobs), crates/athena-core/src/class_trend.rs (the per-class pattern to mirror: analyze/compute/lines, prints nothing on a thin history), crates/athena-core/src/invocation.rs (tokens_lower_bound, prompt_tokens), crates/athena-core/tests/learnings_recorded.rs (the spec-mechanism test pattern using parse_spec, spec_gaps, measure_problem), and crates/athena-cli/src/display.rs (where the trend section is printed). The new code reads the existing token facts on recorded invocations; it adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: extend crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView), crates/athena-core/src/class_trend.rs (pattern), crates/athena-cli/src/display.rs (printing), and docs/spec.toml; the new module is the only file created under crates/ besides a test.
- The status label is `context`, so the measure ends with `status: context`. Adding it to STATUS_MEASURE_LABELS (so the Hestia spec pass accepts it) is part of the change.
- Outcomes held means landed share and mistakes for the same class of work, shown beside tokens per class. It is not a new quality judgement.
- A class with too few landed jobs prints no figure, as the trend line does, so a thin history is not shown as success.
- Not a refusal rule: nothing is newly refused, so no refusal-rate criterion applies.
- Consumers: none found for new config; athena.toml and defaults.toml are not changed.
- Environment: none. No CI, push, PR or records-branch reach is needed.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A13-01 has a mechanism naming the new module path under crates/athena-core/src/ and its test, owner `Athena`, and a measure that ends with `status: context`. The `context` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [content] A new module in crates/athena-core/src/ is registered in lib.rs. It groups tokens loaded per landed job by class of work and reports, for the same class, the landed share and mistake count. `StatusView` has a matching field, and crates/athena-cli/src/display.rs prints a line starting with `context`.
3. [behavior] Run `cargo test -p athena-core`. A unit test builds records with known token counts across two classes and asserts the exact per-class figures and the outcome figures. A second test asserts that a class below the minimum job count yields no figure. A third test asserts that dropping the mechanism (setting it to `none`) makes `spec_gaps` report OLY-B-A13-01, and that the real spec entry is not a gap. These tests fail without the module and pass with it.
4. [behavior] Run `cargo test -p athena-cli`. A display test asserts the `context` line appears in `athena status` output when the data exists and is absent, rather than showing success, when it does not.
5. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or point to the Athena-recorded passes on the final tree. All pass.
6. [content] CHANGELOG.md records the job, and a new appended decision with a Provenance bullet in docs/decisions.md records the choice of tokens-per-landed-job as the compression measure. No existing decisions.md or records/ entry is edited.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/context_load.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/tests/context_compression.rs (create)

## Out of scope

- Changing prompts or prompt token budgets
- Actually reducing the context any role loads
- Editing or annotating existing docs/decisions.md or records/ entries
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests, and fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check exact per-class figures and the status line, not that files exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A class with too few jobs shows nothing rather than success.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is enforced by a test on the spec entry.
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

- path: <state>/jobs/20261030-033234-vm1m/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033234-vm1m/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-B-A13-01 has a mechanism naming the new module path under crates/athena-core/src/ and its test, owner `Athena`, and a measure that ends with `status: context`. The `context` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
A new module in crates/athena-core/src/ is registered in lib.rs. It groups tokens loaded per landed job by class of work and reports, for the same class, the landed share and mistake count. `StatusView` has a matching field, and crates/athena-cli/src/display.rs prints a line starting with `context`.
Run `cargo test -p athena-core`. A unit test builds records with known token counts across two classes and asserts the exact per-class figures and the outcome figures. A second test asserts that a class below the minimum job count yields no figure. A third test asserts that dropping the mechanism (setting it to `none`) makes `spec_gaps` report OLY-B-A13-01, and that the real spec entry is not a gap. These tests fail without the module and pass with it.
Run `cargo test -p athena-cli`. A display test asserts the `context` line appears in `athena status` output when the data exists and is absent, rather than showing success, when it does not.
Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or point to the Athena-recorded passes on the final tree. All pass.
CHANGELOG.md records the job, and a new appended decision with a Provenance bullet in docs/decisions.md records the choice of tokens-per-landed-job as the compression measure. No existing decisions.md or records/ entry is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
