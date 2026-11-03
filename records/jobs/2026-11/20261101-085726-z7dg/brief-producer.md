# Job 20261101-085726-z7dg: End drain cleanly when landed config needs a newer build

## Request (verbatim)

> After a landing that adds a new key to athena.toml, the drain's running binary re-reads the landed config, cannot parse the new key, records the landed job as failed and stops the run with 'repeated failure' even though the job landed. Make the drain treat a config that only the newer landed build understands as a reason to end the run cleanly (and say to reinstall), not as a failure of the job.
> - A test lands a job whose diff adds a key the running config schema does not know; the job is recorded as landed, not failed, and the run ends with a note to reinstall rather than 'repeated failure'.
> - Changes stay in crates/athena-core/src/run/ and the drain code; CHANGELOG.md gets an entry.

## Reading (accepted by the user)

When a landed job's athena.toml adds a key the running binary's schema rejects, the drain should keep the job recorded as landed and end the run with a "reinstall" note, not count a failure or stop on "repeated failure". Goal: a drain operator whose run lands a config-extending job sees a clean stop with reinstall guidance.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Stop enum ~l.923-985, land() ~l.1617, item loop ~l.1700-1762, ItemEnd::Failed -> Stop::RepeatedFailure ~l.2308, RealSteps::caps/priority_caps calling JobConfig::at(head) ~l.3248/3332), crates/athena-core/src/capability.rs (JobConfig::at parses athena.toml with Config::parse, strict on unknown fields), crates/athena-core/src/admission.rs (configuration rule, unknown field), and the file list of crates/athena-core/src/run/. The exact place where the post-landing config parse error becomes a failed job was not pinned down; the producer must locate it by running the new test first.
- The drain code lives in crates/athena-cli/src/drain.rs (and its integration tests in crates/athena-cli/tests/drain.rs); 'the drain code' in the request is read as that, plus crates/athena-core/src/run/ if the landing/record logic is there.
- The new stop is a new Stop variant (or reuses Stop::Rebuilt-style handling) with a label and meaning that tells the user to reinstall; Stop::ALL and display/help text in crates/athena-cli/src/display.rs and main.rs stay consistent with it, which counts as drain code.
- No new record or data file is added under crates/, so check_record_file/AnyRecord are unaffected; the job's recorded state stays landed.
- Only a config parse failure at the landed commit is treated this way; a config that fails to parse on its own (not a newer-schema issue) cannot be told apart cheaply, so the producer may treat any parse error of the landed head config, when the landing itself succeeded, as this case, and says so in a code comment.
- No change to athena.toml, defaults.toml or workflows; the test uses a fixture config with a key the schema lacks.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or the drain.rs unit tests) lands a job whose diff adds an unknown key to athena.toml; `cargo test -p athena-cli` passes, the test asserts the job's recorded outcome is landed (not failed), and the report's stop is not Stop::RepeatedFailure.
2. [behavior] In that test the run's final report/output contains a note telling the user to reinstall, and the run ends cleanly (no failure counted for the item); checked by `cargo test -p athena-cli`.
3. [behavior] A regression test shows a genuine failure (job that does not land twice) still ends with Stop::RepeatedFailure; `cargo test -p athena-cli` passes.
4. [content] The change touches only files under crates/athena-core/src/run/, crates/athena-cli/src/drain.rs, its stop display text, and their tests (git diff --stat), plus CHANGELOG.md.
5. [content] CHANGELOG.md has a new entry describing that a landed config the running build cannot parse ends the run cleanly with a reinstall note instead of a false repeated failure.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass (tested default).
7. [content] The new stop reason (if added) has a label and meaning and is listed wherever stops are enumerated (Stop::ALL, help text in crates/athena-cli/src/main.rs, display), verified by the existing test that every Stop has a meaning.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Making the config parser tolerant of unknown keys
- Automatically reinstalling or rebuilding the binary
- Changes to athena.toml, defaults.toml, or admission rules
- Changes to the athena-gate crate or other crates beyond the drain and run code

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test, clippy, fmt criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in how the drain classifies post-landing config parse errors, not a special case for one key.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the recorded landed state and stop reason by running the test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test runs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Reinstall note and CHANGELOG entry written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change confined to drain and run code.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a test rather than prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 40 cases in 40 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one repair round within the same job. The verifier is only start...
- 20 cases in 18 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make the disk guard and build cleanup one ordered step. Before the free-space check, and again on any low-disk hold, Athena removes the regenerable builds of held, interrupted and finished jobs, then measures free space. Only the unfreeable shortfall holds a job. Held jobs resume automatically once ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

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

A new test in crates/athena-cli/tests/drain.rs (or the drain.rs unit tests) lands a job whose diff adds an unknown key to athena.toml; `cargo test -p athena-cli` passes, the test asserts the job's recorded outcome is landed (not failed), and the report's stop is not Stop::RepeatedFailure.
In that test the run's final report/output contains a note telling the user to reinstall, and the run ends cleanly (no failure counted for the item); checked by `cargo test -p athena-cli`.
A regression test shows a genuine failure (job that does not land twice) still ends with Stop::RepeatedFailure; `cargo test -p athena-cli` passes.
The change touches only files under crates/athena-core/src/run/, crates/athena-cli/src/drain.rs, its stop display text, and their tests (git diff --stat), plus CHANGELOG.md.
CHANGELOG.md has a new entry describing that a landed config the running build cannot parse ends the run cleanly with a reinstall note instead of a false repeated failure.
`cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass (tested default).
The new stop reason (if added) has a label and meaning and is listed wherever stops are enumerated (Stop::ALL, help text in crates/athena-cli/src/main.rs, display), verified by the existing test that every Stop has a meaning.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
