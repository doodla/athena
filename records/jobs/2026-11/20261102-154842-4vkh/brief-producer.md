# Job 20261102-154842-4vkh: Speed up the slowest test binary using supplied CI timings

## Request (verbatim)

> try again on 20261102-001711-pgvq

## Reading (accepted by the user)

Retry of job 20261102-001711-pgvq. Read a timings file with the run id (supplied before the producer starts, or the job is held blocked), add a CHANGELOG.md entry of the ten slowest test binaries, and speed up only the slowest binary without losing coverage. The audience is maintainers reading CHANGELOG.md.

Assumptions:
- Retry of job 20261102-001711-pgvq. Its only candidate change was CHANGELOG.md, a failure note. That work is dropped: the base CHANGELOG.md is used and the note is not kept. Findings addressed: the missing-command-evidence major gets a criterion that names the commands. The no-work blocker and the failure-note major are addressed by forbidding failure notes in CHANGELOG.md and by using supplied timings.
- Per the introspection, the timings come from the orchestrator as a read-only input file with the newest green main run id, extracted from the six shard logs. If no such file is present, the producer must end with a blocked report and leave the repository unchanged. It must not write a CHANGELOG entry about the block. This is my assumption, because the producer cannot reach GitHub.
- Code survey: crates/athena-cli/tests/*.rs holds the test binaries, including lifecycle, live, steward, introspection and drain. Their names come from the file names under athena-cli. The binaries are built by the athena-cli package, so a single one runs with cargo test -p athena-cli --test <name>. CI sharding is in .github/workflows/ci.yml and is not changed. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property, a binary's wall time, is measured directly by cargo's 'finished in' line for that one binary.
- The slowest binary is unknown until the timings are read. The deliverables list the five previously proposed test files with low probability each. Only the one true slowest binary changes. If the slowest is another file, such as an athena-core test, that file is the deliverable instead.
- If the slowdown cannot be fixed without losing coverage, no code changes. CHANGELOG.md still gets the ten-slowest list and states that. This is the request's allowed no-code outcome.

## Acceptance criteria

1. [content] git diff shows CHANGELOG.md gains an entry that lists ten test binaries, slowest first, each with its 'finished in' seconds, and names the CI run id from the supplied timings file. The entry describes only a change to the project. It has no note about job failures, tooling, access or rerunning.
2. [content] The CHANGELOG entry gives the slowest binary's local time before and after the change, each measured by running only that binary. Alternatively it states that the cause cannot be fixed without losing coverage, and then the diff has no code change.
3. [content] git diff touches test code of only the single slowest binary, plus CHANGELOG.md. No other test binary is modified. xtask/shard-timings.txt and .github/workflows/ci.yml are unchanged.
4. [content] Every #[test] function in the changed binary before the change still exists afterwards, and git diff shows no removed test and no weakened assertion.
5. [behavior] Run cargo test -p athena-cli --test <slowest binary name>. It passes, and its 'finished in' time is below the 'before' time stated in CHANGELOG.md. The verifier cites this command and its output as the evidence.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/live.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Running the whole test suite locally
- Changing any test binary other than the single slowest
- Updating xtask/shard-timings.txt or CI sharding, or editing .github/workflows/ci.yml
- Removing or weakening test coverage
- Writing failure or blocked notes into CHANGELOG.md
- Keeping the earlier failure-note CHANGELOG.md change from job 20261102-001711-pgvq
- Changing the orchestrator to fetch CI data

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The behavior criterion runs the single changed binary and compares its time with the 'before' time.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check measured times and tests that run, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the single slowest binary is changed.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust test code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for outside maintainers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer finds the cause of the slow binary before it changes anything.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261102-001711-pgvq raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 5 is unmet: its evidence must cite commands that ran (D43) it cites no command
> blocker (CHANGELOG.md): The requested work was not done: no CI timings were read, no ten-slowest list was written and no binary was sped up. The diff adds only a note that `gh` was not available to the producer. The brief assumed the producer could read CI logs. If that tooling was missing, the producer should have reported the block in its output and left the repository unchanged. Instead it committed a status message as a changelog entry.
> major (CHANGELOG.md:12-16): The CHANGELOG entry records a job failure rather than a change to the project. Under '### Changed' it says 'Slowest-test-binary speedup not done ... Rerun the job with access to the CI logs.' This is an instruction to the job runner. To a maintainer reading the changelog it describes no change to the project. It also wrongly treats 'no CI access' as the request's allowed no-code outcome, which applies only when a fix would cost coverage.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 16 cases in 14 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena one disk lifecycle. Before the low-disk check and before any hold, the writer's regenerable build is removed once the writer has finished and its output is committed or recorded. The guard then measures free space after that removal. Where checks come from GitHub, the guard does not coun...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

24 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

git diff shows CHANGELOG.md gains an entry that lists ten test binaries, slowest first, each with its 'finished in' seconds, and names the CI run id from the supplied timings file. The entry describes only a change to the project. It has no note about job failures, tooling, access or rerunning.
The CHANGELOG entry gives the slowest binary's local time before and after the change, each measured by running only that binary. Alternatively it states that the cause cannot be fixed without losing coverage, and then the diff has no code change.
git diff touches test code of only the single slowest binary, plus CHANGELOG.md. No other test binary is modified. xtask/shard-timings.txt and .github/workflows/ci.yml are unchanged.
Every #[test] function in the changed binary before the change still exists afterwards, and git diff shows no removed test and no weakened assertion.
Run cargo test -p athena-cli --test <slowest binary name>. It passes, and its 'finished in' time is below the 'before' time stated in CHANGELOG.md. The verifier cites this command and its output as the evidence.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
