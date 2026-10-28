# Job 20261027-054214-dn47: Check config edits against the schema at plan time, not only at landing

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The config file and the code that parses it are changed separately, and nothing checks that a new co
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: config-edit-ahead-of-schema-breaks-admission (20261016-185439-64ke)
> Cause: The config file and the code that parses it are changed separately, and nothing checks that a new config section is known to the parser. The new `[parallel]` table was added to athena.toml (by the job's plan or by hand) without a matching s...
> Seen 4 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261016-185428-8vdd (job 20261016-071202-q0xy); prevention it proposed: Run a parse check of athena.toml against the schema as part of the edit and before admission. It should run when a config change is planned, and again on the job's resulting diff. A job whose outcome needs a new config section must deliver ...
> Introspection 20261016-191018-b44f (job 20261016-185517-qcrz); prevention it proposed: Add two code checks. (1) A config-parse check at submit and enqueue time, and again before a job starts. If athena.toml fails to parse under the current schema, refuse the request at once with the parse error and say it is the same cause as...
> Introspection 20261016-191109-w348 (job 20261016-185517-qcrz); prevention it proposed: Change admission so it can accept a config change that comes with the schema change in the same job. For example, validate the changed athena.toml against the schema from the job's own proposed code, or treat config sections the binary does...
> Introspection 20261016-205039-fqgx (job 20261016-200605-xh54); prevention it proposed: Add a code check that runs before any commit that touches athena.toml or the config schema, and before landing. It builds the parser from the working tree being committed and parses the athena.toml in that same tree, and it refuses the comm...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: athena.toml and its parser (crates/athena-data/src/config.rs, strict on unknown fields) are changed separately, and only admission and landing check them. The gate already defers the parse when a change touches crates/athena-data/ (`defers_to_schema_test`), but a plan can still list athena.toml or defaults.toml without any schema file, or name the wrong crate (jobs 20261016-071202-q0xy and 20261016-185517-qcrz did). The fix adds one shared plan-time rule on that same predicate and tests it for the class. It addresses jobs 20261016-071202-q0xy, 20261016-185517-qcrz and 20261016-200605-xh54.

Assumptions:
- Goal: Athena's maintainers need a config edit that is not covered by a schema change to be refused or flagged at plan time, so jobs stop dying at admission or landing with 'unknown field'.
- Job 20261016-071202-q0xy: its `[parallel]` refusal came from the schema being edited in the wrong crate (athena-core) and no check connecting the two. The plan-time rule now catches that case; the parallel feature itself has since landed (athena-data/src/config.rs has `Parallel`, and athena.toml has `[parallel]`).
- Job 20261016-185517-qcrz: the same refusal. The schema deferral in athena_gate (SCHEMA_DIR = crates/athena-data/) already handles the case where the schema file is in the change. The gap that remains is a plan that edits config without a schema file, which this change closes.
- Job 20261016-200605-xh54: it passed verification but the landing gate refused; its four findings (extra files, panic isolation in drain, the display label, gate refusal) concern the parallel feature, not this cause. The gate refusal is covered by the existing deferral rule and the new rule; the other findings no longer apply to this change.
- The rule reuses `athena_gate::defers_to_schema_test` and `SCHEMA_DIR` so plan, admission and gate share one definition. It lives in athena-core (plan or admission module), since athena-core already depends on athena-gate.
- Rule: a plan item whose deliverables modify athena.toml or defaults.toml must also list a modified path under crates/athena-data/, or the plan is flagged. The check cannot see new sections at plan time, so the producer picks the narrowest sound form and documents it.
- The exact location of plan validation was not read in full. The producer should find it in crates/athena-core/src/plan.rs or admission.rs.

## Acceptance criteria

1. [content] The change description names the root cause: config and parser change separately, and nothing connects a new athena.toml section to the schema that must accept it before plan time. It cites jobs 20261016-071202-q0xy, 20261016-185517-qcrz and 20261016-200605-xh54.
2. [behavior] A new test in crates/athena-core (run with `cargo test -p athena-core`) shows that a plan or change which modifies athena.toml without any crates/athena-data/ path is flagged with a message that names the schema file to change. The same test shows that a change that also modifies crates/athena-data/src/config.rs is accepted. The test fails on the base commit and passes with the change.
3. [behavior] The plan-time rule calls `athena_gate::defers_to_schema_test`/SCHEMA_DIR rather than duplicating the path, and a test confirms that the gate, admission and the plan rule agree on the same inputs (`cargo test`).
4. [behavior] Existing gate and admission tests (unknown-field refusal in content.rs, deferral tests) and crates/athena-data/tests/repo_config.rs still pass under `cargo test`.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry for the new check. No changes to athena.toml, defaults.toml or .github/workflows.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/content.rs (modify)

## Out of scope

- Changes to the parallel-jobs feature itself (drain.rs, [parallel] settings)
- Submit/enqueue-time config parse refusal for a config that already fails to parse
- Letting admission build and run candidate code
- Changes to athena.toml, defaults.toml or .github/workflows
- The drain panic-isolation and display-label findings from job 20261016-200605-xh54
- Threads-in-one-process spike, daemon or async runtime, automatic merging of stale jobs, run-level spend cap
- Multi-job `athena go` and the --pr landing phase

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix goes where the config and schema are connected, using the shared SCHEMA_DIR rule, not at one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the class, and fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the new test and check that it fails on the base.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code rule enforces this, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused plan-time check on the existing shared predicate.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

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

The change description names the root cause: config and parser change separately, and nothing connects a new athena.toml section to the schema that must accept it before plan time. It cites jobs 20261016-071202-q0xy, 20261016-185517-qcrz and 20261016-200605-xh54.
A new test in crates/athena-core (run with `cargo test -p athena-core`) shows that a plan or change which modifies athena.toml without any crates/athena-data/ path is flagged with a message that names the schema file to change. The same test shows that a change that also modifies crates/athena-data/src/config.rs is accepted. The test fails on the base commit and passes with the change.
The plan-time rule calls `athena_gate::defers_to_schema_test`/SCHEMA_DIR rather than duplicating the path, and a test confirms that the gate, admission and the plan rule agree on the same inputs (`cargo test`).
Existing gate and admission tests (unknown-field refusal in content.rs, deferral tests) and crates/athena-data/tests/repo_config.rs still pass under `cargo test`.
`cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
CHANGELOG.md has an entry for the new check. No changes to athena.toml, defaults.toml or .github/workflows.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
