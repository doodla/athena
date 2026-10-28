# Job 20261027-160842-avd0: Refuse readings whose criteria rest on the producer's report

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Criteria are written without a defined list of what the verifier can observe (the diff, the repo sna
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: criterion-requires-evidence-verifier-cannot-observe (20261027-011728-xrka)
> Cause: Criteria are written without a defined list of what the verifier can observe (the diff, the repo snapshot, check output). Nothing compares a criterion's required evidence with that list. A criterion that only the producer's report could sat...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261027-011714-9qse (job 20261027-004517-gk9e); prevention it proposed: Add a check at reading or request acceptance that scans each criterion for evidence sources the verifier lacks, such as "producer's report", "the report lists" or "the producer states". The check flags the criterion unless it is restated as...
> Introspection 20261027-011812-anp5 (job 20261027-011733-vrsk); prevention it proposed: Add a reading-acceptance check. Each criterion must name its evidence as a command or Grep the verifier can run, or a file in the snapshot. A criterion whose evidence is the producer's report, summary or claims is refused. The reading and c...
> Introspection 20261027-013024-v1gj (job 20261027-012938-yckz); prevention it proposed: Add a reading-acceptance check. Every criterion must name an evidence source the verifier can observe: a Grep or command it runs, or a file in the snapshot. A criterion that cites the producer's report, or "the producer confirms", is reject...
> Introspection 20261027-023348-vrd5 (job 20261026-232402-6qme); prevention it proposed: Add two things. (1) A rule in the reader brief: every criterion must name evidence the verifier can observe itself, the producer's report is never evidence, and for code-move jobs the standard criteria are a Grep for old module paths outsid...
> Introspection 20261027-030613-vk78 (job 20261026-232423-hvyb); prevention it proposed: Two parts. (1) Add a rule to the reading brief that every criterion must name evidence the verifier can observe itself (the snapshot, commands it runs, its own tools), and that the producer's report is never evidence. (2) Add a code check o...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: nothing compares a criterion's required evidence with what the verifier can observe, so criteria like "the report lists the Grep" (jobs gk9e, vrsk, yckz and two others) are accepted and then cannot be judged. Add a mechanism check in crates/athena-core/src/reading.rs, run next to check_prior_result_and_scope when a reading is built, that refuses any criterion citing the producer's report, summary, claims or statement as its evidence. The error names the criterion and says to restate it as a command or Grep the verifier can run, or a file in the snapshot. Audience: Athena's operator and the verifier.

Assumptions:
- The cause originates at reading acceptance, where criteria are first fixed; the code check is the mechanism, matching what all five introspections proposed.
- The reader prompt has 3 tokens of headroom, so no prompt text is added; the refusal message teaches the reader on retry.
- The refusal is reported the same way as check_prior_result_and_scope (a reading-refused message), applied to every reading with a capability, not only retries.
- The named jobs 20261027-004517-gk9e and 20261027-011733-vrsk are cited as evidence of the failure class, not retried here; their earlier findings (unobservable report criterion) are the class this change prevents, and the gh transport move itself already landed in 20261027-012938-yckz.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-core reading`: a new test shows a reading with the criterion "the producer's report lists the caller-check Grep" is refused with an error naming that criterion, and the same test fails when the new check is removed.
2. [behavior] Run `cargo test --locked -p athena-core reading`: a test covers each phrase family ("the report lists", "the producer states", "the producer confirms", "producer's summary", "producer's claims") and each is refused.
3. [behavior] Run `cargo test --locked -p athena-core reading`: a test shows criteria naming a command, a Grep, or a file path (e.g. "a Grep of crates/ for `gh::` outside gh/ returns no hits", "cargo test --locked passes") are accepted, so ordinary readings are not refused.
4. [behavior] Run `cargo test --locked -p athena-core reading`: a test shows the check is wired into the reading path (the reader's output becomes a refusal message, not a Reading) for a first attempt as well as a retry, not only in the check function.
5. [content] The doc comment on the new check and the CHANGELOG.md entry name the root cause: criteria were written without a defined list of what the verifier can observe, and nothing compared required evidence with that list; jobs 20261027-004517-gk9e, 20261027-011733-vrsk, 20261027-012938-yckz, 20261026-232402-6qme and 20261026-232423-hvyb are cited.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Editing prompt files (reader has 3 tokens of headroom)
- Changing the verifier's handover or giving the verifier the producer's report
- Rewriting criteria of existing jobs
- Changes to athena.toml, defaults.toml or workflows
- Reverting or redoing the gh transport move

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a code check, not prose in a prompt.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at reading acceptance, covering every criterion, not one job's wording.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show refusal and acceptance behavior.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Check is written in Rust in athena-core.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check plus tests and a changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

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

Run `cargo test --locked -p athena-core reading`: a new test shows a reading with the criterion "the producer's report lists the caller-check Grep" is refused with an error naming that criterion, and the same test fails when the new check is removed.
Run `cargo test --locked -p athena-core reading`: a test covers each phrase family ("the report lists", "the producer states", "the producer confirms", "producer's summary", "producer's claims") and each is refused.
Run `cargo test --locked -p athena-core reading`: a test shows criteria naming a command, a Grep, or a file path (e.g. "a Grep of crates/ for `gh::` outside gh/ returns no hits", "cargo test --locked passes") are accepted, so ordinary readings are not refused.
Run `cargo test --locked -p athena-core reading`: a test shows the check is wired into the reading path (the reader's output becomes a refusal message, not a Reading) for a first attempt as well as a retry, not only in the check function.
The doc comment on the new check and the CHANGELOG.md entry name the root cause: criteria were written without a defined list of what the verifier can observe, and nothing compared required evidence with that list; jobs 20261027-004517-gk9e, 20261027-011733-vrsk, 20261027-012938-yckz, 20261026-232402-6qme and 20261026-232423-hvyb are cited.
Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
