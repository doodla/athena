# Job 20261111-061100-smxs: Revise merge-tree landing design note: real race detection, scoring

## Request (verbatim)

> try again on 20261111-055238-5har

## Reading (accepted by the user)

Retry job 20261111-055238-5har: revise the design note for review 20261111-055051-tjym so its race handling rests on real push and host-merge behavior, reconcile the 5.0-minute prediction with the stage it is scored on, and fix roadmap numbering. Audience: Athena maintainers.

Assumptions:
- Retry of job 20261111-055238-5har (rejected: 1 major, 1 minor, 1 nit). Its three changed files (new note, CHANGELOG.md, roadmap.md) are kept as deliverables and revised; each finding is fixed per the criteria.
- Existing code: crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/client.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md
- Goal: a design note for throughput review 20261111-055051-tjym; audience: Athena maintainers who implement and judge the change.
- Environment: none beyond reading the repository; the note describes pushes and PRs but the job runs no push, reads no records, and writes only docs.
- Before writing, the producer reads the push and merge code and, for every claim about a refusal, conflict or race, names the code path that produces it.
- Design note for a change that does not itself add code; no governance files are edited.

## Acceptance criteria

1. [content] New note docs/design/direct-main-merge-tree-landing.md names throughput review 20261111-055051-tjym and cites the figures 2984.3 of 2990.6 median minutes (landing and rebuild) as evidence, and says which figures come from review text.
2. [content] The note has an Existing code section that reuses or extends crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md and docs/design/per-job-main-landing.md, and says how it differs from the two earlier designs and why they did not remove the wait.
3. [content] The note explains the mechanism (fetch host main on verifier pass, Git::merge_tree with base = job's recorded base, commit_tree, push as the job's own PR branch) and why it removes carried-over unlanded commits from the single long-lived local landing branch.
4. [content] Fixes finding 1 (major) of job 20261111-055238-5har: the section on host main moving and two jobs landing at once does not claim a push refusal. It states, citing Git::push_branch in crates/athena-git/src/lib.rs (about lines 430-436) that a plain push to a new PR branch is refused only when the branch exists and is not a descendant, so a main move does not fail the push. It names the real detection point: the host merge (gh/client.rs MergeMethod) preceded by an is_ancestor check that the PR's parent is still the main tip (or a host up-to-date requirement), then refetch and rebuild with merge_tree, with a concrete failure signal per case.
5. [content] Fixes finding 2 (minor): the note states which single measured stage figure is scored against the 5.0-minute prediction (the first-record/real-wait part, about 4.6 minutes by the note's own figures, so the prediction is reconciled with that bound), and section on measurement says the headline 2984.3 median is not the one compared with 5.0; it also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
6. [content] The note lists alternatives considered and rejected, merge_tree conflict handling, and rollback or migration from the local branch.
7. [content] Fixes finding 3 (nit): the roadmap item in docs/design/roadmap.md uses plain sequential numbering (no '3b'), and CHANGELOG.md has an entry for the note.
8. [behavior] Only docs change: git diff shows no change under crates/, athena.toml, defaults.toml or .github/workflows/; cargo build and cargo test still pass.

## Expected deliverables

- docs/design/direct-main-merge-tree-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Editing THROUGHPUT.md or records/
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the cause-to-fix link and the scored measurement.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Figures cited to sources or labeled as estimates.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier checks claims against landing code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261111-055238-5har

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> The note lists alternatives considered and rejected, conflict handling (merge_tree conflict, host main moving during landing), and rollback or migration from the local branch.: Section 5 (alternatives), section 8 (migration and rollback) and the merge_tree-conflict case in section 7 are present and sound. The handling for host main moving is wrong. Section 7 says 'The push is refused as not a fast-forward of the commit's parent' and, for two jobs, 'the second push is refused'. But section 2 pushes to the job's own new branch with Git::push_branch, which lib.rs:430-431 describes as a plain push that 'Git refuses when the branch exists and commit does not descend from it'. A new per-job branch is never refused because main moved. The race really happens when the host merges the pull request (gh/client.rs MergeMethod), and the note does not cover that point.

## Findings you must address

The verifier of job 20261111-055238-5har raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/design/direct-main-merge-tree-landing.md section 7 ('Host main moves while landing', 'Two jobs land at once')): The handling for host main moving and for two jobs landing at once relies on a push refusal that cannot happen. Section 2 step 5 pushes each commit as the job's own new pull request branch with Git::push_branch. lib.rs:430-436: a plain push that git refuses only when the branch already exists and the commit does not descend from it. A move of host main never makes that push fail, so neither the refetch-and-retry nor the 'second push is refused' happens. The real race is at the host merge (gh/client.rs MergeMethod, the host merge endpoint), where the host can squash a pull request built on an old tip without any check. A better design would detect the move at merge time, for example by checking with is_ancestor that the pull request's parent is still the main tip before merging, or by having the host require an up-to-date branch, and then rebuilding with merge_tree from a fresh fetch.

All of the verification's findings:

> major (docs/design/direct-main-merge-tree-landing.md section 7 ('Host main moves while landing', 'Two jobs land at once')): The handling for host main moving and for two jobs landing at once relies on a push refusal that cannot happen. Section 2 step 5 pushes each commit as the job's own new pull request branch with Git::push_branch. lib.rs:430-436: a plain push that git refuses only when the branch already exists and the commit does not descend from it. A move of host main never makes that push fail, so neither the refetch-and-retry nor the 'second push is refused' happens. The real race is at the host merge (gh/client.rs MergeMethod, the host merge endpoint), where the host can squash a pull request built on an old tip without any check. A better design would detect the move at merge time, for example by checking with is_ancestor that the pull request's parent is still the main tip before merging, or by having the host require an up-to-date branch, and then rebuilding with merge_tree from a fresh fetch.
> minor (docs/design/direct-main-merge-tree-landing.md sections 3 and 6): The 5.0-minute prediction does not fit the note's own figures or its measurement method. Section 3 says the saving is 'the real-wait part only', but quotes the real wait from verifier to first record as about 4.6 minutes, so a 5.0-minute saving would exceed the whole wait. Section 6 then measures the headline median against 2984.3, which the note expects to fall by thousands of minutes. The difference from 5.0 will therefore be very large whatever the real saving is. The note should say which figure is scored against 5.0, for example the first-record stage.
> nit (docs/design/roadmap.md:52): The roadmap item is numbered '3b' and placed between items 3 and 4. The '3b.' numbering breaks the list's plain numbered sequence.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261111-055238-5har: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-061100-smxs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-061100-smxs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

New note docs/design/direct-main-merge-tree-landing.md names throughput review 20261111-055051-tjym and cites the figures 2984.3 of 2990.6 median minutes (landing and rebuild) as evidence, and says which figures come from review text.
The note has an Existing code section that reuses or extends crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md and docs/design/per-job-main-landing.md, and says how it differs from the two earlier designs and why they did not remove the wait.
The note explains the mechanism (fetch host main on verifier pass, Git::merge_tree with base = job's recorded base, commit_tree, push as the job's own PR branch) and why it removes carried-over unlanded commits from the single long-lived local landing branch.
Fixes finding 1 (major) of job 20261111-055238-5har: the section on host main moving and two jobs landing at once does not claim a push refusal. It states, citing Git::push_branch in crates/athena-git/src/lib.rs (about lines 430-436) that a plain push to a new PR branch is refused only when the branch exists and is not a descendant, so a main move does not fail the push. It names the real detection point: the host merge (gh/client.rs MergeMethod) preceded by an is_ancestor check that the PR's parent is still the main tip (or a host up-to-date requirement), then refetch and rebuild with merge_tree, with a concrete failure signal per case.
Fixes finding 2 (minor): the note states which single measured stage figure is scored against the 5.0-minute prediction (the first-record/real-wait part, about 4.6 minutes by the note's own figures, so the prediction is reconciled with that bound), and section on measurement says the headline 2984.3 median is not the one compared with 5.0; it also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
The note lists alternatives considered and rejected, merge_tree conflict handling, and rollback or migration from the local branch.
Fixes finding 3 (nit): the roadmap item in docs/design/roadmap.md uses plain sequential numbering (no '3b'), and CHANGELOG.md has an entry for the note.
Only docs change: git diff shows no change under crates/, athena.toml, defaults.toml or .github/workflows/; cargo build and cargo test still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
