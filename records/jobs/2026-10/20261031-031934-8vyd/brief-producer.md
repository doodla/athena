# Job 20261031-031934-8vyd: Re-check rebased candidates only when the rebase touched their paths

## Request (verbatim)

> Priority: urgent
> Async drain build job A7 (docs/design/async-drain.md, decision D145). Re-check rule after rebase (job 7): re-run checks only when the rebase touched paths the candidate changed, and record the share of rebases that needed it. With max_in_flight = 1 drain behaves as today.
> Depends on 20261030-013555-f5qm
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A7 (D145) of async drain: after the ordered landing queue rebases a verified candidate onto the latest main, the drain re-runs checks only when the rebase touched paths the candidate changed, and records the share of rebases that needed a re-check. With max_in_flight = 1 drain behaves as today. Follow-up to failed job 20261030-013555-f5qm (A6); its work is not kept as a candidate and its findings are addressed below. Audience: Sri and the verifier.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (module docs, a LandingQueue field at about line 1018, the Drain landing lock) and crates/athena-core/src/gh/landing.rs (the Rebase enum with Current, Moved, Conflict, Unchecked, and plan_rebase). The entry point is the drain loop and drain_concurrent; the rebase result is judged after verification and before the merge. The new decision is a pure function over (paths changed by the candidate, paths changed by the rebase onto the new main). No new record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The share is measured directly from counted rebases (needed re-check / total rebases) and printed in the drain summary.
- Existing code: crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs and docs/design/async-drain.md (section on the re-check rule, build job 7) are extended and reused, not created.
- Job 20261030-013555-f5qm (A6) failed in plan critique with no candidate, so nothing from it is kept. It is a dependency: if the landing queue and Rebase handling are already in the tree (they appear to be: LandingQueue in drain.rs, Rebase in landing.rs), A7 builds on them. Its findings are addressed as follows. (1) [parallel] max_concurrency > 1 runs with max_in_flight = 1 and lands group jobs one after another onto a moved main: A7 must keep that path landing as today; a test through drain_concurrent with max_concurrency 2 pins it, and a rebase that touches none of the second job's paths does not trigger a re-check. (2) The old job after drop_landed and (3) the Rebase to Rebased mapping, (4) thrash handling and (5) size are A6 matters, not A7's; they apply only if the tree lacks them, in which case the producer reports it instead of building them. The mapping from rebase outcome to re-check is a small pure function with a unit test per arm.
- A rebase that changes no path the candidate changed lands without re-running checks. A rebase that touches an overlapping path re-runs checks before landing. Governed-file fingerprint handling stays as the existing merged-content check in land_pr decides.
- The share is reported in the drain summary output and kept in-memory for the run. No new persisted record is added.

## Acceptance criteria

1. [behavior] Tests in crates/athena-cli/src/drain.rs drive the drain through its real entry point (drain or drain_concurrent) with a landing whose rebase touches a path the candidate changed, and show checks re-run before landing. Run cargo test -p athena-cli.
2. [behavior] A test through the same entry point shows a rebase touching only paths the candidate did not change lands with no check re-run, and the drain summary reports the share of rebases needing a re-check (e.g. 1 of 2) as a number. Run cargo test -p athena-cli.
3. [behavior] With max_in_flight = 1 the existing drain tests pass unchanged: git diff shows no existing test removed or edited, and cargo test passes. A new drain_concurrent test with max_concurrency 2 pins that the second job of a group still lands as today.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry describing the path-scoped re-check after rebase and the recorded share of rebases that needed it, citing D145 and build job 7.
6. [content] Governance files (athena.toml, defaults.toml, .github/workflows) are unchanged; no rule that refuses is newly added beyond re-running checks, so no refusal-rate figure applies.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- docs/design/async-drain.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Raising max_in_flight above 1 or running several producers at once (build job 10)
- Disk guard, check slots and read-ahead
- Recovery of leftover in-flight rows
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Rebuilding the A6 landing queue, send_back and thrash stop if they are absent; the producer reports that instead
- Changes to crates/athena-core/src/state.rs and crates/athena-core/src/run/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise the drain entry point, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain.rs and the changelog.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the share is recorded and checks re-run only on overlap.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

Tests in crates/athena-cli/src/drain.rs drive the drain through its real entry point (drain or drain_concurrent) with a landing whose rebase touches a path the candidate changed, and show checks re-run before landing. Run cargo test -p athena-cli.
A test through the same entry point shows a rebase touching only paths the candidate did not change lands with no check re-run, and the drain summary reports the share of rebases needing a re-check (e.g. 1 of 2) as a number. Run cargo test -p athena-cli.
With max_in_flight = 1 the existing drain tests pass unchanged: git diff shows no existing test removed or edited, and cargo test passes. A new drain_concurrent test with max_concurrency 2 pins that the second job of a group still lands as today.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry describing the path-scoped re-check after rebase and the recorded share of rebases that needed it, citing D145 and build job 7.
Governance files (athena.toml, defaults.toml, .github/workflows) are unchanged; no rule that refuses is newly added beyond re-running checks, so no refusal-rate figure applies.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
