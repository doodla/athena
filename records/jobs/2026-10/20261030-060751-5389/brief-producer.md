# Job 20261030-060751-5389: Check a reading's references to earlier jobs against the repo

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in th
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-assumes-prior-job-deliverable-exists-unverified (20261026-050336-ykx8)
> Cause: Nothing checks a reading's references to earlier jobs against the current repo. A job id named in the request or notes is taken to mean its work has landed. The earlier job may be queued, unmerged or never produced. The critic is the first ...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261026-050325-8red (job 20261026-042558-5p65); prevention it proposed: Add a check at reading acceptance. For every earlier job id, role, prompt or file the reading calls existing, it confirms the artifact is in the snapshot (landed in main, with a path found). If not, the reading must either list it as a deli...
> Introspection 20261026-065033-9gdg (job 20261026-064950-38jx); prevention it proposed: Add a check before a reading is accepted. It lists the roles, prompts and files the reading depends on and confirms each exists in the current snapshot. When one is missing, the check requires the reading either to make any request it queue...
> Introspection 20261026-092111-01tx (job 20261026-065228-02zf); prevention it proposed: Before a reading is accepted, run a check that every field, state, role or function the reading relies on is found in the current snapshot (by grep of the named symbols). It should flag any dependency that is absent, such as a "resolved" st...
> Introspection 20261026-145400-8x0c (job 20261026-145124-ssmw); prevention it proposed: Add a required step to the reading stage: for every existing path, record type, role or function the reading says a feature will use, it must cite the file and line where it exists in the current snapshot, and for each transfer between comp...
> Introspection 20261026-145920-qjpb (job 20261025-001236-brde); prevention it proposed: Add a required step to reading and planning for any request that changes "all" readers, writers or call sites. Before the plan is accepted, the stage must list every real call site found by searching the snapshot, mark each as in scope or o...
> Introspection 20261026-150536-zmp5 (job 20261026-145955-s7e2); prevention it proposed: Require any plan that changes "all readers/callers" of something to include a call-site inventory produced by actually running the scan against the current tree. Each hit must be marked move, allow-listed or out of scope, with a reason and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Add a check in crates/athena-core/src/reading.rs, run when a reading is built, that confirms each earlier job id, path, role, or symbol the reading relies on exists in the snapshot (landed, with a path found). Otherwise the reading must list it as a deliverable to create, or put it in out_of_scope. Audience: the planner, critic and verifier, who now find missing dependencies late. Jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf are the referenced earlier attempts; their candidate work is not touched here.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: build_reading (line 542) is the entry point; check_prior_result_and_scope (2391) and check_code_survey (2999) run after resolution, and the new check goes beside them. The new check adds no record or data file, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by calling the new check on a Reading with fixture snapshot paths.
- Root cause: nothing verifies that artifacts the reading says already exist (paths, files, symbols named in the request or notes) are present in the snapshot; a job id is taken to mean its work landed.
- Scope of the check: paths under crates/, docs/ and similar that the reading's text names as existing and does not list as create deliverables or out_of_scope. A named item that is not found must be a create deliverable or in out_of_scope, else the reading is refused.
- Past jobs 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf are not retried here; their candidate files (code_health.rs and others) are neither kept nor created by this change and are excluded in out_of_scope.
- Governance files (athena.toml, defaults.toml) need no change: the check is code, with no new configuration field.
- Refusal rate: the check refuses more readings; the higher rate is intended; expected 8% of newly refused readings.

## Acceptance criteria

1. [content] The CHANGELOG.md entry and the change description name the root cause: readings take a named earlier job's deliverable as existing without checking the snapshot, and the fix sits in the reading-acceptance check in crates/athena-core/src/reading.rs.
2. [behavior] A unit test in crates/athena-core/src/reading.rs gives a reading that says an existing path or role from a queued earlier job is already there. The check refuses it, and the same reading is accepted when the item is a create deliverable, in out_of_scope, or present in the snapshot. Run: cargo test -p athena-core reading. The test fails without the change.
3. [behavior] The test uses at least two distinct missing dependencies (a path and a symbol or role), so the class is covered and not one job's case. Run: cargo test -p athena-core reading.
4. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 8% of newly refused readings.
5. [behavior] cargo fmt --check, cargo clippy and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Retrying the code-health, context-engineer or other jobs 20261026-042558-5p65, 20261026-064950-38jx, 20261026-065228-02zf; their candidate files (crates/athena-core/src/code_health.rs, steward.rs, status.rs, gh.rs, display.rs, run.rs, introspect.rs, jev.rs, records.rs, briefs.rs) are dropped, not kept
- Changing planner or critic prompts
- Changing athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new test fails without the check; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at reading acceptance, where unverified references are first admitted.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on the refusal and acceptance cases.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a check in code, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check in one file plus notes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that missing dependencies are caught at reading, shown by tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

21 more qualifying classes are left out.

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

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The CHANGELOG.md entry and the change description name the root cause: readings take a named earlier job's deliverable as existing without checking the snapshot, and the fix sits in the reading-acceptance check in crates/athena-core/src/reading.rs.
A unit test in crates/athena-core/src/reading.rs gives a reading that says an existing path or role from a queued earlier job is already there. The check refuses it, and the same reading is accepted when the item is a create deliverable, in out_of_scope, or present in the snapshot. Run: cargo test -p athena-core reading. The test fails without the change.
The test uses at least two distinct missing dependencies (a path and a symbol or role), so the class is covered and not one job's case. Run: cargo test -p athena-core reading.
The change refuses more readings and the higher refusal rate is intended: expected 8% of newly refused readings.
cargo fmt --check, cargo clippy and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
