# Job 20261114-045652-r3mk: Tell the reader the verifier sees only the candidate tree, not base runs

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Criteria are written without a defined list of what the verifier can observe (the diff, the repo sna
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criterion-requires-evidence-verifier-cannot-observe (20261027-011728-xrka)
> Cause: Criteria are written without a defined list of what the verifier can observe (the diff, the repo snapshot, check output). Nothing compares a criterion's required evidence with that list. A criterion that only the producer's report could sat...
> Seen 10 time(s) in 10 jobs. The introspections that record it:
> Introspection 20261027-011714-9qse (job 20261027-004517-gk9e); prevention it proposed: Add a check at reading or request acceptance that scans each criterion for evidence sources the verifier lacks, such as "producer's report", "the report lists" or "the producer states". The check flags the criterion unless it is restated as...
> Introspection 20261027-011812-anp5 (job 20261027-011733-vrsk); prevention it proposed: Add a reading-acceptance check. Each criterion must name its evidence as a command or Grep the verifier can run, or a file in the snapshot. A criterion whose evidence is the producer's report, summary or claims is refused. The reading and c...
> Introspection 20261027-013024-v1gj (job 20261027-012938-yckz); prevention it proposed: Add a reading-acceptance check. Every criterion must name an evidence source the verifier can observe: a Grep or command it runs, or a file in the snapshot. A criterion that cites the producer's report, or "the producer confirms", is reject...
> Introspection 20261027-023348-vrd5 (job 20261026-232402-6qme); prevention it proposed: Add two things. (1) A rule in the reader brief: every criterion must name evidence the verifier can observe itself, the producer's report is never evidence, and for code-move jobs the standard criteria are a Grep for old module paths outsid...
> Introspection 20261027-030613-vk78 (job 20261026-232423-hvyb); prevention it proposed: Two parts. (1) Add a rule to the reading brief that every criterion must name evidence the verifier can observe itself (the snapshot, commands it runs, its own tools), and that the producer's report is never evidence. (2) Add a code check o...
> Introspection 20261027-225528-ym4c (job 20261027-122909-37mg); prevention it proposed: Add a check at reading acceptance that flags any criterion whose evidence is the producer's report, its transcript or its self-described actions. The reading must either rewrite it as an outcome the verifier can observe (for example, the ge...
> Introspection 20261103-231352-6pnd (job 20261103-222737-zzsr); prevention it proposed: Make "fails without the fix" a check that Athena runs itself. For a bug-fix job, Athena would run the new or changed tests against the base tree's production code, with the job's test files overlaid, and record the failing output as a named...
> Introspection 20261104-081744-zgba (job 20261103-231422-8dzf); prevention it proposed: Add a code-checked evidence path for "fails without the fix" criteria. Athena would apply the reverse of the fix to a scratch copy, run the named test command, and record the output among the checks the verifier sees. The reading's criterio...
> Introspection 20261104-091400-5gmh (job 20261104-081830-js1p); prevention it proposed: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the comman...
> Introspection 20261105-170010-e62k (job 20261105-103606-svwx); prevention it proposed: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the v...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader was told which commands the verifier runs but not that it sees only the final tree, so criteria asked for base-vs-candidate runs or the producer's transcript. Fix it in the reader brief's "What the verifier can run" section (information first), and widen the producer-evidence phrase list by two nouns. Jobs 20261027-004517-gk9e, -011733-vrsk and -012938-yckz are addressed as examples of the class.

Assumptions:
- Code survey: read crates/athena-core/src/reading/surveys.rs (check_criteria_evidence, producer_evidence_phrase, REPORT_NOUNS, run at crates/athena-core/src/reading/checks.rs:538 during reading acceptance), crates/athena-context/src/briefs/reader.rs (verifier_can_run_section, criteria_judging_section) and crates/athena-core/src/reading/retry.rs (revert_for revert runs already exist as Athena's 'fails without the fix' evidence). No record or data file is added, so check_record_file and AnyRecord are not touched. The property is measured directly: the rendered reader brief text and check_criteria_evidence's result on sample criteria.
- Goal and audience: readers (the reader role) who write criteria, and the verifier who must judge them. Criteria that need a run on the base commit, a before/after comparison of command output, or the producer's transcript cannot be judged: the verifier sees only the candidate tree, diff and recorded checks (jobs 20261027-004517-gk9e, 20261027-011733-vrsk, 20261027-012938-yckz showed this).
- Previous jobs: 20261027-004517-gk9e (fail: report not visible) and 20261027-011733-vrsk (no change) are addressed by the cause, not by redoing their work; 20261027-012938-yckz already landed and passed. Their candidate files crates/athena-core/src/gh/mod.rs and gh/transport.rs are kept as landed and not touched here; CHANGELOG.md is a deliverable of this job.
- The existing phrase check already refuses 'the producer's report lists ...'; the new brief text gives the reader the missing list rather than adding a refusal, and the added nouns (transcript, log) are a small extension. Expected 1% of newly refused readings.
- No config or workflow file needs a change; consumers of the changed brief text are only the reader prompt assembly.

## Acceptance criteria

1. [content] The text rendered by verifier_can_run_section in crates/athena-context/src/briefs/reader.rs states that the verifier sees only the final candidate tree, cannot check out or run anything on the base commit, and that a before/after comparison or 'fails without the fix' criterion is met only by the diff or by a recorded Athena revert-run check, never by the verifier's own base run.
2. [content] crates/athena-core/src/reading/surveys.rs producer_evidence_phrase also refuses criteria citing the producer's transcript or log, and its doc comment names the root cause (no defined list of verifier-observable evidence).
3. [behavior] Run `cargo test --locked`: a new brief test fails on the base tree and passes now, asserting the reader brief contains the final-tree-only statement; a new reading test asserts check_criteria_evidence refuses a criterion citing 'the producer's transcript' and still accepts 'Grep over crates/ returns 0 hits'.
4. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; Athena's recorded checks on the final tree pass.
5. [content] CHANGELOG.md has a new entry naming the root cause; expected 1% of newly refused readings.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- crates/athena-core/src/gh/mod.rs and gh/transport.rs (the structure-pass G2 move, already landed by 20261027-012938-yckz)
- Building new revert-run or base-tree-run machinery in Athena's checks
- Edits to athena.toml, defaults.toml, workflows, docs/decisions.md or records/
- Changes to the reader's frame prompt file (2 tokens headroom)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the brief text a reader receives and a test that fails without the change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
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

- path: <state>/jobs/20261114-045652-r3mk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045652-r3mk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The text rendered by verifier_can_run_section in crates/athena-context/src/briefs/reader.rs states that the verifier sees only the final candidate tree, cannot check out or run anything on the base commit, and that a before/after comparison or 'fails without the fix' criterion is met only by the diff or by a recorded Athena revert-run check, never by the verifier's own base run.
crates/athena-core/src/reading/surveys.rs producer_evidence_phrase also refuses criteria citing the producer's transcript or log, and its doc comment names the root cause (no defined list of verifier-observable evidence).
Run `cargo test --locked`: a new brief test fails on the base tree and passes now, asserting the reader brief contains the final-tree-only statement; a new reading test asserts check_criteria_evidence refuses a criterion citing 'the producer's transcript' and still accepts 'Grep over crates/ returns 0 hits'.
Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; Athena's recorded checks on the final tree pass.
CHANGELOG.md has a new entry naming the root cause; expected 1% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
