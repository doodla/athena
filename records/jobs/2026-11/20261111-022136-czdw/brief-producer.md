# Job 20261111-022136-czdw: Give OLY-8-05 a critic-challenge mechanism, test and spec entry (retry)

## Request (verbatim)

> try again on 20261029-204942-2zqw

## Reading (accepted by the user)

Retry of job 20261029-204942-2zqw: put the "search for ways the conclusions might be wrong" rule in the versioned critic role prompt (crates/athena-context/prompts/critic.md, frame.critic version bump), test it, and record it as OLY-8-05's mechanism in docs/spec.toml with measure ending `status: planning`. The earlier wrong path prompts/critic.md is replaced by the real path.

Assumptions:
- Goal and audience: the spec reader and verifier can see that Ares (the critic stage) is tasked to search for ways plan conclusions might be wrong, via a versioned prompt rule, a test and a spec mechanism.
- Code survey: read crates/athena-context/src/lib.rs (CRITIC_FRAME = frame.critic v1 from crates/athena-context/prompts/critic.md; budget tests at ~lines 402-541 that check frame wording), crates/athena-core/src/run/planning.rs (critic_round composes CONTRACT + CRITIC_FRAME at line 338), crates/athena-context/src/briefs/planner.rs (critic_brief) and briefs.rs tests. No new record or data file is added, so check_record_file/AnyRecord/round-trip tests are unaffected. The property is measured directly: the composed critic prompt text contains the challenge rule, and frame version is bumped.
- Job 20261029-204942-2zqw findings addressed: (1) challenge rule was in the per-job brief, not the versioned frame: fixed by moving it to crates/athena-context/prompts/critic.md and bumping frame.critic to version 2, removing the brief-only section from planner.rs; (2) behavior evidence not tied to commands: producer must run and cite cargo fmt --check, cargo clippy --all-targets, cargo test; (3) test check failed on drain::parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn, which passed locally and does not touch this change: producer first runs cargo test -p athena-cli --test drain on base and change and reports; if it fails on base too, it is flaky and not fixed here; (4) `status: planning` is an indirect, shared measure: kept, as the accepted reading chose it, and stated as indirect in the measure text.
- Its candidate files are handled: CHANGELOG.md kept (entry updated); docs/spec.toml kept; crates/athena-context/src/briefs.rs kept with the test retargeted at the frame; crates/athena-context/src/briefs/planner.rs modified to drop the brief-only challenge section (or kept as a pointer if the test needs it).
- Prompt headroom: critic set has 345 tokens left; the added rule must be under ~100 tokens. Update any budget figure in docs/system-facts.md only if it states critic size.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-8-05 has mechanism naming existing paths (crates/athena-context/prompts/critic.md and critic_round in crates/athena-core/src/run/planning.rs) instead of "none", keeps owner = "Ares", and its measure ends with `status: planning`; Grep confirms the paths exist and that `athena status` prints a planning line.
2. [content] crates/athena-context/prompts/critic.md contains a rule telling the critic to search for ways the plan's conclusions might be wrong (counterexamples, fragile assumptions, failure modes), and CRITIC_FRAME's version in crates/athena-context/src/lib.rs is bumped from 1 to 2; git diff shows both.
3. [behavior] A new test in crates/athena-context (briefs.rs or lib.rs tests) asserts the composed critic prompt contains the challenge rule; run `cargo test -p athena-context` to see it pass, and it fails when the rule is removed from critic.md.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` all pass, including the prompt token budget tests; run each command. `cargo test -p athena-cli --test drain` is run on the change and any failure is checked against the base.
5. [content] CHANGELOG.md has an entry for the change; no governance file (athena.toml, defaults.toml, workflows) changes; docs/system-facts.md changes only if it states a critic prompt budget figure that moved.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- Creating a new Ares role or prompt
- Changing other requirements' mechanisms
- Changes to athena.toml, defaults.toml or workflows
- Fixing the flaky drain test unless it fails on the base and blocks this job

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the critic prompt rule and test, not just file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and full cargo test required.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-204942-2zqw raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 3 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 12 cases in 12 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required survey step to every design job before the note is drafted. The producer writes an Existing code inventory: for each proposed change, the file and function that does it today, or a statement that none exists. For each rule attributed to a doc, it gives the quoted line. For the scoring...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261111-022136-czdw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-022136-czdw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml requirement OLY-8-05 has mechanism naming existing paths (crates/athena-context/prompts/critic.md and critic_round in crates/athena-core/src/run/planning.rs) instead of "none", keeps owner = "Ares", and its measure ends with `status: planning`; Grep confirms the paths exist and that `athena status` prints a planning line.
crates/athena-context/prompts/critic.md contains a rule telling the critic to search for ways the plan's conclusions might be wrong (counterexamples, fragile assumptions, failure modes), and CRITIC_FRAME's version in crates/athena-context/src/lib.rs is bumped from 1 to 2; git diff shows both.
A new test in crates/athena-context (briefs.rs or lib.rs tests) asserts the composed critic prompt contains the challenge rule; run `cargo test -p athena-context` to see it pass, and it fails when the rule is removed from critic.md.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked` all pass, including the prompt token budget tests; run each command. `cargo test -p athena-cli --test drain` is run on the change and any failure is checked against the base.
CHANGELOG.md has an entry for the change; no governance file (athena.toml, defaults.toml, workflows) changes; docs/system-facts.md changes only if it states a critic prompt budget figure that moved.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
