# Job 20261029-130605-mbpm: Cover every brief heading and addendum in docs/spec.toml and test it

## Request (verbatim)

> Priority: urgent
> Finish moving the owner's brief (the brief file at the repository root) into docs/spec.toml, so the spec holds every requirement and the brief can be retired. After spec layer 2 the spec has 135 requirements but still nothing for the brief's opening goals (including 'gets measurably faster, cheaper, and more correct'), the Part 2 requirements A1 to A13 where not yet covered, and the dated addenda before 2026-10-17 and after 2026-10-24 (among them 2026-10-12 'Chunking and parallel work' and 2026-10-13 'Athena owns acceleration'). Add them in the same format, each with verbatim quotes under their headings, and extend the spec test so it fails when any heading or dated addendum in the brief has no requirement citing it. Record conflicts between the owner's words as open questions quoting both.
> Acceptance:
> - The spec test fails if a heading or dated addendum of the brief has no citing requirement, and passes on the updated spec.
> - The spec has requirements citing the 2026-10-12 and 2026-10-13 addenda and the opening goal about getting measurably faster.
> - The boundary scan and fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: docs/spec.toml, xtask/tests/spec.rs, CHANGELOG.md

## Reading (accepted by the user)

Reads the request as: add sourced requirements to docs/spec.toml for all brief headings not yet cited (opening goals, A1–A13 which sit under "Part 1" in BRIEF.md, the operating model and other pre-Olympus headings, and the addenda dated before 2026-10-17 or after 2026-10-24). Extend xtask/tests/spec.rs so it fails when any brief heading or dated addendum has no citing requirement; record owner conflicts as open questions quoting both sides. The audience is the owner and the verifier who retires the brief.

Assumptions:
- Goal: let the owner retire BRIEF.md because docs/spec.toml cites every heading; audience is the owner and the verifier running the spec test.
- The request's 'Part 2 requirements A1 to A13' are the headings '### A1.' to '### A13.' under '## Part 1: Athena' in BRIEF.md; I take the headings, not the Part number, as authoritative.
- 'Every heading' means every heading in BRIEF.md, including the pre-Olympus ones (Goal, O1–O5, Standards, Boundaries, S1–S7, D1–D3 and so on), plus Olympus subheadings and all dated addenda. Pure structural headings such as the title and 'Addenda' may be exempted by a short explicit list in the test; the producer decides which and writes the list down.
- The sections() helper in xtask/tests/spec.rs currently limits the citable text to Olympus through 2026-10-24 plus two pinned addenda. It must be widened to the whole brief so earlier and later headings can be cited, while keeping the existing negative tests (the 2026-10-25 'Evidence labels Jev' out-of-range test needs rewriting).
- BRIEF.md itself is not deleted in this job; retiring it is a later step, and the spec test still reads it.
- Existing requirement ids OLY-A-22 (2026-10-13 acceleration) and OLY-A-23 exist already. The 2026-10-13 addendum may already be cited, so the producer checks first and adds only what is missing. New ids avoid collisions with existing ones.
- Priority urgent is noted; I found no reason in the request that makes this cut waste.
- Code survey: not needed; this is a docs and test change that touches no crates/ code. The hestia spec review in crates/athena-core/src/hestia.rs reads docs/spec.toml, so the producer runs the workspace tests to catch any count or label assumption.

## Acceptance criteria

1. [behavior] `cargo test -p xtask --test spec` passes on the updated spec, and the new coverage test lists no gaps for any heading or dated addendum of BRIEF.md.
2. [behavior] A negative test in xtask/tests/spec.rs removes the requirements citing one heading (e.g. A6 or the 2026-10-12 'Chunking and parallel work' addendum) and asserts coverage reports exactly that heading as a gap; `cargo test -p xtask --test spec` runs it and passes.
3. [content] docs/spec.toml has requirements whose first source heading is '2026-10-12: Chunking and parallel work', '2026-10-13: Athena owns acceleration', and a heading under 'Goal' whose verbatim quote contains 'measurably faster, cheaper, and more correct'.
4. [content] docs/spec.toml has requirements citing each of A1 to A13 in BRIEF.md and each pre-2026-10-17 and post-2026-10-24 addendum, each with verbatim quotes under its heading and mechanism, owner and measure fields in the existing format.
5. [content] Where BRIEF.md has conflicting owner statements, docs/spec.toml has [[open_question]] entries with two or more sides quoting both; none silently resolves a conflict.
6. [content] CHANGELOG.md has a new entry describing the spec extension and the stricter coverage test.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (whole workspace, including any test of the spec review in crates/athena-core/src/hestia.rs) and the boundary scan all pass.
8. [content] The checked-in spec contains no secrets or personal data (public default); the 'at least 135 active requirements' floor in the spec test is raised to match the new count.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Deleting or editing BRIEF.md
- Changing athena.toml, defaults.toml or workflow files
- Implementing any mechanism the new requirements name; they record mechanism and status only
- Resolving conflicts between the owner's words

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the test failing on a missing heading and passing on the updated spec, not that files exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and the full test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A negative in-memory coverage test proves the new rule fires.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Requirements are written plainly with verbatim cited quotes.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the spec or changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the spec test and checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
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

20 more qualifying classes are left out.

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

`cargo test -p xtask --test spec` passes on the updated spec, and the new coverage test lists no gaps for any heading or dated addendum of BRIEF.md.
A negative test in xtask/tests/spec.rs removes the requirements citing one heading (e.g. A6 or the 2026-10-12 'Chunking and parallel work' addendum) and asserts coverage reports exactly that heading as a gap; `cargo test -p xtask --test spec` runs it and passes.
docs/spec.toml has requirements whose first source heading is '2026-10-12: Chunking and parallel work', '2026-10-13: Athena owns acceleration', and a heading under 'Goal' whose verbatim quote contains 'measurably faster, cheaper, and more correct'.
docs/spec.toml has requirements citing each of A1 to A13 in BRIEF.md and each pre-2026-10-17 and post-2026-10-24 addendum, each with verbatim quotes under its heading and mechanism, owner and measure fields in the existing format.
Where BRIEF.md has conflicting owner statements, docs/spec.toml has [[open_question]] entries with two or more sides quoting both; none silently resolves a conflict.
CHANGELOG.md has a new entry describing the spec extension and the stricter coverage test.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (whole workspace, including any test of the spec review in crates/athena-core/src/hestia.rs) and the boundary scan all pass.
The checked-in spec contains no secrets or personal data (public default); the 'at least 135 active requirements' floor in the spec test is raised to match the new count.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
