# Job 20261109-052134-xz0j: Retry: trim agent prompts and rebase budgets, keeping every hard rule

## Request (verbatim)

> try again on 20261109-035221-jj8g

## Reading (accepted by the user)

Retry of job 20261109-035221-jj8g: rewrite the role and capability prompts leaner and rebase the token budgets, restoring the verifier's cited-document rule and the other dropped guidance. Audience is the maintainer, who found the prompts mediocre; no hard rule may be removed without a stated new home.

Assumptions:
- Job 20261109-035221-jj8g is addressed finding by finding: (major) the verifier's cited-document rule is restored in verifier.md and pinned in a test anchor; (minor) 'Judge user-facing output as its audience would', 'personal data' in the harms list, 'Change nothing', and the reader's producer_model cheaper-vs-default guidance are restored or given a named home; (minor) docs/system-facts.md is kept as a deliverable because system_facts_budgets_match_computed forces its budget lines to follow; (nit) the contract size test stays an exact pin, updated to the new length, not loosened to an upper bound.
- The previous candidate's work is kept as the starting point where it passed: the rewritten prompts, docs/agent-reference.md, CHANGELOG.md and the budget edits. Only the findings above are corrected.
- Code survey: crates/athena-context/src/lib.rs (BudgetSet budgets, CONTRACT size test, frame anchors tests), crates/athena-context/src/briefs.rs (headroom rows), crates/athena-context/prompts/*.md and crates/athena-cap-*/prompts/*.md. A Grep of crates/ for 'cites a document' today finds only verifier.md line 6 (base). A Grep for FRAME_ANCHORS in the base finds nothing, so the anchor list is in the candidate; the producer must locate the equivalent pin list that the tests read.
- Before editing, the producer lists each prompt's hard rules (bullets and must/never lines). Afterward it states for every removed rule where it now lives. This is the prevention the introspection proposed, applied by hand in the decision entry.
- Prior prevention attempts (pre-dispatch headroom figure, running prompt tests after each cut) already exist; this job just follows them and adds the rule list and relocation table.
- Budgets rise nowhere without a matching cut elsewhere; restored rules must fit the verifier's 1300-token budget (the candidate was at 1153).
- docs/decisions.md gets an entry listing each removed passage with file and reason, claiming nothing beyond the diff. It is a docs file, not governance.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: all tests pass, including the athena-context budget tests (every set within its budget), headroom-row tests in crates/athena-context/src/briefs.rs, and system_facts_budgets_match_computed.
2. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.
3. [content] Grep of crates/athena-context/prompts/verifier.md finds the rule 'If the request cites a document, read it and compare the change with it' with 'request_reading_mismatch', and a test in crates/athena-context/src/lib.rs pins that phrase.
4. [content] Grep of the new prompts shows the other dropped guidance is present or relocated: 'Judge user-facing output as its audience would', 'personal data' in the verifier harms, 'Change nothing' for verifier/reader, and the producer_model cheaper-model guidance in reader.md.
5. [content] docs/decisions.md has an entry listing each removed prompt passage with its file, reason and new home (or 'restates the brief'), and every rule in the base prompts is either still in the prompts or listed there; git diff shows no hard rule removed without a stated home.
6. [content] The contract-size test in crates/athena-context/src/lib.rs remains an exact assert_eq! on the new length, not an upper bound.
7. [content] Each rewritten prompt that points to docs/agent-reference.md names a path that exists in the diff, and docs/agent-reference.md contains no claim (e.g. 'every rule is stated in the prompt') contradicted by the prompts.
8. [content] Outcome check: CHANGELOG.md has an entry with measured before/after token counts per prompt set, taken from the headroom rows or check_budget; no budget in lib.rs rises, and docs/system-facts.md budget lines match the computed values.

## Expected deliverables

- crates/athena-context/prompts/contract.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/agent-reference.md (create)
- docs/system-facts.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- The USD [budget] caps in athena.toml and defaults.toml
- Changing which roles exist or the pipeline's logic
- Study-capability behavior beyond prompt wording
- Building the proposed automated prompt-rule diff check as new code
- records/ and owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: before/after token count criterion
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: grep and test criteria check the rules present, not just files
- strangers: Code, commits, docs, and records are written for an outside reader. Here: decision entry and docs written for outside readers
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: restores every dropped rule and keeps the exact size pin
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused prompt-trim change
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261109-035221-jj8g

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> `git diff` shows the always-loaded prompt files under crates/athena-context/prompts/ and the capability prompts rewritten. For each role, the diff drops text that restates defaults, catalog or facts already in the brief, and keeps the role's job, output contract and hard rules.: All ten prompt files are rewritten. Most removals restate the schema or the brief (for example plan-field glosses, the forecast p_* glosses, fails_without, which the schema doc on outputs.rs:479 covers, and the snapshot and absence lines, which the verifier brief carries). But the verifier frame drops "If the request cites a document, read it and compare the change with it. Falling short of it is a request_reading_mismatch." That line is part of the output contract: it says when to set request_reading_mismatch. The brief does not restate it: a Grep of crates/ for 'cites a document' finds nothing. Job 20261014-192705-j4be added it on purpose (its reading-1.json and verification.json are in the evidence snapshot). Also dropped without replacement: 'Judge user-facing output as its audience would', 'personal data' in the list of harms, 'Change nothing', and the reader's producer_model choice guidance.

## Findings you must address

The verifier of job 20261109-035221-jj8g raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-context/prompts/verifier.md): The verifier's cited-document rule is dropped and was not moved anywhere Base line: "If the request cites a document, read it and compare the change with it. Falling short of it is a request_reading_mismatch." It is absent from the new frame and from docs/agent-reference.md, and a Grep of crates/ for 'cites a document' finds no brief that renders it. Job 20261014-192705-j4be added it on purpose: evidence/records/jobs/2026-10/20261014-192705-j4be/reading-1.json line 53 holds a criterion requiring exactly this text. It also makes docs/agent-reference.md line 5 ('Every rule a role must follow is stated in that role's prompt') false. Fix: restore the line in verifier.md and add it to FRAME_ANCHORS. Verifier is at 1153 of 1300, so there is room.

All of the verification's findings:

> major (crates/athena-context/prompts/verifier.md): The verifier's cited-document rule is dropped and was not moved anywhere Base line: "If the request cites a document, read it and compare the change with it. Falling short of it is a request_reading_mismatch." It is absent from the new frame and from docs/agent-reference.md, and a Grep of crates/ for 'cites a document' finds no brief that renders it. Job 20261014-192705-j4be added it on purpose: evidence/records/jobs/2026-10/20261014-192705-j4be/reading-1.json line 53 holds a criterion requiring exactly this text. It also makes docs/agent-reference.md line 5 ('Every rule a role must follow is stated in that role's prompt') false. Fix: restore the line in verifier.md and add it to FRAME_ANCHORS. Verifier is at 1153 of 1300, so there is room.
> minor (crates/athena-context/prompts/verifier.md; crates/athena-context/prompts/reader.md): Other verifier and reader guidance dropped without relocation Verifier: 'Judge user-facing output as its audience would', 'personal data' in the list of harms, and 'Change nothing' are removed; the contract only covers writing personal data and git state. Reader: 'producer_model: the cheaper model for small, mechanical changes; the default when judgment is needed' is removed; the brief lists the choices and a default, not when to pick the cheaper one. None is in docs/agent-reference.md.
> minor (docs/system-facts.md): docs/system-facts.md changed outside the expected deliverables It is not on the expected deliverables list. The edit is forced by system_facts_budgets_match_computed, it was planned, and the CHANGELOG explains it. It changes only the six budget lines.
> nit (crates/athena-context/src/lib.rs:372): The contract size test was loosened from an exact pin to an upper bound assert_eq!(CONTRACT.text.len(), 977) became assert!(CONTRACT.text.len() <= 977). This is reasonable for a rewrite, but the test no longer catches a silent change that keeps the text shorter.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

The work starts from the rejected candidate of job 20261109-035221-jj8g: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-052134-xz0j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-052134-xz0j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked`: all tests pass, including the athena-context budget tests (every set within its budget), headroom-row tests in crates/athena-context/src/briefs.rs, and system_facts_budgets_match_computed.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.
Grep of crates/athena-context/prompts/verifier.md finds the rule 'If the request cites a document, read it and compare the change with it' with 'request_reading_mismatch', and a test in crates/athena-context/src/lib.rs pins that phrase.
Grep of the new prompts shows the other dropped guidance is present or relocated: 'Judge user-facing output as its audience would', 'personal data' in the verifier harms, 'Change nothing' for verifier/reader, and the producer_model cheaper-model guidance in reader.md.
docs/decisions.md has an entry listing each removed prompt passage with its file, reason and new home (or 'restates the brief'), and every rule in the base prompts is either still in the prompts or listed there; git diff shows no hard rule removed without a stated home.
The contract-size test in crates/athena-context/src/lib.rs remains an exact assert_eq! on the new length, not an upper bound.
Each rewritten prompt that points to docs/agent-reference.md names a path that exists in the diff, and docs/agent-reference.md contains no claim (e.g. 'every rule is stated in the prompt') contradicted by the prompts.
Outcome check: CHANGELOG.md has an entry with measured before/after token counts per prompt set, taken from the headroom rows or check_budget; no budget in lib.rs rises, and docs/system-facts.md budget lines match the computed values.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
