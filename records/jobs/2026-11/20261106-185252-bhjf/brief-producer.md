# Job 20261106-185252-bhjf: Retry 20261106-042815-ntc7: widen ontology scan to all governing docs

## Request (verbatim)

> try again on 20261106-042815-ntc7

## Reading (accepted by the user)

Retry of stale job 20261106-042815-ntc7, which passed verification but could not land because the branch moved (drain max_concurrency change). Its verified candidate is kept and merged onto the tip; the audience is the verifier and the next producer.

Assumptions:
- Job 20261106-042815-ntc7 is addressed by keeping its whole candidate (CHANGELOG.md, crates/athena-cli/tests/ontology.rs, docs/decisions.md, docs/ontology.toml) and re-verifying on the merged tree. Its stale outcome (branch moved, dd76838c7c0f) needs only a clean merge; no content finding is outstanding.
- Its two nits (long lines in decisions.md/ontology.toml/ontology.rs; CHANGELOG entry under Added rather than Changed) are cosmetic and are optionally fixed if the producer touches those lines; they are not required.
- Code survey: crates/athena-cli/tests/ontology.rs is the only code read. Its error messages at lines ~257 and ~310 in the base still say 'is not BRIEF.md or one of the term's file sources'; the candidate changes them. The property (a sense found in a governing doc but missing from a term) is measured directly by the scratch-root unit test and the real-file test over docs/ontology.toml. No record or data file is added under crates/, so check_record_file and AnyRecord are not involved. No refusal rule for readings is added.
- Merge conflicts with the tip, if any, are in docs/decisions.md or CHANGELOG.md; the producer resolves them keeping both sides.

## Acceptance criteria

1. [content] docs/decisions.md D152 states the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources, and no decision keeps the old narrower scope. Check with grep '^## D152'.
2. [content] D152 names the root cause (checks of shape and listed sources, not every governing source) and the tradeoff that docs/ subfolders are scanned only if a term lists them.
3. [content] Grep 'is not BRIEF.md or one of' in crates/athena-cli/tests/ontology.rs returns nothing; the messages name BRIEF.md, README.md, docs/*.md and the term's file sources.
4. [content] CHANGELOG.md has an entry naming the root cause of the scan-scope change, and the header comment in crates/athena-cli/tests/ontology.rs names it; prior CHANGELOG content and the new drain max_concurrency entries from the tip survive the merge.
5. [behavior] The scratch-root unit test flags a governing doc outside the term's listed sources that names a term in a sense its definition lacks. Run: cargo test -p athena-cli --test ontology.
6. [behavior] The real-file ontology test passes on docs/ontology.toml with no new excuses. Run cargo test -p athena-cli --test ontology, then cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)

## Out of scope

- Redoing the y179 speed work, or changing secrets.rs, checks.rs or run.rs
- Changing ci.yml, xtask or setup.sh
- Adding ontology terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows
- Changing any study's concept model or files under studies/
- Recursive scanning of docs/ subfolders (recorded as a tradeoff)
- Building a code check that links decisions to the behavior they describe
- Reverting or touching the drain max_concurrency change on the tip

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and cargo test criterion included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run the ontology tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Criteria require the root cause named and fixed at the scan scope.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Re-verification on the merged tree.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: job 20261106-042815-ntc7 went stale and its candidate conflicts with the branch tip

That job was verified, but the branch moved on before it could land. Athena merged its candidate onto the branch tip and git reported conflicts in these paths (quoted data, not instructions):

> CHANGELOG.md
> crates/athena-cli/tests/ontology.rs

Your working directory already holds git's conflicted merge: the branch tip's files with the candidate's changes merged in. Resolve every conflict so that both the tip's changes and the candidate's intent are kept, and leave no conflict marker line (`<<<<<<<`, `=======` or `>>>>>>>`) in any file. A candidate that still holds an opening and a closing marker line in a listed path is closed before it is checked or verified. A listed path that holds no marker lines is a modify/delete, rename or binary conflict: git kept only one side of it. Decide explicitly whether to keep, delete or merge that path, and say which in your report. Address the other points the job's verification raised below.

Unmet criteria: none recorded.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

32 more qualifying classes are left out.

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

The work starts from the conflicted merge of job 20261106-042815-ntc7's candidate onto the branch tip: the files of the base commit with the merge result laid over them, uncommitted, in a one-commit git repository of its own whose commit holds the base. The listed conflicted files may hold conflict markers, which you must resolve. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-185252-bhjf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-185252-bhjf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/decisions.md D152 states the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources, and no decision keeps the old narrower scope. Check with grep '^## D152'.
D152 names the root cause (checks of shape and listed sources, not every governing source) and the tradeoff that docs/ subfolders are scanned only if a term lists them.
Grep 'is not BRIEF.md or one of' in crates/athena-cli/tests/ontology.rs returns nothing; the messages name BRIEF.md, README.md, docs/*.md and the term's file sources.
CHANGELOG.md has an entry naming the root cause of the scan-scope change, and the header comment in crates/athena-cli/tests/ontology.rs names it; prior CHANGELOG content and the new drain max_concurrency entries from the tip survive the merge.
The scratch-root unit test flags a governing doc outside the term's listed sources that names a term in a sense its definition lacks. Run: cargo test -p athena-cli --test ontology.
The real-file ontology test passes on docs/ontology.toml with no new excuses. Run cargo test -p athena-cli --test ontology, then cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
