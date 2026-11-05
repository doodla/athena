# Job 20261105-170035-1704: Retry: design note on many agents, few concurrent builds

## Request (verbatim)

> try again on 20261105-103606-svwx

## Reading (accepted by the user)

Retry of job 20261105-103606-svwx: write one design note in docs/design/ for running dozens of agents at once on this 4-CPU, 15 GB, ~27 GB-disk machine, with agent slots separate from queued build slots and shared compiled output. The audience is the owner, who decides whether to fund it. The earlier candidate, docs/design/many-agents-few-builds.md, is dropped and rewritten, fixing each of its findings.

Assumptions:
- Existing code: crates/athena-data/src/config.rs (MAX_CONCURRENCY_CEILING), crates/athena-core/src/disk.rs (disk checks), crates/athena-core/src/run/mod.rs (run pipeline and cargo invocation), crates/athena-core/src/state.rs (remove_produce at about line 272) and docs/design/warm-baseline-build.md (shared baseline build) exist and are not created by this job; the design builds on them.
- Job 20261105-103606-svwx was rejected (verdict fail). Its one-file candidate, docs/design/many-agents-few-builds.md, is dropped; the retry starts from base and writes a fresh note. The same filename may be reused; it is a create, since the candidate is not in the repository.
- Finding fixes for 20261105-103606-svwx: (a) lanes vs per-run slots: the note settles that a job holds its lane for the whole job (preferred lane) and only a CPU build slot per cargo run, so trimming happens per job and lane size is sized for one job's output; (b) remove_produce is located in state.rs, not disk.rs; (c) the note says in one line what the roughly 77 excluded job directories are (no outcome.json, or still in flight); (d) the CHANGELOG finding does not apply: the request says one new file only, so CHANGELOG.md is excluded.
- The failed criterion 8 (cargo test and clippy evidence) was an evidence-visibility problem in Athena's flow, not a defect in the note. The introspection's proposed follow-up (recorded command runs for criteria) is out of scope here. The doc-only change is checked by the recorded check run of cargo test and cargo clippy, which the verifier may also run itself.
- Measurements come from job records of the last 3 days (2026-11-03 to 2026-11-05); the producer reads them from the files it has and cites job ids. Any figure it cannot measure is labeled an estimate.
- Machine arithmetic: 4 CPUs, 15 GB memory, about 27 GB free disk; at 10 to 12 GB per target directory, disk allows about 2 builds today.

## Acceptance criteria

1. [content] Exactly one new file under docs/design/ is added and no other path changes (check with git status and git diff); CHANGELOG.md, athena.toml, defaults.toml and workflows are untouched.
2. [content] The design has an Existing code section that reuses or extends crates/athena-data/src/config.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/state.rs and docs/design/warm-baseline-build.md, saying what each already does and what the design builds on.
3. [content] A measurements section splits job wall time over the last 3 days into model time, local build and test time, and waiting, with numbers. Each figure cites the job record ids or files it came from, unmeasured numbers are labeled estimates, and one line says what the jobs left out of the sample are.
4. [content] The note defines agent slots and build slots separately, including a queue for build slots, and states how a job acquires and releases a build slot around cargo runs. It states whether a lane or target directory is held per job or per cargo run, and that choice is consistent across all sections.
5. [content] The note proposes a way for jobs to share compiled output, compares at least two alternatives, and gives an expected per-job disk figure against the current 10 to 12 GB.
6. [content] The note states how many agents and how many builds this machine can run at once, with arithmetic from 4 CPUs, 15 GB memory and about 27 GB disk, and says whether MAX_CONCURRENCY_CEILING must be raised.
7. [content] A build order lists small changes, each landable alone. Each names the files or functions it touches (for example config.rs, run/mod.rs, state.rs for remove_produce, disk.rs) and states how its effect will be measured.
8. [behavior] The doc-only change leaves the build green: the recorded check run of cargo test and cargo clippy --all-targets --locked -- -D warnings passes, and the verifier can run cargo test and cargo clippy itself.

## Expected deliverables

- docs/design/many-agents-few-builds.md (create)
- docs/design/agents-and-builds.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing any proposed change in code
- Editing athena.toml, defaults.toml or workflows
- Changing MAX_CONCURRENCY_CEILING now
- Editing existing design docs
- Modifying CHANGELOG.md (the request allows one new file only)
- Making criteria that name commands verifiable from recorded evidence (the introspection's follow-up request)

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The note is written for the owner as an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Measurements cite job ids and files; estimates are labeled.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note compares alternatives and records the recommended build order.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file; each proposed step is landable alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the note's content, measurements and arithmetic, not just that the file exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must still pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier checks the note against the criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261105-103606-svwx raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 8 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo clippy --all-targets --locked -- -D warnings`

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

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-170035-1704/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-170035-1704/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Exactly one new file under docs/design/ is added and no other path changes (check with git status and git diff); CHANGELOG.md, athena.toml, defaults.toml and workflows are untouched.
The design has an Existing code section that reuses or extends crates/athena-data/src/config.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/state.rs and docs/design/warm-baseline-build.md, saying what each already does and what the design builds on.
A measurements section splits job wall time over the last 3 days into model time, local build and test time, and waiting, with numbers. Each figure cites the job record ids or files it came from, unmeasured numbers are labeled estimates, and one line says what the jobs left out of the sample are.
The note defines agent slots and build slots separately, including a queue for build slots, and states how a job acquires and releases a build slot around cargo runs. It states whether a lane or target directory is held per job or per cargo run, and that choice is consistent across all sections.
The note proposes a way for jobs to share compiled output, compares at least two alternatives, and gives an expected per-job disk figure against the current 10 to 12 GB.
The note states how many agents and how many builds this machine can run at once, with arithmetic from 4 CPUs, 15 GB memory and about 27 GB disk, and says whether MAX_CONCURRENCY_CEILING must be raised.
A build order lists small changes, each landable alone. Each names the files or functions it touches (for example config.rs, run/mod.rs, state.rs for remove_produce, disk.rs) and states how its effect will be measured.
The doc-only change leaves the build green: the recorded check run of cargo test and cargo clippy --all-targets --locked -- -D warnings passes, and the verifier can run cargo test and cargo clippy itself.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
