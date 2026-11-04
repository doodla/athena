# Job 20261029-160024-80qj: Design warm baseline build to cut checks-stage time

## Request (verbatim)

> Priority: high
> 
> Throughput design: The checks stage (fmt, clippy, test) is the biggest cost: 14.1 of the 25.4-minute median per landed
> 
> Athena's throughput review 20261029-155931-wfkr named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The checks stage (fmt, clippy, test) is the biggest cost: 14.1 of the 25.4-minute median per landed job, more than twice the producer's 6.8 minutes. The checks run in the verifier's fresh snapshot with an empty per-job, per-side CARGO_TARGE...
> Stage: checks
> Proposed change: Change how the pipeline builds code. Today every check run starts from an empty build directory. Instead, the pipeline keeps one warm baseline build of main, keyed by a hash of rust-toolchain.toml and Cargo.lock, and refreshes it in the bac...
> Predicted saving: 8.0 minutes per landed job; the median was 25.4 minutes per landed job when the review ran.
> Why: I based this on reading THROUGHPUT.md and the design documents; I ran nothing and timed nothing. **What was measured (THROUGHPUT.md):** the median per landed job over the last 20 is 25.4 min. Checks take 14.1, producer 6.8, verifier 2.5, re...
> 
> A change that implements this design must cite throughput review 20261029-155931-wfkr in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for a warm baseline build of main (keyed by hash of rust-toolchain.toml and Cargo.lock, refreshed in the background) so checks stop starting from an empty target dir. It cites review 20261029-155931-wfkr, states the 8.0-minute predicted saving and how the next review measures it. rust-toolchain.toml is not changed.

Assumptions:
- Goal: a design note for the warm-baseline build change; audience is Athena's maintainer and the next throughput review, which scores the predicted saving.
- The job is docs-only: it designs the change but does not implement it; implementation would be a later job citing review 20261029-155931-wfkr.
- rust-toolchain.toml is named in the request only as an input to the cache key; this job does not change it.
- Existing code: docs/ design documents and THROUGHPUT.md are read for context; the note is new and creates no code.

## Acceptance criteria

1. [content] A new design note exists under docs/design/ (e.g. docs/design/warm-baseline-build.md) that describes the warm baseline build of main keyed by a hash of rust-toolchain.toml and Cargo.lock, its background refresh, and how checks (fmt, clippy, test) start from it instead of an empty CARGO_TARGET_DIR; checkable by reading the file.
2. [content] The note cites throughput review 20261029-155931-wfkr and the measured figures from THROUGHPUT.md (14.1 of 25.4 minutes median per landed job in checks; producer 6.8), labeling any other number as an estimate.
3. [content] The note states the predicted saving of 8.0 minutes per landed job and names how the next throughput review measures it (the median minutes per landed job over the last 20, with checks-stage minutes compared against 14.1 and 25.4).
4. [content] The note lists alternatives considered (e.g. shared target dir, sccache, no change) and risks such as stale cache and correctness of verification from a warm build, and reviews them.
5. [content] rust-toolchain.toml is only read and cited as the cache key input; git diff shows no change to it, nor to athena.toml, defaults.toml, Cargo.lock or any file outside docs/design/.

## Expected deliverables

- docs/design/warm-baseline-build.md (create)
- CHANGELOG.md (modify)

## Out of scope

- rust-toolchain.toml (read as a cache key input only; must not change)
- Implementing the warm baseline build in crates/ or xtask/
- Changes to athena.toml, defaults.toml, or .github/workflows/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Note cites THROUGHPUT.md figures and labels estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note includes alternatives and risks.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new design note only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Note states how the next review measures the saving.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

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

A new design note exists under docs/design/ (e.g. docs/design/warm-baseline-build.md) that describes the warm baseline build of main keyed by a hash of rust-toolchain.toml and Cargo.lock, its background refresh, and how checks (fmt, clippy, test) start from it instead of an empty CARGO_TARGET_DIR; checkable by reading the file.
The note cites throughput review 20261029-155931-wfkr and the measured figures from THROUGHPUT.md (14.1 of 25.4 minutes median per landed job in checks; producer 6.8), labeling any other number as an estimate.
The note states the predicted saving of 8.0 minutes per landed job and names how the next throughput review measures it (the median minutes per landed job over the last 20, with checks-stage minutes compared against 14.1 and 25.4).
The note lists alternatives considered (e.g. shared target dir, sccache, no change) and risks such as stale cache and correctness of verification from a warm build, and reviews them.
rust-toolchain.toml is only read and cited as the cache key input; git diff shows no change to it, nor to athena.toml, defaults.toml, Cargo.lock or any file outside docs/design/.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
