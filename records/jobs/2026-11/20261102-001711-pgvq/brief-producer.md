# Job 20261102-001711-pgvq: Speed up the slowest test binary using newest green CI timings

## Request (verbatim)

> Find the slowest tests from GitHub's CI timings and speed up the worst one.
> - Read per-test or per-binary times from the newest green CI run on main (the six test shards' logs list each binary's 'finished in' time); do not run the whole suite locally, which takes longer than the writer's time limit.
> - CHANGELOG.md lists the ten slowest test binaries with their times and the run they came from.
> - Change only the single slowest binary's tests, keeping what they cover, and show its time before and after with a local run of that one binary; if the cause cannot be fixed without losing coverage, say so and change no code.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

Read each test binary's "finished in" time from the six shard logs of the newest green main CI run, record the ten slowest in CHANGELOG.md with the run id, then speed up only the single slowest binary's tests without losing coverage, showing its local before/after time. If the cause cannot be fixed without losing coverage, change no code and say so. Audience: maintainers reading CHANGELOG.md.

Assumptions:
- Code survey: I read crates/athena-cli/tests/ (26 integration binaries, e.g. lifecycle.rs, live.rs, steward.rs), crates/athena-core/tests and crates/athena-data/tests listings, plus xtask/shard-timings.txt; that file holds only estimates (athena-cli --lib 40s and --test lifecycle 40s are its largest), so it is not the real CI data and the real slowest binary must come from CI logs.
- The slowest binary is unknown to me; it is likely one of crates/athena-cli/tests/*.rs or the athena-cli lib unit tests, decided by the CI logs.
- The producer can read CI logs via GitHub tooling (e.g. gh); the run id is recorded in CHANGELOG.md. Existing CHANGELOG.md has no ten-slowest list yet, so a new entry is added.
- Local before/after times come from running only that one binary (cargo test -p <pkg> --test <name>), reported in the CHANGELOG entry; the whole suite is not run.
- No governance files change; xtask/shard-timings.txt is not updated, since it is an estimates file maintained by a separate command.
- Alternative outcome: if no coverage-preserving speedup exists, only CHANGELOG.md changes and it says so.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry listing ten test binaries, slowest first, each with its 'finished in' seconds, and naming the GitHub run id (newest green main run) they came from.
2. [content] The CHANGELOG entry states the slowest binary's local time before and after the change, from running only that binary, or states that the cause cannot be fixed without losing coverage and no code changed.
3. [content] The diff touches test code of only the single slowest binary (plus CHANGELOG.md); no other test binary is modified.
4. [content] Every test function present in the changed binary before the change still exists afterwards with equivalent assertions (git diff shows no removed #[test] without a replacement and no weakened assertions).
5. [behavior] Run cargo test on only the changed binary (cargo test -p <package> --test <name>); it passes, and its reported 'finished in' time is below the 'before' time stated in CHANGELOG.md.

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
- Updating xtask/shard-timings.txt or CI sharding
- Removing or weakening test coverage

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the slowest binary's measured time falls, not just that edits were made.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Times are cited from a named CI run id or a local run, not estimated.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the single slowest binary is changed.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier re-runs the one changed binary.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion checks the binary's timing and passing, not file existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry is written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

CHANGELOG.md has a new entry listing ten test binaries, slowest first, each with its 'finished in' seconds, and naming the GitHub run id (newest green main run) they came from.
The CHANGELOG entry states the slowest binary's local time before and after the change, from running only that binary, or states that the cause cannot be fixed without losing coverage and no code changed.
The diff touches test code of only the single slowest binary (plus CHANGELOG.md); no other test binary is modified.
Every test function present in the changed binary before the change still exists afterwards with equivalent assertions (git diff shows no removed #[test] without a replacement and no weakened assertions).
Run cargo test on only the changed binary (cargo test -p <package> --test <name>); it passes, and its reported 'finished in' time is below the 'before' time stated in CHANGELOG.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
