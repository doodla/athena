# Job 20261109-201651-ggxj: Design early per-job PRs and dropping the local staging branch

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing and rebuild stage takes 1811.5 of the 1859.9-minute median per landed job (THROUGHPUT.md
> 
> Athena's throughput review 20261109-201520-z1xg named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing and rebuild stage takes 1811.5 of the 1859.9-minute median per landed job (THROUGHPUT.md, measured). Most of that is not real waiting. Each job's commit sits on one long-lived local staging branch, and every later pull request c...
> Stage: landing and rebuild
> Proposed change: Open each job's pull request when its producer finishes, not after the verifier, and drop the local staging branch. When the producer ends, Athena builds the job's single landing commit on the host's current main (merge_tree and commit_tree...
> Predicted saving: 7.0 minutes per landed job; the median was 1859.9 minutes per landed job when the review ran.
> Why: What I read: THROUGHPUT.md and these design notes: continuous-landing, async-drain, native-pr-landing, per-job-landing-branches, per-job-main-landing, agent-and-build-slots, warm-baseline-build and ci-routing-baseline. I ran nothing. Where ...
> 
> A change that implements this design must cite throughput review 20261109-201520-z1xg in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for opening each job's PR when the producer finishes, building the landing commit on current main, and dropping the long-lived staging branch, to cut the landing and rebuild wait. Audience: Athena maintainers and the next throughput review, which scores the stated saving.

Assumptions:
- Existing code: docs/design/continuous-landing.md, docs/design/native-pr-landing.md, docs/design/per-job-landing-branches.md, docs/design/per-job-main-landing.md, docs/design/async-drain.md, docs/design/agent-and-build-slots.md, docs/design/warm-baseline-build.md and docs/design/ci-routing-baseline.md are existing designs the new note builds on.
- Environment: the design discusses GitHub pull requests and CI; the job itself is docs-only, reads only the repository copy, writes one markdown file, and makes no pushes or network calls.
- Goal: remove the landing and rebuild wait (1811.5 of 1859.9 median minutes per landed job); audience is maintainers and the next throughput review.
- The request text must cite throughput review 20261109-201520-z1xg; the note cites it and THROUGHPUT.md.
- Predicted saving is 7.0 minutes per landed job against a 1859.9-minute median; the note states this and how the next review measures it.
- Only docs are changed; no code, config or governance files, so no code survey or consumers are needed.
- Term staging branch: the long-lived local git branch on which each job's commit sits until its pull request lands.

## Acceptance criteria

1. [content] A new design note exists under docs/design/ that describes opening each job's PR when the producer finishes, building the single landing commit on the host's current main, and dropping the local staging branch.
2. [content] The design has an Existing code section that reuses or extends docs/design/per-job-main-landing.md, docs/design/native-pr-landing.md, docs/design/per-job-landing-branches.md and docs/design/continuous-landing.md, saying what is kept, changed or superseded.
3. [content] The note explains, citing THROUGHPUT.md, how the change removes the landing and rebuild wait (1811.5 of 1859.9 median minutes) and lists alternatives considered.
4. [content] The note states a saving of 7.0 minutes per landed job against the 1859.9-minute median, labeled as a prediction, and names how the next throughput review measures the minutes after the change lands.
5. [content] The note cites throughput review 20261109-201520-z1xg and states that a change implementing it must cite that review in its request text.
6. [content] Only files under docs/design/ are added or changed, per git diff.

## Expected deliverables

- docs/design/early-pr-no-staging-branch.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/ or xtask/
- Editing athena.toml, defaults.toml or workflows
- Editing existing design notes beyond optional cross-links

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite THROUGHPUT.md or are labeled predictions.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The note says how the next review measures the saving.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design note only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261109-201651-ggxj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-201651-ggxj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new design note exists under docs/design/ that describes opening each job's PR when the producer finishes, building the single landing commit on the host's current main, and dropping the local staging branch.
The design has an Existing code section that reuses or extends docs/design/per-job-main-landing.md, docs/design/native-pr-landing.md, docs/design/per-job-landing-branches.md and docs/design/continuous-landing.md, saying what is kept, changed or superseded.
The note explains, citing THROUGHPUT.md, how the change removes the landing and rebuild wait (1811.5 of 1859.9 median minutes) and lists alternatives considered.
The note states a saving of 7.0 minutes per landed job against the 1859.9-minute median, labeled as a prediction, and names how the next throughput review measures the minutes after the change lands.
The note cites throughput review 20261109-201520-z1xg and states that a change implementing it must cite that review in its request text.
Only files under docs/design/ are added or changed, per git diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
