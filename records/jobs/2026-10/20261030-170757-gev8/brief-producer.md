# Job 20261030-170757-gev8: Make refusal-rule plans run against a corpus of real request shapes

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intende
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-for-new-refusal-rule-untested-against-real-request-shapes-fails-planning-rounds (20261029-181827-rnrn)
> Cause: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intended behavior and hand-checking a few cases. It never executes them against a corpus of real past requests and readings. Specification gaps (un...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261029-181811-bs86 (job 20261029-111202-2bpc); prevention it proposed: Before the critic's first round, the planning stage for any job that adds or changes a reading-refusal or matching rule should have to do two things. First, list every input form the accepted reading names and map each to a parser case in t...
> Introspection 20261030-170739-hq09 (job 20261030-060751-5389); prevention it proposed: Add a planning-stage gate for any plan that adds or changes a refusal or matching rule. The gate runs the plan's stated rule, as a small executable spec, against a stored corpus of real past requests and readings. The corpus must include th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Retries 20261029-111202-2bpc and 20261030-060751-5389 (both failed in round 3 of planning). Root cause: refusal and matching rules (parts parser, stems, stop list) are specified by reasoning and hand-checked on a few cases, never executed against real request and reading shapes. Fix: add a stored corpus of real request shapes, with a test that runs every reading-acceptance check against it, and a planner rule that requires this for any plan adding or changing such a rule. Audience: the Athena planner, critic and maintainers.

Assumptions:
- Root cause named in the change description: refusal and matching rules are never executed against real request and reading shapes before they are planned or landed, so spec gaps (numbered N. and N) parts, stop-list/stem order, cited-but-unrelied jobs, brand-word symbols) surface only in critic rounds.
- Job 20261029-111202-2bpc: its reading-coverage code change (parts parser, stems) is not retried here; its findings (missing N./N) parts, stem/stop-list order on the Steward bullet, quoted-complaint exemption, rebuilt-text nit) become corpus cases and planner guidance, so the class is covered. No candidate files were recorded, so none are kept; its out_of_scope items carry over (no athena.toml, defaults.toml, workflows, records/, reader or verifier prompts).
- Job 20261030-060751-5389: its findings (citing a job is not relying on it, GitHub/env-var/let-binding not symbols, crate-relative paths, run_pin_tests.rs scope) become corpus cases and planner guidance; its candidate is dropped, no files kept; out_of_scope carries over (no planner/critic behavior changes beyond the one added planner rule, no athena.toml or defaults.toml).
- Code survey: read crates/athena-core/src/reading.rs (is_bullet at 2239, request_text_acceptance 2249, check_reading_acceptance 3013 as the gate every reading passes, check_governance_coverage 3089) and crates/athena-context/src/briefs.rs (READING_RULES at 1681, rule-section test near 3362) and crates/athena-context/prompts/planner.md (15 lines, 494 tokens of headroom). The corpus is a Rust test fixture, not a record file under records/, so check_record_file and AnyRecord are not involved; if a data file is added under crates/, the round-trip test must cover it. The property is measured directly by calling check_reading_acceptance with fixture requests and readings.
- The corpus holds request shapes seen in the cited jobs (Steward three-bullet acceptance, numbered 1./2./3. parts, (a)(b)(c), root-cause requests citing several rejected jobs, retry complaints) with expected accept/refuse outcomes. Text rebuilt from the brief is labeled so in the fixture.
- Prompt budget: only planner.md is edited (about 100 tokens added, under its 494 headroom); the reader and verifier prompts are untouched.

## Acceptance criteria

1. [content] The change description names the root cause (refusal and matching rules are not executed against real request and reading shapes before landing) and the fix sits in the shared gate test plus the planner rule, not in one job's check; checkable in the diff and CHANGELOG.md.
2. [content] crates/athena-context/prompts/planner.md gains a rule: a plan that adds or changes a refusal or matching rule must list the input forms it handles, run the rule against the stored corpus, and cite the results in the plan; the planner prompt stays within its 1300-token budget.
3. [behavior] A new corpus test in crates/athena-core (run with cargo test -p athena-core) feeds at least eight real request shapes through check_reading_acceptance, including Steward bullets, N./N)/(N) numbered parts, a root-cause request citing rejected jobs the reading never relies on, and a retry complaint, asserting the expected accept or refuse for each. It fails on the code before the change (no corpus exists) and passes after.
4. [behavior] A mechanism test fails if any check_* function called by check_reading_acceptance is not exercised by the corpus test, so a new refusal rule cannot land without corpus cases; run with cargo test -p athena-core.
5. [behavior] A briefs test in crates/athena-context (cargo test -p athena-context) asserts the planner brief contains the corpus rule; it fails before and passes after.
6. [behavior] Expected 0% of newly refused readings: the change adds tests and a planner rule and does not tighten any refusal rule, so the refusal rate is unchanged.
7. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass across the workspace.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Implementing the parts-coverage check or the prior-job existence check from jobs 20261029-111202-2bpc and 20261030-060751-5389; they are separate follow-ups that will use this corpus.
- Editing athena.toml, defaults.toml, workflows, records/, or the reader, critic, producer and verifier prompts.
- Candidate files of jobs 20261029-084645-sza7, 20261029-110953-bw7k and 20261026-042558-5p65 and similar rejected jobs: dropped, not kept.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Corpus and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New corpus and mechanism tests; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the planning and gate seam rather than one rule.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that every acceptance check is covered by the corpus.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the gate on request shapes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that real request shapes get the right outcome.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Fixtures state their source.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures contain no secrets or personal data.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: corpus, test and one planner rule.
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

The change description names the root cause (refusal and matching rules are not executed against real request and reading shapes before landing) and the fix sits in the shared gate test plus the planner rule, not in one job's check; checkable in the diff and CHANGELOG.md.
crates/athena-context/prompts/planner.md gains a rule: a plan that adds or changes a refusal or matching rule must list the input forms it handles, run the rule against the stored corpus, and cite the results in the plan; the planner prompt stays within its 1300-token budget.
A new corpus test in crates/athena-core (run with cargo test -p athena-core) feeds at least eight real request shapes through check_reading_acceptance, including Steward bullets, N./N)/(N) numbered parts, a root-cause request citing rejected jobs the reading never relies on, and a retry complaint, asserting the expected accept or refuse for each. It fails on the code before the change (no corpus exists) and passes after.
A mechanism test fails if any check_* function called by check_reading_acceptance is not exercised by the corpus test, so a new refusal rule cannot land without corpus cases; run with cargo test -p athena-core.
A briefs test in crates/athena-context (cargo test -p athena-context) asserts the planner brief contains the corpus rule; it fails before and passes after.
Expected 0% of newly refused readings: the change adds tests and a planner rule and does not tighten any refusal rule, so the refusal rate is unchanged.
cargo fmt --check, cargo clippy --all-targets and cargo test pass across the workspace.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
