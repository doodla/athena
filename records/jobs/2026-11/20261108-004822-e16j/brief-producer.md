# Job 20261108-004822-e16j: Retry: trim change/study verifier and study producer prompts safely

## Request (verbatim)

> try again on 20261024-232057-r4pj

## Reading (accepted by the user)

Retry of job 20261024-232057-r4pj: trim brief-repeating text from the change verifier, study verifier and study producer prompt sets so each is at least 100 tokens under 1300 (≤1200), keeping every phrase tests pin (e.g. the five verifier-frame snapshot phrases at crates/athena-context/src/lib.rs:531-541), and record only verified claims in decisions/changelog/system-facts.

Assumptions:
- Job 20261024-232057-r4pj is addressed finding by finding: (1) the blocker/test failure from removing snapshot wording is fixed by keeping verbatim all five phrases asserted by the_verifier_frame_limits_snapshot_runs_to_the_snapshot ("a snapshot, not the real repository", "no records/, no owner files, no history", "a result about the snapshot only", "say so explicitly in evidence and findings", "never report it as a result on the real repository") and cutting elsewhere, e.g. the change and study verifier/producer capability prompts, which already free most of the room; (2) the D166 misstatement is fixed by writing the new decisions entry only from what was run and by not claiming the producer brief states that Athena reruns checks; (3) the behavior criteria name the commands to be run (cargo test --locked, clippy) so evidence cites commands that ran.
- Code survey: read crates/athena-context/src/lib.rs (prompt constants and frame tests such as the verifier-frame snapshot test at line 531 and the critic frame test), and the prompts under crates/athena-context/prompts, crates/athena-cap-change/prompts and crates/athena-cap-study/prompts; prompt sizes are measured by crates/athena-cli budgets test; no records or data files are added, so check_record_file/AnyRecord are not involved. The producer should grep crates/ for string literals in tests that assert prompt text before each cut.
- The earlier candidate's files are kept as a starting point only in intent: CHANGELOG.md, both verifier.md files, athena-context producer.md and verifier.md, decisions.md, system-facts.md are all deliverables again; the retry starts from base, so those edits are redone, with the athena-context verifier.md snapshot bullet restored to its pinned wording.
- The study producer measured 1257 tokens, not the 1276 in the request; the target is at most 1200 for the change verifier, study verifier and study producer, budgets unchanged at 1300.
- Audience: the repo maintainers and the verifier who check that prompts stay lean and tests pass.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes and the printed sizes for the change verifier, study verifier and study producer are each at most 1200, with budget values unchanged.
2. [behavior] Run `cargo test --locked -p athena-context --lib` then `cargo test --locked` for the whole workspace: all pass, including the_verifier_frame_limits_snapshot_runs_to_the_snapshot, and `cargo clippy --locked --all-targets` is clean.
3. [content] `git diff` shows no test file (*.rs under tests/ or #[cfg(test)] code) modified; the diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md.
4. [content] crates/athena-context/prompts/verifier.md still contains verbatim the five phrases asserted at crates/athena-context/src/lib.rs:534-538, and every other phrase asserted by tests for the edited prompts is still present (checkable with Grep against test literals).
5. [content] docs/decisions.md has a new entry listing each removed passage or group with its file and why it was removable, making no claim about tests or brief contents beyond what the diff and the test run show; a reader can match each bullet to the diff.
6. [content] CHANGELOG.md has an entry describing the prompt trim and the new headroom; docs/system-facts.md budget lines equal the computed sizes (the system_facts_budgets_match_computed test passes).
7. [content] No prompt rule is added or changed in meaning; removed text only repeats brief content or backs no current rule or test; nothing secret or personal is committed (standing defaults: outcome-check, behavior-evidence, small, public, strangers).

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/contract.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Changing any budget value, any test file, athena.toml or defaults.toml
- Adding new rules or instructions to the prompts
- Trimming the reader, planner or critic prompts beyond what shared frames imply
- Adding new checks or tooling for prompt-edit jobs (a prior prevention already tried)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require running budgets, workspace tests and clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check sizes and test passes, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt trim.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Named commands for verifier.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261024-232057-r4pj raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 2 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (crates/athena-context/prompts/verifier.md (snapshot bullet); test at crates/athena-context/src/lib.rs:531): Removing the snapshot wording from the verifier frame breaks an existing test, so the workspace test suite fails. `cargo test --locked -p athena-context --lib the_verifier_frame_limits_snapshot_runs_to_the_snapshot` panics: "verifier frame lacks: no records/, no owner files, no history". The test also requires "a result about the snapshot only" and "say so explicitly in evidence and findings", and both were removed too. The CI test check failed for the same reason. To fix it, keep these phrases and cut elsewhere. The cap-change verifier and producer frame changes already free most of the needed room.
> major (docs/decisions.md D166 (Decision, Tradeoff)): D166 calls removed text repetition or rule-less when a test enforces it, and says no test was affected. D166 says "no brief or test changed" and lists the snapshot clause as removable. The snapshot wording backs the test above, so the decision record misstates what the change does.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261108-004822-e16j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-004822-e16j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes and the printed sizes for the change verifier, study verifier and study producer are each at most 1200, with budget values unchanged.
Run `cargo test --locked -p athena-context --lib` then `cargo test --locked` for the whole workspace: all pass, including the_verifier_frame_limits_snapshot_runs_to_the_snapshot, and `cargo clippy --locked --all-targets` is clean.
`git diff` shows no test file (*.rs under tests/ or #[cfg(test)] code) modified; the diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md.
crates/athena-context/prompts/verifier.md still contains verbatim the five phrases asserted at crates/athena-context/src/lib.rs:534-538, and every other phrase asserted by tests for the edited prompts is still present (checkable with Grep against test literals).
docs/decisions.md has a new entry listing each removed passage or group with its file and why it was removable, making no claim about tests or brief contents beyond what the diff and the test run show; a reader can match each bullet to the diff.
CHANGELOG.md has an entry describing the prompt trim and the new headroom; docs/system-facts.md budget lines equal the computed sizes (the system_facts_budgets_match_computed test passes).
No prompt rule is added or changed in meaning; removed text only repeats brief content or backs no current rule or test; nothing secret or personal is committed (standing defaults: outcome-check, behavior-evidence, small, public, strangers).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
