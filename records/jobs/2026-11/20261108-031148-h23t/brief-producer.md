# Job 20261108-031148-h23t: Retry firstmate M2 study; fix spec coverage so cargo test passes

## Request (verbatim)

> try again on 20261108-023653-70wy

## Reading (accepted by the user)

Retry of job 20261108-023653-70wy: redo the firstmate long-running-sessions (M2) study at the same pin for Sri and outside readers. The last attempt's study content was sound. It was rejected because cargo test failed on spec coverage of the BRIEF addenda from 2026-11-04 on, so this retry also adds the missing requirements to docs/spec.toml.

Assumptions:
- Job 20261108-023653-70wy is addressed as follows. The red cargo test (6 xtask spec tests: 'no requirement for' the BRIEF addenda 2026-11-04 to 2026-11-08) is fixed by adding requirements to docs/spec.toml. The c9 claim is fixed by citing only that a dead pid is reclaimed. The c17 and intro limits[3] nit is fixed by naming Pi, omp and OpenCode. The intro what[4] nit is fixed by adding the only-nudged exception. The rules.rs change is kept as a deliverable (refactor plus M2 tests), and the CI-skip limit is an existing M1 pattern, left as is.
- The typed request 'try again' takes precedence over the introspection's proposed baseline-check feature. That feature and the 'separate job' framing are out of scope here; the spec fix is included only so that cargo test goes green.
- Code survey: read xtask/tests/spec.rs (coverage test at line 323) and docs/spec.toml (format header lines 1-33; each BRIEF heading with text must be the first source heading of some requirement, with mechanism, owner, measure ending 'status: <label>', and verbatim quotes). The earlier candidate's crates/athena-cap-study/src/rules.rs changes are the study rule tests. No record or data file is added under crates/, so no check_record_file or AnyRecord change applies.
- Pinned commit is 6407052f4c521e39401ce2305dfa45d201249827, same as the M1 study; the M2 topics come from BRIEF.md milestone M2.
- Neither athena.toml, defaults.toml nor any workflow needs to change.

## Acceptance criteria

1. [content] studies/firstmate-long-running-sessions/ holds pin.json (sha 6407052f4c521e39401ce2305dfa45d201249827), claims.json, model.json, findings.json, rationales.json, intro.json and validation.json in the same shape as studies/firstmate-work-handling/.
2. [content] Every claim cites file and line sources at the pin with quotes. Claim c9 states only that a dead pid is reclaimed. The bounded-retry claim and the intro name Pi, omp and OpenCode. The intro sentence about the update restarting secondmates includes the only-nudged exception. Every intro sentence maps to a claim id.
3. [content] The model covers state, resumption, crash survival, unattended operation, self-update, drift and context rot as states, transitions, guards and counters, and records gaps in the source as findings.
4. [behavior] Run `cargo test --locked` and `cargo clippy` from the repo root. Both pass with no failures, including the study rule tests for the new directory and the six xtask/tests/spec.rs coverage tests.
5. [content] docs/spec.toml gains requirements whose first source heading is each BRIEF.md addendum from 2026-11-04 to 2026-11-08 that lacked one, each with verbatim quotes, mechanism, owner and a measure ending in a valid status label.
6. [content] git diff shows no change under studies/firstmate-work-handling/, and CHANGELOG.md has an entry for the M2 study.

## Expected deliverables

- studies/firstmate-long-running-sessions/pin.json (create)
- studies/firstmate-long-running-sessions/claims.json (create)
- studies/firstmate-long-running-sessions/model.json (create)
- studies/firstmate-long-running-sessions/findings.json (create)
- studies/firstmate-long-running-sessions/rationales.json (create)
- studies/firstmate-long-running-sessions/intro.json (create)
- studies/firstmate-long-running-sessions/validation.json (create)
- CHANGELOG.md (modify)
- crates/athena-cap-study/src/rules.rs (modify)
- docs/spec.toml (modify)

## Out of scope

- Changes under studies/firstmate-work-handling/
- Studying Hermes Agent or Athena itself
- Changes to athena.toml, defaults.toml or workflows
- M3 and later milestones
- Building the baseline test check proposed by the introspection
- Changes to xtask code or BRIEF.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass including the study rule tests and xtask spec tests.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: every claim cites file, lines and a verbatim quote at the pin; gaps are recorded as findings.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: the intro is written for outside readers and each sentence traces to claim ids.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: the spec coverage gap that blocked the last attempt is fixed rather than worked around.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: the verifier runs cargo test and clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check that tests exercise the new study and the spec coverage.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261108-023653-70wy raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/tests/spec.rs:323 (the_real_spec_covers_every_heading_and_addendum_of_the_brief) and 5 related tests): `cargo test` fails, so criterion 3 is unmet. The failing xtask spec tests were almost certainly already failing before this change. `cargo test -p xtask --test spec` gives 24 passed and 6 failed, with 'no requirement for: [addendum 2026-11-04 … addendum 2026-11-08]'. The diff touches neither BRIEF.md nor xtask, so this change did not cause it. The fix (adding spec requirements for the new addenda) is outside this job's scope and needs separate work.

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

- path: <state>/jobs/20261108-031148-h23t/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-031148-h23t/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

studies/firstmate-long-running-sessions/ holds pin.json (sha 6407052f4c521e39401ce2305dfa45d201249827), claims.json, model.json, findings.json, rationales.json, intro.json and validation.json in the same shape as studies/firstmate-work-handling/.
Every claim cites file and line sources at the pin with quotes. Claim c9 states only that a dead pid is reclaimed. The bounded-retry claim and the intro name Pi, omp and OpenCode. The intro sentence about the update restarting secondmates includes the only-nudged exception. Every intro sentence maps to a claim id.
The model covers state, resumption, crash survival, unattended operation, self-update, drift and context rot as states, transitions, guards and counters, and records gaps in the source as findings.
Run `cargo test --locked` and `cargo clippy` from the repo root. Both pass with no failures, including the study rule tests for the new directory and the six xtask/tests/spec.rs coverage tests.
docs/spec.toml gains requirements whose first source heading is each BRIEF.md addendum from 2026-11-04 to 2026-11-08 that lacked one, each with verbatim quotes, mechanism, owner and a measure ending in a valid status label.
git diff shows no change under studies/firstmate-work-handling/, and CHANGELOG.md has an entry for the M2 study.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
