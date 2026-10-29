# Job 20261024-232059-mymj: Extend docs/spec.toml to BRIEF sections 7-10, philosophy and addenda

## Request (verbatim)

> Build the second layer of the cohesive spec (decision 3C): extend docs/spec.toml, which covers BRIEF.md 'Project Olympus' sections 1 to 6, to sections 7 to 10, the governing philosophy and the addenda of 2026-10-17 to 2026-10-24, in the same format and checked by the same test.
> Acceptance:
> - Every new requirement cites verbatim quotes under their headings, and the xtask spec test passes.
> - Conflicts between Sri's words are recorded as open questions quoting both.
> - The boundary scan (xtask/tests/policy.rs) still passes.
> Files: docs/spec.toml, xtask/tests/spec.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Add sourced requirements (and open questions for conflicting words) to docs/spec.toml for Olympus sections 7 to 10, "Governing philosophy", and the addenda dated 2026-10-17 to 2026-10-24. Widen xtask/tests/spec.rs so its section cutter no longer stops at "## 7." and coverage covers the new parts; record the decision and a changelog line. Audience: Sri and the verifier checking quotes against BRIEF.md.

Assumptions:
- Goal: a cohesive, sourced spec of the whole Olympus brief; audience is Sri and the verifier, who check quotes against BRIEF.md.
- Addenda 2026-10-25 and 2026-10-26 are out of scope because the request names 2026-10-17 to 2026-10-24.
- The existing test's sections() function ends at `## 7.` and coverage_gaps only checks sections 1 to 5 and R1 to R8; both need extending in the same test file, and this is the required change to 'the same test'.
- Headings used must be unique within the Olympus-onward text; the brief has two `## Addenda` headings but the new spec cites only dated addendum headings.
- Quote placement: text under a heading runs only to the next heading of any level, so quotes must be cited under the nearest subheading (e.g. Phase N, Metric integrity).
- Decision numbering in docs/decisions.md follows the existing D-number format; the request's 'decision 3C' is cited by name in the new entry.
- Boundary scan in xtask/tests/policy.rs is unaffected as no governance files change; not read in detail.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask --test spec`: it passes, including the_real_spec_passes, so every new requirement and open-question side quotes BRIEF.md verbatim under a heading from sections 7 to 10, Governing philosophy, or the 2026-10-17 to 2026-10-24 addenda.
2. [behavior] Run `cargo test -p xtask --test policy`: the boundary scan still passes.
3. [content] xtask/tests/spec.rs no longer stops cutting sections at the `## 7.` line; it reads through the 2026-10-24 addenda and stops before the 2026-10-25 addendum, so no new requirement cites 2026-10-25 or later text. Coverage tests now also require at least one requirement under each of sections 7, 8, 9, 10, Governing philosophy, and each addendum dated 2026-10-17 to 2026-10-24 (checked by Grep of the test).
4. [behavior] The test file has in-memory negative cases for the new reach, run by `cargo test -p xtask --test spec`: a non-verbatim quote under a section 7+ heading fails, a missing section 8 requirement is reported as a coverage gap, and a heading dated after 2026-10-24 is rejected.
5. [content] Where Sri's words conflict across sections or addenda (for example the 2026-10-17 to 2026-10-24 addenda against earlier text, such as the history exemptions and trailer rules), docs/spec.toml has an [[open_question]] with two or more sides, each quoting both statements verbatim; the conflicting requirements are not silently resolved. Existing superseded/active status is used only where the brief says so.
6. [content] New requirement ids follow the existing OLY-N-NN pattern and existing entries for sections 1 to 6 are unchanged (git diff shows only additions there).
7. [content] docs/decisions.md gets an entry recording that the second layer (decision 3C) extends the spec, with the alternatives considered; CHANGELOG.md gets a matching line. No secrets or personal data are added.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/spec.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Addenda dated 2026-10-25 and later
- Changes to sections 1 to 6 entries in docs/spec.toml
- Changes to BRIEF.md, athena.toml, defaults.toml or xtask/tests/policy.rs
- Resolving open questions; they are only recorded

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Statements and the decision entry are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every requirement cites verbatim quotes checked by the spec test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the spec and policy tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the quote-check test and negative cases rather than checking file existence.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data added.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the spec, its test, and two log files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Quote rules are enforced by the spec test, not prose.
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

19 more qualifying classes are left out.

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

Run `cargo test -p xtask --test spec`: it passes, including the_real_spec_passes, so every new requirement and open-question side quotes BRIEF.md verbatim under a heading from sections 7 to 10, Governing philosophy, or the 2026-10-17 to 2026-10-24 addenda.
Run `cargo test -p xtask --test policy`: the boundary scan still passes.
xtask/tests/spec.rs no longer stops cutting sections at the `## 7.` line; it reads through the 2026-10-24 addenda and stops before the 2026-10-25 addendum, so no new requirement cites 2026-10-25 or later text. Coverage tests now also require at least one requirement under each of sections 7, 8, 9, 10, Governing philosophy, and each addendum dated 2026-10-17 to 2026-10-24 (checked by Grep of the test).
The test file has in-memory negative cases for the new reach, run by `cargo test -p xtask --test spec`: a non-verbatim quote under a section 7+ heading fails, a missing section 8 requirement is reported as a coverage gap, and a heading dated after 2026-10-24 is rejected.
Where Sri's words conflict across sections or addenda (for example the 2026-10-17 to 2026-10-24 addenda against earlier text, such as the history exemptions and trailer rules), docs/spec.toml has an [[open_question]] with two or more sides, each quoting both statements verbatim; the conflicting requirements are not silently resolved. Existing superseded/active status is used only where the brief says so.
New requirement ids follow the existing OLY-N-NN pattern and existing entries for sections 1 to 6 are unchanged (git diff shows only additions there).
docs/decisions.md gets an entry recording that the second layer (decision 3C) extends the spec, with the alternatives considered; CHANGELOG.md gets a matching line. No secrets or personal data are added.
Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
