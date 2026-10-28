# Job 20261028-213554-pnjn: Fix newcomer-rated gaps on the firstmate study site page

## Request (verbatim)

> try again on 20261027-123701-77zh

## Reading (accepted by the user)

Retries job 20261027-123701-77zh (dispatched, never verified, no candidate): redo four fixes so the generated Athena site page, rated 3/5 by a newcomer, has no unexplained f2 gap, no early study-id paragraph, a one-sentence firstmate definition before "Words used first", and README install and example request in Next steps.

Assumptions:
- Job 20261027-123701-77zh had no candidate and no findings, so nothing is kept or dropped from it; all five of its deliverable files are re-listed here and its out_of_scope items are repeated below.
- Code survey: I read crates/athena-data/src/records.rs (AnyRecord at line 2386, check_record_file at line 2686) and crates/athena-cap-study/src/rules.rs (line 1435 loads studies/firstmate-work-handling for the study rules). The page is built by xtask/src/site.rs, entry `cargo run -p xtask -- site`, which loads pin, claims, model, rationales, findings and intro, and fails on an uncited element. The change adds no record or data file under crates/. studies/ JSON is not a records/ file, so check_record_file and AnyRecord are untouched. The study-rules test in crates/athena-cap-study/src/rules.rs (cargo test) must keep passing when findings.json or intro.json change. Properties are measured directly by building the page and grepping the generated index.html, not through a stand-in.
- README.md Quick start (lines 12-16) gives `./setup.sh` and `athena ask "add a changelog"`; these are the install command and example request, copied verbatim.
- For f2, the producer chooses between renumbering the findings and adding a stated reason on the page. Renumbering means editing claim-subject references too.
- The audience is a newcomer to Athena who rates the page. The change is not a refusal-rule change.

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`. In the generated index.html under target/site, the Limits of this study finding ids are consecutive, or the page states why f2 is missing. Check by grep.
2. [behavior] In the generated index.html, the text 'Its study id, firstmate-work-handling' is absent from the header. If it is kept, it appears after the Claims and sources content.
3. [behavior] In the generated index.html, a one-sentence plain description of firstmate with a claim link appears before the 'Words used first' heading. Check the byte offsets with grep.
4. [behavior] In the generated index.html, the Next steps section contains `./setup.sh` and `athena ask "add a changelog"` (HTML-escaped), matching README.md Quick start. Check by grep.
5. [behavior] Run `cargo test -p xtask`, `cargo test`, `cargo clippy` and `cargo fmt --check`. All pass. The build still errors on an element that cites no claim.
6. [content] Only xtask/src/site.rs, studies/firstmate-work-handling/intro.json and findings.json, CHANGELOG.md and docs/design/explainer-site.md change. athena.toml, defaults.toml, workflows, claims.json, model.json, rationales.json and pin.json are untouched.
7. [content] A newcomer reading top to bottom learns what firstmate is before any firstmate-specific term, and can install and try Athena from Next steps without leaving the page.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)

## Out of scope

- Claims, model, rationales and pin data of the study (claims.json, model.json, rationales.json, pin.json)
- Athena README content, athena.toml, defaults.toml and workflows
- Glossary rewrites, the repetition of yolo rules, and showing cost or checker output on the page
- Records under records/
- Committing the rebuilt site output

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the generated page, not the activity.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria grep the built index.html.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the page.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page is written for a newcomer.
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

18 more qualifying classes are left out.

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

Run `cargo run -p xtask -- site`. In the generated index.html under target/site, the Limits of this study finding ids are consecutive, or the page states why f2 is missing. Check by grep.
In the generated index.html, the text 'Its study id, firstmate-work-handling' is absent from the header. If it is kept, it appears after the Claims and sources content.
In the generated index.html, a one-sentence plain description of firstmate with a claim link appears before the 'Words used first' heading. Check the byte offsets with grep.
In the generated index.html, the Next steps section contains `./setup.sh` and `athena ask "add a changelog"` (HTML-escaped), matching README.md Quick start. Check by grep.
Run `cargo test -p xtask`, `cargo test`, `cargo clippy` and `cargo fmt --check`. All pass. The build still errors on an element that cites no claim.
Only xtask/src/site.rs, studies/firstmate-work-handling/intro.json and findings.json, CHANGELOG.md and docs/design/explainer-site.md change. athena.toml, defaults.toml, workflows, claims.json, model.json, rationales.json and pin.json are untouched.
A newcomer reading top to bottom learns what firstmate is before any firstmate-specific term, and can install and try Athena from Next steps without leaving the page.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
