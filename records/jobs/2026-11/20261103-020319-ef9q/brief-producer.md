# Job 20261103-020319-ef9q: Add decision-brief renderer with refusal checks to xtask site

## Request (verbatim)

> Give the owner one-screen visual decision briefs instead of text, and check them before they are shown.
> The owner decides things Athena cannot (Steward proposals above their limit, paused lines of work, design choices) but cannot read long text. Today those arrive as long log lines in athena status.
> - Store the owner's style rules verbatim in docs/conventions/owner-style.md (new file): 'Use HTML to help me understand what we're discussing. Show me with big pictures and little words without witholding key details. Use your own taste, avoid lazy design, integrate real-world pictures/icons/visuals as needed but only selectively when it makes sense. When you do use prose/detail: Be clear, concise/terse, and avoid jargon. Explain things as if the viewer is new here, but dont use weird metaphors.'
> - For each item that waits for a person, render one self-contained HTML brief with the same xtask site renderer: what is being decided, the options, what each costs and risks, and Athena's recommendation, in at most about 120 words of prose plus one diagram or chart; every claim links to the record it came from. Publish them on the existing explainer site under a decisions page.
> - The audience role reads each brief against owner-style.md and rates it; a brief rated 3 or lower is rewritten once before it is published, and the rating is recorded.
> - athena status shows each waiting item as one line with the brief's link.
> - Tests: a waiting Steward proposal produces a brief with the required parts and a link from status; a brief with no recommendation or over the word limit is refused by the renderer.
> - CHANGELOG.md gets an entry; docs/design/ gets a short design note.
> - Before planning, list what else reads the explainer site, the audience role and the status output, with a test or a reason for each.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

First slice of owner decision briefs: store the owner's style rules verbatim, add a renderer in the xtask site that produces one self-contained HTML brief per waiting item and refuses briefs without a recommendation or over ~120 words, and publish a decisions page. The audience-role rating and rewrite, and the status line links, are follow-up requests.

Assumptions:
- Goal: the owner (a non-reader of long text) gets one-screen visual briefs; the audience is the owner and the verifier judging against owner-style.md.
- Code survey: read xtask/src/site.rs and xtask/src/main.rs (site entry point) and crates/athena-core/src/steward.rs and status.rs (where waiting proposals and status lines come from); the briefs add no records or data files under crates/, so check_record_file/AnyRecord are not affected.
- Scope cut to fit a single run: renderer, refusal rules, decisions page, style file, design note, changelog. Waiting items are limited to Steward proposals in this slice.
- The brief input is a plain struct (title, options with cost/risk, recommendation, record links); word count covers prose fields only, limit 120 with small tolerance.
- Refusal rules apply only to new briefs, not existing readings, so the replay refusal-rate criterion does not apply.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's quoted style rules verbatim, including 'Show me with big pictures and little words without witholding key details.' (checked by Grep).
2. [behavior] Run `cargo test -p xtask`: a test builds a brief from a waiting Steward proposal and asserts the HTML has the decision, options, cost and risk per option, a recommendation, one diagram (inline SVG), and a link to the source record for each claim.
3. [behavior] Run `cargo test -p xtask`: a brief with no recommendation is refused with an error naming the missing part, and a brief with prose over the word limit (about 120 words) is refused with an error naming the count.
4. [behavior] Run `cargo run -p xtask -- site` (or the existing site command) and `ls` the output: a decisions page exists under the explainer site, is linked from the site index, and each brief is self-contained (no external script or stylesheet references, checked by Grep).
5. [content] CHANGELOG.md has an entry for decision briefs, and docs/design/ has a short design note with alternatives and a section listing what else reads the explainer site, the audience role and the status output, each with a test or a reason; it states that rating/rewrite and status links are follow-ups.
6. [content] Existing site renderer tests still pass because the change extends xtask/src/site.rs rather than adding a second renderer; no change to athena.toml, defaults.toml or workflows.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/site.rs (modify)
- xtask/src/main.rs (modify)
- docs/design/owner-decision-briefs.md (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Audience-role rating of briefs against owner-style.md, rewrite-once for ratings of 3 or lower, and recording the rating (follow-up request 1)
- athena status one-line-per-waiting-item with brief links (follow-up request 2)
- Briefs for paused lines of work and design choices (follow-up request 3)
- Repository-wide fmt, clippy or test run as its own criterion

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Renderer and tests in Rust within xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for brief parts and refusals.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design note and docs written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Cut to renderer slice; rest named as follow-ups.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests and the site build.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Refusals enforced in the renderer, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 40 cases in 40 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one repair round within the same job. The verifier is only start...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

25 more qualifying classes are left out.

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

docs/conventions/owner-style.md exists and contains the owner's quoted style rules verbatim, including 'Show me with big pictures and little words without witholding key details.' (checked by Grep).
Run `cargo test -p xtask`: a test builds a brief from a waiting Steward proposal and asserts the HTML has the decision, options, cost and risk per option, a recommendation, one diagram (inline SVG), and a link to the source record for each claim.
Run `cargo test -p xtask`: a brief with no recommendation is refused with an error naming the missing part, and a brief with prose over the word limit (about 120 words) is refused with an error naming the count.
Run `cargo run -p xtask -- site` (or the existing site command) and `ls` the output: a decisions page exists under the explainer site, is linked from the site index, and each brief is self-contained (no external script or stylesheet references, checked by Grep).
CHANGELOG.md has an entry for decision briefs, and docs/design/ has a short design note with alternatives and a section listing what else reads the explainer site, the audience role and the status output, each with a test or a reason; it states that rating/rewrite and status links are follow-ups.
Existing site renderer tests still pass because the change extends xtask/src/site.rs rather than adding a second renderer; no change to athena.toml, defaults.toml or workflows.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
