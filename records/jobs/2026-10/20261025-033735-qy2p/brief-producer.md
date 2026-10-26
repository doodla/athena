# Job 20261025-033735-qy2p: Show producers the recurring failure classes in their brief

## Request (verbatim)

> Priority: high
> Show the planner and critic the recurring failure classes (BRIEF.md addendum of 2026-10-25): introspections classify every failure (172 records, 73 classes; for example plan-tested-only-against-job-commits-not-realistic-branch-history 5 times, reading-omits-prior-verifier-finding 7 times), but the planner and critic never see them because records/ is left out of their copy. Add to their briefs the classes seen in at least three distinct jobs, each with its title, count and the prevention the introspections proposed, capped to fit the budgets.
> Acceptance:
> - A brief test with fixture introspections shows the qualifying classes listed with counts and preventions, and one seen in two jobs left out.
> - The planner and critic budgets still pass.
> Files: crates/athena-context/src/briefs.rs, crates/athena-core/src/run.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a "Recurring failure classes" section to the producer brief, built from the real records/ introspections and class registry. It lists classes seen in at least 3 distinct jobs with title, case count and proposed prevention, capped and quoted as data, with the producer budget kept passing. Planner and critic briefs stay untouched.

Assumptions:
- Goal: producers, who write the change, avoid repeat failure classes; the user's reply redirects the section from planner/critic briefs to the producer brief.
- Threshold, cap and data quoting are as in the earlier reading; count is of cases, threshold is distinct jobs.
- Missing records/ yields no section rather than an error.
- docs/system-facts.md budget lines may need regenerating if the producer budget figures change.

## Acceptance criteria

1. [behavior] A new test in crates/athena-context/src/briefs.rs, run with `cargo test -p athena-context`, builds fixture class summaries and asserts producer_brief lists a class seen in 3+ distinct jobs with its title, case count and proposed prevention, and omits a class with many cases in only 2 jobs.
2. [behavior] The test shows the count is of cases while the threshold counts distinct jobs: 5 cases in 2 jobs is omitted; 3 cases in 3 jobs is listed.
3. [behavior] A test shows the cap: with many qualifying classes the most frequent come first, output stays within a fixed limit and states how many were left out. With none qualifying, no empty heading appears.
4. [behavior] A test shows prevention and title text is quoted as data (fenced or labeled), and the section text says it is not instructions.
5. [behavior] `cargo test -p athena-cli --test budgets` passes, so the producer budget still passes and docs/system-facts.md budget lines match the computed ones.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass.
7. [content] planner_brief, critic_brief and reader_brief and their inputs are unchanged.
8. [content] crates/athena-core/src/run.rs reads introspections and the class registry from the real records/ and passes the summaries in ProducerInputs at the producer_brief call site.
9. [content] CHANGELOG.md has an entry saying the producer brief now carries recurring failure classes (seen in at least 3 distinct jobs), with the cap.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)

## Out of scope

- Changes to the planner, critic or reader briefs
- Changing athena.toml job_plan setting
- Editing records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check the listed output, not that types exist.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures use invented data; no personal data.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Budget test and system-facts match.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Producer brief only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

A new test in crates/athena-context/src/briefs.rs, run with `cargo test -p athena-context`, builds fixture class summaries and asserts producer_brief lists a class seen in 3+ distinct jobs with its title, case count and proposed prevention, and omits a class with many cases in only 2 jobs.
The test shows the count is of cases while the threshold counts distinct jobs: 5 cases in 2 jobs is omitted; 3 cases in 3 jobs is listed.
A test shows the cap: with many qualifying classes the most frequent come first, output stays within a fixed limit and states how many were left out. With none qualifying, no empty heading appears.
A test shows prevention and title text is quoted as data (fenced or labeled), and the section text says it is not instructions.
`cargo test -p athena-cli --test budgets` passes, so the producer budget still passes and docs/system-facts.md budget lines match the computed ones.
`cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass.
planner_brief, critic_brief and reader_brief and their inputs are unchanged.
crates/athena-core/src/run.rs reads introspections and the class registry from the real records/ and passes the summaries in ProducerInputs at the producer_brief call site.
CHANGELOG.md has an entry saying the producer brief now carries recurring failure classes (seen in at least 3 distinct jobs), with the cap.

## Report

End with the producer report as your structured output.
