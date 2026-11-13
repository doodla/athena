# Job 20261113-023315-gk4a: Retry OLY-B-A13-01: tokens-per-class status line, drain-safe

## Request (verbatim)

> try again on 20261030-033234-vm1m

## Reading (accepted by the user)

Retry of job 20261030-033234-vm1m: re-deliver the context-load mechanism (new athena-core module, `context` line in `athena status`, spec entry ending `status: context`) and fix its findings: unrelated-looking drain failure, D43 evidence, tighter mistake figures, and an end-to-end display test. Audience is the Athena owner reading `athena status` and the spec gap list.

Assumptions:
- Prior job 20261030-033234-vm1m is retried from the base; its candidate is re-made, keeping all nine changed files as deliverables (including xtask/shard-timings.txt, which the ci_shards test requires for the new test file).
- Findings addressed: (1) check-test failure on athena-cli drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn was not reproduced and the change touches no drain code; the producer runs `cargo test -p athena-cli --test drain` and reports the result, and if it still fails on the final tree it is investigated rather than dismissed as flaky. (2) D43: criteria name each command as its own simple command. (3) shard-timings.txt is now a listed deliverable. (4) mistake figures: count only mistakes whose job is among the counted closed jobs, so numerator and denominator cover the same set; unattached mistakes are not folded in. (5) the display test is made end to end where feasible.
- The introspection's proposal to change the test check's kept output (failing-test extraction, base-tree rerun) is a separate change to the check pipeline; it is out of scope here. The typed request, 'try again', takes precedence.
- Code survey: crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, mistake_report at ~595, StatusView), crates/athena-core/src/hestia.rs (spec_gaps ~1291, label acceptance ~1230), crates/athena-cli/src/display.rs (status printing); the new module needs lib.rs registration. It reads existing records only and adds no new record type or data file, so check_record_file and AnyRecord are not touched.
- Existing code: reuse mistake_report in crates/athena-core/src/status.rs and the existing StatusView/STATUS_MEASURE_LABELS and spec_gaps in crates/athena-core/src/hestia.rs; extend them rather than building parallel paths.
- Current docs/spec.toml entry OLY-B-A13-01 has mechanism `none` and measure `The spec gap list; status: requirement` (read at line 2866).
- Minimum landed jobs per class is 3, as before; thin classes yield no figure.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A13-01 has a mechanism naming crates/athena-core/src/context_load.rs and its test, owner `Athena`, and a measure ending with `status: context`; `context` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [content] crates/athena-core/src/context_load.rs is registered in crates/athena-core/src/lib.rs; it groups tokens loaded per landed job by class of work with landed share and mistakes for the same class, counting only mistakes of the counted jobs. StatusView has a matching field and crates/athena-cli/src/display.rs prints a line starting with `context`.
3. [behavior] Run `cargo test -p athena-core --test context_compression`: tests with known token counts across two classes assert exact per-class and outcome figures; a thin class yields no figure; setting the mechanism to `none` makes spec_gaps report OLY-B-A13-01 while the real entry is not a gap. These fail without the module.
4. [behavior] Run `cargo test -p athena-cli`: display tests assert the `context` line appears in status output when data exists and is absent when it does not, and `--test drain` passes including parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test --locked` each as its own command, or cite Athena-recorded passes on the final tree; all pass, including the ci_shards timings test with the line for context_compression in xtask/shard-timings.txt.
6. [content] CHANGELOG.md records the job and a new appended decision with a Provenance bullet in docs/decisions.md records tokens-per-landed-job as the compression measure; no existing decisions.md or records/ entry is edited.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/context_load.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/tests/context_compression.rs (create)
- xtask/shard-timings.txt (modify)

## Out of scope

- Changing prompts or prompt token budgets
- Actually reducing the context any role loads
- Editing or annotating existing docs/decisions.md or records/ entries
- Changing athena.toml, defaults.toml or workflows
- Changing the test check's kept-output extraction or base-tree rerun (separate change)
- Changes to drain code

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the printed status line and per-class figures, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-033234-vm1m raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 5 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`

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

- path: <state>/jobs/20261113-023315-gk4a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-023315-gk4a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-B-A13-01 has a mechanism naming crates/athena-core/src/context_load.rs and its test, owner `Athena`, and a measure ending with `status: context`; `context` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
crates/athena-core/src/context_load.rs is registered in crates/athena-core/src/lib.rs; it groups tokens loaded per landed job by class of work with landed share and mistakes for the same class, counting only mistakes of the counted jobs. StatusView has a matching field and crates/athena-cli/src/display.rs prints a line starting with `context`.
Run `cargo test -p athena-core --test context_compression`: tests with known token counts across two classes assert exact per-class and outcome figures; a thin class yields no figure; setting the mechanism to `none` makes spec_gaps report OLY-B-A13-01 while the real entry is not a gap. These fail without the module.
Run `cargo test -p athena-cli`: display tests assert the `context` line appears in status output when data exists and is absent when it does not, and `--test drain` passes including parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn.
Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test --locked` each as its own command, or cite Athena-recorded passes on the final tree; all pass, including the ci_shards timings test with the line for context_compression in xtask/shard-timings.txt.
CHANGELOG.md records the job and a new appended decision with a Provenance bullet in docs/decisions.md records tokens-per-landed-job as the compression measure; no existing decisions.md or records/ entry is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
