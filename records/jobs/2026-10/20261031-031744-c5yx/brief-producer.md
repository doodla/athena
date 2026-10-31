# Job 20261031-031744-c5yx: Add FIFO LandingQueue to Drain landing, without rebase

## Request (verbatim)

> Priority: urgent
> Async drain A6, first half: an ordered landing queue in the drain, with no rebase
> 
> Build job A6 (docs/design/async-drain.md, decision D145) failed planning twice for size (20261030-013555-f5qm, 20261031-030119-tw3v). It is split. This half adds only the queue; the rebase before landing is a separate request that depends on this one.
> 
> Add a first-in-first-out landing queue to `Drain` in crates/athena-cli/src/drain.rs, as a small pure `LandingQueue` struct. A worker joins the queue when its run passes, keeps its place through the existing landing retry (the two land calls in `attempts`), and leaves on every exit through a drop guard. Only the queue head takes the `landing` mutex and lands. Landing itself is unchanged: no rebase, and a landing that fails behaves as today. Queue order is the order runs pass, not the order landing commits were stacked; say so in a short note in docs/design/async-drain.md. The `[parallel] max_concurrency` > 1 path is otherwise unchanged.
> 
> Acceptance:
> - Unit tests of `LandingQueue`: joins are served in order; a worker that leaves (drop) while not at the head is removed; the head keeps its place across a retry.
> - A drain test with two passed workers shows they land in the order they passed.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, docs/design/async-drain.md, CHANGELOG.md

## Reading (accepted by the user)

Retry of failed jobs 20261030-013555-f5qm and 20261031-030119-tw3v, cut to the queue only: a pure FIFO `LandingQueue` in drain.rs, joined when a run passes, held through the two land calls in `attempts`, left via a drop guard; only the head takes the `landing` mutex. No rebase, no landing change. Audience: Sri and the verifier of the build jobs.

Assumptions:
- Prior jobs 20261030-013555-f5qm and 20261031-030119-tw3v produced no candidate (failed in planning for size); nothing from them is kept, so no files of theirs are carried over. Their findings are addressed by the split: 20261030-013555-f5qm's rebase, send_back/drop_landed, thrash and landing.rs findings no longer apply because this half has none of that; its max_concurrency > 1 finding is addressed by leaving landing unchanged and keeping that path otherwise as is. 20261031-030119-tw3v's left-behind records-loss finding and gh/mod.rs finding no longer apply (no rebase, no landing.rs or gh changes); its pass-order-vs-stacking-order finding is fixed by the design-doc note the request asks for; its size finding is addressed by the small scope.
- Code survey: crates/athena-cli/src/drain.rs: `Drain` has `landing: Mutex<()>` (line ~1041, built ~1859); `Drain::land` (~1330) takes the mutex then calls `steps.land`; `attempts` (~1376) loops, sets `passed` on `RunStep::Passed` and calls `self.land` once per loop pass, up to ATTEMPTS, which are the two land calls. The queue is joined once `passed` is set, kept through both calls, and left through a drop guard on every exit of the section. Waiting uses the existing `wake` condvar or a new Condvar paired with the state mutex. No record or data file is added, so no records validation or AnyRecord change.
- Queue order is the order runs pass (join order), not landing-commit stacking order; stated in a short note in docs/design/async-drain.md section 6, which also marks that the rebase is a later request.
- The drain test uses the existing test Steps fakes in drain.rs's tests module and drives two workers via the real drain entry, with the second worker's run passing first or gating by channel so pass order is deterministic.
- No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli drain`: new unit tests of `LandingQueue` pass, showing joins are served in order, a member that leaves (drop guard) while not at the head is removed without blocking others, and the head keeps its place across a retry (the same slot serves both land calls).
2. [behavior] Run `cargo test -p athena-cli drain`: a new drain test with two workers that pass one after the other shows the two landings happen in the order the runs passed, even if the later-started worker is ready first.
3. [behavior] Run `cargo test -p athena-cli`: with max_in_flight = 1 every pre-existing drain test passes unchanged; the diff in drain.rs tests only adds tests, and no existing test body is edited.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
5. [content] In crates/athena-cli/src/drain.rs `LandingQueue` is a small pure struct (no mutex or threads inside); the drop guard releases the slot on every exit of `attempts`; the diff leaves `Steps::land`, the lander and crates/athena-core untouched, and adds no rebase.
6. [content] docs/design/async-drain.md has a short note that queue order is the order runs pass, not the order landing commits were stacked, and that rebase comes in a later request.
7. [content] CHANGELOG.md has an entry for the ordered landing queue that says it does not rebase.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Rebase before landing, conflict return to the producer, send_back/drop_landed, thrash stop (the separate dependent request)
- Any change to crates/athena-core (gh/landing.rs, gh/mod.rs, run/) or how landing itself works
- Raising max_in_flight above 1 (build job 10) and the `[parallel] max_concurrency` path beyond the queue
- athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and drain tests; fmt, clippy -D warnings and cargo test criterion.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check landing order and unchanged serial behavior by running the tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Order, drop and retry-place criteria run real code through tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Queue only; rebase left to a later request.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc note and CHANGELOG entry written for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
- 11 cases in 10 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena a disk lifecycle. Before the checks stage, it should automatically delete the finished producer build folder and any other regenerable artifacts (verify-side builds, caches) once they are no longer needed. If free space is still below the guard, it should hold the job instead of failing ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

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

Run `cargo test -p athena-cli drain`: new unit tests of `LandingQueue` pass, showing joins are served in order, a member that leaves (drop guard) while not at the head is removed without blocking others, and the head keeps its place across a retry (the same slot serves both land calls).
Run `cargo test -p athena-cli drain`: a new drain test with two workers that pass one after the other shows the two landings happen in the order the runs passed, even if the later-started worker is ready first.
Run `cargo test -p athena-cli`: with max_in_flight = 1 every pre-existing drain test passes unchanged; the diff in drain.rs tests only adds tests, and no existing test body is edited.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
In crates/athena-cli/src/drain.rs `LandingQueue` is a small pure struct (no mutex or threads inside); the drop guard releases the slot on every exit of `attempts`; the diff leaves `Steps::land`, the lander and crates/athena-core untouched, and adds no rebase.
docs/design/async-drain.md has a short note that queue order is the order runs pass, not the order landing commits were stacked, and that rebase comes in a later request.
CHANGELOG.md has an entry for the ordered landing queue that says it does not rebase.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
