# Job 20261113-000055-2fae: Add a verification status line with a working false-positive count

## Request (verbatim)

> try again on 20261030-033225-g36k

## Reading (accepted by the user)

Retry of job 20261030-033225-g36k: give OLY-B-A7-04 a mechanism by adding a `verification` line to `athena status` (verifier cost, rejections caught, false positives, escapes), where false positives reuse gate_counts' landed-retry/same-change overrule detection instead of impossible Fail+Landed conditions. Audience: Apollo and the owner reading status.

Assumptions:
- Job 20261030-033225-g36k (rejected, final verdict fail) is retried from the base; its candidate is dropped and rebuilt. Findings addressed: (1 blocker) false-positive count was always 0 because it tested Fail-with-final-Pass and rejected-then-landed, both impossible (verify.rs pass needs agent Pass; landing bails unless final Pass) - fixed by counting a rejection as a false positive when a landed retry ('try again on <job>') with the same change exists, via logic factored out of crates/athena-core/src/gate_counts.rs (same_change and the landed-retry scan) and shared; (2 minor) CHANGELOG and docs text described the impossible definition - rewritten to the retry-overrule definition; (3 minor) planning.rs was changed only to widen visibility of LaterCause/later_causes - kept as a deliverable (visibility-only change, no behavior change); (4 nit) the line prints false positives as 'of which' a subset of caught.
- Code survey: read crates/athena-core/src/gate_counts.rs (gate_counts, same_change at line 93, which already detects a landed 'try again on' retry with the same change), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView, tests using 'try again on'), crates/athena-core/src/status/planning.rs (later_causes) and throughput.rs. Entry point is the status view build in status.rs, rendered by crates/athena-cli/src/display.rs. No new record or data file under crates/ is added, so check_record_file/AnyRecord is not touched.
- Existing code: crates/athena-core/src/gate_counts.rs (overrule detection, reused and factored for sharing), crates/athena-core/src/status/planning.rs (later-harm citation reused for escapes), crates/athena-core/src/status.rs (label registry), crates/athena-cli/src/display.rs.
- Environment: the status replay reads the real records snapshot via `athena status --snapshot <evidence directory>` (read-only, local files: history.json, verification and request records); expected counts come from history.json and git log of the local repository (read-only, no push, no CI).
- Moving the gate_counts helper must not change gate_counts output; its existing tests must still pass.
- Governance: docs/spec.toml is a deliverable; athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [content] docs/spec.toml OLY-B-A7-04 has a `mechanism` naming the verification report in crates/athena-core/src/status/verification.rs (not none), keeps owner Apollo, and a `measure` ending with `status: verification`. Check with Grep.
2. [content] STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs maps the label `verification` to a new StatusView field, and crates/athena-cli/src/display.rs prints a line starting with `verification` from it. Check with Grep.
3. [content] verification.rs computes false positives by calling the overrule detection shared with crates/athena-core/src/gate_counts.rs (landed retry with same change), not by testing Fail+Landed combinations; Grep shows the shared function used by both modules.
4. [behavior] A unit test in crates/athena-core builds Records in the shape the real writers produce: rejected and passed verifications, job totals, a rejected job later retried ('try again on <job>') with the same change that landed, and a landed job later reverted. It asserts verifier cost, caught, false positives (1) and escapes. It fails with the computation stubbed and passes with it. Run `cargo test`.
5. [behavior] Real records: run `cargo run -p athena-cli -- status --snapshot <evidence directory>`; the verification line's false-positive count equals the number of rejected jobs in history.json that have a landed 'try again on' retry with the same change (counted independently from history.json and git log commit times, expected range: non-zero, since the real records hold hundreds of retry requests), and caught is at least that number; the unchanged status lines are identical before and after.
6. [behavior] With empty Records a test asserts the verification line contains `none yet` and shows no zero counts as success. Run `cargo test`.
7. [behavior] A test in crates/athena-cli/tests/introspection.rs renders athena_cli::display::status and asserts a line starting with `verification` showing cost, caught, false positives and escaped counts. Run `cargo test`.
8. [behavior] The xtask spec test accepts the new measure marker and label, and existing gate_counts tests still pass. Run `cargo test`.
9. [behavior] fmt, clippy and test pass: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` (recorded on the final tree).
10. [content] CHANGELOG.md has an added entry using the retry-overrule false-positive definition; docs/enforcement.md gets an M35 row; no existing entry of docs/decisions.md or anything under records/ is edited.

## Expected deliverables

- crates/athena-core/src/status/verification.rs (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gate_counts.rs (modify)
- crates/athena-core/src/status/planning.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- docs/enforcement.md (modify)

## Out of scope

- Changing verifier behavior, verdict rules or gates
- Editing existing records/ or docs/decisions.md entries
- Other spec requirements
- Building the generic replay check for all status measures (the class-level prevention); only this measure is replayed on real records

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criterion compares the printed count with an independent count from history.json.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty records print `none yet`.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and CLI tests plus fmt, clippy, test criterion.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-033225-g36k raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (crates/athena-core/src/status/verification.rs:51-53): The false-positive count is always 0 on real records: both conditions it tests are impossible, and the existing overrule detection is not reused The code counts `(agent_verdict == Fail && !rejected) || (rejected && landed(job))`. In verify.rs:663, `pass` requires `agent_verdict == Some(Verdict::Pass)`, so a verifier fail with a final pass cannot happen. In gh/landing.rs:861, landing bails when `final_verdict != Verdict::Pass`, so a job rejected by the final verdict cannot land. The real records snapshot has 161 verification.json files where the verifier said fail and 0 with a final pass. It also has 257 request.json files containing 'try again on', which is the actual overrule path. gate_counts.rs already detects that path: a landed retry ('try again on <job>') with the same change counts as overruling the gate that ended the job. The reading said to count rejections 'overruled or whose job landed by the owner/retry unchanged' and to reuse existing detection. As built, the status line shows `false positives 0` as if measured, and the unit test only passes by building an impossible Fail+Landed job. Simpler, correct alternative: count a rejection as a false positive when gate_counts' landed_retries/same_change logic finds a landed retry with the same change (factor that logic out and share it).

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

- path: <state>/jobs/20261113-000055-2fae/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-000055-2fae/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml OLY-B-A7-04 has a `mechanism` naming the verification report in crates/athena-core/src/status/verification.rs (not none), keeps owner Apollo, and a `measure` ending with `status: verification`. Check with Grep.
STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs maps the label `verification` to a new StatusView field, and crates/athena-cli/src/display.rs prints a line starting with `verification` from it. Check with Grep.
verification.rs computes false positives by calling the overrule detection shared with crates/athena-core/src/gate_counts.rs (landed retry with same change), not by testing Fail+Landed combinations; Grep shows the shared function used by both modules.
A unit test in crates/athena-core builds Records in the shape the real writers produce: rejected and passed verifications, job totals, a rejected job later retried ('try again on <job>') with the same change that landed, and a landed job later reverted. It asserts verifier cost, caught, false positives (1) and escapes. It fails with the computation stubbed and passes with it. Run `cargo test`.
Real records: run `cargo run -p athena-cli -- status --snapshot <evidence directory>`; the verification line's false-positive count equals the number of rejected jobs in history.json that have a landed 'try again on' retry with the same change (counted independently from history.json and git log commit times, expected range: non-zero, since the real records hold hundreds of retry requests), and caught is at least that number; the unchanged status lines are identical before and after.
With empty Records a test asserts the verification line contains `none yet` and shows no zero counts as success. Run `cargo test`.
A test in crates/athena-cli/tests/introspection.rs renders athena_cli::display::status and asserts a line starting with `verification` showing cost, caught, false positives and escaped counts. Run `cargo test`.
The xtask spec test accepts the new measure marker and label, and existing gate_counts tests still pass. Run `cargo test`.
fmt, clippy and test pass: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` (recorded on the final tree).
CHANGELOG.md has an added entry using the retry-overrule false-positive definition; docs/enforcement.md gets an M35 row; no existing entry of docs/decisions.md or anything under records/ is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
