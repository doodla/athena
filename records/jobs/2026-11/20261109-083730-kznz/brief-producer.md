# Job 20261109-083730-kznz: Require reader questions to be precise customer decisions with defaults

## Request (verbatim)

> Also, clarifications. I'm the customer essentially. Athena is allowed to ask/clarify what I want. But my time is valuable, so it should know exactly what it needs from me before coming to me.

## Reading (accepted by the user)

The customer wants Athena free to ask him clarifying questions, but only once the repository and defaults can't settle the point, and with the exact decision, alternatives and fallback default stated. I read this as a refusal rule on the reader's `question` field, mirroring the planner's `User:`/`Host:` open-questions rule, plus tests, a CHANGELOG entry and a doc note.

Assumptions:
- Goal and audience: the customer (Sri) wants Athena to be able to ask him questions, but only after it has worked out exactly what it needs, so his time is spent on real decisions. The reader's `question` field is the place where this is enforced today and it is not checked. Planner open questions are already gated by admission::open_questions_findings (User:/Host: prefix).
- Code survey: read crates/athena-core/src/admission.rs (open_questions_findings, has_addressee, says_code_answers; the planner-side rule is called from plan_findings), crates/athena-core/src/reading.rs (build_reading at ~line 702, `question` handling at ~lines 762 and 1202, many tests that set `question` to short strings) and crates/athena-core/src/reading/{terms,outcome,surveys,checks}.rs (each skips checks when `reading.question.is_some()`). The reader entry point is build_reading, which first builds the Reading from the model output and then runs the per-check functions. The new rule is a check run there and measures the question text directly. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The refusal corpus in crates/athena-core/tests/refusal_corpus.rs (open_question_cases) is the model for corpus tests.
- The ask is read as: apply to the reader the same discipline as the planner (settle what the repository can settle, ask only what only the customer can decide, with alternatives and a default). The reader's reading must still be complete under its stated assumption, as the reader brief already says.
- Reuse admission's has_addressee logic (make it pub(crate) or share it) rather than duplicate it. This is an extension of existing code, not a new tool.
- Prompt files are out of scope because the reader prompt has only 71 tokens of headroom. The refusal finding itself feeds back the required format.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs (or a new file under crates/athena-core/src/reading/) holds a reader-question rule, wired into build_reading, that gives a finding for a question that does not begin with `User:` (same prefix normalisation as admission::has_addressee), offers fewer than two concrete alternatives, or names no default the job runs on if unanswered. A Grep for the rule's function name finds its definition and its call site.
2. [behavior] `cargo test -p athena-core` passes. New unit tests show that a question like `User: Which greeting: (a) hello or (b) hi? Default: (a).` is accepted. Questions with no `User:` prefix, with no alternatives, or with no default are each refused. A question that says the code answers it is refused. A reading with no question is unaffected.
3. [content] The rule's doc comment says the change refuses more readings and the higher refusal rate is intended, and states `expected 3% of newly refused readings`. The reason given is that questions the repository or defaults could settle cost the customer time.
4. [behavior] The refusal finding text, shown by the new tests, tells the reader what to do: settle the question from the repository or defaults, or rewrite it as `User:` + the decision + the alternatives + the default. An outcome check: a stranger reading the finding knows exactly what a question must contain before it reaches the customer.
5. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes. The existing reading tests that set `question` to short strings such as `which file?` are updated to the new format and still pass.
6. [content] CHANGELOG.md has an entry for the rule. docs/design/job-planning.md or another existing doc under docs/ records that the reader's question follows the same addressee rule as the planner's open questions, with the alternatives.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading/questions.rs (create)

## Out of scope

- Changing how the customer's answers are delivered or stored
- Planner open-questions rule behavior
- Prompt files (reader token headroom is 71)
- athena.toml, defaults.toml and workflow changes
- Learned-defaults or 'dumb context questions' memory features

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the intent from the existing planner open-questions rule instead of asking the customer.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and clippy/fmt criteria for the new rule.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The reader's question has no gate, so the rule is added at the reader boundary, mirroring the planner fix.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check accepted and refused question texts by running tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-083730-kznz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-083730-kznz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading.rs (or a new file under crates/athena-core/src/reading/) holds a reader-question rule, wired into build_reading, that gives a finding for a question that does not begin with `User:` (same prefix normalisation as admission::has_addressee), offers fewer than two concrete alternatives, or names no default the job runs on if unanswered. A Grep for the rule's function name finds its definition and its call site.
`cargo test -p athena-core` passes. New unit tests show that a question like `User: Which greeting: (a) hello or (b) hi? Default: (a).` is accepted. Questions with no `User:` prefix, with no alternatives, or with no default are each refused. A question that says the code answers it is refused. A reading with no question is unaffected.
The rule's doc comment says the change refuses more readings and the higher refusal rate is intended, and states `expected 3% of newly refused readings`. The reason given is that questions the repository or defaults could settle cost the customer time.
The refusal finding text, shown by the new tests, tells the reader what to do: settle the question from the repository or defaults, or rewrite it as `User:` + the decision + the alternatives + the default. An outcome check: a stranger reading the finding knows exactly what a question must contain before it reaches the customer.
`cargo clippy --all-targets` is clean and `cargo fmt --check` passes. The existing reading tests that set `question` to short strings such as `which file?` are updated to the new format and still pass.
CHANGELOG.md has an entry for the rule. docs/design/job-planning.md or another existing doc under docs/ records that the reader's question follows the same addressee rule as the planner's open questions, with the alternatives.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
