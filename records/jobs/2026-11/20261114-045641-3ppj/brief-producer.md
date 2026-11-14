# Job 20261114-045641-3ppj: Make matching-helper changes visible to the refusal corpus mechanism

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intende
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: plan-for-new-refusal-rule-untested-against-real-request-shapes-fails-planning-rounds (20261029-181827-rnrn)
> Cause: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intended behavior and hand-checking a few cases. It never executes them against a corpus of real past requests and readings. Specification gaps (un...
> Seen 11 time(s) in 11 jobs. The introspections that record it:
> Introspection 20261029-181811-bs86 (job 20261029-111202-2bpc); prevention it proposed: Before the critic's first round, the planning stage for any job that adds or changes a reading-refusal or matching rule should have to do two things. First, list every input form the accepted reading names and map each to a parser case in t...
> Introspection 20261030-170739-hq09 (job 20261030-060751-5389); prevention it proposed: Add a planning-stage gate for any plan that adds or changes a refusal or matching rule. The gate runs the plan's stated rule, as a small executable spec, against a stored corpus of real past requests and readings. The corpus must include th...
> Introspection 20261030-171844-a950 (job 20261030-170805-jz9t); prevention it proposed: Add a plan-time step for any plan that adds or changes a matching, parsing or refusal rule. The planner must list the rule's exemption and match windows as concrete input and output cases, taken from real past reading phrasings. Athena woul...
> Introspection 20261103-001617-5g6g (job 20261102-161126-hn31); prevention it proposed: Require every plan for a new refusal or matching rule to include accept and refuse cases drawn from real past readings and requests. Include the compliant shape the refusal message recommends (a fix stage that checks its speed-up by timing ...
> Introspection 20261105-073335-5945 (job 20261105-051400-dzr4); prevention it proposed: Before a plan for a new refusal or matching rule goes to the critic, run a check that applies the plan's draft matcher to every request and changelog entry in the repo's refusal corpus and records the matched and refused counts and examples...
> Introspection 20261107-182355-vdrj (job 20261107-110049-k8c2); prevention it proposed: Before the planning critic runs on any job that adds or changes a reading-refusal or matching rule, run the draft matcher automatically over the corpus of real past requests, including Athena's generated root-cause requests, and over the jo...
> Introspection 20261111-064803-f4da (job 20261029-212839-tcjb); prevention it proposed: Before a reading or plan for a new matching or refusal rule is accepted, Athena should run the draft matcher over the real input shapes. These are the bullets steward.rs generates, including the long-title "as described" case, and the Accep...
> Introspection 20261111-072628-k5y7 (job 20261111-064834-q749); prevention it proposed: When a plan adds a refusal or matching rule, Athena should run a mechanical replay before the critic rounds. The replay should run the new rule over every existing test helper and scenario that reaches it, plus recorded real request shapes,...
> Introspection 20261113-112803-sms6 (job 20261030-060839-1njk); prevention it proposed: Before planning ends, run any new reading-refusal rule automatically over a corpus of recorded past requests and accepted readings, and add the false refusals to the plan. The corpus should include deferral wording, alternatives, optional p...
> Introspection 20261113-112837-4mph (job 20261030-060843-f4s2); prevention it proposed: Add a planning step that, for any new reading-refusal rule, lists every caller of the checking function by search, including CLI commands the replay invokes, and requires each in the dependents list with a test. Also add a mechanical repair...
> Introspection 20261113-193229-qz92 (job 20261113-190407-4318); prevention it proposed: Before the plan is accepted, run the planned reading rule against the reader guide's own example wording and the recorded real code readings, and have the planner state the refused share per part. Block the plan if any guide-following form ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Goal: planners and reviewers of a new refusal or matching rule (parts parser, stems, stop list) get a mechanism that exposes untested helpers, not just new `check_*` steps. D148 and refusal_corpus.rs already landed the corpus; its stated Limit is that a helper, stem list or stop list adds no step, so I add a test for that gap. Jobs 20261029-111202-2bpc, 20261030-060751-5389 and 20261030-170805-jz9t are cited evidence; their failed plans and candidates are not kept.

Assumptions:
- Root cause: refusal and matching rules were specified by reasoning and hand-checked, never executed against stored real request and reading shapes. D148 covers new `check_*` steps only; its Limit names helpers, `is_bullet`, `request_text_acceptance`, stem lists and stop lists as uncovered. This job closes that Limit.
- Code survey: read crates/athena-core/tests/refusal_corpus.rs (INPUT_FORMS, cases(), corpus_covers_the_listed_input_forms, reading_source, function_body, acceptance_check_names), crates/athena-core/src/reading/checks.rs (check_reading_acceptance at 492, check_governance_coverage at 624), crates/athena-core/src/reading.rs, crates/athena-context/src/briefs/planner.rs (REFUSAL_CORPUS_RULE) and docs/decisions.md D148. Entry point is refusing_check; the test is a unit-level source scan plus gate calls, adds no record or data file, and the property is measured by running the gate on corpus cases directly.
- Existing code: crates/athena-core/tests/refusal_corpus.rs (reading_source, function_body, INPUT_FORMS, cases) and docs/decisions.md D148 are extended, not replaced.
- Jobs 20261029-111202-2bpc, 20261030-060751-5389 and 20261030-170805-jz9t: their findings (N./N) parts, stop list and stem order, cited-but-not-relied jobs, brand words as symbols, exemption windows) are already cases or input forms in the corpus; the new test keeps them from regressing. Their candidates changed no recorded files, so no work is kept; their listed deliverables are dropped except the corpus file, CHANGELOG.md and docs/decisions.md, which this job also changes.
- Changing no refusal rule: the change refuses 0% of past accepted readings (expected 0% of newly refused readings). The prior jobs' production rules (parts-coverage, prior-job existence) are not re-attempted.
- Information, not refusal: the planner already sees REFUSAL_CORPUS_RULE; this job only adds a mechanism test and a decision record, so no prompt token budget is used.

## Acceptance criteria

1. [content] crates/athena-core/tests/refusal_corpus.rs gains a test that scans the reading sources (reading_source) for matching helpers (is_bullet, request_text_acceptance and each stop-list or stem-list const or fn) and fails when one is not named by a corpus case form tag or a pinned-helper list in that file; the diff shows the test and the list.
2. [behavior] A test fails without the change and passes with it: a second test feeds the scanner a source with one invented stop-list const and asserts the scan reports it uncovered, while the real source passes. Verifier runs `cargo test -p athena-core --test refusal_corpus`.
3. [behavior] A reader of the corpus (a planner) can check the outcome: each corpus case for the stop list and stems states its source job and expected accept or refuse outcome, and the full corpus passes when run with `cargo test -p athena-core --test refusal_corpus`.
4. [content] docs/decisions.md ends with a new appended decision that names the root cause (matching helpers invisible to the corpus mechanism), amends D148's Limit, states expected 0% of newly refused readings, and has a Provenance bullet. No existing entry is edited.
5. [content] CHANGELOG.md records the job and the root cause; athena.toml, defaults.toml, workflows and records/ are unchanged.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass; Athena's recorded checks on the final tree count as evidence.

## Expected deliverables

- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Retrying the parts-coverage and prior-job-existence checks of jobs 20261029-111202-2bpc, 20261030-060751-5389 and 20261030-170805-jz9t; their candidate files are dropped, not kept
- Changing planner, critic or reader prompt files (reader has 2 tokens of headroom)
- Changing athena.toml, defaults.toml, workflows or records/
- Tightening any refusal rule or running the planner's draft matcher at plan time (the planner has Read, Grep and Glob only)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the scan failing on an invented uncovered helper and passing on the real source.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The gap is closed by a test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045641-3ppj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045641-3ppj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/tests/refusal_corpus.rs gains a test that scans the reading sources (reading_source) for matching helpers (is_bullet, request_text_acceptance and each stop-list or stem-list const or fn) and fails when one is not named by a corpus case form tag or a pinned-helper list in that file; the diff shows the test and the list.
A test fails without the change and passes with it: a second test feeds the scanner a source with one invented stop-list const and asserts the scan reports it uncovered, while the real source passes. Verifier runs `cargo test -p athena-core --test refusal_corpus`.
A reader of the corpus (a planner) can check the outcome: each corpus case for the stop list and stems states its source job and expected accept or refuse outcome, and the full corpus passes when run with `cargo test -p athena-core --test refusal_corpus`.
docs/decisions.md ends with a new appended decision that names the root cause (matching helpers invisible to the corpus mechanism), amends D148's Limit, states expected 0% of newly refused readings, and has a Provenance bullet. No existing entry is edited.
CHANGELOG.md records the job and the root cause; athena.toml, defaults.toml, workflows and records/ are unchanged.
cargo fmt --check, cargo clippy and cargo test pass; Athena's recorded checks on the final tree count as evidence.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
