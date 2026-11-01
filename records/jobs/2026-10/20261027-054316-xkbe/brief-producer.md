# Job 20261027-054316-xkbe: Compare failing tests to the base so job-caused failures are separate

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing separates a failure the job caused from one already on the base, or from a clash with a conc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: failing-check-blocks-all-criteria-unjudged-no-baseline (20261017-075355-rwrh)
> Cause: Nothing separates a failure the job caused from one already on the base, or from a clash with a concurrent config and test change. The check stage has no baseline run and no mapping from tests to the change. A failing check also ends the jo...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-075336-m1y1 (job 20261017-044408-wcwy); prevention it proposed: Add a code check at admission and before handover. Admission would run the full test suite on the base commit and record which tests fail. Before handover the producer must run the same suite, and the system would compare the two. A test th...
> Introspection 20261017-184719-0s4h (job 20261017-182453-a5sf); prevention it proposed: Before the producer starts, run the repo's full check suite on the base commit and record which checks already fail. At verification, compare the failures after the job to that baseline. Failures that already existed are reported separately...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the baseline records only whether a whole check (e.g. "test") failed on the base, so a failure is either wholly pre-existing or wholly the job's; a different test failing, or a clash with a concurrent config/test change, cannot be told apart (jobs 20261017-044408-wcwy and 20261017-182453-a5sf). Fix it where it originates by recording failing test names in the baseline and comparing per test at verification.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (is_pre_existing, mark_pre_existing, VerdictInputs.baseline), crates/athena-core/src/run/checks.rs (take_baseline runs the configured checks on the base snapshot via crates/athena-core/src/checks.rs baseline_runs), crates/athena-core/src/run/verification.rs (verifier always runs; failed check fails the final verdict unless pre-existing) and crates/athena-data/src/records.rs (Baseline::failed_before matches a check by name only; Baseline::validate). The Baseline record is already in AnyRecord, so a new optional field on CheckRun/Baseline must keep serde defaults so old records still parse and the record round-trip test and check_record_file still pass.
- Cause as read: pre-existing detection is per check name, so (a) a check failing on the base masks a new, different test failure in the same check, and (b) a failure from a clash with a concurrent change is judged as the job's own. The name-only comparison is the origin; the fix is test-level.
- Fix shape: parse failing test names from cargo test output (the 'failures:' list) into CheckRun for baseline and candidate runs; is_pre_existing treats a failed check as pre-existing only if every failing test also failed on the base; failures new relative to the base are reported separately in the finding with their names. Where no test names can be parsed, behaviour falls back to the current name-level rule.
- Jobs 20261017-044408-wcwy and 20261017-182453-a5sf: their candidate changes are not retried or kept here; their findings are addressed by the general mechanism (failing test names classified against the base), not by editing their files. The athena.toml / docs/design/job-planning.md clashes are not edited.
- Audience: the verifier and the user reading a rejected job, who need to see which failures the job caused. Check names in the stage are the existing ones (fmt, clippy, test).

## Acceptance criteria

1. [content] The change description names the root cause (baseline compares whole checks by name, with no per-test comparison) and the fix is in crates/athena-data/src/records.rs and crates/athena-core/src/verify.rs / checks.rs, not in any one job's files; check with git diff.
2. [behavior] A unit test in crates/athena-core/src/verify.rs fails without the change and passes with it: a baseline where test X failed and a candidate where tests X and Y fail is NOT pre-existing and the verdict fails naming Y; where only X fails it is pre-existing. Run cargo test -p athena-core.
3. [behavior] A parser test shows failing test names are extracted from real cargo test output of the shapes quoted in the two jobs (the 'failures:' list), and output with no list falls back to the name-level rule. Run cargo test -p athena-core.
4. [behavior] A Baseline/CheckRun record written before the change (no test-name field) still parses and validates; the records round-trip test and check_record_file pass. Run cargo test.
5. [behavior] The finding or note for a failed check separates tests already failing on the base from tests newly failing, so the user can see what the job caused; covered by a test on the produced finding text. Run cargo test -p athena-core.
6. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test --locked pass.
7. [content] No governance file (athena.toml, defaults.toml, workflows) is changed; CHANGELOG.md has an entry describing the root cause.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Re-landing the auditor role or the ontology file from the two earlier jobs; their candidate files (auditor.rs, ontology.toml, tests, docs) are dropped, not kept
- Fixing the max_concurrency test or the banned-word wording in docs, which are separate instances
- Changing athena.toml, defaults.toml or workflows
- Rebasing or merging concurrent changes automatically

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix the per-check-name baseline comparison itself
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: tests for the parser and the verdict; fmt, clippy, test pass
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: all changes in Rust
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria run cargo test on the new tests
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: enforced in verdict code, not prose
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused change to baseline comparison
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
- 11 cases in 10 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle. Before the checks stage, it should automatically delete the finished producer build folder and any other regenerable artifacts (verify-side builds, caches) once they are no longer needed. If free space is still below the guard, it should hold the job instead of failing ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

22 more qualifying classes are left out.

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

The change description names the root cause (baseline compares whole checks by name, with no per-test comparison) and the fix is in crates/athena-data/src/records.rs and crates/athena-core/src/verify.rs / checks.rs, not in any one job's files; check with git diff.
A unit test in crates/athena-core/src/verify.rs fails without the change and passes with it: a baseline where test X failed and a candidate where tests X and Y fail is NOT pre-existing and the verdict fails naming Y; where only X fails it is pre-existing. Run cargo test -p athena-core.
A parser test shows failing test names are extracted from real cargo test output of the shapes quoted in the two jobs (the 'failures:' list), and output with no list falls back to the name-level rule. Run cargo test -p athena-core.
A Baseline/CheckRun record written before the change (no test-name field) still parses and validates; the records round-trip test and check_record_file pass. Run cargo test.
The finding or note for a failed check separates tests already failing on the base from tests newly failing, so the user can see what the job caused; covered by a test on the produced finding text. Run cargo test -p athena-core.
cargo fmt --check, cargo clippy --all-targets and cargo test --locked pass.
No governance file (athena.toml, defaults.toml, workflows) is changed; CHANGELOG.md has an entry describing the root cause.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
