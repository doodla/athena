# Job 20261108-142640-egnb: Add pinned-versions and explicit-tool-paths defaults for the reader

## Request (verbatim)

> try again on 20261027-044511-kktq

## Reading (accepted by the user)

Retry of job 20261027-044511-kktq. Add two [[default]] entries to defaults.toml, each with its whole rule in the first sentence (the only part reader render() shows), and raise the reader budget by the smallest round amount instead of splitting or trimming existing defaults. The audience is the reader step, which should stop asking about version pins and tool paths or silent fallback.

Assumptions:
- Retry of job 20261027-044511-kktq. Its candidate changed CHANGELOG.md, crates/athena-cli/tests/budgets.rs, defaults.toml and docs/system-facts.md. Every file is dropped and redone from the base, and each is a deliverable below except none are kept as-is. Findings addressed: (1) the five split defaults (buy-first, tested, public, verified, lazy-requests) are NOT split or shortened, and a test pins their rendered lines; (2) the new first sentences carry the bump command and the no-silent-fallback halves, and the test asserts the rendered context, not the model; (3) docs/system-facts.md is now a listed deliverable because the budgets test requires its reader line to match the computed figure; (4) the misread of render() is fixed: render shows only the first sentence (crates/athena-data/src/defaults.rs:58-83).
- Code survey: crates/athena-data/src/defaults.rs (parse, render, first_sentence: a '. ' ends the rendered text); crates/athena-cli/tests/budgets.rs (computed_rows, system_facts_budgets_match_computed, every_role_set_is_within_its_budget use defaults().render()); crates/athena-context/src/lib.rs:175 (BudgetSet::Reader => 2_000); crates/athena-core/src/run_pin_tests.rs:1625 (hard-codes 'of 2000 tokens used' for the reader line). No record or data file is added under crates/, so check_record_file and AnyRecord are not involved. The property is measured on defaults().render() itself.
- The reader set has 3 tokens of headroom (1997 of 2000), so two new rendered lines (about 40-50 tokens each, estimated at ceil(bytes/3)) will not fit. The reader budget in crates/athena-context/src/lib.rs is raised to the smallest multiple of 50 that fits, and the changelog states the old and new figure and why.
- The bump command is the existing `cargo run -p xtask -- repin` (xtask/src/main.rs:16, README.md:252-256), so the pinned-versions first sentence names it. Both entries are marked prose-only, with no enforcement code, matching the existing prose-only entries. Enforcing them in the runner is out of scope.
- Statements are one sentence each so the whole rule renders. Any '. ' inside a statement ends the render, so a command like 'cargo run -p xtask -- repin' must contain no period-space. applies_to is ['config','build','code'] for both. source cites this request and job 20261027-044511-kktq.
- Governance: defaults.toml is the requested deliverable and is named in criteria. athena.toml and .github/workflows are not needed, because the budget lives in crates/athena-context/src/lib.rs, not athena.toml.
- Docs that quote the reader figure change with it: docs/system-facts.md line 22 (checked by system_facts_budgets_match_computed) and docs/design/context-engineer.md line 54 (prose table, not test-checked).

## Acceptance criteria

1. [content] defaults.toml gains a [[default]] with id "pinned-versions" whose statement is ONE sentence (no '. ' inside) saying config version pins such as claude_version stay exact and never float, are bumped only by `cargo run -p xtask -- repin`, and a refusal from a mismatch names that command, marked prose-only. Check with Grep on defaults.toml.
2. [content] defaults.toml gains a [[default]] with id "explicit-tool-paths" whose statement is ONE sentence saying a check never relies on inherited environment or ambient tools, the runner passes every tool path explicitly, output records whether the preferred tool or the fallback ran, and a silent fallback is a defect, marked prose-only. Both entries have applies_to and a source citing this request or job 20261027-044511-kktq. Check with Grep on defaults.toml.
3. [content] git diff shows no change to the statement, applies_to or source of any existing default in defaults.toml: no sentence splitting or shortening of buy-first, tested, public, verified, lazy-requests, root-cause or behavior-evidence. Only two entries are appended.
4. [behavior] A new test in crates/athena-cli/tests/budgets.rs builds defaults().render() from the real defaults.toml and asserts it contains both new ids and the key phrases of every half: 'claude_version', 'xtask -- repin', 'never float', 'explicit', 'fallback', 'defect', and the recorded tool used. Run `cargo test -p athena-cli --test budgets` and see it pass.
5. [behavior] A second new test asserts the rendered line of every pre-existing default equals its pre-change line (listed verbatim in the test), so any change to an existing default's rendered text fails. Run `cargo test -p athena-cli --test budgets` and see it pass.
6. [behavior] The reader budget in crates/athena-context/src/lib.rs is raised only by the smallest round amount that fits, and no other budget changes. docs/system-facts.md, docs/design/context-engineer.md and the hard-coded figure in crates/athena-core/src/run_pin_tests.rs match the new figures. Run `cargo test` and see every_role_set_is_within_its_budget, system_facts_budgets_match_computed and the run_pin test pass.
7. [content] CHANGELOG.md has an entry for the two defaults that states the old and new reader budget and why it was raised. No other files change beyond the listed deliverables.
8. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/system-facts.md (modify)
- docs/design/context-engineer.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing mechanisms in the runner that enforce the version-pin or tool-path policies.
- Adding a bump command or changing claude_version handling; the existing `cargo run -p xtask -- repin` is only named.
- Changing athena.toml, CI workflows, or any budget other than the reader's.
- Splitting, shortening or dropping any existing default's rule or rendered text.
- Edits to docs/decisions.md history.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New render tests plus fmt, clippy and cargo test as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the rendered reader context contains the behavior phrases, not just the ids.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The new entries are marked prose-only, and the render and budget tests enforce their presence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: two entries, one budget raise and the figures that follow from it.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the root cause of the earlier failure: the render shows only the first sentence and the budget had no room.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and the changelog are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the entries or the changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-044511-kktq raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (defaults.toml (buy-first, tested, public, verified, lazy-requests)): The budget was met by splitting five existing defaults so that the reader's first-sentence render drops their rules DefaultsModel::render uses first_sentence() (crates/athena-data/src/defaults.rs:58-83). Before the change these five statements were one sentence each, so the reader saw all of each rule. After it, the reader sees only 'Adopt good, maintained tools and crates before building.', 'Every module has passing tests.', 'Everything committed is public: no secrets, credentials, or personal data.', 'Done means independently verified.' and 'Requests are short.'. The lost clauses include the core reader rule 'Recover hidden requirements from the repository before asking'. The CHANGELOG says this was done to fit the budget. Simpler, sound alternatives: write each new entry so its first sentence carries the whole rule, or shorten wording without adding sentence breaks, or raise the reader budget as the reading allowed in crates/athena-context/src/lib.rs.
> major (defaults.toml (pinned-versions, explicit-tool-paths); crates/athena-cli/tests/budgets.rs): The reader's default context lacks the bump-command and silent-fallback halves of the new policies, so the reader can still ask those questions The rendered lines are 'Version pins in config, such as claude_version, stay exact and never float.' and 'A check never relies on inherited environment or ambient tools.'. The explicit runner tool paths, recording the tool used, 'silent fallback is a defect', the bump command and the 'Prose-only' marks never reach the reader. The request names 'silent runner fallback' as a question the reader keeps asking, and it asks for a test that shows the reading step's context includes these policies. The new test checks those phrases against the model instead, and its comment admits this.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading assumed the reader renders each default in full, so two new entries would cost about 100 tokens and the fix would be to trim root-cause and behavior-evidence. In fact render() shows only each statement's first sentence. So trimming the long later sentences saves nothing, and whatever a policy puts after its first sentence never reaches the reader. The reading should have required each new policy's first sentence to carry the whole rule the reader needs, including the bump command and the silent-fallback rule.

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

- path: <state>/jobs/20261108-142640-egnb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-142640-egnb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml gains a [[default]] with id "pinned-versions" whose statement is ONE sentence (no '. ' inside) saying config version pins such as claude_version stay exact and never float, are bumped only by `cargo run -p xtask -- repin`, and a refusal from a mismatch names that command, marked prose-only. Check with Grep on defaults.toml.
defaults.toml gains a [[default]] with id "explicit-tool-paths" whose statement is ONE sentence saying a check never relies on inherited environment or ambient tools, the runner passes every tool path explicitly, output records whether the preferred tool or the fallback ran, and a silent fallback is a defect, marked prose-only. Both entries have applies_to and a source citing this request or job 20261027-044511-kktq. Check with Grep on defaults.toml.
git diff shows no change to the statement, applies_to or source of any existing default in defaults.toml: no sentence splitting or shortening of buy-first, tested, public, verified, lazy-requests, root-cause or behavior-evidence. Only two entries are appended.
A new test in crates/athena-cli/tests/budgets.rs builds defaults().render() from the real defaults.toml and asserts it contains both new ids and the key phrases of every half: 'claude_version', 'xtask -- repin', 'never float', 'explicit', 'fallback', 'defect', and the recorded tool used. Run `cargo test -p athena-cli --test budgets` and see it pass.
A second new test asserts the rendered line of every pre-existing default equals its pre-change line (listed verbatim in the test), so any change to an existing default's rendered text fails. Run `cargo test -p athena-cli --test budgets` and see it pass.
The reader budget in crates/athena-context/src/lib.rs is raised only by the smallest round amount that fits, and no other budget changes. docs/system-facts.md, docs/design/context-engineer.md and the hard-coded figure in crates/athena-core/src/run_pin_tests.rs match the new figures. Run `cargo test` and see every_role_set_is_within_its_budget, system_facts_budgets_match_computed and the run_pin test pass.
CHANGELOG.md has an entry for the two defaults that states the old and new reader budget and why it was raised. No other files change beyond the listed deliverables.
`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
