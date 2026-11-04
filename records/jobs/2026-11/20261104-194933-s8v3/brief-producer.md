# Job 20261104-194933-s8v3: Record lifted budget caps; keep job_usd 50 so tests pass unchanged

## Request (verbatim)

> try again on 20261104-171104-yb47

## Reading (accepted by the user)

Retry of job 20261104-171104-yb47, whose candidate was rejected for editing six test and code files to pin job_usd 60. This reading keeps job_usd at 50, which already covers the 47.00 stage-cap reserve, so no crates/ file changes. It confirms the lifted caps in athena.toml and records them in CHANGELOG.md for the owner.

Assumptions:
- Goal: the owner (4 Nov) wants Athena's per-agent budgets and timeouts lifted; the audience is the owner and the verifier checking athena.toml and the changelog.
- Job 20261104-171104-yb47 addressed: its major finding (six files outside scope edited: config.rs, admission.rs, reading.rs, run/tests.rs, run_pin_tests.rs, drain.rs) is fixed by dropping all of those edits. Its job_usd 60 and test-pin edits are dropped, not kept. Its CHANGELOG.md and athena.toml edits are kept only as redone on the base.
- Read from athena.toml in the snapshot: reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, producer_timeout_s 5400, verifier_timeout_s 3600, planner_usd 3.00, critic_usd 3.00, max_rounds 5 already hold the target values; job_usd is 50.00. Reserve is 2+10+5+5*(3+3)=47, so 50 covers it and admission accepts.
- Tests pin job_usd = 50.00 (admission.rs, run_pin_tests.rs, reading.rs, config.rs tests), so raising it to 60 would force test edits. Keeping 50 meets the goal without that. The earlier 60 came from a 40 refusal that no longer applies.
- If the snapshot's athena.toml differs from the values above, the producer corrects only value lines; if a test or admission rule then refuses, it stops and reports rather than editing crates/.
- The change is a CHANGELOG.md Unreleased entry naming each raised cap with old and new value; the old values are the ones in the earlier request.

## Acceptance criteria

1. [content] git diff file list contains only CHANGELOG.md, plus athena.toml only if a value line needed correcting; nothing under crates/ (including config.rs, admission.rs, reading.rs, tests) changes.
2. [content] athena.toml [budget] has reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00, producer_timeout_s 5400, verifier_timeout_s 3600, and [job_plan] has planner_usd 3.00, critic_usd 3.00, max_rounds 5; no key is added or removed (Grep athena.toml).
3. [content] job_usd (50) is at least the stage-cap reserve 2+10+5+5*(3+3)=47, checkable by arithmetic from athena.toml, so admission accepts.
4. [content] CHANGELOG.md Unreleased section names each lifted cap with its old and new value (reader 0.50->2.00, producer 3.00->10.00, verifier 2.00->5.00, job 12.00->50.00, producer timeout 3000->5400, verifier timeout 2400->3600, planner 0.75->3.00, critic 0.75->3.00, max_rounds 3->5) and says it follows the owner's 4 Nov direction.
5. [behavior] Run `cargo test --locked` with no test file edited; it passes, including athena-data repo_config tests and the admission budget tests.

## Expected deliverables

- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Raising job_usd to 60 or any change to tests that pin job_usd 50.00
- Any change under crates/, including crates/athena-data/src/config.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/tests.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-cli/tests/drain.rs
- Adding or removing config keys
- Changing other budget values such as reader_timeout_s or check_timeout_s

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the caps are in place, admission arithmetic holds, and cargo test passes unchanged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; no code or test edits.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Drops the job_usd 60 choice that forced test edits, rather than patching tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry lists old and new values for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and reads the diff file list.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261104-171104-yb47 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-data/src/config.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/tests.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-cli/tests/drain.rs): Six files outside the expected deliverables (athena.toml, CHANGELOG.md) were changed, including config.rs, which the brief lists as out of scope; the request said 'Files changed: athena.toml only'. `git diff HEAD~1 --stat` lists 8 files. All six extra edits are test-only (pins of 50.0 → 60.0, replace() literals, reader-cost fixtures 31.5 → 41.5, critic rows +10). They are needed for `cargo test` to pass, because the base tests pin job_usd = 50.00. They still go beyond the scope the user stated and accepted. The fix is to amend scope and criterion 3 to allow updating the tests that pin these values, not to change the code.
> major: The reading does not match the request, so passing criteria do not show the request was met The request and the accepted reading say the change touches only athena.toml and that config.rs needs no change. Neither noticed that the repository's tests pin the shipped job_usd: config.rs tests assert 50.0, admission/run tests replace the literal "job_usd = 50.00", and the drain test asserts caps.run 50.0. Raising job_usd to 60 therefore cannot pass criterion 4 (`cargo test`) without editing test files, so criteria 3 and 4 conflict as written. The schema really is unchanged. The scope should have said "athena.toml, CHANGELOG.md, and updates to the tests that pin these values".

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

git diff file list contains only CHANGELOG.md, plus athena.toml only if a value line needed correcting; nothing under crates/ (including config.rs, admission.rs, reading.rs, tests) changes.
athena.toml [budget] has reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00, producer_timeout_s 5400, verifier_timeout_s 3600, and [job_plan] has planner_usd 3.00, critic_usd 3.00, max_rounds 5; no key is added or removed (Grep athena.toml).
job_usd (50) is at least the stage-cap reserve 2+10+5+5*(3+3)=47, checkable by arithmetic from athena.toml, so admission accepts.
CHANGELOG.md Unreleased section names each lifted cap with its old and new value (reader 0.50->2.00, producer 3.00->10.00, verifier 2.00->5.00, job 12.00->50.00, producer timeout 3000->5400, verifier timeout 2400->3600, planner 0.75->3.00, critic 0.75->3.00, max_rounds 3->5) and says it follows the owner's 4 Nov direction.
Run `cargo test --locked` with no test file edited; it passes, including athena-data repo_config tests and the admission budget tests.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
