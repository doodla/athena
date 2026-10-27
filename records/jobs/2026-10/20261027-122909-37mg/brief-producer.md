# Job 20261027-122909-37mg: Fix newcomer gaps on the firstmate study site page

## Request (verbatim)

> On the generated site page for the firstmate-work-handling study: (1) under Limits of this study, either renumber the findings so there is no gap after f1, or say on the page why f2 is missing; (2) remove the header paragraph that explains the study id and the studies/ folder, or move it to the end; (3) say in one plain sentence what firstmate is before the 'Words used first' list; (4) add to Next steps an install command and one example request taken from Athena's README. Keep every claim link. Rebuild with `cargo run -p xtask -- site` and report the commands that were run.
> 
> Acceptance:
> - The change is made as described: Newcomer rating 3/5 on the Athena site: missing f2 id, insider study-id paragraph, firstmate explained late, no install command or example request
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: studies/

## Reading (accepted by the user)

The page is built by xtask/src/site.rs from studies/firstmate-work-handling/*.json. I read the request as four edits for a newcomer reader. They are: no gap in finding ids, the study-id paragraph moved to the end of the page, a one-sentence "what firstmate is" before "Words used first", and an install command plus an example request from Athena's README in Next steps. After the edits, run the site rebuild.

Assumptions:
- The generator xtask/src/site.rs and its tests xtask/tests/site_page.rs must change even though the request says Files: studies/. The study-id paragraph and Next steps are hard-coded in site.rs.
- Finding ids are renumbered, f3 to f2 and f4 to f3, in findings.json. Any other references (validation.json, tests, fixtures, CHANGELOG) are updated to match. If a reference cannot be updated safely, the page instead states why f2 is missing.
- The study-id paragraph is moved to a short closing note after Claims and sources, so the sentence about citing claims stays near the top.
- The firstmate sentence comes from the existing intro.json 'what' text, with its claim links c1, c11, c21 and c22. It is placed before the Words used first heading, probably as a new intro.json field.
- The install command is './setup.sh' and the example request is 'athena ask "add a changelog"', both from README.md Quick start.
- The audience is a newcomer reading the generated site page, and the Athena site is the target of the stated 3/5 newcomer rating.

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`, then read target/site/index.html with Read. Under 'Limits of this study' the finding ids run f1, f2, f3 with no gap, or the page says why f2 is absent.
2. [behavior] In the rebuilt page, the paragraph explaining the study id and the studies/ folder is not in the header. It is absent or sits at the end of the page, after the claims.
3. [behavior] In the rebuilt page, one plain sentence saying what firstmate is appears before the 'Words used first' heading. It carries claim-id links.
4. [behavior] In the rebuilt page, the Next steps section contains the install command './setup.sh' and the example request 'athena ask "add a changelog"', both matching README.md Quick start.
5. [content] Every existing claim link and bracketed claim id still resolves to a claim on the page. The site generator's claim-citation checks still pass.
6. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass. Updated tests in xtask/tests/site_page.rs assert the new page order and the new text.
7. [content] The producer's report lists the commands it ran, including `cargo run -p xtask -- site`.
8. [content] No unrelated behavior changes. No governance file (athena.toml, defaults.toml, workflows) is touched, and no other study or page changes.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- xtask/tests/site_page.rs (modify)
- studies/firstmate-work-handling/validation.json (modify)
- xtask/tests/fixtures/bad_firstmate_page.html (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing any claim, claim source, or the pinned firstmate commit
- Changing other pages or other studies
- Rewording the glossary or the transition cards
- Changing README.md or athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass, with site tests updated for the new layout.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Rebuild the page with cargo run -p xtask -- site and check the rendered output for each of the four changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered page text and order, not that the code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the single study page and its generator.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New page text is plain sentences written for an outside newcomer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The finding-id gap is fixed in the data (findings.json), not by hiding it in the renderer.
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

Run `cargo run -p xtask -- site`, then read target/site/index.html with Read. Under 'Limits of this study' the finding ids run f1, f2, f3 with no gap, or the page says why f2 is absent.
In the rebuilt page, the paragraph explaining the study id and the studies/ folder is not in the header. It is absent or sits at the end of the page, after the claims.
In the rebuilt page, one plain sentence saying what firstmate is appears before the 'Words used first' heading. It carries claim-id links.
In the rebuilt page, the Next steps section contains the install command './setup.sh' and the example request 'athena ask "add a changelog"', both matching README.md Quick start.
Every existing claim link and bracketed claim id still resolves to a claim on the page. The site generator's claim-citation checks still pass.
`cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass. Updated tests in xtask/tests/site_page.rs assert the new page order and the new text.
The producer's report lists the commands it ran, including `cargo run -p xtask -- site`.
No unrelated behavior changes. No governance file (athena.toml, defaults.toml, workflows) is touched, and no other study or page changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
