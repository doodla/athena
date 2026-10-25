# Job 20261024-093833-vava: Make the first producer report list every criterion with evidence

## Request (verbatim)

> Most producers are sent back once because their first report does not account for every acceptance criterion: jobs 20261024-025358-kb23, -033758-c2h3, -045548-7qht, -055549-0qcy, -033935-c4w4 and -082859-skh5 each had a send-back, and the second producer run cost 9 to 17 minutes, often for a report that only restates what was already done. Find why the first report misses criteria (the producer prompt, the report schema, or how criteria are shown), make the first report list every criterion with its evidence, and measure the send-back rate before and after on the jobs' records.

## Reading (accepted by the user)

Goal: stop the producer's first report from missing acceptance criteria, which costs the user a 9–17 minute second run. Fix the cause in the change producer prompt and the rendered brief (not the budget-tight shared prompt), then measure the send-back rate before and after from job records.

Assumptions:
- Audience: the user, who pays for the send-back, and the verifier who judges the change.
- Read in this copy: run.rs unmet_criteria already ignores numbering, kind tag, case, spacing and trailing punctuation, but any other wording difference counts as unmet. A paraphrased or merged entry therefore triggers a send-back, and the send-back message may only restate what was done.
- Read in this copy: the shared producer.md has one line on criteria_check, and the CriterionCheckOutput schema asks for the text 'without its number or kind tag'. The producer must retype criteria from memory, so wording drifts. Rendering the criteria so they can be copied verbatim is the likely fix.
- Records of the named jobs are absent from my copy, so the cause is not yet measured. The producer must read the records, if it can, to confirm the cause and compute the before rate for the six jobs. If it cannot read them, it names that gap and does not invent numbers.
- The after rate can only come from jobs run after the change. The change reports this honestly as a small sample or as not yet measurable, and gives a repeatable way to compute it.
- Per the user's reply, new producer instructions go in crates/athena-cap-change/prompts/producer.md or the rendered brief in briefs.rs, net of removals. The shared producer.md is not grown (study at 1276/1300). The change verifier prompt is not touched (1298/1300).
- The design call and alternatives (prompt only, schema only, tolerant matching) are recorded in docs/decisions.md.

## Acceptance criteria

1. [content] docs/decisions.md has a note naming the cause of the missed criteria. It cites crates/athena-core/src/run.rs (unmet_criteria, criterion_key), the criteria rendering in crates/athena-context/src/briefs.rs, the producer prompts, and the CriterionCheckOutput docs in outputs.rs. It says which of prompt, schema and display contributed and lists the alternatives considered (prompt only, schema only, tolerant matching).
2. [content] The note states the send-back rate before the change for kb23, c2h3, 7qht, 0qcy, c4w4 and skh5, with the records read to get it. Each number is marked measured or estimate, with its sample size. Any unreadable record is named, and the after rate is stated for jobs run after the change, or the note says there are none yet.
3. [behavior] Run `cargo test -p athena-context`: a test shows the rendered producer brief lists every accepted criterion verbatim, with a copyable form (no number or kind tag needed to match), and instructs one criteria_check entry per criterion, in order, with evidence, including criteria already satisfied.
4. [content] New producer instructions are in crates/athena-cap-change/prompts/producer.md or the rendered brief. Shared crates/athena-context/prompts/producer.md and crates/athena-cap-change/prompts/verifier.md are not made longer, and `cargo test --locked -p athena-cli --test budgets -- --nocapture` passes.
5. [behavior] Run `cargo test -p athena-core`: a test shows a first report with a met entry for every criterion is not sent back, and a report omitting one criterion is sent back once, with the send-back naming the missing criterion's exact text.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding to the change verifier prompt (at 1298 of 1300 tokens)
- Growing the shared producer.md beyond its budget
- Changing the verifier or planner behavior
- Editing records/ or the named jobs' outputs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests in athena-context and athena-core; cargo test and clippy must be clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The finding cites file locations; the rates are marked measured or estimate, with sample size.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria check that the brief and the send-back logic behave as intended, not that text exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The finding names which of prompt, schema or display causes the misses before the fix.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The before and after send-back rates are the outcome criterion.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision note is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief, the change producer prompt and, if needed, the field docs.
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

## Report

End with the producer report as your structured output.
