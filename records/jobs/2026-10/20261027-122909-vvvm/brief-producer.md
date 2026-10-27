# Job 20261027-122909-vvvm: Enforce rendered-page assertions for firstmate study page criteria

## Request (verbatim)

> Add a convention, and a test in xtask that enforces it for the firstmate-work-handling study: when a job's acceptance criteria are about what a generated study page contains or must not contain, the producer adds assertions on the rendered HTML from `cargo run -p xtask -- site`, runs them before handover, and a failure blocks handover. Write the convention where producer briefs and the reading stage pick it up. Start with the firstmate page: assert the glossary entry and the next-step link come before the first transition card, assert which model-limit findings are rendered, and assert that agent-decided transitions are checked through the model's trigger_evaluated_by field. Keep a fixture that must fail.
> 
> Acceptance:
> - The change is made as described: Firstmate study page follow-up jobs are rejected one after another; nothing checks the rendered page against the criteria before the verifier sees it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261027-085315-n35f.json

## Reading (accepted by the user)

Add a convention that jobs whose criteria concern what a generated study page contains must add assertions on the rendered HTML from `cargo run -p xtask -- site`, run them before handover, and treat failure as blocking. Implement the firstmate page checks as an xtask test (glossary and next-step link before first transition card, rendered model-limit findings, agent-decided transitions checked via trigger_evaluated_by), plus a fixture page that must fail. Audience: producers and the reader/verifier stages that judge study page jobs.

Assumptions:
- Prompt token budgets prevent editing the reader prompt (3 tokens left) and verifier (2 left); the producer prompt for change has 219 tokens left, so a short rule goes there and fuller text in docs/.
- Existing unit tests in xtask/src/site.rs partly cover these checks; the new integration test in xtask/tests exercises the built HTML output and adds the failing fixture.
- The records/introspections file named under Files is context only and is not changed.
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [behavior] `cargo test -p xtask` passes, including a new test that renders the real firstmate-work-handling study and asserts the glossary section and the `#next-steps` link both appear before the first transition card in the HTML.
2. [behavior] The new test asserts exactly which model-limit findings from studies/firstmate-work-handling/model.json are present in the rendered HTML (and that others are absent), and `cargo test -p xtask` passes.
3. [behavior] The new test asserts that every transition whose model `trigger_evaluated_by` is `agent` is rendered as agent-decided, derived from model.json rather than hard-coded, and passes.
4. [behavior] A committed fixture page (e.g. a deliberately wrong HTML file) is run through the same assertion helpers by a test that expects failure (glossary after first card, missing link, wrong findings, wrong evaluator), proving each check can fail; `cargo test -p xtask` passes.
5. [content] The convention is written in a producer prompt/brief location (crates/athena-cap-change/prompts/producer.md, within its token headroom) stating: criteria about generated page contents require HTML assertions on `cargo run -p xtask -- site` output, run before handover, and failure blocks handover. Reader prompt and other budget-tight prompts are not grown.
6. [content] The convention is also recorded in a doc under docs/ (stranger-readable) naming the xtask test as its enforcing mechanism.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass with no unrelated behavior changes to site rendering.

## Expected deliverables

- xtask/tests/site_page.rs (create)
- xtask/tests/fixtures/bad_firstmate_page.html (create)
- crates/athena-cap-change/prompts/producer.md (modify)
- docs/design/requirements-map.md (modify)
- docs/conventions/rendered-page-assertions.md (create)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the rendered page layout or study content
- Editing the reader, verifier, or study prompts
- Changing governance files such as athena.toml or defaults.toml
- Assertions for studies other than firstmate-work-handling

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask test plus must-fail fixture; fmt, clippy, test pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Convention enforced by an xtask test rather than prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered HTML behavior, not file existence.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Convention documented for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to firstmate page.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Addresses missing pre-verifier render checks rather than one page fix.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
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

`cargo test -p xtask` passes, including a new test that renders the real firstmate-work-handling study and asserts the glossary section and the `#next-steps` link both appear before the first transition card in the HTML.
The new test asserts exactly which model-limit findings from studies/firstmate-work-handling/model.json are present in the rendered HTML (and that others are absent), and `cargo test -p xtask` passes.
The new test asserts that every transition whose model `trigger_evaluated_by` is `agent` is rendered as agent-decided, derived from model.json rather than hard-coded, and passes.
A committed fixture page (e.g. a deliberately wrong HTML file) is run through the same assertion helpers by a test that expects failure (glossary after first card, missing link, wrong findings, wrong evaluator), proving each check can fail; `cargo test -p xtask` passes.
The convention is written in a producer prompt/brief location (crates/athena-cap-change/prompts/producer.md, within its token headroom) stating: criteria about generated page contents require HTML assertions on `cargo run -p xtask -- site` output, run before handover, and failure blocks handover. Reader prompt and other budget-tight prompts are not grown.
The convention is also recorded in a doc under docs/ (stranger-readable) naming the xtask test as its enforcing mechanism.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass with no unrelated behavior changes to site rendering.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
