# Job 20261103-075205-wbtm: Render owner decision briefs from open Steward proposals on the site

## Request (verbatim)

> Owner decision briefs, part 1 of 3: render one-screen briefs from the real waiting Steward proposals.
> An earlier try was rejected because it read a new file format nothing writes, used relative record links that do not open on the published site, and gave every proposal the same fixed cost, risk and 'Approve'.
> - Add docs/conventions/owner-style.md (new) holding the owner's style rules verbatim: 'Use HTML to help me understand what we're discussing. Show me with big pictures and little words without witholding key details. Use your own taste, avoid lazy design, integrate real-world pictures/icons/visuals as needed but only selectively when it makes sense. When you do use prose/detail: Be clear, concise/terse, and avoid jargon. Explain things as if the viewer is new here, but dont use weird metaphors.'
> - In xtask/src/site.rs, read open Steward proposals (WorkItem records of kind Proposal, through athena-data, which xtask already depends on) and render one brief each on a decisions page linked from the index: the title, the rung, what it would change, the evidence it cites, and a recommendation taken from the proposal's own fields. Where a field is absent, the brief says so rather than inventing it.
> - Every record link uses the existing source_url so it opens at the pinned commit on GitHub.
> - The renderer refuses a brief over about 120 words of prose or with no recommendation line.
> - Tests in xtask build briefs from two fixture proposals with different fields and show the briefs differ, the links are GitHub URLs, and an over-long brief is refused.
> - CHANGELOG.md gets an entry.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

Add the owner's style rules as a doc, and have xtask's site build read open WorkItem proposals via athena-data and render one short brief each on a decisions page linked from the index. For the owner reviewing waiting proposals; briefs use only the proposal's own fields, say when one is absent, link records at the pinned commit, and are refused if over ~120 words or lacking a recommendation line.

Assumptions:
- Code survey: read xtask/src/site.rs (entry point build(root,out) renders the study then writes index.html; source_url(base, sha, path, start, end) and github_base exist; escape() for HTML) and crates/athena-data/src/records.rs (WorkItem has id, at, role, kind, key, title, rung, source, evidence: Vec<Reference>; WorkItemKind::Proposal) plus crates/athena-core/src/work_decisions.rs open_proposals and status.rs WorkProposalView. The brief is built from WorkItem fields directly. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Existing code: crates/athena-data/src/records.rs (WorkItem, Reference, RefKind) and xtask/src/site.rs (source_url, escape, build) are reused, not created. athena-core's open_proposals filters out decided items; xtask may not depend on athena-core, so the open filter may need decisions read via athena-data or a dependency added to xtask/Cargo.toml (governance-adjacent; only if needed).
- WorkItem has no 'what it would change' or explicit recommendation field. 'What it would change' is taken from the title and rung (and source); the recommendation line is derived from the proposal's own fields, e.g. the rung and the cited evidence count, and states plainly when the field is absent (e.g. 'No recommendation recorded') rather than a fixed 'Approve'. Such an explicit absence line counts as the recommendation line; the refusal fires when the line is missing altogether.
- Records live in the repository's record store read at the pinned commit; the decisions page loads them in build(), and a missing store yields an explicit 'no open proposals' page, not silent success.
- Record links go through source_url with the pinned GitHub base and sha, never relative paths. Path-kind references link to the file; job/commit references link to their record path or commit URL on GitHub.
- Word limit counts visible prose words of a brief (about 120); the exact counting is an implementation choice documented in code.
- Standing defaults: rust, tested (new unit tests in xtask), strangers, public, behavior-evidence, outcome-check, small.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
2. [content] CHANGELOG.md has a new entry describing the owner decision briefs page.
3. [content] xtask/src/site.rs reads WorkItem records of kind Proposal through athena-data and not from any new file format; Grep shows no new reader of an unwritten format.
4. [behavior] Run `cargo test -p xtask`: a test builds briefs from two fixture proposals with different title, rung, source and evidence fields and asserts the two briefs' text differs, including different recommendation lines (not a fixed 'Approve', cost or risk).
5. [behavior] Run `cargo test -p xtask`: a test asserts every record link in the rendered briefs starts with https://github.com/ and contains the pinned sha, with no relative links (no href beginning with a path or '../').
6. [behavior] Run `cargo test -p xtask`: a test asserts a brief over ~120 words of prose is refused with an error, and a brief with no recommendation line is refused with an error.
7. [behavior] Run `cargo test -p xtask`: a test shows an absent field (e.g. a proposal with no evidence) renders an explicit 'not recorded' statement rather than an invented value.
8. [content] The rendered index page contains a link to the decisions page (checked in the index render test), and the decisions page is written by build() next to index.html.
9. [content] The change refuses more briefs than before only in the new renderer; the existing study page rendering is unchanged (existing xtask site tests still pass).

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/src/decisions.rs (create)
- xtask/src/main.rs (modify)

## Out of scope

- Parts 2 and 3 of the owner decision briefs series
- Approving or rejecting proposals from the page
- Cost or risk figures not present in the proposal's fields
- Changes to athena.toml, defaults.toml or workflows
- Any new record or file format
- Repository-wide fmt, clippy or test runs as their own criteria

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover the briefs, links and refusals.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Briefs and docs are written for a first-time reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures contain no personal data or secrets.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check differing briefs, GitHub links and refusals.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that real proposals produce distinct, linked, bounded briefs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside site.rs plus doc and changelog.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty proposal list renders an explicit 'none waiting' statement, not silent success.
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

docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
CHANGELOG.md has a new entry describing the owner decision briefs page.
xtask/src/site.rs reads WorkItem records of kind Proposal through athena-data and not from any new file format; Grep shows no new reader of an unwritten format.
Run `cargo test -p xtask`: a test builds briefs from two fixture proposals with different title, rung, source and evidence fields and asserts the two briefs' text differs, including different recommendation lines (not a fixed 'Approve', cost or risk).
Run `cargo test -p xtask`: a test asserts every record link in the rendered briefs starts with https://github.com/ and contains the pinned sha, with no relative links (no href beginning with a path or '../').
Run `cargo test -p xtask`: a test asserts a brief over ~120 words of prose is refused with an error, and a brief with no recommendation line is refused with an error.
Run `cargo test -p xtask`: a test shows an absent field (e.g. a proposal with no evidence) renders an explicit 'not recorded' statement rather than an invented value.
The rendered index page contains a link to the decisions page (checked in the index render test), and the decisions page is written by build() next to index.html.
The change refuses more briefs than before only in the new renderer; the existing study page rendering is unchanged (existing xtask site tests still pass).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
