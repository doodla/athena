# Job 20261030-060852-dy62: Add pin-policy and hermetic-check defaults so readings settle them

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Requests about a pin deadlock state the problem, not the policy that should govern it. Nothing in th
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: design-policy-choice-unspecified-needs-user-answer (20261016-142810-vhs2)
> Cause: Requests about a pin deadlock state the problem, not the policy that should govern it. Nothing in the defaults records stances such as exact pins versus floating ones, or who owns bumping them. The reading step has no way to settle such a p...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261016-142800-y4vb (job 20261016-005805-p806); prevention it proposed: Add a written defaults entry for version pins and similar the person running Athena-owned config: pins stay exact, are never auto-floated, and are bumped by the the person running Athena through a single explicit command. That command sets the pin to the installed version, ...
> Introspection 20261025-183726-fqgr (job 20261025-183644-t67q); prevention it proposed: Add a written default: a check never depends on inherited environment or ambient tools. The runner passes every path a check needs explicitly. Any choice between a preferred tool and a fallback is recorded in the check's output, and a silen...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: defaults.toml records no stance on version pins (exact, never auto-floated, who bumps them) or on checks that depend on ambient tools or environment. So the reader had to ask the user (jobs 20261016-005805-p806 and 20261025-183644-t67q). The fix adds two written defaults, makes the reader prompt budget fit them, and adds tests proving the reader prompt carries them.

Assumptions:
- Goal and audience: the reader role, and the person running Athena who would otherwise be asked; they should find pin and hermetic-check stances already written in defaults.toml.
- Code survey: crates/athena-data/src/defaults.rs (loads defaults.toml, renders the 'The user's default expectations (version N)' block, has a test that the model parses). crates/athena-context/src/lib.rs (BudgetSet token budgets, Contract = 350; the reader set is 2000 and has 2 tokens left). crates/athena-core/src/run_pin_tests.rs line 1625 (a test that hardcodes 'of 2000 tokens used' for the reader). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Job 20261016-005805-p806 is addressed by writing the policy that its request left open: pins stay exact, are never auto-floated, and are bumped only by the person running Athena through one explicit command. Its candidate (patch-tolerant claude::accepts in claude.rs, xtask claude_accepts, wiring.rs, live.rs, README.md) is dropped, not kept; its findings (D154 link missing from decision-links.toml, duplicated rule, unrun full test) do not apply because none of that code is re-landed.
- Job 20261025-183644-t67q is addressed by writing the second default: a check never depends on inherited environment or ambient tools, the runner passes each path explicitly, and any preferred-versus-fallback tool choice is recorded in the check output. It made no changes, so no file is kept or dropped; its nextest work (athena.toml, ci.yml, checks.rs, gh.rs, config.rs, run.rs, xtask, policy.rs) is out of scope here.
- Existing code: defaults.toml, crates/athena-data/src/defaults.rs and crates/athena-context/src/lib.rs are extended, not replaced; new entries follow the existing [[default]] shape (id, classes, text).
- Consumers: defaults.toml is read by crates/athena-data/src/defaults.rs and rendered into the reader prompt; the reader budget in crates/athena-context/src/lib.rs bounds it.
- defaults.toml 'version' stays 1 unless the defaults loader requires a bump for added entries; the producer decides after reading defaults.rs.
- Changing a prompt budget and the reader prompt content is an agent-behavior change: only the reader role is affected.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry for version pins whose text says pins are exact, are never auto-floated, and are bumped only by the person running Athena through one explicit command that sets the pin to the installed version; it is applied to the classes code, config and build.
2. [content] defaults.toml has a second new [[default]] entry saying a check never depends on inherited environment or ambient tools, that the runner passes every path a check needs explicitly, and that any preferred-versus-fallback tool choice is recorded in the check output.
3. [content] The reader token budget in crates/athena-context/src/lib.rs is raised enough for the new entries (same file and same change as the reader budget line in crates/athena-core/src/run_pin_tests.rs line 1625, updated to match), and docs/decisions.md gets a new appended decision, with a Provenance bullet, naming the root cause, the two jobs, and the budget change; no existing entry is edited.
4. [content] CHANGELOG.md has an entry naming the root cause (defaults silent on pin and ambient-tool policy). athena.toml, workflows and the files changed by the two earlier candidates (claude.rs, wiring.rs, live.rs, README.md, xtask, checks.rs, gh.rs) are not changed; defaults.toml is the only governance file changed.
5. [behavior] Run `cargo test` and check that a new test in crates/athena-data/src/defaults.rs parses the real defaults.toml and asserts both new ids exist with the key phrases (exact, never auto-floated, ambient). The test fails on the base tree, where the entries are absent, and passes with the change. The same test also checks the rendered reader prompt contains both entries, so the property is checked for the class and not for one job.
6. [behavior] Run `cargo test`: the reader prompt-budget test (run_pin_tests.rs) and the context budget tests pass, which shows the reader set fits its raised budget. Recorded fmt, clippy (-D warnings) and test checks pass on the final tree.
7. [behavior] Outcome check: feeding the p806 request text through the defaults rendering shows the pin stance in the block the reader sees, so the reader can settle it without a question. This is checked by the new test's assertion on the rendered text.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Patch-tolerant version acceptance in claude.rs and xtask claude_accepts, wiring.rs, live.rs, README.md (job 20261016-005805-p806 candidate): dropped.
- An xtask repin command that rewrites athena.toml: the default names the command; building it is a separate job.
- cargo-nextest adoption, athena.toml [tools] pins, ci.yml, checks.rs, gh.rs, config.rs, run.rs (job 20261025-183644-t67q): separate job.
- Editing athena.toml, workflows or other prompt files.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus passing cargo test and clippy.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the defaults, the origin shared by both jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered reader prompt, not file existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome criterion checks the reader can settle the policy unasked.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces the defaults and the budget.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261030-060852-dy62/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060852-dy62/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] entry for version pins whose text says pins are exact, are never auto-floated, and are bumped only by the person running Athena through one explicit command that sets the pin to the installed version; it is applied to the classes code, config and build.
defaults.toml has a second new [[default]] entry saying a check never depends on inherited environment or ambient tools, that the runner passes every path a check needs explicitly, and that any preferred-versus-fallback tool choice is recorded in the check output.
The reader token budget in crates/athena-context/src/lib.rs is raised enough for the new entries (same file and same change as the reader budget line in crates/athena-core/src/run_pin_tests.rs line 1625, updated to match), and docs/decisions.md gets a new appended decision, with a Provenance bullet, naming the root cause, the two jobs, and the budget change; no existing entry is edited.
CHANGELOG.md has an entry naming the root cause (defaults silent on pin and ambient-tool policy). athena.toml, workflows and the files changed by the two earlier candidates (claude.rs, wiring.rs, live.rs, README.md, xtask, checks.rs, gh.rs) are not changed; defaults.toml is the only governance file changed.
Run `cargo test` and check that a new test in crates/athena-data/src/defaults.rs parses the real defaults.toml and asserts both new ids exist with the key phrases (exact, never auto-floated, ambient). The test fails on the base tree, where the entries are absent, and passes with the change. The same test also checks the rendered reader prompt contains both entries, so the property is checked for the class and not for one job.
Run `cargo test`: the reader prompt-budget test (run_pin_tests.rs) and the context budget tests pass, which shows the reader set fits its raised budget. Recorded fmt, clippy (-D warnings) and test checks pass on the final tree.
Outcome check: feeding the p806 request text through the defaults rendering shows the pin stance in the block the reader sees, so the reader can settle it without a question. This is checked by the new test's assertion on the rendered text.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
