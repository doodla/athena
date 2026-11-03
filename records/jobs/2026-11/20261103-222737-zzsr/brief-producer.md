# Job 20261103-222737-zzsr: Don't count needs-accepting re-reads toward the drain job limit

## Request (verbatim)

> try again on 20261103-193428-df2m

## Reading (accepted by the user)

Retry of job 20261103-193428-df2m: when a drain re-read ends as needs accepting, give its --job-limit slot back so the drain goes on to the next item. Audience: the operator running `athena drain`. This time the report must cite a command and its output for every behavior criterion, including the run against the old counter.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. `run_job` checks `st.started >= self.limits.max_jobs` (~line 1433) and then increments `st.started` (~1439). The `_ if run.needs_accepting` arm (~1537) never gives the slot back. `needs_accepting: true` comes only from the re-read result (~3572). Other `st.started` uses are at ~1889, 2008, 2555 and 2896. The tests are in crates/athena-cli/tests/drain.rs; the help text is in crates/athena-cli/src/main.rs. No record or data file is added.
- Job 20261103-193428-df2m is retried. Its candidate (decrement of `st.started` in the needs-accepting arm, docs, test, CHANGELOG) was sound. It was rejected only because behavior criterion 2 cited no command that ran (D43). So the same four files are kept as deliverables and the retry must record the commands. The nit about driving `Source::Reread` is optional; the stub's `needs_accepting` run result is accepted.
- The old-counter demonstration is done with git-free means the verifier can see: the test's assertions (b must land, stop != JobLimit) and the recorded `cargo test` output. The producer should run the new test with the decrement temporarily removed, record the failing output, then restore the fix.
- No governance files change; athena.toml, defaults.toml and workflows are untouched.
- The cost of the re-read still counts against the money budget.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with `max_jobs: 1` has item `a` end as needs accepting and item `b` runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1` and `stop != Stop::JobLimit`, and it passes.
2. [behavior] Fails without the fix, with a command that ran: with the `st.started` decrement in the needs-accepting arm of `run_job` removed, `cargo test -p athena-cli --test drain` fails on the new test (b not run, Stop::JobLimit). The recorded check output shows that failing run and the passing run with the fix.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the `_ if run.needs_accepting` arm of `run_job` returning the slot (`st.started` decremented under the state lock). The JobLimit check and the passed, held and failed outcomes are unchanged.
4. [behavior] Run `cargo test -p athena-cli --test drain`: `the_job_limit_stops_the_loop`, the urgent-queued-request test and the two existing needs-accepting tests still pass. `git diff` shows no assertion removed or weakened.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`; all are clean.
6. [content] The drain module docs, the `Limits::max_jobs` doc and the `--job-limit` help in crates/athena-cli/src/main.rs say a re-read that only needs accepting does not count toward the job limit. Grep finds that wording. CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the re-read's cost still counts against the budget.
- Changing how failed, held or budget-short outcomes count toward the job limit.
- Auto-accepting readings whose scope differs.
- Changes to athena.toml, defaults.toml, workflows or any other governance file.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test, plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name the test assertions and the commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the job counter in run_job, not in the one stop reason.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criterion 2 requires recorded runs with and without the fix.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, help and CHANGELOG are updated for an outside reader.
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
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

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

Run `cargo test -p athena-cli --test drain`. A new test with `max_jobs: 1` has item `a` end as needs accepting and item `b` runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1` and `stop != Stop::JobLimit`, and it passes.
Fails without the fix, with a command that ran: with the `st.started` decrement in the needs-accepting arm of `run_job` removed, `cargo test -p athena-cli --test drain` fails on the new test (b not run, Stop::JobLimit). The recorded check output shows that failing run and the passing run with the fix.
`git diff` of crates/athena-cli/src/drain.rs shows the `_ if run.needs_accepting` arm of `run_job` returning the slot (`st.started` decremented under the state lock). The JobLimit check and the passed, held and failed outcomes are unchanged.
Run `cargo test -p athena-cli --test drain`: `the_job_limit_stops_the_loop`, the urgent-queued-request test and the two existing needs-accepting tests still pass. `git diff` shows no assertion removed or weakened.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`; all are clean.
The drain module docs, the `Limits::max_jobs` doc and the `--job-limit` help in crates/athena-cli/src/main.rs say a re-read that only needs accepting does not count toward the job limit. Grep finds that wording. CHANGELOG.md has an entry.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
