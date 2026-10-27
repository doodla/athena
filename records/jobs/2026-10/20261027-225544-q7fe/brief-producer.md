# Job 20261027-225544-q7fe: Fix firstmate study page newcomer gaps (retry of 20261027-122909-37mg)

## Request (verbatim)

> try again on 20261027-122909-37mg

## Reading (accepted by the user)

Retry of job 20261027-122909-37mg for newcomer readers of the generated firstmate-work-handling site page. Make the four page edits (no f2 gap, study-id paragraph moved to end, one-sentence "what firstmate is" before "Words used first", README install command and example in Next steps) and rebuild. Its only major finding, the unverifiable "producer's report" criterion, is replaced by a criterion the verifier can run itself.

Assumptions:
- Job 20261027-122909-37mg's major finding (criterion 7, producer report not visible to verifier) is addressed by dropping that criterion and replacing it with a verifier-runnable one; the producer should still mention commands run, but no criterion depends on it.
- Its minor finding (page file outside working dir) is addressed by checking via cargo run and the site_page tests, which render the real page. Its nit is fixed: the closing note says 'This page's study id' instead of 'Its'.
- Page is built by xtask/src/site.rs from studies/firstmate-work-handling/*.json; the earlier deliverable list and out-of-scope list are kept.
- Audience: a newcomer reading the Athena site; install command and example come from README.md Quick start (./setup.sh and athena ask "add a changelog").

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`: it exits successfully and regenerates the firstmate study page on the final snapshot.
2. [behavior] Run `cargo test -p xtask`: the site_page tests render the real page and assert that under 'Limits of this study' finding ids run f1, f2, f3 with no gap (or the page states why f2 is absent).
3. [behavior] Tests in xtask/tests/site_page.rs assert the study-id/studies/ folder paragraph is not in the header and appears at the end of the page after the claims, introduced with 'This page's study id' rather than an unclear 'Its'.
4. [behavior] Tests assert one plain sentence saying what firstmate is, carrying claim-id links, appears before the 'Words used first' heading.
5. [behavior] Tests assert Next steps contains './setup.sh' and 'athena ask "add a changelog"', both matching README.md Quick start.
6. [content] Every existing claim link and bracketed claim id still resolves to a claim on the page; claim-citation checks pass; no claim, claim source or pinned commit changes.
7. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass.
8. [content] No governance file (athena.toml, defaults.toml, workflows), README.md, other study or other page changes.

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

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered page content via tests and cargo run, not producer activity.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass; tests updated.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the page text and order.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the site build itself.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the one page.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261027-122909-37mg

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> The producer's report lists the commands it ran, including `cargo run -p xtask -- site`.: Neither the brief nor the snapshot contains a producer report. The candidate commit message is just 'candidate', and the CHANGELOG entry lists no commands. From what I can see, I cannot confirm that the producer reported its commands. For the user's benefit: I ran `cargo run -p xtask -- site` myself and it succeeded.

## Findings you must address

The verifier of job 20261027-122909-37mg raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Can't confirm that the producer reported the commands it ran (criterion 7) The brief and the snapshot contain no producer report. The commit message is 'candidate', and the CHANGELOG lists no commands. The request explicitly asks to 'report the commands that were run'.

All of the verification's findings:

> major: Can't confirm that the producer reported the commands it ran (criterion 7) The brief and the snapshot contain no producer report. The commit message is 'candidate', and the CHANGELOG lists no commands. The request explicitly asks to 'report the commands that were run'.
> minor (target/verify/site/index.html): The page file couldn't be read directly, so the rendered-page checks rely on the integration test `cargo run -p xtask -- site` writes to <CARGO_TARGET_DIR>/site/index.html, which is outside the working directory. The policy hook refused `ls` on it, and I did not try to get around that. The behavior was checked instead through the site_page tests, which render the real page with the xtask binary. This is a limit of the snapshot, not a defect in the change.
> nit (xtask/src/site.rs:566-570): The 'About this page' note opens with an unclear 'Its' The closing section reads 'About this page' / 'Its study id, <code>…</code>, is the name of the folder…'. The pronoun has no clear antecedent in the paragraph. 'This page's study id' would be plainer for a newcomer.

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

The work starts from the rejected candidate of job 20261027-122909-37mg: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo run -p xtask -- site`: it exits successfully and regenerates the firstmate study page on the final snapshot.
Run `cargo test -p xtask`: the site_page tests render the real page and assert that under 'Limits of this study' finding ids run f1, f2, f3 with no gap (or the page states why f2 is absent).
Tests in xtask/tests/site_page.rs assert the study-id/studies/ folder paragraph is not in the header and appears at the end of the page after the claims, introduced with 'This page's study id' rather than an unclear 'Its'.
Tests assert one plain sentence saying what firstmate is, carrying claim-id links, appears before the 'Words used first' heading.
Tests assert Next steps contains './setup.sh' and 'athena ask "add a changelog"', both matching README.md Quick start.
Every existing claim link and bracketed claim id still resolves to a claim on the page; claim-citation checks pass; no claim, claim source or pinned commit changes.
`cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass.
No governance file (athena.toml, defaults.toml, workflows), README.md, other study or other page changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
