# Job 20261106-045349-f6x6: Add plan/critic convention for content mirroring a governing source

## Request (verbatim)

> Priority: high
> 
> Add a plan and critic convention: when a job writes content that summarises or mirrors a governing source (definitions taken from briefs or docs, cache keys taken from build inputs, seeded records), the plan must name that source and list every place in it that the content depends on. That means every mention of a term, or every compiled-in file for a cache key. At least one acceptance criterion must compare the written content against that list, not just check its structure. The critic must reject a plan that leaves this out. Test it with a plan fixture that is missing the list and is rejected. Add a CHANGELOG entry.
> 
> Introspections that record the repeated cause: 20261017-182254-my85, 20261025-230247-cm9r, 20261102-102715-564y, 20261106-030930-hb84
> 
> Acceptance:
> - The change is made as described: Content a job writes from a partial view of its governing source (a term's senses, a cache key's inputs) is caught only at the final verdict; 3 jobs in class f62n
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261017-182305-f62n.json

## Reading (accepted by the user)

Planner prompt requires a plan that writes content summarising or mirroring a governing source (term definitions, cache keys, seeded records) to name the source and list every dependent place (each term mention, each compiled-in file). The critic prompt rejects plans without it. A fixture plan missing the list is shown rejected, and CHANGELOG gets an entry. Audience: Athena's planner and critic stages, and maintainers judging failure class f62n.

Assumptions:
- Code survey: read crates/athena-context/prompts/planner.md and critic.md (plain prompts, 966 and 902 of 1300 tokens, so about 334 and 398 left), crates/athena-core/src/admission.rs (dependents_findings, the shape check run on plans at line ~310, with unit tests) and crates/athena-core/tests/refusal_corpus.rs. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Reading: a prompt convention plus a test. Planner prompt gains a rule (name the source, list every dependent place, and require one criterion comparing written content to that list). Critic prompt gains a rule to raise a major finding (verdict revise or reject) when it is missing. The fixture test is an admission-level plan fixture without the list, asserted to yield a major finding. If a prompt-only critic rule cannot be tested deterministically, the check is an admission.rs shape rule, which is the assumption I take.
- The existing `dependents` section and its shape check are the model; the new list is a separate plan section or an extension of it, the producer's design choice, kept small.
- Failure-class record records/failure-classes/2026-10/20261017-182305-f62n.json is a reference and is not modified; records/ never changes.
- athena.toml and defaults.toml need no change; the planner and critic prompt token budgets must not be exceeded.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md states that a job writing content that summarises or mirrors a governing source must name the source and list every place in it the content depends on (every mention of a term, every compiled-in file for a cache key), and must have at least one criterion comparing the written content to that list, not just its structure.
2. [content] crates/athena-context/prompts/critic.md instructs the critic to reject (major finding) a plan that writes such mirrored content without the named source and complete list, or without a comparing criterion.
3. [behavior] A test with a plan fixture that lacks the source list is rejected (a major finding or refusal naming the missing list), while the same fixture with the list is accepted; check with `cargo test`.
4. [content] CHANGELOG.md has an entry for the convention, citing failure class f62n.
5. [content] Prompt sets stay within their token budgets (planner 1300, critic 1300), shown by the existing prompt budget test in `cargo test`.
6. [behavior] No unrelated behavior changes: `cargo test` passes including existing tests, `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- docs/design/job-planning.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Editing records/ including the f62n failure-class record or the four introspections
- Changing athena.toml or defaults.toml
- Changes to reader, producer or verifier prompts
- Retroactively auditing past jobs' plans

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test for the rejection; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: fmt, clippy and test criteria run by the verifier.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by a tested plan check where possible, with the critic prompt as backup.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion checks a plan missing the list is rejected.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Prompt edits plus one fixture test and a CHANGELOG entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Prompt and changelog wording is plain and self-contained.
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
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

31 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-045349-f6x6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-045349-f6x6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/prompts/planner.md states that a job writing content that summarises or mirrors a governing source must name the source and list every place in it the content depends on (every mention of a term, every compiled-in file for a cache key), and must have at least one criterion comparing the written content to that list, not just its structure.
crates/athena-context/prompts/critic.md instructs the critic to reject (major finding) a plan that writes such mirrored content without the named source and complete list, or without a comparing criterion.
A test with a plan fixture that lacks the source list is rejected (a major finding or refusal naming the missing list), while the same fixture with the list is accepted; check with `cargo test`.
CHANGELOG.md has an entry for the convention, citing failure class f62n.
Prompt sets stay within their token budgets (planner 1300, critic 1300), shown by the existing prompt budget test in `cargo test`.
No unrelated behavior changes: `cargo test` passes including existing tests, `cargo clippy` is clean and `cargo fmt --check` passes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
