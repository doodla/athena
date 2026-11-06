# Job 20261102-223344-sqs9: Check seeded ontology terms against every governing source, not a listed

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a seeded data file is checked for shape and for the sources the author chose to list, so a sense of a term that lives in another governing source goes unseen until the final verdict. The fix is in crates/athena-cli/tests/ontology.rs, where the claim check should scan every governing doc for each term. This also covers the 0wtc, y179 and kq9v retries; the audience is the verifier and the next producer.

Assumptions:
- Code survey: crates/athena-cli/tests/ontology.rs. scanned_files() scans only BRIEF.md plus the sources the term itself lists, so a sense found in another governing doc (README.md, docs/*.md) escapes the check unless the author listed that doc. The file also holds claim_problems and the excuse mechanism. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly: each term name is searched in the governing files, and the test fails when a hit is neither a listed sense nor an excused hit.
- Job 20261017-173508-0wtc: its Hermes (tool vs Olympus role) finding is already fixed in the repo (docs/ontology.toml has senses and the test has claim checks). The remaining gap is the scan scope, which this change closes. Its files are kept: CHANGELOG.md, docs/decisions.md, docs/ontology.toml and crates/athena-cli/tests/ontology.rs; the last is modified, the others only if a newly found hit needs a sense or an excuse.
- Job 20261025-214509-y179: a speed-target job whose goal was missed. It is a different instance of the same cause, a claim never measured before handover. Its code changes (secrets.rs memo, checks.rs poll schedule, D126, CHANGELOG) are kept untouched. The speed work itself is not redone here, and the 7-minute goal is a separate request.
- Job 20261031-172040-kq9v: the stale-binary finding is already addressed in the repo. xtask BUILD_INPUT_GLOBS is the single list, xtask/tests/policy.rs requires each athena-bin key in ci.yml to match it, and ci.yml and xtask/src/main.rs are kept as they are. The remaining minor findings (stamp on non-CI builds, commit reporting, dirty-tree skip) are not re-opened.
- Existing code: crates/athena-cli/tests/ontology.rs and docs/ontology.toml are extended, not replaced.
- Scan scope becomes BRIEF.md, README.md and docs/*.md except docs/ontology.toml, plus the listed sources. Decision ids and CHANGELOG.md are excluded.

## Acceptance criteria

1. [content] The change description names the root cause: checks verified shape and listed sources, not each term's content against every governing source. Evidence: the CHANGELOG.md entry and the header comment in crates/athena-cli/tests/ontology.rs.
2. [content] scanned_files in crates/athena-cli/tests/ontology.rs scans BRIEF.md, README.md and docs/*.md for every term, not only the term's listed sources. Grep the function to check.
3. [behavior] A new unit test builds a scratch root where a governing doc outside the term's listed sources names a term in a sense its definition lacks. It fails against the old scanned_files and passes with the change. Run: cargo test -p athena-cli --test ontology.
4. [behavior] The real-file test passes on docs/ontology.toml with any new senses or well-formed excuses for newly found hits. Run: cargo test -p athena-cli --test ontology.
5. [content] CHANGELOG.md has an entry for the change. docs/decisions.md records the call only if the scan scope is a design choice.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing the y179 speed work, or changing secrets.rs, checks.rs or run.rs
- Changing ci.yml, xtask or setup.sh (the kq9v input-set check already exists)
- Adding terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows
- Changing any study's concept model or files under studies/

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the scan scope of the check, not at one term.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new failing-then-passing test plus fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A scratch-root test shows a missed sense being flagged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test file, with the data file and changelog only if needed.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust test code only.
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
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261102-223344-sqs9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261102-223344-sqs9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: checks verified shape and listed sources, not each term's content against every governing source. Evidence: the CHANGELOG.md entry and the header comment in crates/athena-cli/tests/ontology.rs.
scanned_files in crates/athena-cli/tests/ontology.rs scans BRIEF.md, README.md and docs/*.md for every term, not only the term's listed sources. Grep the function to check.
A new unit test builds a scratch root where a governing doc outside the term's listed sources names a term in a sense its definition lacks. It fails against the old scanned_files and passes with the change. Run: cargo test -p athena-cli --test ontology.
The real-file test passes on docs/ontology.toml with any new senses or well-formed excuses for newly found hits. Run: cargo test -p athena-cli --test ontology.
CHANGELOG.md has an entry for the change. docs/decisions.md records the call only if the scan scope is a design choice.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
