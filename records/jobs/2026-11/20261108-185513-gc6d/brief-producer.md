# Job 20261108-185513-gc6d: Design per-job landing branches with a CHANGELOG entry

## Request (verbatim)

> try again on 20261108-183842-fsaz

## Reading (accepted by the user)

Retry job 20261108-183842-fsaz: write the per-job landing branches design note (one job, one commit, one PR cut from main) for review 20261108-183648-5eya, now with the CHANGELOG.md entry, corrected stage path, and a stronger measurement rule.

Assumptions:
- Retry of job 20261108-183842-fsaz: its major finding (CHANGELOG.md not modified) is fixed by listing CHANGELOG.md as a deliverable and requiring an entry; its minor finding on status.rs vs status/throughput.rs is fixed by a criterion; its minor finding on weak attribution is fixed by requiring the per-phase split measure. Its three candidate files are kept as deliverables.
- Existing code: docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs are existing and are reused or extended by the design.
- Goal: a design note for throughput review 20261108-183648-5eya; audience is Athena maintainers and the next throughput review.
- The proposed pre-verifier changelog check from the introspection is a separate job and is out of scope here; the new request as typed asks only for the design with a CHANGELOG entry.
- Environment: the design discusses pull requests, but nothing is pushed or opened by this job; it writes only repository files locally, with no CI or history reads.

## Acceptance criteria

1. [content] A new Markdown file docs/design/per-job-landing-branches.md exists in the diff and describes landing each job as one commit in one pull request cut from current main, instead of stacking on a long-lived local branch.
2. [content] The note cites throughput review 20261108-183648-5eya and the figures 1537.1 of 1582.1 median minutes (THROUGHPUT.md); Grep for the review id in the new file finds it.
3. [content] The note states the predicted saving (1530.0 minutes per landed job) labeled as an estimate, and says the next throughput review measures it using the per-phase split from build job 1, so that a concurrent continuous-landing change is separated from this one.
4. [content] The design has an Existing code section that reuses or extends docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs, naming what is kept and what changes.
5. [content] The note lists alternatives considered (keep stacking with squash, rebase the stack) with reasons for rejection, and risks such as conflicts and landing order when cutting from main.
6. [content] Any reference to StageMinutes or the stage definition names crates/athena-core/src/status/throughput.rs, not status.rs (Grep 'StageMinutes' in the verifier's snapshot confirms the path).
7. [content] CHANGELOG.md is modified in the diff with an entry for the per-job landing design note, in the style of the existing continuous-landing.md entry (git diff shows a CHANGELOG.md hunk).
8. [content] docs/design/continuous-landing.md and docs/design/squash-chain-landing.md each gain only a short pointer to the new note (git diff). The diff touches only docs/design/ and CHANGELOG.md, and contains no secrets or personal data.

## Expected deliverables

- docs/design/per-job-landing-branches.md (create)
- docs/design/continuous-landing.md (modify)
- docs/design/squash-chain-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the landing change in crates/ or xtask/
- Implementing the pre-verifier changelog check
- Changes to athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design note with alternatives and risks.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cites THROUGHPUT.md figures and labels the saving an estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Names how the next throughput review measures the saving.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One note plus two pointers and a changelog entry.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261108-183842-fsaz

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria: none recorded.

## Findings you must address

The verifier of job 20261108-183842-fsaz raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (CHANGELOG.md): CHANGELOG.md is an expected deliverable and the changelog default applies, but the change does not modify it. The deliverables list says 'CHANGELOG.md (modify)', and the changelog default says the change records itself in CHANGELOG.md. git diff HEAD~1 --stat shows only the three docs/design/ files. Earlier design-only notes have entries, such as CHANGELOG.md line 490 for continuous-landing.md (D155) and line 497 for warm-baseline-build.md, so this one should too.

All of the verification's findings:

> major (CHANGELOG.md): CHANGELOG.md is an expected deliverable and the changelog default applies, but the change does not modify it. The deliverables list says 'CHANGELOG.md (modify)', and the changelog default says the change records itself in CHANGELOG.md. git diff HEAD~1 --stat shows only the three docs/design/ files. Earlier design-only notes have entries, such as CHANGELOG.md line 490 for continuous-landing.md (D155) and line 497 for warm-baseline-build.md, so this one should too.
> minor (docs/design/per-job-landing-branches.md section 1): The note says the stage is defined in crates/athena-core/src/status.rs, but StageMinutes is defined in crates/athena-core/src/status/throughput.rs. Grep shows 'pub struct StageMinutes' at crates/athena-core/src/status/throughput.rs:102. status.rs only uses it in tests. Section 8 ('the stage definition in status.rs') has the same imprecision.
> minor (docs/design/per-job-landing-branches.md sections 5 and 7): The attribution rule is weak if continuous landing ships in the same window. Section 5 credits a drop to this change if every landed pull request holds one commit. That shows the design was in force, but it does not separate this saving from a saving made by continuous-landing.md in the same window. The per-phase split from build job 1 is the stronger measure, and the note could say the review uses it.

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
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

The work starts from the rejected candidate of job 20261108-183842-fsaz: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-185513-gc6d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-185513-gc6d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new Markdown file docs/design/per-job-landing-branches.md exists in the diff and describes landing each job as one commit in one pull request cut from current main, instead of stacking on a long-lived local branch.
The note cites throughput review 20261108-183648-5eya and the figures 1537.1 of 1582.1 median minutes (THROUGHPUT.md); Grep for the review id in the new file finds it.
The note states the predicted saving (1530.0 minutes per landed job) labeled as an estimate, and says the next throughput review measures it using the per-phase split from build job 1, so that a concurrent continuous-landing change is separated from this one.
The design has an Existing code section that reuses or extends docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs, naming what is kept and what changes.
The note lists alternatives considered (keep stacking with squash, rebase the stack) with reasons for rejection, and risks such as conflicts and landing order when cutting from main.
Any reference to StageMinutes or the stage definition names crates/athena-core/src/status/throughput.rs, not status.rs (Grep 'StageMinutes' in the verifier's snapshot confirms the path).
CHANGELOG.md is modified in the diff with an entry for the per-job landing design note, in the style of the existing continuous-landing.md entry (git diff shows a CHANGELOG.md hunk).
docs/design/continuous-landing.md and docs/design/squash-chain-landing.md each gain only a short pointer to the new note (git diff). The diff touches only docs/design/ and CHANGELOG.md, and contains no secrets or personal data.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
