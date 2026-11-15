# Job 20261114-045711-1e6d: Tell the reader to require end-to-end, can-fail behavior criteria

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request described a command ("works the queue within a budget") without stating the end-to-end e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criteria-check-presence-not-behavior (20261014-184842-whyd)
> Cause: The request described a command ("works the queue within a budget") without stating the end-to-end effect: queued items must be landed, and the rating must be of the page after those changes. Nothing required the build to prove that the rat...
> Seen 8 time(s) in 8 jobs. The introspections that record it:
> Introspection 20261015-070659-2kqw (job 20261015-064153-wekz); prevention it proposed: Require that a loop-style command's acceptance criteria state the observable outcome, not the presence of parts. For drain, a test must run one pass with a stubbed queue item. It must assert that `athena land --pr` is invoked for the item. ...
> Introspection 20261015-081905-xw6c (job 20261015-063409-hekk); prevention it proposed: Require any test of a protection or denial to prove it can fail. The test must show the write succeeds on an unlocked snapshot under the same user and environment, then show it is refused once locked. It must fail loudly, not pass silently,...
> Introspection 20261015-160328-7077 (job 20261015-152305-6h8k); prevention it proposed: Add a code check, and a matching test, that the source check covers every content file of the page. The check should enumerate the page's text-bearing inputs (claims.json, intro.json and any others) and fail if any of them is not compared w...
> Introspection 20261015-163343-435x (job 20261015-160432-g1tq); prevention it proposed: A mutation-style check on negative tests that a criterion names. The check would run the named test against the code with the guard disabled or the invariant inverted, and require the test to fail there. It would also require the test to as...
> Introspection 20261101-222454-36yp (job 20261031-031939-wz33); prevention it proposed: Require every behaviour criterion to be demonstrated by a test that exercises the real production path end to end, not a default or no-op hook. Here, a landing through the drain must produce a record commit on the records branch. Split mult...
> Introspection 20261103-074940-hcey (job 20261103-020319-ef9q); prevention it proposed: For any job that renders or reports from repository records, require two things at plan and criteria stage. First, each criterion must be demonstrated by running the real command on the real (or a copied snapshot of the) repository and asse...
> Introspection 20261103-235051-6bj9 (job 20261103-201548-qt18); prevention it proposed: Add a test, required by the reading for any rendered owner-facing text, that builds cards from at least two real or realistic Steward proposals with different content. It must assert three things. The question line is not the title with a f...
> Introspection 20261112-023747-fs2m (job 20261030-001211-6mzd); prevention it proposed: Require every criterion for a new mechanism to name its consumer. The criterion would say which non-test code reads the new data and what observable behavior changes. Its demonstration would run that code path, for example resolving the mod...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause of criteria-check-presence-not-behavior (jobs 20261015-064153-wekz, 20261015-063409-hekk, 20261015-152305-6h8k and others): the reader's brief never told it that a criterion for a loop, protection or rendered output must name the real end-to-end run, its observable effect, and proof it can fail, so criteria checked parts. Add that as a prose-only reading rule in the reader brief, not a refusal rule, with a test that the brief carries it.

Assumptions:
- Goal and audience: the reader agent that writes acceptance criteria, and the verifier and owners who judge them; the goal is that criteria check the requested behavior, not the presence of parts.
- Code survey: read crates/athena-context/src/briefs/reader.rs (READING_RULES array of 13 lines, rendered under '# Reading rules'), crates/athena-core/src/reading/checks.rs and reading/surveys.rs (check_criteria_evidence, the refusal checks that stay unchanged) and crates/athena-context/src/briefs.rs (brief tests). The entry point is the reader brief builder; no record or data file is added, so no AnyRecord or check_record_file round trip is touched.
- Existing code: extends crates/athena-context/src/briefs/reader.rs READING_RULES, following the existing prose-only 'Reused field' rule and its Root cause comment; docs/decisions.md.
- Information is preferred over refusal per the request; a refusal rule cannot judge whether a criterion exercises the end-to-end path, so the rule is marked prose only.
- The reader prompt file has 2 tokens left, so the change goes in the code-held READING_RULES, not the prompt files; if a test counts the brief's tokens, the rule is kept short enough to fit, and a separate budget raise is out of scope.
- Kept/dropped files of the earlier candidates: crates/athena-cli/src/drain.rs, tests/drain.rs, display.rs, main.rs, lib.rs, crates/athena-core/src/{steward,pin,run,verify,run_pin_tests,lib}.rs, crates/athena-git/src/snapshot.rs, README.md and athena.toml are all dropped (not changed here); CHANGELOG.md and docs/decisions.md are kept as deliverables.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/reader.rs READING_RULES gains one new line, marked `(prose only)`, saying: for a loop-style command (drain, queue), protection or denial, or rendered or reported output, a behavior criterion names the real production path run end to end, the observable effect (for example an item landed, a record written, a page changed) and a demonstration that the check fails when the effect or guard is absent; the array length is updated to match. Preceded by a Root cause comment citing jobs 20261015-064153-wekz, 20261015-063409-hekk and 20261015-152305-6h8k.
2. [behavior] A new unit test in crates/athena-context/src/briefs.rs (or reader.rs tests) builds the reader brief and asserts it contains the new rule text; it fails on the code before the change. Verifier runs `cargo test -p athena-context`.
3. [behavior] No refusal check is added: `cargo test -p athena-core` still passes, including tests/refusal_corpus.rs unchanged, so expected 0% of newly refused readings; the rule is information only because giving the reader the information can prevent the failure.
4. [content] A new appended entry in docs/decisions.md names the root cause (the reader was not told that criteria must check the end-to-end effect and that a check can fail), the alternatives (a refusal rule, rejected), and has a Provenance bullet; CHANGELOG.md records the change. No existing entry is edited.
5. [behavior] Athena's recorded fmt, clippy and test checks pass on the final tree (`cargo fmt --check`, `cargo clippy`, `cargo test`).
6. [content] Job 20261015-064153-wekz findings (drain not landing via `athena land --pr`, unbudgeted retry read) are not re-implemented: that candidate's files are dropped, and the class is addressed by the reader rule. Same for 20261015-063409-hekk (root-run write tests that check nothing) and 20261015-152305-6h8k (source check missing intro.json); the new rule names those patterns.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)

## Out of scope

- Adding any refusal check to athena-core reading checks
- Changing prompt files or token budgets
- Changing drain, verifier snapshot locking or source-check code from the earlier jobs
- Changes to athena.toml, defaults.toml or CI workflows
- Editing existing entries of docs/decisions.md or records/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the brief carries the rule and tests fail before the change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the reader brief where criteria originate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule line, one test, docs.
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

- path: <state>/jobs/20261114-045711-1e6d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045711-1e6d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs/reader.rs READING_RULES gains one new line, marked `(prose only)`, saying: for a loop-style command (drain, queue), protection or denial, or rendered or reported output, a behavior criterion names the real production path run end to end, the observable effect (for example an item landed, a record written, a page changed) and a demonstration that the check fails when the effect or guard is absent; the array length is updated to match. Preceded by a Root cause comment citing jobs 20261015-064153-wekz, 20261015-063409-hekk and 20261015-152305-6h8k.
A new unit test in crates/athena-context/src/briefs.rs (or reader.rs tests) builds the reader brief and asserts it contains the new rule text; it fails on the code before the change. Verifier runs `cargo test -p athena-context`.
No refusal check is added: `cargo test -p athena-core` still passes, including tests/refusal_corpus.rs unchanged, so expected 0% of newly refused readings; the rule is information only because giving the reader the information can prevent the failure.
A new appended entry in docs/decisions.md names the root cause (the reader was not told that criteria must check the end-to-end effect and that a check can fail), the alternatives (a refusal rule, rejected), and has a Provenance bullet; CHANGELOG.md records the change. No existing entry is edited.
Athena's recorded fmt, clippy and test checks pass on the final tree (`cargo fmt --check`, `cargo clippy`, `cargo test`).
Job 20261015-064153-wekz findings (drain not landing via `athena land --pr`, unbudgeted retry read) are not re-implemented: that candidate's files are dropped, and the class is addressed by the reader rule. Same for 20261015-063409-hekk (root-run write tests that check nothing) and 20261015-152305-6h8k (source check missing intro.json); the new rule names those patterns.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
