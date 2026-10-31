# Job 20261030-013549-2cab: Create and remove a per-job worktree and target dir with a disk bound

## Request (verbatim)

> Priority: high
> Async drain build job A4 (docs/design/async-drain.md, decision D145). Per-job worktree start (job 4): create and remove a worktree and its target directory per job, with the disk bound of section 5 computed from job A1's measured T and S. Depends on 20261030-013143-as0j and 20261029-142844-ynv3 and 20261029-210827-18fc (A1 needs its measurement). With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A4 (D145, section 5): each job gets its own git worktree and CARGO_TARGET_DIR, created at producer start and removed when the job ends. The number of jobs allowed in flight is capped by a disk bound computed from the latest measured T and S (job A1's measurement record from 20261029-210827-18fc), with max_in_flight = 1 unchanged. The audience is Sri and the drain verifier.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (guard, sweep_idle_targets, CargoBuilder, measure_build, BuildMeasurement use), crates/athena-core/src/run/mod.rs (Athena::go -> go_with_stages, the real entry point; kept_work_dir), crates/athena-core/src/run/producer.rs (producer_snapshot uses StateDir::work_dir, jobs/<id>/work), crates/athena-data/src/config.rs (PipelineConfig max_in_flight, reserve_disk_gb = 10). The disk bound formula is design section 5: in_flight = floor((free_gb - reserve_disk_gb - max_check_slots x T) / (T + S)). T and S come from the latest athena.build-measurement/1 record; with none, the design estimates T about 6 GB, S about 0.2 GB are used. No new record type or data file is added, so the records allow-list (check_record_file, AnyRecord round-trip) is not touched.
- Job 20261030-013143-as0j (A3, in-flight table) landed and is kept as is. The worktree it records is StateDir::work_dir; this job makes that directory a real git worktree and removes it. Its minor finding (a moved row keeps the stage verifier) and its nit (section number) are not in this job's files and stay out of scope.
- Job 20261029-142844-ynv3 (A1, rejected, no numbers measured) is dropped: its docs-only candidate changed CHANGELOG.md, docs/decisions.md and docs/design/async-drain.md and none of it is redone here. Its need for numbers was met by 20261029-210827-18fc (landed, kept as is), whose `athena measure-build` writes the T and S record this job reads. Its minor findings (extra store.rs, display.rs and test files; sccache stats not isolated) are not touched. If no record exists the bound uses the labelled design estimates.
- Worktree creation and removal run through git worktree add/remove on the state directory path jobs/<id>/work with a per-job CARGO_TARGET_DIR; removal also runs on failure and when a job ends, so a finished job leaves neither directory.
- With max_in_flight = 1 the drain runs as today, existing drain tests unchanged. Only a bound that would allow zero in-flight jobs on a small disk is applied, and it holds the start rather than failing the job.

## Acceptance criteria

1. [behavior] A test through Athena::go (the real entry point) shows a job's producer runs in a fresh git worktree with its own target directory under the job's state directory, and that after the job ends both directories are gone, on success and on a failed or rejected job. Run: cargo test -p athena-core -p athena-cli.
2. [behavior] A test in crates/athena-core/src/disk.rs shows the in-flight limit equals floor((free_gb - reserve_disk_gb - check_slots x T) / (T + S)) using T and S from the latest build-measurement record, and that with no record the design estimates (6 GB, 0.2 GB) are used. Run: cargo test -p athena-core disk.
3. [behavior] A test shows that when the bound allows fewer in-flight jobs than max_in_flight, the extra start waits rather than failing, and that with max_in_flight = 1 behavior is unchanged and the existing tests in crates/athena-cli/tests/drain.rs and lifecycle.rs pass without edits to their assertions. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry for per-job worktree start naming the disk bound and the source of T and S.
6. [content] The diff changes no file under athena.toml, defaults.toml, .github/workflows or records/, and does not change crates/athena-data/src/config.rs.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Running more than one producer at once or raising max_in_flight above 1 (build job 10)
- Rebase at landing time, check slots, read-ahead, conflict return
- Acting on leftover in-flight rows (recovery)
- Fixing A3's moved-row stage finding or A1/measure-build findings
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Re-running or changing the build measurement itself

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for worktree lifecycle and the bound; fmt, clippy, test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run through cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dirs created and removed, and the computed limit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to run and disk modules.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

22 more qualifying classes are left out.

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

A test through Athena::go (the real entry point) shows a job's producer runs in a fresh git worktree with its own target directory under the job's state directory, and that after the job ends both directories are gone, on success and on a failed or rejected job. Run: cargo test -p athena-core -p athena-cli.
A test in crates/athena-core/src/disk.rs shows the in-flight limit equals floor((free_gb - reserve_disk_gb - check_slots x T) / (T + S)) using T and S from the latest build-measurement record, and that with no record the design estimates (6 GB, 0.2 GB) are used. Run: cargo test -p athena-core disk.
A test shows that when the bound allows fewer in-flight jobs than max_in_flight, the extra start waits rather than failing, and that with max_in_flight = 1 behavior is unchanged and the existing tests in crates/athena-cli/tests/drain.rs and lifecycle.rs pass without edits to their assertions. Run: cargo test.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry for per-job worktree start naming the disk bound and the source of T and S.
The diff changes no file under athena.toml, defaults.toml, .github/workflows or records/, and does not change crates/athena-data/src/config.rs.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
