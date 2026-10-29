# Job 20261029-135024-p78m: Design async drain without assuming sccache; add D145 proposed

## Request (verbatim)

> try again on 20261029-073414-yg89

## Reading (accepted by the user)

Retry of job 20261029-073414-yg89: write docs/design/async-drain.md (five points, measured 42-minute baseline, target, disk budget, conflict handling, ordered build jobs) for Sri and the verifier, with the sccache premise fixed per D84, plus proposed decision D145 and a CHANGELOG entry.

Assumptions:
- Job 20261029-073414-yg89 findings addressed: (major) sccache treated as active although D84 removed it: the retry states the cache is off, recomputes the disk budget and recheck estimate without it, marks cache-dependent numbers conditional, and adds an early build job to find the 0-hit cause and measure hits across per-job worktrees; re-enabling is a separate costed option. (minor) job 10, not job 7, raises max_in_flight; (minor) max_check_slots added to the [drain.pipeline] block; (minor) the target is computed per landed job, with failed jobs' check-slot use worked in using 35 of 61 landed.
- The previous candidate's three changed files (docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md) are kept as deliverables but rewritten from base since attempts are discarded; the retry starts from base.
- docs/decision-links.toml was listed earlier; it is kept as a possible deliverable only if a check requires a link entry for D145. Governance files athena.toml, defaults.toml and workflows are untouched.
- The doc includes a 'Premises checked' section listing each dependency (build cache, drain, producer, verifier, disk guard, config limits) with the repo path or decision showing its current state, produced by reading the repo and docs/decisions.md before writing.
- Next free decision id is D145 (after D144); the entry uses the Decision/Alternatives/Tradeoff/Reversal cost/Provenance format.

## Acceptance criteria

1. [content] docs/design/async-drain.md has a section for each of the five points: read-ahead and hard-job planning; early producer start in own worktree bounded by disk; one ordered landing queue with rebase-or-return-to-producer; CI waits and self-rebuild as background awaits; ordering of records commits with several jobs in flight.
2. [content] Grep of docs/design/async-drain.md for sccache finds no statement treating it as active; each mention says D84 removed it or labels it a proposed option, and a 'Premises checked' section lists each dependency (build cache, drain, producer, verifier, disk guard, config limits) with a repo path or decision id.
3. [content] The design states the measured baseline (median 42 minutes; producer 15, checks 13 to 15, landing 6 to 8, verifier 2.3, reading and planning up to 5; 35 of 61 landed over 2026-10-27 to 29) and a target in minutes per landed job with stage-by-stage arithmetic that includes failed jobs' use of the check slot; unmeasured or cache-dependent figures are labeled estimate or conditional.
4. [content] The disk budget per in-flight job is computed without a shared cache, with the implied in-flight limit; the [drain.pipeline] config block includes max_in_flight, max_check_slots, read_ahead and reserve_disk_gb; the text names job 10 as the one that raises max_in_flight.
5. [content] Failure and conflict handling covers rebase conflict returning to the producer, failed CI, failed self-rebuild, crash mid-flight and thrash stop; alternatives are listed; ordered build jobs each sized for one producer run state dependencies, with an early job measuring cache hits across worktrees.
6. [content] docs/decisions.md has D145 with heading marked (proposed), in the existing format, linking docs/design/async-drain.md; CHANGELOG.md has an Unreleased entry naming the design doc and the proposed decision.
7. [behavior] Run git diff --stat: only docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md (and docs/decision-links.toml if needed) change; nothing under crates/ or xtask/. Run cargo test and cargo clippy and confirm they pass.

## Expected deliverables

- docs/design/async-drain.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Any source code change in crates/ or xtask/
- Implementing the async drain; only the build job list is written
- Changes to athena.toml, defaults.toml, or workflows (including re-enabling sccache)
- Changes to records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Baseline numbers cited from the request; unmeasured figures labeled estimate or conditional.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc lists alternatives and a premise check before build jobs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the per-landed-job minutes target and conflict behavior, not mere file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for outside readers.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy confirmed to still pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-073414-yg89 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/design/async-drain.md section 4 'Shared build cache', section 5 step 2, section 9 table, section 12): The design treats sccache as an active shared build cache, but D84 removed it (0 hits, per-job paths suspected) and nothing re-enables it. The disk budget, the recheck estimate and the cheap-producer argument rest on that cache. Section 4: 'D79 and D84 put sccache in front of rustc ... so the second job's crates compile mostly from cache.' docs/decisions.md D84 (line 1846): 'The sccache wiring is removed from athena.toml ... Re-adding the cache needs a measured cause first', and it suspects 'a per-job CARGO_TARGET_DIR and per-job snapshot paths' as the cause of the 0 hits. A search of every *.toml for 'sccache' finds nothing. The design adds per-job worktrees, which make those path differences larger. None of the ten build jobs re-enables sccache or measures its hits. A more correct design would state that the cache is off, add an early job that finds the 0-hit cause and measures hits across per-job worktrees, and redo the disk budget and recheck estimates without the cache until that job is done.

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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/design/async-drain.md has a section for each of the five points: read-ahead and hard-job planning; early producer start in own worktree bounded by disk; one ordered landing queue with rebase-or-return-to-producer; CI waits and self-rebuild as background awaits; ordering of records commits with several jobs in flight.
Grep of docs/design/async-drain.md for sccache finds no statement treating it as active; each mention says D84 removed it or labels it a proposed option, and a 'Premises checked' section lists each dependency (build cache, drain, producer, verifier, disk guard, config limits) with a repo path or decision id.
The design states the measured baseline (median 42 minutes; producer 15, checks 13 to 15, landing 6 to 8, verifier 2.3, reading and planning up to 5; 35 of 61 landed over 2026-10-27 to 29) and a target in minutes per landed job with stage-by-stage arithmetic that includes failed jobs' use of the check slot; unmeasured or cache-dependent figures are labeled estimate or conditional.
The disk budget per in-flight job is computed without a shared cache, with the implied in-flight limit; the [drain.pipeline] config block includes max_in_flight, max_check_slots, read_ahead and reserve_disk_gb; the text names job 10 as the one that raises max_in_flight.
Failure and conflict handling covers rebase conflict returning to the producer, failed CI, failed self-rebuild, crash mid-flight and thrash stop; alternatives are listed; ordered build jobs each sized for one producer run state dependencies, with an early job measuring cache hits across worktrees.
docs/decisions.md has D145 with heading marked (proposed), in the existing format, linking docs/design/async-drain.md; CHANGELOG.md has an Unreleased entry naming the design doc and the proposed decision.
Run git diff --stat: only docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md (and docs/decision-links.toml if needed) change; nothing under crates/ or xtask/. Run cargo test and cargo clippy and confirm they pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
