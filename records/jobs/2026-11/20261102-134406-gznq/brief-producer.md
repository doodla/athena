# Job 20261102-134406-gznq: Check declared build-input sets against the real compiled-in inputs

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> Introspection 20261102-102715-564y (job 20261031-172040-kq9v); prevention it proposed: Add a code check, run in the repo tests and before verification, that computes the athena binary's real input set (every file the build compiles in, including everything xtask input_files should list) and fails if the hashFiles globs in eve...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a declared list (CI cache globs, input_files, BUILD_PATHS, ontology terms) is checked for shape but never against the source it summarizes, so errors surface only at the final verdict. Add a code check that derives the athena binary's real input set (every include_str!/include_bytes! target under crates/ plus BUILD_PATHS) and fails if the ci.yml athena-bin hashFiles globs or xtask input_files miss any. Audience: Athena maintainers and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (BUILD_PATHS at line 20 is the repo's definition of build inputs), crates/athena-context/src/lib.rs and briefs.rs (include_str! of prompts and docs/system-facts.md), and grep of include_str! across crates/*/src. Entry point is a new test in crates/athena-core (cargo test), run before verification; it reads files from the workspace root, scans sources for include_str!/include_bytes! targets, resolves them relative to the source file, and compares with BUILD_PATHS and the hashFiles globs parsed from .github/workflows/ci.yml. No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected; the allow-list is untouched. The property is measured directly: the real include set versus the declared globs, not a stand-in.
- Prior job 20261031-172040-kq9v: its finding (athena-bin key omitted prompts and docs/system-facts.md) already appears fixed in the repo's ci.yml (globs crates/**, xtask/**, docs/system-facts.md). This change keeps that work as is and adds the mechanism the finding's minor item asked for, so the hand-kept list cannot drift again. The stamp-on-non-CI-build, restored build-commit and dirty-tree findings are not addressed here; they are minor, local, and listed out of scope.
- Prior job 20261017-173508-0wtc (ontology term contradicts BRIEF.md): addressed at the class level only through the shared mechanism of checking a declared list against its governing source; the ontology content itself is left as is and not touched here. Prior job 20261025-214509-y179 (speed target not met, no baseline): its code changes (secrets.rs memo, checks.rs poll schedule) are kept untouched; the measure-first stage is out of scope because it is a pipeline change needing its own job.
- The check parses hashFiles patterns with a simple glob match (prefix/** and exact file names), enough for the current ci.yml; a pattern form it cannot parse fails loudly rather than passing.

## Acceptance criteria

1. [content] The change description (CHANGELOG.md entry) names the root cause: declared input lists were checked for structure but not against the real compiled-in inputs, citing jobs 20261031-172040-kq9v, 20261017-173508-0wtc and 20261025-214509-y179 and saying how each prior result is addressed or why it is out of scope.
2. [content] A new test under crates/athena-core derives the input set from every include_str!/include_bytes! target under crates/*/src plus BUILD_PATHS, and asserts each is covered by the athena-bin hashFiles globs in .github/workflows/ci.yml (all jobs) and by BUILD_PATHS; it reads ci.yml rather than a copy.
3. [behavior] Negative test: run `cargo test -p athena-core` on in-memory fixtures where a glob list omits a prompt .md file or docs/system-facts.md; the check returns a failure naming the missing path. The same fixtures with the full list pass. The test fails against the pre-fix key from job kq9v (only *.rs and Cargo.toml globs).
4. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: all pass on the real repository, so the current ci.yml satisfies the new check.
5. [content] The check is a refusal rule over repository files, not over readings; no replay of past readings is affected (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-core/tests/build_inputs.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Changing docs/ontology.toml, crates/athena-cli/tests/ontology.rs or any study (job 0wtc work stays dropped/unchanged)
- Changing crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs or the test-speed goal (job y179 work kept as is)
- Changing xtask/src/main.rs, xtask/tests/policy.rs or the setup.sh stamp and dirty-tree behavior (job kq9v work kept as is)
- A measure-first stage for speed-target jobs
- Changing athena.toml, defaults.toml, tool pins or action SHAs

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the shared cause: declared lists never checked against their real source.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a test, not prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test plus fmt, clippy and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Negative fixture test checks the failure happening.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check; ontology and pipeline changes excluded.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check written in Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG names cause and jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 18 cases in 16 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle owned by the build's state. Once a producer finishes and its changes are committed, the regenerable writer build is removed before the free-space check, and the guard excludes the held job's own build from the space it requires. A held job on low disk automatically recla...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

24 more qualifying classes are left out.

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

The change description (CHANGELOG.md entry) names the root cause: declared input lists were checked for structure but not against the real compiled-in inputs, citing jobs 20261031-172040-kq9v, 20261017-173508-0wtc and 20261025-214509-y179 and saying how each prior result is addressed or why it is out of scope.
A new test under crates/athena-core derives the input set from every include_str!/include_bytes! target under crates/*/src plus BUILD_PATHS, and asserts each is covered by the athena-bin hashFiles globs in .github/workflows/ci.yml (all jobs) and by BUILD_PATHS; it reads ci.yml rather than a copy.
Negative test: run `cargo test -p athena-core` on in-memory fixtures where a glob list omits a prompt .md file or docs/system-facts.md; the check returns a failure naming the missing path. The same fixtures with the full list pass. The test fails against the pre-fix key from job kq9v (only *.rs and Cargo.toml globs).
Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: all pass on the real repository, so the current ci.yml satisfies the new check.
The check is a refusal rule over repository files, not over readings; no replay of past readings is affected (expected 0% of newly refused readings).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
