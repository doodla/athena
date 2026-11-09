# Job 20261017-205554-zaj2: Hide other studies from study jobs; configure Hermes Agent as a system

## Request (verbatim)

> Give each study job a fresh context: a job studying one system must not see another system's study, as BRIEF.md Part 2 S1 requires ("each studied by Athena with a fresh context, with no access to other instances' notes beyond labeled examples in the skills"). Today agent snapshots hide only records/ and owner-excluded paths (crates/athena-git/src/snapshot.rs), so a study job can read studies/firstmate-work-handling/; the study capability's design promised separation enforced in code (docs/design/study-capability-m1.md) and it was never built. Then configure Hermes Agent as the second studied system: repo https://github.com/NousResearch/hermes-agent at tag v2026.9.24 (an annotated tag on commit f97608f178d1ffeca59860195ab7da295f7c8e5f, release v0.21.5).
> Plan: (1) For a study job naming system b, the reader's, producer's and verifier's snapshots leave out every other studies/<id>/ directory; change jobs still see all studies. (2) A hidden directory is never reported as deleted and lands unchanged. (3) In athena.toml, add [capabilities.study.systems.hermes-agent] with that URL and repo_ref = "v2026.9.24", keeping firstmate as it is. Athena must have been reinstalled from a main that holds the systems schema before this job runs.
> Acceptance:
> - cargo test: the repository's athena.toml parses; with a fake pinner, Hermes pins at v2026.9.24 and firstmate still pins the SHA in its pin.json.
> - cargo test: in a test repository with studies/a and studies/b, the snapshots of a study job naming b lack studies/a, and a change job's snapshots hold both.
> - cargo test: a hidden study directory is never reported as deleted, and studies/a is byte-identical after a fake-driver study job naming b runs to landing.
> - clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry; the change to athena.toml is listed in the reading and named in a criterion.
> Out of scope: running Hermes, studying it, and the site.

## Reading (accepted by the user)

For a study job naming system b, snapshots for reader, producer and verifier omit every other studies/<id>/ directory, hidden directories are never counted as deleted and land unchanged, and change jobs are unaffected. Then add [capabilities.study.systems.hermes-agent] to athena.toml pinned at v2026.9.24, keeping firstmate. Audience: the owner enforcing BRIEF.md Part 2 S1 fresh-context separation.

Assumptions:
- Code survey: read crates/athena-git/src/snapshot.rs (Snapshot::create calls materialize, which drops records/ and athena-exclude paths via classify; the entry point), crates/athena-core/src/run/producer.rs (producer_snapshot), run/verification.rs, reading.rs, run/planning.rs (other Snapshot::create callers), run/pin.rs (pin_repo; study_pin lets studies/<id>/pin.json override repo_ref, otherwise the pinner resolves repo_ref), and crates/athena-data/src/config.rs (repo_url, repo_ref). Resolution and pinning run before dispatch; snapshots are created per role afterwards. The property is measured directly: files present on disk in each snapshot, and the change list / landed tree compared to the base. No new record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Assumption: the systems schema ([capabilities.study.systems.<id>] with repo_url and repo_ref, and a way to select the system a study job names) exists in the base commit, as the request states. The copy I read lacks it. If the base lacks it the athena.toml entry will not parse, and the producer must stop and report instead of inventing the schema.
- Design: Snapshot::create gains a list of hidden path prefixes (the other studies/<id>/ directories). The core computes it from the study id the job names and passes it for every role's snapshot of a study job. Change jobs pass an empty list. The hidden prefixes are removed in materialize alongside records/.
- The hidden directories exist at the base. Change-list and landing logic must treat a path hidden from the snapshot as absent from both sides, not as deleted, so landing leaves studies/a byte-identical. This extends the existing handling of records/ and athena-exclude.
- Existing code: crates/athena-git/src/snapshot.rs, crates/athena-core/src/run/pin.rs, crates/athena-core/src/run/producer.rs, docs/design/study-capability-m1.md.
- Consumers: crates/athena-core/src/run/pin.rs, crates/athena-cap-study/src/lib.rs (reads the study section of athena.toml).
- Term ladder: not applicable; no structural terms are used.
- Out of scope as stated: running Hermes, studying it, and the site. firstmate's config and studies/firstmate-work-handling/ stay unchanged.

## Acceptance criteria

1. [behavior] Run cargo test: a test that parses the repository's own athena.toml passes, and with a fake pinner Hermes pins at v2026.9.24 from https://github.com/NousResearch/hermes-agent while firstmate still pins the SHA in studies/firstmate-work-handling/pin.json.
2. [behavior] Run cargo test: in a test repository with studies/a and studies/b, the reader, producer and verifier snapshots of a study job naming b lack studies/a and hold studies/b, and a change job's snapshots hold both.
3. [behavior] Run cargo test: a hidden study directory is never reported as deleted in a change list, and studies/a is byte-identical after a fake-driver study job naming b runs to landing.
4. [content] athena.toml (listed as a deliverable) contains [capabilities.study.systems.hermes-agent] with repo_url https://github.com/NousResearch/hermes-agent.git (or the URL without .git, matching firstmate's form) and repo_ref = "v2026.9.24", and the firstmate entry is unchanged in the diff.
5. [behavior] Run cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check: both are clean.
6. [content] CHANGELOG.md has a new entry describing study snapshot isolation, the hidden-directory landing rule and the Hermes Agent system; the diff shows it.
7. [content] The change refuses no more readings; it only hides paths from study snapshots, so no refusal-rate change is intended (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/pin.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/study-capability-m1.md (modify)

## Out of scope

- Running Hermes Agent
- Studying Hermes Agent or writing studies/hermes-agent
- The site
- Changing the firstmate configuration or its existing study

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Isolation criteria check the snapshot contents and landed bytes by running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the hiding mechanism plus the one config entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
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

- path: <state>/jobs/20261017-205554-zaj2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205554-zaj2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test: a test that parses the repository's own athena.toml passes, and with a fake pinner Hermes pins at v2026.9.24 from https://github.com/NousResearch/hermes-agent while firstmate still pins the SHA in studies/firstmate-work-handling/pin.json.
Run cargo test: in a test repository with studies/a and studies/b, the reader, producer and verifier snapshots of a study job naming b lack studies/a and hold studies/b, and a change job's snapshots hold both.
Run cargo test: a hidden study directory is never reported as deleted in a change list, and studies/a is byte-identical after a fake-driver study job naming b runs to landing.
athena.toml (listed as a deliverable) contains [capabilities.study.systems.hermes-agent] with repo_url https://github.com/NousResearch/hermes-agent.git (or the URL without .git, matching firstmate's form) and repo_ref = "v2026.9.24", and the firstmate entry is unchanged in the diff.
Run cargo clippy --all-targets --locked -- -D warnings and cargo fmt --all --check: both are clean.
CHANGELOG.md has a new entry describing study snapshot isolation, the hidden-directory landing rule and the Hermes Agent system; the diff shows it.
The change refuses no more readings; it only hides paths from study snapshots, so no refusal-rate change is intended (expected 0% of newly refused readings).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
