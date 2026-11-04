# Job 20261103-231422-8dzf: Retry 193428-df2m: re-reads needing acceptance skip the job limit

## Request (verbatim)

> try again on 20261103-193428-df2m once the verifier can see the 'fails without the fix' run; the code in candidate b3e381696a71 was correct and only that evidence was missing.
> Depends on 20261103-231410-asf2

## Reading (accepted by the user)

Retry of rejected job 20261103-193428-df2m (code was correct; only the 'fails without the fix' evidence was missing). Re-apply the change so a drain re-read ending as needs accepting gives its --job-limit slot back, with a regression test at job limit 1. It depends on job 20261103-231410-asf2, which lets the verifier see Athena's own revert-and-run record. For operators of `athena drain`.

Assumptions:
- Code Survey: Code survey: crates/athena-cli/src/drain.rs: run_job takes a slot at `st.started += 1` (~line 1439) after the `st.started >= max_jobs` check (~1433); the `_ if run.needs_accepting` arm (~1537) is where the slot is returned. `needs_accepting` comes only from the re-read result (~3572 in the test stub). crates/athena-cli/tests/drain.rs holds the drain tests and its stub. The property is measured directly through the drain report (jobs outcomes, landed, stop reason), not through a stand-in. No record or data file is added, so records validation is unaffected. No refusal rule is added.
- Job 20261103-193428-df2m: its candidate's four files (CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-cli/tests/drain.rs) are kept as deliverables and redone from base, because earlier changes were discarded. The fix is unchanged: a decrement of st.started under the state lock in the needs-accepting arm, plus doc/help wording. Its one major finding (behavior criterion 2 had no recorded command) is addressed by the dependency 20261103-231410-asf2: Athena runs the planned revert demonstration and shows the verifier the record. Its nit (the test uses the stub rather than Source::Reread) is accepted as is.
- Job 20261103-231410-asf2 is a dependency, not part of this change. This job relies on the verifier receiving the recorded failing run of the new test against the old counter, and it does not touch the mechanism that provides it.
- The criteria below name a plan with one demonstration: revert the decrement in drain.rs, then run the new test.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with max_jobs 1 has item a end as needs accepting and item b runnable. It asserts jobs[0].outcome == "needs accepting", landed == 1 and stop != Stop::JobLimit, and it passes.
2. [behavior] Fails without the fix: with the decrement of st.started in the needs-accepting arm of run_job reverted on a scratch copy, `cargo test -p athena-cli --test drain` fails on the new test. The verifier sees this as Athena's recorded revert run under 'Checks Athena ran'.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the needs-accepting arm in run_job giving the slot back (`st.started` decremented under the state lock). The JobLimit check and the failed, held and passed paths are unchanged.
4. [behavior] `cargo test -p athena-cli --test drain` still passes the_job_limit_stops_the_loop and the existing needs-accepting tests, and `git diff` shows no removed or weakened asserts.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` pass.
6. [content] Grep of crates/athena-cli/src/drain.rs and crates/athena-cli/src/main.rs for the --job-limit help and the Limits::max_jobs doc finds wording that a re-read that only needs accepting does not count toward the limit. CHANGELOG.md has a matching entry.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the re-read cost still counts against the budget.
- Changing how failed, held or budget-short outcomes count toward the job limit.
- Auto-accepting readings whose scope differs.
- The mechanism that shows the verifier the revert run (job 20261103-231410-asf2).
- Governance files: athena.toml, defaults.toml and workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the drain outcome and stop reason by running the tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the job counter, not at the one symptom.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, the same four files as the earlier attempt.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261103-193428-df2m raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 2 is unmet: its evidence must cite commands that ran (D43) it cites no command

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
- 9 cases in 7 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Make "fails without the fix" a check that Athena runs itself. For a bug-fix job, Athena would run the new or changed tests against the base tree's production code, with the job's test files overlaid, and record the failing output as a named check. It would then run them on the final tree. The verifi...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

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

Run `cargo test -p athena-cli --test drain`. A new test with max_jobs 1 has item a end as needs accepting and item b runnable. It asserts jobs[0].outcome == "needs accepting", landed == 1 and stop != Stop::JobLimit, and it passes.
Fails without the fix: with the decrement of st.started in the needs-accepting arm of run_job reverted on a scratch copy, `cargo test -p athena-cli --test drain` fails on the new test. The verifier sees this as Athena's recorded revert run under 'Checks Athena ran'.
`git diff` of crates/athena-cli/src/drain.rs shows the needs-accepting arm in run_job giving the slot back (`st.started` decremented under the state lock). The JobLimit check and the failed, held and passed paths are unchanged.
`cargo test -p athena-cli --test drain` still passes the_job_limit_stops_the_loop and the existing needs-accepting tests, and `git diff` shows no removed or weakened asserts.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` pass.
Grep of crates/athena-cli/src/drain.rs and crates/athena-cli/src/main.rs for the --job-limit help and the Limits::max_jobs doc finds wording that a re-read that only needs accepting does not count toward the limit. CHANGELOG.md has a matching entry.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
