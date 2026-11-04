# Job 20261104-171135-911m: Lift agent caps in athena.toml with job_usd covering their sum

## Request (verbatim)

> try again on 20261104-140950-gzp9

## Reading (accepted by the user)

Retry of job 20261104-140950-gzp9: apply the nine cap values to athena.toml, raise [budget] job_usd to 50.00 so it covers the $47.00 required by the new caps (2 + 5×(3+3) + 10 + 5), and update the config.rs tests that pin the old values. Audience: the Athena owner and verifier.

Assumptions:
- Job 20261104-140950-gzp9 failed: its athena.toml-only edit (kept here as a deliverable) left job_usd 40.00 below the 47.00 required, and four config.rs tests pin old values. This retry fixes both: raises job_usd and updates those tests. The earlier out_of_scope on config.rs is dropped because the tests read athena.toml; the schema code itself stays unchanged.
- Code survey: crates/athena-data/src/config.rs. Config::budget_required = reader_usd + max_rounds*(planner_usd+critic_usd) + producer_usd + verifier_usd; check_budget refuses job_usd below it. Tests at ~lines 855-936 and ~1163 pin 0.75, max_rounds 3, job_usd 12.0 and the replace string 'job_usd = 12.00'. Other crates' tests (athena-cli/tests, athena-core) may read athena.toml and are checked by running the workspace tests.
- job_usd = 50.00 is chosen (47.00 needed plus headroom); the user's other eight values are as originally stated. CHANGELOG.md was a prior deliverable but is not required; listed with low probability.
- The change adds no new refusal rule; it only changes values, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [content] athena.toml [budget] reads reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00 (at least 47.00), producer_timeout_s 5400, verifier_timeout_s 3600; [job_plan] planner_usd 3.00, critic_usd 3.00, max_rounds 5; git diff of athena.toml shows only those value substitutions, no key added or removed, reader_timeout_s, check_timeout_s, planner_timeout_s, critic_timeout_s unchanged.
2. [behavior] Run `cargo test --locked -p athena-data`; it passes, including the repository-config budget tests updated to the new values and a test that Config::parse(REPO_CONFIG).check_budget() is ok.
3. [behavior] Run `cargo test --locked --workspace`; it passes, so no other test pinning old athena.toml values remains broken; and `cargo clippy --locked --workspace --all-targets` is clean.
4. [content] Changes to crates/athena-data/src/config.rs are confined to the test module (git diff shows no change outside #[cfg(test)]); the schema and check logic are untouched.
5. [content] job_usd (50.00) is no less than reader_usd + max_rounds*(planner_usd+critic_usd) + producer_usd + verifier_usd = 47.00, so admission's budget check passes on the new config.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-core/src/admission.rs (modify)

## Out of scope

- Changing config schema or non-test logic in crates/athena-data/src/config.rs
- Changing keys not listed (reader_timeout_s, check_timeout_s, planner_timeout_s, critic_timeout_s)
- Editing defaults.toml or workflows
- Adding a new command to compute minimum job_usd

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check admission budget passes and tests run, not just that values were edited.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the job_usd-below-sum invariant and the pinned tests rather than only the nine values.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single config change plus the tests that pin it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

27 more qualifying classes are left out.

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

athena.toml [budget] reads reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00 (at least 47.00), producer_timeout_s 5400, verifier_timeout_s 3600; [job_plan] planner_usd 3.00, critic_usd 3.00, max_rounds 5; git diff of athena.toml shows only those value substitutions, no key added or removed, reader_timeout_s, check_timeout_s, planner_timeout_s, critic_timeout_s unchanged.
Run `cargo test --locked -p athena-data`; it passes, including the repository-config budget tests updated to the new values and a test that Config::parse(REPO_CONFIG).check_budget() is ok.
Run `cargo test --locked --workspace`; it passes, so no other test pinning old athena.toml values remains broken; and `cargo clippy --locked --workspace --all-targets` is clean.
Changes to crates/athena-data/src/config.rs are confined to the test module (git diff shows no change outside #[cfg(test)]); the schema and check logic are untouched.
job_usd (50.00) is no less than reader_usd + max_rounds*(planner_usd+critic_usd) + producer_usd + verifier_usd = 47.00, so admission's budget check passes on the new config.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
