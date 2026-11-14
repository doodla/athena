# Job 20261114-013056-pkhp: Add version-pin and hermetic-check defaults; widen reader budget

## Request (verbatim)

> try again on 20261114-012353-pbvc

## Reading (accepted by the user)

Retry of job 20261114-012353-pbvc (planning stopped on minor critic findings; no candidate). Root cause: defaults.toml is silent on version-pin policy and ambient-tool checks, so the reader asked the user (jobs 20261016-005805-p806, 20261025-183644-t67q). Add two prose-only defaults naming the existing repin command, raise the reader budget to about 2200, and record decision D181.

Assumptions:
- Goal and audience: the reader role (and the user who currently gets asked about pins) must find written stances on version pins and hermetic checks in defaults.toml, so such requests stop needing a user answer.
- Previous result, job 20261014-placeholder is not involved; job 20261114-012353-pbvc failed in planning with only minor findings and wrote no code. Its four minor findings are each addressed: (1) both entries are marked prose-only; (2) the version-pins entry names `cargo run -p xtask -- repin` (D154), which already exists, so no command is built and its factual slip no longer applies; (3) D181 states it departs from D170's rule that budgets never rise to fit a set; (4) new lines avoid words the boundary scan bans, checked by running the policy and budgets tests.
- Earlier jobs 20261016-005805-p806 (patch-tolerant version acceptance in claude.rs, xtask, wiring.rs, live.rs, README.md) is dropped; job 20261025-183644-t67q (nextest, athena.toml tools pins, ci.yml, checks.rs, gh.rs, config.rs, run.rs) is a separate job. Neither candidate's files are kept.
- Code survey: crates/athena-data/src/defaults.rs (render prints only the first sentence, cut at the first '. '; REPO_DEFAULTS at ~line 89; test parses the real defaults.toml), crates/athena-context/src/lib.rs (reader budget at ~line 176), crates/athena-core/src/run_pin_tests.rs (reader budget literal 'of 2000' at ~line 1625), crates/athena-core budgets.rs (computes reader set from defaults.toml and compares with docs/system-facts.md), xtask/tests/policy.rs (boundary scan). No record or data file is added under crates/, so check_record_file and AnyRecord are unaffected. The property is measured directly by rendering the real defaults and counting tokens.
- Environment: none; no CI, push or records read is needed.
- Consumers of defaults.toml: crates/athena-data/src/defaults.rs (parse and render), crates/athena-context (prompt composition), budgets tests, docs/system-facts.md figures.
- Estimate (not measured): the two new rendered lines add about 140-145 tokens, so the reader set lands near 2140, under 2200.
- The first sentence of each new entry must carry the stance, since render prints only that sentence.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry id version-pins whose first sentence says pins are exact, never auto-floated, and changed only by the person running Athena through the existing `cargo run -p xtask -- repin` command; scoped to code, config and build and marked prose-only.
2. [content] defaults.toml has a new [[default]] entry id hermetic-checks whose first sentence says a check never depends on inherited environment or ambient tools, the runner passes every path a check needs, and any preferred-versus-fallback tool choice is recorded in the check output; marked prose-only. defaults.toml is the only governance file changed (athena.toml, workflows, xtask unchanged per git diff).
3. [content] The reader budget at crates/athena-context/src/lib.rs is raised to about 2200, and the literal at crates/athena-core/src/run_pin_tests.rs (~1625) and any figure in docs/system-facts.md match it.
4. [content] docs/decisions.md gets a new appended decision D181 with a Provenance bullet naming the root cause, jobs 20261016-005805-p806 and 20261025-183644-t67q, and stating it departs from D170's rule that budgets never rise to fit a set; no existing entry is edited. CHANGELOG.md has an entry naming the root cause (defaults silent on pin and ambient-tool policy). No repin command is built.
5. [behavior] Run `cargo test`: a new test in crates/athena-data/src/defaults.rs parses the real defaults.toml, asserts both ids exist with key phrases (exact, never auto-floated, repin, ambient), and asserts the rendered reader prompt contains both; it fails on the base tree and passes with the change. The reader prompt budget tests, budgets tests and xtask policy test also pass.
6. [behavior] Run `cargo fmt --check` and `cargo clippy` (warnings denied); both pass. Recorded checks on the final tree also count.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Building a repin command: it already exists (D154, xtask/src/main.rs).
- Patch-tolerant version acceptance in claude.rs, xtask claude_accepts, wiring.rs, live.rs, README.md (job 20261016-005805-p806 candidate): dropped.
- cargo-nextest adoption, athena.toml [tools] pins, ci.yml, checks.rs, gh.rs, config.rs, run.rs (job 20261025-183644-t67q): separate job.
- Editing athena.toml, workflows or other prompt files.
- Changing planning or critic acceptance rules (the introspection's design change): separate job.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criterion checks the real defaults render into the reader prompt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: New entries marked prose-only since nothing enforces them.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in defaults.toml where the policy gap originates.
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

- path: <state>/jobs/20261114-013056-pkhp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-013056-pkhp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] entry id version-pins whose first sentence says pins are exact, never auto-floated, and changed only by the person running Athena through the existing `cargo run -p xtask -- repin` command; scoped to code, config and build and marked prose-only.
defaults.toml has a new [[default]] entry id hermetic-checks whose first sentence says a check never depends on inherited environment or ambient tools, the runner passes every path a check needs, and any preferred-versus-fallback tool choice is recorded in the check output; marked prose-only. defaults.toml is the only governance file changed (athena.toml, workflows, xtask unchanged per git diff).
The reader budget at crates/athena-context/src/lib.rs is raised to about 2200, and the literal at crates/athena-core/src/run_pin_tests.rs (~1625) and any figure in docs/system-facts.md match it.
docs/decisions.md gets a new appended decision D181 with a Provenance bullet naming the root cause, jobs 20261016-005805-p806 and 20261025-183644-t67q, and stating it departs from D170's rule that budgets never rise to fit a set; no existing entry is edited. CHANGELOG.md has an entry naming the root cause (defaults silent on pin and ambient-tool policy). No repin command is built.
Run `cargo test`: a new test in crates/athena-data/src/defaults.rs parses the real defaults.toml, asserts both ids exist with key phrases (exact, never auto-floated, repin, ambient), and asserts the rendered reader prompt contains both; it fails on the base tree and passes with the change. The reader prompt budget tests, budgets tests and xtask policy test also pass.
Run `cargo fmt --check` and `cargo clippy` (warnings denied); both pass. Recorded checks on the final tree also count.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
