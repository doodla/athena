# Job 20261103-015346-82n3: Add athena focus command and urgent/high priority caps in plan

## Request (verbatim)

> try again on 20261031-082442-92qg

## Reading (accepted by the user)

Retry of job 20261031-082442-92qg (dispatched, never verified, no findings): add `athena focus "<text>"` backed by a new focus record, shown in `athena status`, plus athena.toml caps (3 urgent, 10 high) that make `athena plan` demote excess items one tier in memory. Audience: the person running Athena, who needs priority to order work again.

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (PriorityRecord near line 525, AnyRecord enum near line 2623, with its validation and round-trip test), crates/athena-core/src/plan.rs (Plan::build at line 520), crates/athena-core/src/store.rs (priorities loaded as Vec<PriorityRecord>), crates/athena-core/src/reading.rs (builds NewFile::record(&AnyRecord::Priority(..))) and crates/athena-cli/tests/*.rs (e.g. drain.rs builds priority records). The change adds a Focus variant to AnyRecord, so the records validation allow-list, path and schema name and the AnyRecord round-trip test must cover it. Plan demotion is measured directly on Plan output, not through a stand-in.
- Job 20261031-082442-92qg is retried from the base commit. It was never verified and recorded no findings, so there is nothing to fix; its approach is kept as guidance. Its 12 changed files are all kept as deliverables, including crates/athena-cli/src/drain.rs, which the earlier reading missed.
- The focus record directory (records/focus/) does not exist at the base commit. This change adds it only at runtime, when a test or user runs `athena focus`. No file under records/ is created or edited by hand.
- The focus record holds text, set_by (git user.name) and an ordering counter or timestamp. The latest record is the current focus.
- The schema struct for the caps defaults to urgent=3 and high=10, so existing configs and reading digests do not change. Demotion happens in memory in plan only. A person's priority record always keeps its item at that tier.
- Demotion order: person-set items first, most recent first, then the rest in plan order. Urgent excess drops to high and counts in the high pass. High excess drops to normal.

## Acceptance criteria

1. [behavior] Run `cargo test`: the new crates/athena-cli/tests/focus.rs runs `athena focus "speed first"` in a temp repo, asserts a focus record file exists in the new focus records directory, that `athena focus` prints "speed first", and that `athena status` output contains it.
2. [behavior] Run `cargo test`: a test sets a second focus after the first and asserts `athena focus` and `athena status` print only the later text.
3. [behavior] Run `cargo test`: a plan test with 5 urgent items asserts exactly 3 are treated as urgent, a person-set urgent item (via a priority record) is among them, and the plan output has one demotion line for each of the 2 demoted items.
4. [behavior] Run `cargo test`: a test asserts the high cap of 10 demotes excess high items to normal, a person's priority record still wins for its own item, and no request or priority record file is modified by planning.
5. [content] athena.toml sets the urgent cap to 3 and the high cap to 10, and crates/athena-data/src/config.rs declares matching schema fields with those defaults; Grep shows both.
6. [content] crates/athena-data/src/records.rs adds the focus record to AnyRecord with validation (non-empty text and set_by), a path and a schema name. The AnyRecord round-trip test covers it, and check_record_file accepts it.
7. [content] CHANGELOG.md has an entry describing `athena focus`, the status line and the priority caps.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/focus.rs (create)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- The steward triage pass that uses the focus (a separate request)
- Re-tiering or rewriting existing queue items or priority records
- Editing anything under records/ by hand
- Changes to defaults.toml, .github/workflows and prompt files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for focus and caps; cargo test and clippy -D warnings must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run focus, status and plan and check their output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the plan demotes the excess items and focus persists, not that the wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to focus and the caps.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verified by running the tests and commands.
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

Run `cargo test`: the new crates/athena-cli/tests/focus.rs runs `athena focus "speed first"` in a temp repo, asserts a focus record file exists in the new focus records directory, that `athena focus` prints "speed first", and that `athena status` output contains it.
Run `cargo test`: a test sets a second focus after the first and asserts `athena focus` and `athena status` print only the later text.
Run `cargo test`: a plan test with 5 urgent items asserts exactly 3 are treated as urgent, a person-set urgent item (via a priority record) is among them, and the plan output has one demotion line for each of the 2 demoted items.
Run `cargo test`: a test asserts the high cap of 10 demotes excess high items to normal, a person's priority record still wins for its own item, and no request or priority record file is modified by planning.
athena.toml sets the urgent cap to 3 and the high cap to 10, and crates/athena-data/src/config.rs declares matching schema fields with those defaults; Grep shows both.
crates/athena-data/src/records.rs adds the focus record to AnyRecord with validation (non-empty text and set_by), a path and a schema name. The AnyRecord round-trip test covers it, and check_record_file accepts it.
CHANGELOG.md has an entry describing `athena focus`, the status line and the priority caps.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
