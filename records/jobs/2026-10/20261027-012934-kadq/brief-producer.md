# Job 20261027-012934-kadq: Take the seeded candidate when a seeded retry's producer adds nothing

## Request (verbatim)

> Priority: urgent
> A retry seeded from the previous attempt's kept candidate fails as "the producer made no change" when the kept candidate already does the work and only evidence was missing. Job 20261027-011733-vrsk (G2 retry): the seed already moved the Transport items, the producer checked with git status, correctly made no new change, and run.rs (around line 1764) failed the job, so the complete candidate never reached checks and the verifier. Fix: when a retry is seeded from a kept candidate and the producer adds nothing, take the seeded candidate as the candidate and go on to admission, checks and the verifier as usual. A retry from the base with no change still fails as today.
> Acceptance:
> - A test seeds a retry from a kept candidate, has a fake producer make no change, and shows the seeded candidate reaches checks and the verifier and can land.
> - A test shows an unseeded run whose producer makes no change still fails as "the producer made no change".
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, CHANGELOG.md

## Reading (accepted by the user)

In run.rs (~line 1761) the idle-producer check fails a seeded retry whose work tree is unchanged since the seed, even though the seed already differs from the base. Fix: fail only when the change list against the base is empty; when seeded and unchanged but the change list is non-empty, keep the seeded candidate and continue to admission, checks and the verifier. Audience: the Athena operator whose retries lose complete candidates.

Assumptions:
- changes is the snapshot's change list against the base, so a seeded unchanged tree has a non-empty list; the fix is to fail only when changes.is_empty().
- A seeded retry whose seed equals the base (empty change list) still fails as today.
- Tests go in run.rs's existing test module using its fake producer; no changes to snapshot.rs are needed.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] A new test in run.rs seeds a retry from a kept candidate with a fake producer that makes no change; the run reaches checks and the verifier and lands. Run: cargo test -p athena-core.
2. [behavior] A new test shows an unseeded run whose fake producer makes no change still fails with the outcome 'the producer made no change'. Run: cargo test -p athena-core.
3. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
4. [content] CHANGELOG.md has an entry describing that a seeded retry whose producer adds nothing now goes on with the seeded candidate.
5. [content] The condition near run.rs line 1761 fails only when there is no change against the base; the comment explains the seeded case.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing Snapshot::unchanged_since_seed in athena-git
- Changing how retry seeds are chosen or stored
- Changing the checks or verifier logic

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests; fmt, clippy -D warnings and tests must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise the seeded no-change run reaching the verifier and landing, and the unseeded failure.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the idle-producer condition itself, not the one job.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the seeded candidate reaches the verifier.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in run.rs plus changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
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

A new test in run.rs seeds a retry from a kept candidate with a fake producer that makes no change; the run reaches checks and the verifier and lands. Run: cargo test -p athena-core.
A new test shows an unseeded run whose fake producer makes no change still fails with the outcome 'the producer made no change'. Run: cargo test -p athena-core.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry describing that a seeded retry whose producer adds nothing now goes on with the seeded candidate.
The condition near run.rs line 1761 fails only when there is no change against the base; the comment explains the seeded case.

## Report

End with the producer report as your structured output.
