# Job 20261029-180923-wx3a: Give OLY-2-15 (Dionysus exploration) a bounded-experiment mechanism

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-2-15 a mechanism
> 
> Requirement OLY-2-15 in docs/spec.toml is active and has no mechanism.
> Requirement: Dionysus handles exploration: proposing unconventional solutions and bounded experiments.
> Owner: Dionysus
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-2-15
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-2-15 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core module, exploration.rs, that models a Dionysus bounded experiment (hypothesis, unconventional approach, hard bound) and refuses unbounded or empty ones. Then record it as OLY-2-15's mechanism and measure in docs/spec.toml, with a test that fails without it. Audience: the Hestia spec review and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, the allowed `status:` labels), crates/athena-core/src/hestia.rs (spec review, parse_spec, SPEC_FILE, and the test that parses include_str!(docs/spec.toml)), and crates/athena-core/src/proposals.rs (an existing record-backed proposal flow). No code mentions experiments yet. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: extends crates/athena-core/src/lib.rs (module list) and follows the style of crates/athena-core/src/manage.rs and crates/athena-core/src/proposals.rs; docs/spec.toml holds the requirement entry.
- Design call: no printed status label fits exploration, so the measure ends with `status: not shown` and names the test. Adding a status line is out of scope to keep the change small.
- Design call: a pure-code module, with no language-model role and no new crate (buy-first: nothing off the shelf models this). Not a new refusal rule on readings, so no refusal-rate criterion applies.
- Goal: Dionysus's exploration duty gets a checkable mechanism; the audience is the Hestia spec review and the verifier.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-2-15 has a mechanism naming crates/athena-core/src/exploration.rs (not "none"), keeps owner "Dionysus", and a measure ending with `status: not shown` that names the test (cargo test -p athena-core exploration). Check with Grep on docs/spec.toml.
2. [content] crates/athena-core/src/exploration.rs exists, is declared in crates/athena-core/src/lib.rs, and defines a bounded-experiment type (hypothesis, unconventional approach, bound, outcome) with a validate function.
3. [behavior] Run `cargo test -p athena-core exploration`: tests show that an experiment with no bound or an empty hypothesis is refused, that a bounded one is accepted, and that an experiment which exceeds its bound is reported as stopped. Each test fails if validate is stubbed to accept everything.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass. This includes the existing spec tests in hestia.rs and xtask that parse docs/spec.toml.

## Expected deliverables

- crates/athena-core/src/exploration.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- A new language-model Dionysus role or prompt
- A new `athena status` line
- Changes to other requirements in docs/spec.toml
- Changes to athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs the tests that exercise refusal and acceptance.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: No existing crate fits; plain Rust, recorded as a design call.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-180923-wx3a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-180923-wx3a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-2-15 has a mechanism naming crates/athena-core/src/exploration.rs (not "none"), keeps owner "Dionysus", and a measure ending with `status: not shown` that names the test (cargo test -p athena-core exploration). Check with Grep on docs/spec.toml.
crates/athena-core/src/exploration.rs exists, is declared in crates/athena-core/src/lib.rs, and defines a bounded-experiment type (hypothesis, unconventional approach, bound, outcome) with a validate function.
Run `cargo test -p athena-core exploration`: tests show that an experiment with no bound or an empty hypothesis is refused, that a bounded one is accepted, and that an experiment which exceeds its bound is reported as stopped. Each test fails if validate is stubbed to accept everything.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass. This includes the existing spec tests in hestia.rs and xtask that parse docs/spec.toml.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
