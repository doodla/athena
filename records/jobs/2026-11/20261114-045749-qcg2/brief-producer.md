# Job 20261114-045749-qcg2: Check decision links in the producer loop; key them on ids

## Request (verbatim)

> Priority: high
> 
> Add a check, run inside the producer's loop before handover, for any job that edits docs/decisions.md. It fails if an entry in docs/decision-links.toml no longer resolves. Better: key those entries on decision ids, not line numbers. It also lists sentences in docs/ that still state the claim of an amended or superseded decision. Then re-queue the D08 amendment, which is still undone. Also record job 20261104-153450-y72x under failure class 20261104-153440-xf17 with 20261010-225741-yvk0, so the repeat is counted.
> 
> Introspections that record the repeated cause: 20261104-153423-esay, 20261106-012825-ay9k, 20261106-042756-h268, 20261106-191104-tvy1, 20261108-010633-1v8s
> 
> Acceptance:
> - The change is made as described: Amending a decision in docs/decisions.md leaves contradicting docs or breaks the line references in decision-links.toml; only the final verdict catches it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/decisions.md., docs/decision-links.toml, docs/

## Reading (accepted by the user)

Add a pre-handover check, run for any job that edits docs/decisions.md, that fails when a docs/decision-links.toml [[link]] no longer resolves, re-keys links on decision ids instead of line numbers, and lists docs/ lines that still cite an amended or superseded decision. It follows up jobs 20261104-153450-y72x and 20261010-225741-yvk0, whose line-shift and stale-doc failures it prevents; the D08 re-queue and failure-class recording are queue and records actions outside the diff.

Assumptions:
- Goal and audience: producers and verifiers of any job that amends or supersedes a decision should be stopped by a mechanism before handover, not by the final verdict; the user judges the result.
- Code survey: read crates/athena-data/src/decision_links.rs (the one parser of [[governs]] and [[link]], pure rule `unreconciled`), crates/athena-core/src/run/checks.rs (DECISION_LINKS const, the pre-verifier governs check, line 620-689), and grep hits in crates/athena-cli/src/backtest.rs and crates/athena-cli/src/decisions.rs, crates/athena-core/src/run/tests.rs. The producer will extend the checks.rs gate and keep decision_links.rs as the shared parser. docs/decision-links.toml is data, not a record file under crates/, so no AnyRecord or check_record_file change is expected.
- Existing code: crates/athena-data/src/decision_links.rs, crates/athena-core/src/run/checks.rs, crates/athena-cli/src/backtest.rs, docs/decision-links.toml and docs/design/decision-sets.md are extended, not created.
- Environment: none; everything runs locally over the working tree with no CI, push or history reads.
- Field meaning: [[link]] `line` is documented in docs/decision-links.toml lines 3-4 as the line in docs/decisions.md of the phrase. The change makes the pair (older, newer) the key; `line` becomes optional or is dropped. Config: athena.toml unchanged.
- The sentence listing is advisory output (does not fail): lines in docs/ outside decisions.md that cite the id of a decision the change marks amended or superseded. A heuristic on ids; it cannot read meaning.
- D08 re-queue and recording job 20261104-153450-y72x under failure class 20261104-153440-xf17 with 20261010-225741-yvk0 are Athena queue and records actions; records/ never changes in this diff.
- Job 20261104-153450-y72x is addressed this way: its backtest findings (every link off by +3) cannot recur because links no longer depend on line numbers; its docs/design/athena-v0.md finding belongs to the dropped candidate. Its changed code and docs (snapshot.rs, lib.rs, producer.rs, verification.rs, run/mod.rs, run_pin_tests.rs, briefs.rs, enforcement.md) are dropped, not kept. Job 20261010-225741-yvk0's missing D08 amendment is handled by the re-queue.
- Existing decisions.md entries are not edited; the change is recorded in a new appended decision with a Provenance bullet, and a new [[link]] only if it amends an earlier decision (it may amend D156).

## Acceptance criteria

1. [behavior] A unit test in crates/athena-data/src/decision_links.rs (or athena-core checks tests) shows the new check fails when a [[link]] names an older/newer pair not found in docs/decisions.md and passes when decisions.md shifts by added lines; run `cargo test --locked` and it passes.
2. [behavior] The check runs in the producer loop only for jobs whose changed paths include docs/decisions.md, and a test in crates/athena-core/src/run/tests.rs shows a job not touching it is not checked and one touching it with a broken link is held before the verifier; run `cargo test --locked -p athena-core`.
3. [content] docs/decision-links.toml [[link]] entries are keyed by older/newer decision ids; no entry needs a line number to resolve, and the header comment no longer tells editors to update line numbers.
4. [behavior] `cargo test --locked -p athena-cli --test backtest` passes, including the confirmed-list tests, and a test shows inserting lines into a copy of decisions.md before a phrase does not produce a condition-a finding.
5. [behavior] A test shows the sentence listing returns the docs/ lines (outside decisions.md) that cite an amended or superseded decision id and does not fail the job; run `cargo test --locked`.
6. [content] docs/decisions.md gains a new appended decision (next free id, Provenance bullet) recording the check; no existing entry is edited; docs/design/decision-sets.md and CHANGELOG.md describe the check and the id keying.
7. [behavior] The check refuses nothing in the current repository: the real docs/decision-links.toml and docs/decisions.md pass it, shown by `cargo test --locked` (existing backtest tests on the real log).
8. [behavior] No unrelated behavior changes and fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy --locked --all-targets` and `cargo test --locked` on the final tree pass.

## Expected deliverables

- crates/athena-data/src/decision_links.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- docs/decision-links.toml (modify)
- docs/decisions.md (modify)
- docs/design/decision-sets.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rebuilding the ATHENA-COMMITS.txt history feature or any change to crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-context/src/briefs.rs (dropped from the earlier candidates)
- docs/enforcement.md and docs/design/athena-v0.md edits belonging to the dropped history feature
- Rewriting existing docs sentences automatically; the listing is advisory
- Edits or annotation of existing docs/decisions.md entries, and anything under records/
- Edits to athena.toml, defaults.toml or workflows
- Re-queuing job D08 and recording failure class 20261104-153440-xf17 (queue and records actions by Athena)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the check, id keying and listing; cargo test, clippy and fmt required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in the producer loop, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Links keyed on ids removes the line-number cause rather than updating lines.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that run the check on a broken and a shifted log.
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

- path: <state>/jobs/20261114-045749-qcg2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045749-qcg2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A unit test in crates/athena-data/src/decision_links.rs (or athena-core checks tests) shows the new check fails when a [[link]] names an older/newer pair not found in docs/decisions.md and passes when decisions.md shifts by added lines; run `cargo test --locked` and it passes.
The check runs in the producer loop only for jobs whose changed paths include docs/decisions.md, and a test in crates/athena-core/src/run/tests.rs shows a job not touching it is not checked and one touching it with a broken link is held before the verifier; run `cargo test --locked -p athena-core`.
docs/decision-links.toml [[link]] entries are keyed by older/newer decision ids; no entry needs a line number to resolve, and the header comment no longer tells editors to update line numbers.
`cargo test --locked -p athena-cli --test backtest` passes, including the confirmed-list tests, and a test shows inserting lines into a copy of decisions.md before a phrase does not produce a condition-a finding.
A test shows the sentence listing returns the docs/ lines (outside decisions.md) that cite an amended or superseded decision id and does not fail the job; run `cargo test --locked`.
docs/decisions.md gains a new appended decision (next free id, Provenance bullet) recording the check; no existing entry is edited; docs/design/decision-sets.md and CHANGELOG.md describe the check and the id keying.
The check refuses nothing in the current repository: the real docs/decision-links.toml and docs/decisions.md pass it, shown by `cargo test --locked` (existing backtest tests on the real log).
No unrelated behavior changes and fmt, clippy and test pass: Athena's recorded `cargo fmt --check`, `cargo clippy --locked --all-targets` and `cargo test --locked` on the final tree pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
