# Job 20261108-023653-70wy: Resume the firstmate study with milestone M2: long-running sessions

## Request (verbatim)

> Get the study going again

## Reading (accepted by the user)

The firstmate study has only M1 (work handling) done and validated; study work has stalled for weeks. I read "get the study going again" as starting the next milestone, M2 (long-running sessions: state, resumption, crash survival, unattended operation, drift) on firstmate at the same pinned commit. The audience is Sri and outside readers of the published study.

Assumptions:
- Goal: restart study progress after a three-week stall (BRIEF.md log, 2026-11-08). The next step is M2 because M1 exists and its validation passes.
- Same system and pin as the M1 study (firstmate at 6407052f...). The pinned source is supplied to the producer as in the earlier job.
- Code survey: crates/athena-cap-study/src/rules.rs holds the study validation rules and the kept citation test. The study records are typed JSON under studies/<id>/, which is data, not a crates/ file, so no check_record_file or AnyRecord change is expected.
- This adds no new refusal rule, so there is no refusal-rate criterion.
- Environment: no CI workflow, push, PR or records read is needed. Reaching the pinned firstmate source depends on the study capability's mount, which I could not inspect.
- The earlier study's site generation and Pages publishing are out of this job. If a generator reads studies/ automatically, the new study may appear without a change.

## Acceptance criteria

1. [content] A new directory studies/firstmate-long-running-sessions/ holds pin.json, claims.json, model.json, findings.json, rationales.json, intro.json and validation.json, in the same shape as studies/firstmate-work-handling/, and pin.json uses sha 6407052f4c521e39401ce2305dfa45d201249827.
2. [content] Every claim in claims.json cites file and line sources at the pinned commit with quotes, and validation.json lists every rule from the existing study's validation.json with pass true. A reader can follow each intro.json sentence to a claim id.
3. [behavior] Run `cargo test` and `cargo clippy` from the repo root. Both pass, including the kept study rule tests in crates/athena-cap-study that check citations, quotes and the model for the new study directory.
4. [content] The model covers the M2 topics from BRIEF.md (state, resumption, crash survival, unattended operation, self-update, drift, context rot) as states, transitions, guards and counters. Where the pinned source says nothing, the study records the gap as a finding instead of guessing.
5. [content] The existing studies/firstmate-work-handling/ files are unchanged (git diff shows no change under that path).

## Expected deliverables

- studies/firstmate-long-running-sessions/pin.json (create)
- studies/firstmate-long-running-sessions/claims.json (create)
- studies/firstmate-long-running-sessions/model.json (create)
- studies/firstmate-long-running-sessions/findings.json (create)
- studies/firstmate-long-running-sessions/rationales.json (create)
- studies/firstmate-long-running-sessions/intro.json (create)
- studies/firstmate-long-running-sessions/validation.json (create)
- CHANGELOG.md (modify)

## Out of scope

- Changes to the M1 study files under studies/firstmate-work-handling/
- Studying Hermes Agent or Athena itself
- Changes to athena.toml, defaults.toml or workflows
- M3 and later milestones

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite pinned-source lines, and numbers are measured or labeled as estimates.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Validation rules and cargo test are run independently.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target from BRIEF.md and the existing studies/ contents.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study, one milestone.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check validation results and citations, not that files exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: intro.json is written for outside readers.

## Pinned source

Athena resolved the studied repository to one commit before this job began:

- url: https://github.com/kunchenguid/firstmate.git
- ref: 6407052f4c521e39401ce2305dfa45d201249827
- sha: 6407052f4c521e39401ce2305dfa45d201249827

Cite this url, ref and full SHA exactly as given; they are real, not placeholders. The tree is mounted read-only in your working directory.

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

- path: <state>/jobs/20261108-023653-70wy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-023653-70wy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new directory studies/firstmate-long-running-sessions/ holds pin.json, claims.json, model.json, findings.json, rationales.json, intro.json and validation.json, in the same shape as studies/firstmate-work-handling/, and pin.json uses sha 6407052f4c521e39401ce2305dfa45d201249827.
Every claim in claims.json cites file and line sources at the pinned commit with quotes, and validation.json lists every rule from the existing study's validation.json with pass true. A reader can follow each intro.json sentence to a claim id.
Run `cargo test` and `cargo clippy` from the repo root. Both pass, including the kept study rule tests in crates/athena-cap-study that check citations, quotes and the model for the new study directory.
The model covers the M2 topics from BRIEF.md (state, resumption, crash survival, unattended operation, self-update, drift, context rot) as states, transitions, guards and counters. Where the pinned source says nothing, the study records the gap as a finding instead of guessing.
The existing studies/firstmate-work-handling/ files are unchanged (git diff shows no change under that path).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
