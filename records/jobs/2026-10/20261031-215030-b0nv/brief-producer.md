# Job 20261031-215030-b0nv: Scan only deliverable paths and content criteria for boundary words

## Request (verbatim)

> Priority: high
> Stop refusing a reading over a boundary word in its prose; keep the check where the word can reach a file
> 
> The reading check (`check_reading_acceptance` in crates/athena-core/src/reading.rs) refuses any reading whose free text holds a boundary word. Readings are records, and records are outside the boundary scan (xtask/tests/policy.rs skips records/). The risk the check guards against is a word reaching a tracked file, and the writer already gets the word list and runs the scan itself (D73). Meanwhile readings keep being refused for prose alone: one request was refused six times on 2026-10-31, and another twice on 2026-10-30.
> 
> Keep the check on deliverable paths and on acceptance criteria that tell the writer what text to write into a file. Drop it for the summary, assumptions, out-of-scope and other prose fields. Update the module docs and docs/decisions.md with the reason.
> 
> Acceptance:
> - A test shows a reading whose assumptions or summary contain a boundary word is accepted.
> - A test shows a reading whose deliverable path, or an acceptance criterion quoting text to write into a file, contains a boundary word is still refused.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Narrow `check_boundary_and_repo_files` in reading.rs so boundary words are refused only in deliverable paths and in content-kind acceptance criteria (text to write into a file). Title, summary, assumptions, out_of_scope and other prose fields stop being scanned. Audience: readers, whose readings keep being refused for prose alone, and the maintainers who read the decision log.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: `check_reading_acceptance` (about line 3014) runs `check_boundary_and_repo_files` last (about line 2771), which scans every field from `reading_text_fields` (about line 2709) and each deliverable path with `athena_context::briefs::boundary_scan`. The repository-file checks (create/modify/delete against the base commit) stay unchanged. This is an in-memory text check, so the existing unit tests in reading.rs measure it directly; it adds no record or data file, so `check_record_file`, `AnyRecord` and the round-trip test are not involved.
- Assumed meaning of 'acceptance criteria that tell the writer what text to write into a file': criteria with kind `content` are scanned, and criteria with kind `behavior` are not, since content criteria describe file text and behavior criteria name commands to run.
- The title, summary, decline_reason, question, departures, hard_reason, urgent_reason, assumptions, out_of_scope and defaults_applied are no longer scanned. The request-text scan (around line 2257, on the user's own request) is a different check and stays.
- Existing tests in reading.rs that expect refusal for summary, assumptions and similar fields (e.g. `a_reading_whose_summary_holds_a_boundary_word_is_refused_and_the_word_is_named`, `a_boundary_word_in_any_text_field_or_path_is_refused_with_the_field_named`) are rewritten to match the new scope. Banned words in tests are still built from pieces so the file passes the boundary scan.
- The latest decision in docs/decisions.md is D152, so the new entry is D153 and cites D73. The CHANGELOG.md entry goes under the unreleased heading.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading.rs builds a reading with a boundary word in its summary and in an assumption, and `check_reading_acceptance` returns Ok; `cargo test -p athena-core` passes.
2. [behavior] A unit test shows a reading whose deliverable path holds a boundary word is still refused with the path and word named, and a reading with a `content` acceptance criterion holding a boundary word is still refused naming `acceptance_criteria`; `cargo test -p athena-core` passes.
3. [behavior] A unit test shows a `behavior` criterion, out_of_scope, title and defaults_applied text holding a boundary word are accepted; `cargo test -p athena-core` passes.
4. [behavior] The change narrows a refusal rule, so it refuses fewer readings and none newly: expected 0% of newly refused readings.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] The doc comments of `check_boundary_and_repo_files` and the module docs in crates/athena-core/src/reading.rs state which fields are scanned, which are not, and why: readings are records outside the boundary scan, and the writer already gets the word list (D73).
7. [content] docs/decisions.md has a new D153 entry giving the reason, the alternatives considered and the cited refusals (six on 2026-10-31, two on 2026-10-30), and CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the boundary word list or the boundary scan in athena_context::briefs.
- Changing the request-text scan that refuses a user's request holding a boundary word.
- Changing the repository-file checks for deliverable actions.
- Changing xtask/tests/policy.rs, athena.toml, defaults.toml or any prompt file.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New and rewritten unit tests; fmt, clippy -D warnings and the test suite must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests show the outcome: prose with a boundary word is accepted and a file-bound word is still refused.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that exercise acceptance and refusal.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs and the decision entry state the reason for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is in Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is narrowed to where a word can reach a file, not special-cased per refused request.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 16 cases in 14 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena one disk lifecycle. Before the low-disk check and before any hold, the writer's regenerable build is removed once the writer has finished and its output is committed or recorded. The guard then measures free space after that removal. Where checks come from GitHub, the guard does not coun...
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

A new unit test in crates/athena-core/src/reading.rs builds a reading with a boundary word in its summary and in an assumption, and `check_reading_acceptance` returns Ok; `cargo test -p athena-core` passes.
A unit test shows a reading whose deliverable path holds a boundary word is still refused with the path and word named, and a reading with a `content` acceptance criterion holding a boundary word is still refused naming `acceptance_criteria`; `cargo test -p athena-core` passes.
A unit test shows a `behavior` criterion, out_of_scope, title and defaults_applied text holding a boundary word are accepted; `cargo test -p athena-core` passes.
The change narrows a refusal rule, so it refuses fewer readings and none newly: expected 0% of newly refused readings.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
The doc comments of `check_boundary_and_repo_files` and the module docs in crates/athena-core/src/reading.rs state which fields are scanned, which are not, and why: readings are records outside the boundary scan, and the writer already gets the word list (D73).
docs/decisions.md has a new D153 entry giving the reason, the alternatives considered and the cited refusals (six on 2026-10-31, two on 2026-10-30), and CHANGELOG.md has an entry for the change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
