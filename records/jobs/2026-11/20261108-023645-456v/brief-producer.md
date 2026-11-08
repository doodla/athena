# Job 20261108-023645-456v: Merge duplicate failure classes instead of only adding new ones

## Request (verbatim)

> Lessons become checks, not knowledge. They created 158 separate failure classes that are never merged.

## Reading (accepted by the user)

Sri's note (BRIEF.md, 2026-11-08) says the introspection step only ever adds failure classes (158 so far, none merged). I read it as: make the class registry merge-capable and stop code adding near-duplicates. The audience is Sri and the introspection and planner prompts that read the class list.

Assumptions:
- Code survey: crates/athena-core/src/introspect.rs (entry: the introspection answer handler around lines 1570-1620 where `failure_class == "new"` mints a FailureClass with ClassStatus::Proposed and a fresh RecordId; the registry is read at lines 424-434 from records/failure-classes; Jev class matching runs in shadow via ask_jev, then code narrows by class_of_work and category), crates/athena-data/src/records/introspection.rs (ClassStatus has only `Proposed`; FailureClass is deny_unknown_fields), crates/athena-core/src/labeled.rs and hestia.rs (they read classes, so merged classes must resolve to their target when counting recurrence). The records validation allow-list already accepts athena.failure-class/1 through AnyRecord; any new field or status variant must round-trip in the existing records round-trip test and check_record_file. Class count and recurrence are measured directly on a fixture registry in tests, not through a stand-in.
- Existing code: docs/design/defaults-loop.md section 3.4 already says the user may rename or merge classes, but no code does; this job builds that merge and extends the introspect.rs registry path.
- The real 158 classes live under records/, which this job cannot read or edit. So the job delivers the mechanism (a merge status, resolution of merged ids, duplicate prevention at creation) and does not rewrite existing records.
- Term ladder: not used. Merge means a class record marked merged into a target class id; counts and lookups then treat it as its target.
- Assumed mechanism: add `ClassStatus::Merged` with an optional `merged_into` id, a code path that refuses to create a new class whose normalised name equals an existing live class (reusing it instead), and resolution of merged ids in registry reads, earlier-case selection and recurrence counts. A CLI command for a user-driven merge is optional and only added if small.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [behavior] `cargo test -p athena-core introspect` includes a test where a new-class answer whose normalised name equals an existing live class reuses that class id and adds no registry record.
2. [behavior] A test merges class B into class A. Registry reads and earlier-case selection then count B's introspections under A, and the live class count drops by one. Run with `cargo test -p athena-core`.
3. [behavior] A FailureClass with status merged and merged_into round-trips through the records round-trip test and passes check_record_file. An old `proposed` record still deserializes. Run with `cargo test -p athena-data`.
4. [behavior] `cargo test` passes workspace-wide, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] docs/design/defaults-loop.md section 3.4 and CHANGELOG.md describe the merge behavior. They say the existing 158 classes are not rewritten by this change and mark any unbuilt part as design.
6. [content] A decision entry in docs/decisions.md records the call: dedupe at creation plus merge status, with the alternatives of periodic model clustering and a manual-only merge, and why.

## Expected deliverables

- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-data/src/records.rs (modify)
- docs/design/defaults-loop.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)

## Out of scope

- Rewriting or merging the existing 158 records under records/
- Clustering existing classes with a model
- Changing the lessons-into-defaults or skills work in the other brief lines
- Prompt file edits (token headroom is near zero)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require passing cargo test, clippy and fmt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at class creation, which is where the duplicates come from, plus a merge path.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show reuse and merge happening.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Dedupe is enforced in code rather than by prompt.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision entry are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Scoped to one mechanism and leaves the data migration out.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261108-023645-456v/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-023645-456v/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

`cargo test -p athena-core introspect` includes a test where a new-class answer whose normalised name equals an existing live class reuses that class id and adds no registry record.
A test merges class B into class A. Registry reads and earlier-case selection then count B's introspections under A, and the live class count drops by one. Run with `cargo test -p athena-core`.
A FailureClass with status merged and merged_into round-trips through the records round-trip test and passes check_record_file. An old `proposed` record still deserializes. Run with `cargo test -p athena-data`.
`cargo test` passes workspace-wide, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
docs/design/defaults-loop.md section 3.4 and CHANGELOG.md describe the merge behavior. They say the existing 158 classes are not rewritten by this change and mark any unbuilt part as design.
A decision entry in docs/decisions.md records the call: dedupe at creation plus merge status, with the alternatives of periodic model clustering and a manual-only merge, and why.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
