# Job 20261101-000734-04qa: Add design existing-code reading check with reader rule line

## Request (verbatim)

> try again on 20261027-054248-rjf1

## Reading (accepted by the user)

Retry of job 20261027-054248-rjf1 (which failed in planning, no candidate, nothing kept). Add `check_design_existing_code` to the reading gate, refusing design readings that lack an `Existing code:` assumption naming existing repo paths and a criterion requiring an Existing code section, and teach the rule in READING_RULES so a correct reading passes on the first read. Audience: the reader agent and the maintainers who judge design jobs.

Assumptions:
- Job 20261027-054248-rjf1 failed in planning with no candidate and no changed files; its work is dropped and this retry starts from the base. Addressing its critique: (major) READING_RULES gets a sixth line, because BudgetSet::Reader covers only CONTRACT, READER_FRAME, defaults and catalog (athena-context/src/lib.rs:279-283), so the reader-prompt headroom claim of 3 tokens does not apply to it; (minor) the criterion must match the phrase 'existing code section' plus 'reuse' or 'extend', so 'the existing code in section 3 is unchanged' and 'no survey needed' are refused; (minor) paths the reading lists as create deliverables are ignored when deciding whether the Existing code: assumption names a path; (nit) the doc comment has no blank lines and the signature is exactly `pub fn check_design_existing_code(`.
- Code survey: crates/athena-core/src/reading.rs (refusing_check at ~3042 with step! calls, check_code_survey at 3375, names_crates_path at 3303, the doc-comment test at ~7958), crates/athena-core/tests/refusal_corpus.rs (base_reading, repo_files, INPUT_FORMS, code_survey_refusal_is_declared_intended), crates/athena-context/src/briefs.rs (READING_RULES [&str; 5] at 1694, line tests near 3389-3412), crates/athena-context/src/lib.rs (BudgetSet::Reader). No record or data file is added, so check_record_file and AnyRecord do not apply. The property is measured by running the real refusing_check on constructed readings.
- Existing code: the check reads only the reading (deliverable paths, assumptions, criteria); it accepts a path that starts with crates/ or docs/ and is not a create deliverable. It does not search the repo, since the reader has no such step.
- Design readings with a question or a decline, and readings of other classes, are not affected by the new check.
- The new rule line is at most 220 bytes, READING_RULES stays under 2400 bytes, and the added text has no banned words or email-like text.
- Expected refusal rate: 5% of newly refused readings, since only design-class readings are affected.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: unit tests in crates/athena-core/src/reading.rs show a design reading with an `Existing code:` assumption naming a path that is not a create deliverable, and a criterion requiring an Existing code section that reuses or extends it, is accepted. They fail without the change.
2. [behavior] Run `cargo test -p athena-core`: a design reading whose criterion contains 'existing code' and 'section' but says the opposite (for example 'no survey needed' or 'the existing code in section 3 is unchanged') is refused.
3. [behavior] Run `cargo test -p athena-core`: a design reading whose `Existing code:` assumption names only the file the job creates is refused, as is one with no such assumption.
4. [behavior] Run `cargo test -p athena-core`: readings of class code, tests, docs and config, and design readings with a question or a decline, are still accepted.
5. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a new case built from a design reading written from request text alone is refused by check_design_existing_code, is declared intended, and the guard that every step! has a refusing case passes.
6. [behavior] Run `cargo test -p athena-context`: READING_RULES has six lines, one naming `Existing code:` and `check_design_existing_code`; the line-count, per-line 220-byte, section under 2400 bytes, every-check-has-a-rule-line and budget tests pass.
7. [content] Grep of crates/athena-core/src/reading.rs shows `step!(check_design_existing_code(reading));` in refusing_check, and a doc comment without blank lines naming the root cause (no inventory of existing code before a design is accepted) and jobs 20261026-043137-zc0v and 20261026-065046-0c1d; the refusal text names the `Existing code:` label and what to list.
8. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings.
9. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass.
10. [content] git diff shows no change to athena.toml, defaults.toml, .github/workflows, docs/design/jev-code-health.md or docs/decisions.md.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing or reverting the landed work of jobs 20261026-043137-zc0v and 20261026-065046-0c1d (docs/design/jev-code-health.md, docs/decisions.md); it is kept
- Changing jev.rs, labeled.rs or the score-jev command
- Any model-call or repository-search step in the reader; the check reads only the reading
- Changing athena.toml, defaults.toml or workflows
- A general check that resolves token-budget claims in assumptions

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in athena-core and athena-context.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and corpus tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on accepted and refused readings.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the reading gate and in the reader brief, for the whole design class.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A refusing check plus a corpus guard enforce the rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
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

Run `cargo test -p athena-core`: unit tests in crates/athena-core/src/reading.rs show a design reading with an `Existing code:` assumption naming a path that is not a create deliverable, and a criterion requiring an Existing code section that reuses or extends it, is accepted. They fail without the change.
Run `cargo test -p athena-core`: a design reading whose criterion contains 'existing code' and 'section' but says the opposite (for example 'no survey needed' or 'the existing code in section 3 is unchanged') is refused.
Run `cargo test -p athena-core`: a design reading whose `Existing code:` assumption names only the file the job creates is refused, as is one with no such assumption.
Run `cargo test -p athena-core`: readings of class code, tests, docs and config, and design readings with a question or a decline, are still accepted.
Run `cargo test -p athena-core --test refusal_corpus`: a new case built from a design reading written from request text alone is refused by check_design_existing_code, is declared intended, and the guard that every step! has a refusing case passes.
Run `cargo test -p athena-context`: READING_RULES has six lines, one naming `Existing code:` and `check_design_existing_code`; the line-count, per-line 220-byte, section under 2400 bytes, every-check-has-a-rule-line and budget tests pass.
Grep of crates/athena-core/src/reading.rs shows `step!(check_design_existing_code(reading));` in refusing_check, and a doc comment without blank lines naming the root cause (no inventory of existing code before a design is accepted) and jobs 20261026-043137-zc0v and 20261026-065046-0c1d; the refusal text names the `Existing code:` label and what to list.
The change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass.
git diff shows no change to athena.toml, defaults.toml, .github/workflows, docs/design/jev-code-health.md or docs/decisions.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
