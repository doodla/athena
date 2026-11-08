# Job 20261027-044511-kktq: Add version-pin and check-tool-path defaults to defaults.toml

## Request (verbatim)

> Add two entries to defaults.toml, in the form the reading step already reads. (1) Version pins in operator config, such as claude_version, stay exact and never float. The operator bumps them with one explicit command, and any refusal caused by a mismatch names that command. (2) A check never relies on inherited environment or ambient tools: the runner passes every tool path a check needs explicitly. When a check can use a preferred tool or a fallback, its output records which one it used, and a silent fallback counts as a defect. Done when both entries are in defaults.toml, a test shows the reading step's default context includes them, and the existing defaults and budget tests pass. Today defaults.toml contains neither policy (read: a case-insensitive grep found no match for either).
> 
> Acceptance:
> - The change is made as described: The reading step keeps asking the user policy questions that a written default should answer (version pins; check tool paths and silent runner fallback)
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: defaults.toml

## Reading (accepted by the user)

Add two [[default]] entries to defaults.toml in the existing id/statement/applies_to/source form: exact version pins bumped by one named command, and checks that get explicit tool paths and record any fallback. The audience is the reader step, which should stop asking the user these questions. A test shows the reader's default context contains both.

Assumptions:
- Goal: the reader step answers version-pin and check-tool-path questions from written defaults instead of asking the user. Audience: the reader, producer and verifier that receive defaults.
- Code survey: crates/athena-cli/tests/budgets.rs (every_role_set_is_within_its_budget composes the reader from defaults().render()), crates/athena-context/src/lib.rs (BudgetSet::Reader = 2_000 tokens, check_budget), crates/athena-core/src/proposals.rs (DefaultsModel::parse and defaults.toml editing). defaults.toml is data parsed at runtime, not a record file under crates/, so check_record_file and AnyRecord do not apply.
- Budget risk: the reader prompt has 3 tokens left, and it renders every default. Two new entries add roughly 100 tokens, so the producer must shorten existing long statements (root-cause, behavior-evidence) or the budget test fails. Raising the 2_000 limit in crates/athena-context/src/lib.rs is a fallback only if trimming cannot pass, and it is not planned.
- The new test goes in an existing test file in crates/athena-cli/tests/ or the crates/athena-core tests, wherever defaults() can be imported; no governance file other than defaults.toml changes.
- Exact applies_to: ["config", "build", "code"] for both entries. Both are prose-only unless a mechanism exists, and the statement says so.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry with id "pinned-versions" (or similar). Its statement says version pins in config, such as claude_version, stay exact and never float, are bumped only by one explicit command, and any refusal caused by a mismatch names that command. Check with Grep on defaults.toml.
2. [content] defaults.toml has a second new [[default]] entry whose statement says a check never relies on inherited environment or ambient tools, and the runner passes every tool path a check needs explicitly. It also says that when a check can use a preferred tool or a fallback, its output records which one was used, and a silent fallback is a defect. Check with Grep on defaults.toml.
3. [content] Both entries have applies_to and a source citing the request or job, matching the form of the other entries, so DefaultsModel::parse accepts the file. The diff touches only defaults.toml plus a test file, and the version field and existing entries keep their meaning.
4. [behavior] A new test builds the reader's default context from the real defaults.toml (defaults().render(), as in crates/athena-cli/tests/budgets.rs) and asserts the text contains both new ids and key phrases. Run cargo test and see it pass.
5. [behavior] The existing defaults tests and every_role_set_is_within_its_budget pass under cargo test; the reader set stays within its 2,000-token budget, which has 3 tokens of headroom today, by keeping the new statements terse and tightening wordy existing statements without dropping their content.
6. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test all pass.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing mechanisms that enforce the version-pin or tool-path policies in the runner.
- Adding a bump command or changing any claude_version handling.
- Changing athena.toml or CI workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass, with a new test for the reader's default context.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the rendered reader context contains both entries.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to defaults.toml plus one test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The entries state whether they are prose-only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The reader context must show the policies so that questions are no longer asked.
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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-044511-kktq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-044511-kktq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] entry with id "pinned-versions" (or similar). Its statement says version pins in config, such as claude_version, stay exact and never float, are bumped only by one explicit command, and any refusal caused by a mismatch names that command. Check with Grep on defaults.toml.
defaults.toml has a second new [[default]] entry whose statement says a check never relies on inherited environment or ambient tools, and the runner passes every tool path a check needs explicitly. It also says that when a check can use a preferred tool or a fallback, its output records which one was used, and a silent fallback is a defect. Check with Grep on defaults.toml.
Both entries have applies_to and a source citing the request or job, matching the form of the other entries, so DefaultsModel::parse accepts the file. The diff touches only defaults.toml plus a test file, and the version field and existing entries keep their meaning.
A new test builds the reader's default context from the real defaults.toml (defaults().render(), as in crates/athena-cli/tests/budgets.rs) and asserts the text contains both new ids and key phrases. Run cargo test and see it pass.
The existing defaults tests and every_role_set_is_within_its_budget pass under cargo test; the reader set stays within its 2,000-token budget, which has 3 tokens of headroom today, by keeping the new statements terse and tightening wordy existing statements without dropping their content.
cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
