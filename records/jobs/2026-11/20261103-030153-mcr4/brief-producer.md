# Job 20261103-030153-mcr4: Check landed athena.toml against the installed build, in every drain

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: A job can add a config field together with the code that reads it. The build that lands it, and the
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: self-landing-config-field-unknown-to-installed-build (20261025-010251-8gqz)
> Cause: A job can add a config field together with the code that reads it. The build that lands it, and the drain that keeps running afterward, is the older one, which does not know the field. Nothing checks that the installed binary accepts the la...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261101-085638-8zkm (job 20261101-061448-sdya); prevention it proposed: Two things together. (1) Before landing a job whose diff adds or changes an athena.toml field, parse the post-landing athena.toml with the currently installed binary. If it rejects the file, hold the landing, or land it and then rebuild and...
> Introspection 20261103-030136-jnn4 (job 20261101-085726-z7dg); prevention it proposed: Two code changes together. First, before dispatching or landing a job that adds a config field, check that the running build accepts the key. If not, land the job on a path that rebuilds the installed build first, or hold the job with a cle...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a job can land a new athena.toml field while the installed build (which keeps running) cannot parse it, and nothing checks that at the landing. Job 20261101-085726-z7dg added a drain-only post-landing check; this change moves the check into the landing outcome and fixes that job's open finding so the whole class ends the run cleanly with a reinstall note.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (State::land at ~1643, Steps::config_after_landing at ~528 and ~3576, Stop::NeedsReinstall at ~954, finish/rate at ~2338, set_stop keeps only the first reason), crates/athena-core/src/capability.rs (JobConfig::at parses athena.toml at a commit), crates/athena-cli/tests/drain.rs (real-steps tests). The entry is the drain's land step under the landing lock; the config is read with the running build's schema directly via JobConfig::at at the new head, not a stand-in. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not involved.
- Job 20261101-085726-z7dg (landed, pass): its Stop::NeedsReinstall and post-landing config read are KEPT. Its findings: (1) set_stop keeps only the first reason, so under drain_concurrent an earlier stop hides the reinstall note and rating still reads the unreadable config - FIXED by keying skip-rating and the note on State.unreadable_config.is_some(), not on the stop reason; (2) any parse error counts as 'newer schema' - kept, the error text stays in the note; (3) the real-steps test not shown to fail on base - addressed by a test that fails without the change.
- Job 20261101-061448-sdya (landed, pass): its work is kept unchanged. Its two minor findings (duplicate origin parser in gh/client.rs vs gh/early.rs; full CI wait when the CI-branch push fails) are unrelated to this failure class and are not addressed; its changed files are excluded in out_of_scope.
- Goal and audience: the operator running athena drain; a landed job that adds a config key must never be recorded failed, and the run must end with a reinstall note. Also the failure of the same class when a non-first worker or a non-drain path reads the landed config.
- Default applied: the check is enforced by mechanism (a test), not prose; no email addresses or personal data are written anywhere.

## Acceptance criteria

1. [content] The CHANGELOG.md entry and code comments name the root cause: the landing never checks that the running build can read the config it lands, and the fix is where the landed head is read, not at one job.
2. [behavior] A test in crates/athena-cli/tests/drain.rs or drain.rs lands a job whose athena.toml diff adds a key the running schema rejects, while another worker has already set a stop reason: the job is recorded landed (not failed), rating is skipped, the reinstall note with the parse error is reported, and the run does not end as 'repeated failure'. It fails on the base tree and passes with the change. Run: cargo test -p athena-cli drain.
3. [behavior] A second test covers the class, not one job: each Steps/lander path that reports Landed (stub and real steps) yields the same reinstall outcome for any unreadable landed config, and a readable landed config still continues normally. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test all pass.
5. [content] Files from job 20261101-061448-sdya (CI wait, gh/client.rs, routing.rs, config.rs, athena.toml) are unchanged in the diff (git diff).

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- All work of job 20261101-061448-sdya: crates/athena-core/src/gh/client.rs, routing.rs, run/mod.rs, run/tests.rs, run/verification.rs, crates/athena-data/src/config.rs, athena.toml (kept, not changed)
- Making the config parser tolerant of unknown keys
- Automatically reinstalling or rebuilding the binary
- Changes to athena.toml, defaults.toml, admission rules, or .github/workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the run ending cleanly with job landed, via tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix keyed on the unreadable-config state, not the stop reason or one job.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email addresses or personal data written.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 40 cases in 40 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one repair round within the same job. The verifier is only start...
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

The CHANGELOG.md entry and code comments name the root cause: the landing never checks that the running build can read the config it lands, and the fix is where the landed head is read, not at one job.
A test in crates/athena-cli/tests/drain.rs or drain.rs lands a job whose athena.toml diff adds a key the running schema rejects, while another worker has already set a stop reason: the job is recorded landed (not failed), rating is skipped, the reinstall note with the parse error is reported, and the run does not end as 'repeated failure'. It fails on the base tree and passes with the change. Run: cargo test -p athena-cli drain.
A second test covers the class, not one job: each Steps/lander path that reports Landed (stub and real steps) yields the same reinstall outcome for any unreadable landed config, and a readable landed config still continues normally. Run: cargo test.
cargo fmt --check, cargo clippy --all-targets and cargo test all pass.
Files from job 20261101-061448-sdya (CI wait, gh/client.rs, routing.rs, config.rs, athena.toml) are unchanged in the diff (git diff).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
