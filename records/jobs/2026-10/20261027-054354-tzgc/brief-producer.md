# Job 20261027-054354-tzgc: Skip Priority lines when taking a request's title

## Request (verbatim)

> Request titles in athena plan and athena status take the request's first line, so every request that starts with a 'Priority:' line shows the title 'Priority: high' or 'Priority: urgent'. Take the title from the first line that is not a Priority line.
> Acceptance:
> - A test shows a request whose first line is 'Priority: urgent' gets its title from the next non-empty line in plan and status output.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/plan.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

A request's title comes from its first line even when that is 'Priority: urgent'. The title will come from the first non-empty line that is not a Priority line, so plan and status show a useful title. The fix goes in one shared helper in plan.rs, used by the title code in store.rs that both outputs call.

Assumptions:
- Goal/audience: operators reading `athena plan` and `athena status` need distinct request titles instead of 'Priority: high'.
- Code survey: crates/athena-core/src/store.rs `title()` (line ~150) falls back to `req.text.lines().next()` when no reading exists; status.rs:711 calls job.title(); reading/topics.rs:374 builds the PlanEntry title from job.title(). plan.rs `Priority::from_text` (line ~97) already detects Priority lines (trimmed, any case, `Priority:` prefix). reading.rs:1360 also truncates the first line, which is probably the subject for the reading fallback. No record or data file is added, so check_record_file and AnyRecord are not touched.
- The shared helper lives in plan.rs (e.g. `pub fn title_line(text: &str) -> &str`), and store.rs title() calls it. store.rs is therefore edited even though the request's file list omits it. A reading-derived title is unchanged.
- If every line is blank or a Priority line, the title falls back to the first line, as before.
- Existing code: crates/athena-core/src/plan.rs Priority::from_text and crates/athena-core/src/store.rs title() are reused and extended.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new unit test, run with `cargo test -p athena-core`, builds a request whose text starts with 'Priority: urgent', then a blank line, then 'Fix the widget'. It asserts the title is 'Fix the widget' in both the plan entry (topics/plan path) and the status output (status.rs).
2. [behavior] Tests cover a lowercase/indented 'priority: high' line, a request with no Priority line (the title is still its first line), and a request made only of Priority lines (falls back to the first line). `cargo test` passes.
3. [behavior] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
4. [content] The diff to crates/athena-core/src/plan.rs adds the shared title helper, and the diff to crates/athena-core/src/status.rs adds or updates a status test; the title logic in store.rs uses the helper instead of `lines().next()`.
5. [content] CHANGELOG.md has a new entry under the unreleased section describing the title fix.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Changing how priority itself is parsed or ranked
- Changes to titles that come from a reading
- Governance files such as athena.toml and defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests; cargo test, fmt and clippy -D warnings pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared title source in store.rs so plan and status both change, not just the two outputs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the tests on the resulting titles.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One helper plus one call site and tests.
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
- 11 cases in 11 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the final verdict. For a job whose criteria name specific job ids, the check reads the live records and lists, per id, whether an outcome record exists and whether a closed job cites a commit hash found in git log. It reports any gap to the producer while it can sti...

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

- path: <state>/jobs/20261027-054354-tzgc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054354-tzgc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test, run with `cargo test -p athena-core`, builds a request whose text starts with 'Priority: urgent', then a blank line, then 'Fix the widget'. It asserts the title is 'Fix the widget' in both the plan entry (topics/plan path) and the status output (status.rs).
Tests cover a lowercase/indented 'priority: high' line, a request with no Priority line (the title is still its first line), and a request made only of Priority lines (falls back to the first line). `cargo test` passes.
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
The diff to crates/athena-core/src/plan.rs adds the shared title helper, and the diff to crates/athena-core/src/status.rs adds or updates a status test; the title logic in store.rs uses the helper instead of `lines().next()`.
CHANGELOG.md has a new entry under the unreleased section describing the title fix.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
