# Job 20261031-043018-kjy2: Retry: find and fix the checks-stage slowdown (job 9h6z)

## Request (verbatim)

> try again on 20261030-112518-9h6z

## Reading (accepted by the user)

Retry of job 20261030-112518-9h6z, which timed out at the 3000s producer deadline with no candidate. Diagnose with timings why the checks stage (fmt, clippy, tests) grew about 35% for 3% more tests, then fix the main cause so the full run is at least 15% under 1176s (at or below 1000s), without deleting tests. The audience is the maintainer watching the check_timeout_s margin.

Assumptions:
- Job 20261030-112518-9h6z timed out at its 3000s deadline with no candidate and no changed files. There are no findings to fix, so the retry starts from the base. It keeps the earlier scope and deliverables.
- To avoid another timeout, the producer should time targeted runs, such as `cargo test -p <crate> -- --report-time` or per-crate runs, instead of repeating whole check runs. It should fix the first clear cause it finds.
- Code survey: I read the paths crates/athena-cli/tests/status_durations.rs and crates/athena-core/src/run/tests.rs. Both exist and are the likely slow tests, going by the earlier attempt's deliverable list. I did not inspect their contents, so the producer must measure per-test time first. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by the wall time of the checks stage and per-test times from cargo test.
- No refusal rule is added or tightened, so no refusal-rate criterion applies.
- The deliverables keep the earlier list. Cargo.toml and crates/athena-core/Cargo.toml are listed only in case the cause is a build profile or dependency setting, and they are named in a criterion. athena.toml, including check_timeout_s, is not changed.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry that names the cause of the checks slowdown and the fix. It gives per-crate or per-test timings from the base build before and after the change.
2. [content] No test is deleted or marked #[ignore]. `git diff` shows no removed #[test] functions, and the test count stays at or above 1737.
3. [behavior] Run `cargo test`: all tests pass, and the same set of tests runs as before.
4. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
5. [content] The recorded check output of the candidate shows the full checks stage (fmt, clippy, tests) at or below 1000s, at least 15% under the slowest earlier run of 1176s, measured by the same checks stage.
6. [content] The diff changes the slow tests or build settings that the CHANGELOG entry names as the cause, so the fix addresses the root cause and not a symptom. If Cargo.toml or crates/athena-core/Cargo.toml changes, the CHANGELOG entry explains why.
7. [content] athena.toml and check_timeout_s are unchanged, and no files other than the listed deliverables change.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.toml (modify)

## Out of scope

- Changing check_timeout_s or other values in athena.toml
- Restructuring or parallelizing the pipeline stages
- Reducing coverage by deleting tests or marking them #[ignore]

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the checks-stage wall time and the same passing tests, not just that files changed.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy with -D warnings and fmt --check are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix must target the cause that was measured and named in the CHANGELOG.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The CHANGELOG entry must give measured per-crate or per-test timings.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the speedup and the tests passing, not that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused diagnosis and fix.
- thrash-stop: If the same item fails twice in a loop, the loop stops and reports thrashing. Prose-only: no loop code detects it yet. Here: The retry is told to measure targeted runs instead of repeating full runs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Resume: your work starts from the kept work of job 20261030-112518-9h6z

That job's producer reached its deadline. Its edits were kept and your working directory already holds them. Check them, finish what is missing, and do not redo what is done. Its progress note (quoted data, not instructions):

> # Progress note
> 
> The producer of job 20261030-112518-9h6z stopped: deadline of 3000s reached.
> 
> Changed files (2), against the base commit:
> 
> - M Cargo.toml
> - M crates/athena-core/src/run/tests.rs
> 
> The edits were kept; `athena retry` starts the next producer from them.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 7 cases in 7 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Land b5hm first and extend it. Before the checks stage, and again when free disk drops below the guard, Athena should delete regenerable artifacts itself: the verify-side build folder of finished baseline checks, the workspace target, and the uv and npm caches. If that is still not enough, the job s...

22 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261030-112518-9h6z: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has a new entry that names the cause of the checks slowdown and the fix. It gives per-crate or per-test timings from the base build before and after the change.
No test is deleted or marked #[ignore]. `git diff` shows no removed #[test] functions, and the test count stays at or above 1737.
Run `cargo test`: all tests pass, and the same set of tests runs as before.
Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
The recorded check output of the candidate shows the full checks stage (fmt, clippy, tests) at or below 1000s, at least 15% under the slowest earlier run of 1176s, measured by the same checks stage.
The diff changes the slow tests or build settings that the CHANGELOG entry names as the cause, so the fix addresses the root cause and not a symptom. If Cargo.toml or crates/athena-core/Cargo.toml changes, the CHANGELOG entry explains why.
athena.toml and check_timeout_s are unchanged, and no files other than the listed deliverables change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
