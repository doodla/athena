# Job 20261106-042815-ntc7: Amend D152 to match the widened ontology claim-check scan scope

## Request (verbatim)

> try again on 20261102-223344-sqs9

## Reading (accepted by the user)

Retry of job 20261102-223344-sqs9. Its code (scanned_files now scans BRIEF.md, README.md and docs/*.md) is kept, and the rejection finding is fixed: decision D152 still says the check scans only BRIEF.md and the term's file sources. The audience is the verifier and the next producer.

Assumptions:
- Job 20261102-223344-sqs9 is addressed finding by finding. (1) Major, D152 Decision (1) contradicting the code: fixed by amending D152 in docs/decisions.md (or adding a decision that supersedes it). (2) Minor, two error messages in crates/athena-cli/tests/ontology.rs (about lines 338 and 397) still say 'not BRIEF.md or one of the term's file sources': fixed to name BRIEF.md, README.md, docs/*.md and the term's file sources. (3) Minor, docs/ subfolders not scanned: the reading accepted by the user named docs/*.md and the new request asks only for documentation edits. This is not changed in code. D152's Tradeoff records that docs/ subfolders are scanned only when a term lists them as a source.
- The candidate's two changed files, CHANGELOG.md and crates/athena-cli/tests/ontology.rs, are kept as deliverables. The candidate is not visible in the repository I read: scanned_files there still scans only BRIEF.md plus the term's file sources (line 249). The producer must therefore start from the candidate; if it is absent, it must redo the scanned_files change, the scratch-root test and the header comment before amending the decision.
- Code survey: I read crates/athena-cli/tests/ontology.rs (scanned_files at line 249, term_hits at 266, the two message strings at about 338 and 397) and docs/decisions.md D152 (lines 4111-4157). D152 Decision (1) says 'it scans `BRIEF.md` and the term's file sources'. No record file under crates/ is added, so check_record_file and AnyRecord do not apply. The scan scope is measured directly by running the ontology test against a scratch root.
- Existing decisions that state the old scope must be amended or superseded. Grep docs/decisions.md for 'BRIEF.md and the term' and 'file sources' to find any besides D152. D152's wording is amended in place, with a dated note saying the scope widened and why.
- The widened scan is non-recursive over docs/. D152 should say so, and say that reaching subfolders needs the term to list the file.

## Acceptance criteria

1. [content] docs/decisions.md D152 Decision (1) no longer says the check scans only BRIEF.md and the term's file sources. It states that the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources. Check by reading D152 (grep '^## D152').
2. [content] No decision in docs/decisions.md states the old narrower scope. Grep for 'BRIEF.md` and the term' and 'file sources' and confirm every hit is amended or is the superseding text.
3. [content] D152, or a decision that supersedes it, names the root cause, which is checks of shape and of listed sources and not of every governing source. It also states the tradeoff that docs/ subfolders are scanned only if a term lists them.
4. [content] The two error messages in crates/athena-cli/tests/ontology.rs now name BRIEF.md, README.md, docs/*.md and the term's file sources. Grep for 'is not BRIEF.md or one of' returns nothing.
5. [content] CHANGELOG.md has an entry for the scan-scope change that names the root cause. The header comment in crates/athena-cli/tests/ontology.rs also names it.
6. [behavior] The scratch-root unit test fails against the old scanned_files and passes with the change. A governing doc outside the term's listed sources names a term in a sense its definition lacks, and the test requires it to be flagged. Run: cargo test -p athena-cli --test ontology.
7. [behavior] The real-file ontology test passes on docs/ontology.toml with no new excuses. Run: cargo test -p athena-cli --test ontology. Also run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

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
- Recursive scanning of docs/ subfolders (job sqs9's third finding, recorded as a tradeoff instead)
- Building a code check that links decisions to the behavior they describe (the introspection's proposed prevention)

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decision names the root cause and states the scan scope where the check is defined.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The scratch-root test and the real-file test are run, not just checked for existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the prior job's files plus decisions.md change. Subfolder recursion is left out.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision text is written for an outside reader and cites its sources.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running the ontology tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261102-223344-sqs9

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> CHANGELOG.md has an entry for the change. docs/decisions.md records the call only if the scan scope is a design choice.: The CHANGELOG entry is present. But D152 in docs/decisions.md, decision point (1), already records the scan scope as a design choice: "For each term it scans `BRIEF.md` and the term's file sources". The change widens that scope and leaves D152 untouched (the diff touches nothing under docs/), so the decision now contradicts the code. Scan scope is clearly a design choice here, so docs/decisions.md should have been updated or a superseding decision added.

## Findings you must address

The verifier of job 20261102-223344-sqs9 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/decisions.md:4124-4125 (D152, Decision (1))): Decision D152 still says the claim check scans only BRIEF.md and the term's file sources, so the governing decision now contradicts the code. D152 reads "For each term it scans `BRIEF.md` and the term's file sources". The candidate makes scanned_files also scan README.md and every docs/*.md, and does not touch docs/decisions.md, which is on the deliverables list. This is the same failure class the job exists to prevent: a governing document disagreeing with what it describes. docs/decision-links.toml does not link D152 to ontology.rs, so the decision-reconciliation check could not catch it. Fix: amend D152 point (1), or add a decision that supersedes it, stating the new scope, why it is the root cause, and that it is non-recursive.

All of the verification's findings:

> major (docs/decisions.md:4124-4125 (D152, Decision (1))): Decision D152 still says the claim check scans only BRIEF.md and the term's file sources, so the governing decision now contradicts the code. D152 reads "For each term it scans `BRIEF.md` and the term's file sources". The candidate makes scanned_files also scan README.md and every docs/*.md, and does not touch docs/decisions.md, which is on the deliverables list. This is the same failure class the job exists to prevent: a governing document disagreeing with what it describes. docs/decision-links.toml does not link D152 to ontology.rs, so the decision-reconciliation check could not catch it. Fix: amend D152 point (1), or add a decision that supersedes it, stating the new scope, why it is the root cause, and that it is non-recursive.
> minor (crates/athena-cli/tests/ontology.rs:358 and :417): Two error messages still describe the old scan scope. "sense file {:?} is not BRIEF.md or one of the term's file sources" and "excuse ... names {:?}, which is not BRIEF.md or one of the term's file sources". Both should now say the file is not a governing doc (BRIEF.md, README.md, docs/*.md) or one of the term's file sources.
> minor (crates/athena-cli/tests/ontology.rs:260-270): Governing docs in subfolders of docs/ are not scanned unless a term lists them. read_dir(root.join("docs")) reads only the top level, so docs/design/*.md and docs/conventions/ are skipped. D152's own provenance cites docs/design/job-planning.md and docs/design/native-pr-landing.md as governing sources. The accepted reading named docs/*.md, so this matches the reading, but the same gap remains for documents one level down. A recursive walk of docs/**/*.md would close it.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

30 more qualifying classes are left out.

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

The work starts from the rejected candidate of job 20261102-223344-sqs9: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-042815-ntc7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-042815-ntc7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/decisions.md D152 Decision (1) no longer says the check scans only BRIEF.md and the term's file sources. It states that the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources. Check by reading D152 (grep '^## D152').
No decision in docs/decisions.md states the old narrower scope. Grep for 'BRIEF.md` and the term' and 'file sources' and confirm every hit is amended or is the superseding text.
D152, or a decision that supersedes it, names the root cause, which is checks of shape and of listed sources and not of every governing source. It also states the tradeoff that docs/ subfolders are scanned only if a term lists them.
The two error messages in crates/athena-cli/tests/ontology.rs now name BRIEF.md, README.md, docs/*.md and the term's file sources. Grep for 'is not BRIEF.md or one of' returns nothing.
CHANGELOG.md has an entry for the scan-scope change that names the root cause. The header comment in crates/athena-cli/tests/ontology.rs also names it.
The scratch-root unit test fails against the old scanned_files and passes with the change. A governing doc outside the term's listed sources names a term in a sense its definition lacks, and the test requires it to be flagged. Run: cargo test -p athena-cli --test ontology.
The real-file ontology test passes on docs/ontology.toml with no new excuses. Run: cargo test -p athena-cli --test ontology. Also run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
