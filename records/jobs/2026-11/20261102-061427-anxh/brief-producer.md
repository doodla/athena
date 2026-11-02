# Job 20261102-061427-anxh: Free writer build before the checks disk gate; gate only local runs

## Request (verbatim)

> In the checks stage, remove the writer's build folder before the free-disk check, not after it. Today the focus-and-caps job was held for low disk (9.0 GB free, 10 GB needed) while its own 9.8 GB writer build was still on disk. When the checks will come from GitHub's CI run, the stage needs no local build space at all, so the free-disk requirement should not apply until a local fallback run is actually needed.
> - A test: a job whose writer build alone takes free disk below the requirement is not held; the build is removed first and the checks go ahead.
> - A test: with a GitHub origin and a green CI run, the checks stage does not hold for low disk; the requirement applies only before a local check run.
> - CHANGELOG.md gets an entry.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

In check_and_verify, the free-disk gate currently runs before free_producer_build, so the job's own writer build can hold it. The gate moves after that removal and applies only just before a local check run, so a green or red CI result never triggers it. The audience is the Athena operator, who wants fewer false disk holds.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (check_and_verify: disk_gate at ~1413, free_producer_build at ~1463, with_ci_source and run_routed_with_record call at ~1500), crates/athena-core/src/routing.rs (run_routed_with_record, Fallback::run, run_local, Exec trait), crates/athena-core/src/disk.rs (disk_gate, LowDisk), crates/athena-core/src/run/verification.rs (verifier gate), and crates/athena-cli/tests/drain.rs (simulated_free harness). No records or data files are added, so the records allow-list is not touched.
- The CI check results are read inside run_routed_with_record, and the local fallback is run_local through Exec::exec. So the gate goes at the start of the local run (SnapExec or a wrapper around Exec, or Fallback::run), and a low result is returned so check_and_verify holds the job via hold_for_disk as it does today. How that is threaded through is the producer's choice.
- A resumed job with recorded_checks still skips the gate and the checks, as before.
- This relaxes a refusal rule, so no newly refused readings are expected: expected 0% of newly refused readings.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (using with_free_space and simulated_free) has a writer build whose size takes simulated free disk below min_free_gb. The job is not held for disk, the build directory is removed first, and the checks run to a verdict. Run `cargo test -p athena-cli --test drain` and check that this test passes.
2. [behavior] A new test with a GitHub origin and a green CI run for the candidate, and free disk below min_free_gb, shows that the checks stage does not hold for low disk and reuses the CI result (the 'ci   checks reused' line). A companion case with no usable CI result shows the disk gate still holds the job before the local check run. Run `cargo test -p athena-cli --test drain` and check that both pass.
3. [content] In crates/athena-core/src/run/mod.rs check_and_verify, the up-front disk_gate call before the checks is gone. free_producer_build runs before any disk_gate in the checks stage, and the gate is called only on the local-run path (the fallback run). Check by reading the diff.
4. [content] CHANGELOG.md has a new entry saying that the writer build is removed before the checks free-disk check and that the requirement applies only before a local check run.
5. [content] The existing verifier-stage gate (verifier_environment_problem) and the dispatch and producer gates are unchanged, and the existing disk-hold tests still pass.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)

## Out of scope

- Changing the dispatch, producer or verifier-stage disk gates
- Changing the min_free_gb default or config
- Changing CI-result resolution or routing rules

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests; the repo-wide fmt, clippy and test run is deliberately not a separate criterion, per the request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the hold or no-hold behavior through tests that run the drain.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The gate is reordered and made conditional on a local run, rather than raising the disk limit.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and comments are written for an outside reader.
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

A new test in crates/athena-cli/tests/drain.rs (using with_free_space and simulated_free) has a writer build whose size takes simulated free disk below min_free_gb. The job is not held for disk, the build directory is removed first, and the checks run to a verdict. Run `cargo test -p athena-cli --test drain` and check that this test passes.
A new test with a GitHub origin and a green CI run for the candidate, and free disk below min_free_gb, shows that the checks stage does not hold for low disk and reuses the CI result (the 'ci   checks reused' line). A companion case with no usable CI result shows the disk gate still holds the job before the local check run. Run `cargo test -p athena-cli --test drain` and check that both pass.
In crates/athena-core/src/run/mod.rs check_and_verify, the up-front disk_gate call before the checks is gone. free_producer_build runs before any disk_gate in the checks stage, and the gate is called only on the local-run path (the fallback run). Check by reading the diff.
CHANGELOG.md has a new entry saying that the writer build is removed before the checks free-disk check and that the requirement applies only before a local check run.
The existing verifier-stage gate (verifier_environment_problem) and the dispatch and producer gates are unchanged, and the existing disk-hold tests still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
