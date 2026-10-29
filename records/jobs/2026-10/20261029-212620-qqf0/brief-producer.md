# Job 20261029-212620-qqf0: Measure build cache and disk (A1) and accept D145 in decisions.md

## Request (verbatim)

> try again on 20261029-142844-ynv3

## Reading (accepted by the user)

Retry of job 20261029-142844-ynv3 for Sri and the verifier: write a real '### A1 measurements' entry in docs/decisions.md (sccache stats, D84 cause, hits across two worktrees, T and S with units and commands), keep D145 marked accepted by the brain under Sri's delegation (2026-10-29), and a CHANGELOG entry. Where a tool cannot run, the entry gives the tried command and the specific error in that field's place, never a placeholder.

Assumptions:
- Prior job 20261029-142844-ynv3 failed because its A1 entry held only 'not measured' lines (major), the CHANGELOG presented an unperformed measurement as 'Added' (minor), and the design's S estimate of about 0.2 GB was missing (nit). This retry addresses each: the producer first tries `which sccache`, `sccache --show-stats`, `df -h`, `git worktree add` and `du`, and records actual outputs; the CHANGELOG wording states exactly what was measured; the entry quotes the design's estimates (T about 6 GB, S about 0.2 GB, docs/design/async-drain.md lines 103-104) next to any measured or unmeasured value.
- The prior candidate's changes to CHANGELOG.md, docs/decisions.md and docs/design/async-drain.md (D145 accepted heading and status line) are kept as a starting point and reworked, not dropped; the D145 acceptance part had no findings.
- Where the environment truly lacks sccache, disk or git worktree support, the field holds the exact command tried and the error text, plus the design's estimate labeled as an estimate. This is the only allowed alternative to a number.
- The verifier cannot run sccache or du, so measurement criteria are checked by reading the entry in the diff; cargo checks are the only runnable behavior.
- No Rust code changes, so no code survey is needed; crates/athena-core/src/disk.rs was a listed deliverable in the earlier job but its candidate did not change it, and it is excluded here.

## Acceptance criteria

1. [content] docs/decisions.md has a '### A1 measurements' section that gives the `sccache --show-stats` output from inside a job's build, or the exact command tried and the specific error text explaining why it could not run.
2. [content] The section states the cause found for D84's 0 sccache hits, or says explicitly the cause was not found and lists what was tried.
3. [content] The section gives cache hit counts from two per-job worktrees with different target directories and the commands used, or the command tried and the error text for each that could not run.
4. [content] The section gives T (one cold target/) and S (worktree source size) as numbers with units and the du or equivalent command used. If one cannot be measured, the command tried and error appear, plus the design's estimate (T about 6 GB, S about 0.2 GB) labeled as an estimate. No field is empty, 'not measured' alone, or a placeholder.
5. [content] The D145 heading in docs/decisions.md says accepted, decided by the brain under Sri's delegation, 2026-10-29, and no longer says proposed; the status line in docs/design/async-drain.md agrees.
6. [content] CHANGELOG.md has an entry naming the A1 measurements and the D145 acceptance, worded to say only what was actually measured.
7. [content] git diff shows no change to athena.toml, defaults.toml, any file under .github/workflows/, or any file under crates/.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, so existing drain tests are unchanged and max_in_flight = 1 drain behavior is as today.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Any athena.toml change or re-enabling sccache
- Implementing read-ahead, per-job worktrees or max_in_flight
- Changing drain behavior or existing drain tests
- crates/athena-core/src/disk.rs and any other file under crates/ (listed in the earlier job but never changed by its candidate; dropped)
- Inventing measurements; an honest failure reason is allowed only where the tool truly cannot run

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require real values or specific failure reasons in each field, not a plan.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every number cites its command or is labeled an estimate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Cargo fmt, clippy and test are run by the verifier.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs only, three files.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Entry contains no secrets or personal paths beyond repo-relative ones.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261029-142844-ynv3

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> docs/decisions.md contains a measurements entry for A1 that states the sccache --show-stats output or the reason it could not run, the cause found for D84's 0 hits (or an explicit statement that it was not found), the hit counts across two worktrees with different target directories, and numeric values for T (one cold target/) and S (worktree source size) with units and the commands used.: I read the diff. The new '### A1 measurements' section gives the reason sccache --show-stats did not run (the job's command restriction) and says plainly that the cause of D84's 0 hits was not found. It gives no hit counts: 'Hits: unknown; misses: unknown'. It gives no numbers for T or S, only 'not measured' plus the commands a later job should run. The only figure is the design's T estimate of 6 GB. S has no value at all, so the design's 0.2 GB estimate is not carried over. The criterion asks for numeric values for T and S with units, and those are missing.

## Findings you must address

The verifier of job 20261029-142844-ynv3 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/decisions.md, '### A1 measurements' section): The measurement job measured nothing: no hit counts, no T, no S, and the D84 cause was not found. The request's core deliverable is 'Measure ... T and S ... Record the numbers'. In the candidate, every measured item says 'not measured' or 'not obtained', and S has neither a number nor an estimate. Criterion 1 asks for numeric T and S with units. Recording an honest 'could not run' is better than inventing numbers, but it does not deliver A1. Simpler fix: rerun this job with a shell that can run sccache, du and git worktree, or have Sri run the listed commands and record the output.

All of the verification's findings:

> major (docs/decisions.md, '### A1 measurements' section): The measurement job measured nothing: no hit counts, no T, no S, and the D84 cause was not found. The request's core deliverable is 'Measure ... T and S ... Record the numbers'. In the candidate, every measured item says 'not measured' or 'not obtained', and S has neither a number nor an estimate. Criterion 1 asks for numeric T and S with units. Recording an honest 'could not run' is better than inventing numbers, but it does not deliver A1. Simpler fix: rerun this job with a shell that can run sccache, du and git worktree, or have Sri run the listed commands and record the output.
> minor (CHANGELOG.md:12): The CHANGELOG presents a measurement that never happened as an 'Added' measurements entry. The CHANGELOG bullet starts 'A1 measurements entry in docs/decisions.md'. The text that follows does say nothing was measured. A reader skimming the heading could still think A1 is done.
> nit (docs/decisions.md, A1 measurements, S line): The entry does not give the design's S estimate. docs/design/async-drain.md:103-104 gives 'S about 0.2 GB' as an estimate. The new entry repeats only the T estimate of 6 GB.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

20 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261029-142844-ynv3: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/decisions.md has a '### A1 measurements' section that gives the `sccache --show-stats` output from inside a job's build, or the exact command tried and the specific error text explaining why it could not run.
The section states the cause found for D84's 0 sccache hits, or says explicitly the cause was not found and lists what was tried.
The section gives cache hit counts from two per-job worktrees with different target directories and the commands used, or the command tried and the error text for each that could not run.
The section gives T (one cold target/) and S (worktree source size) as numbers with units and the du or equivalent command used. If one cannot be measured, the command tried and error appear, plus the design's estimate (T about 6 GB, S about 0.2 GB) labeled as an estimate. No field is empty, 'not measured' alone, or a placeholder.
The D145 heading in docs/decisions.md says accepted, decided by the brain under Sri's delegation, 2026-10-29, and no longer says proposed; the status line in docs/design/async-drain.md agrees.
CHANGELOG.md has an entry naming the A1 measurements and the D145 acceptance, worded to say only what was actually measured.
git diff shows no change to athena.toml, defaults.toml, any file under .github/workflows/, or any file under crates/.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, so existing drain tests are unchanged and max_in_flight = 1 drain behavior is as today.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
