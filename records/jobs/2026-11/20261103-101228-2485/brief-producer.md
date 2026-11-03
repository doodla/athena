# Job 20261103-101228-2485: Retry owner decision briefs: distinct fixture sources, fetch records in

## Request (verbatim)

> try again on 20261103-092929-s7ts
> The last attempt met 8 of 9 criteria. Fix the two remaining points and keep everything else:
> - Give the two fixture proposals different sources (and different titles, rungs and evidence), and assert each brief shows its own proposal's source, so a hard-coded source line would fail the test.
> - The Pages workflow in .github/workflows/ checks out only main, so the published decisions page would always say the records branch is missing. Fetch the records branch in that workflow before the site is rendered, and add a test or workflow check that the fetch is present.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

Retry of job 20261103-092929-s7ts for the owner, who reviews Steward proposals on the published site. Redo the owner-style doc and xtask decisions page, and fix the two findings: fixture proposals now differ in source with per-brief source assertions, and pages.yml fetches the records branch before rendering, guarded by a test.

Assumptions:
- Job 20261103-092929-s7ts is retried from base; its work is kept as the approach and re-created: docs/conventions/owner-style.md, xtask/src/decisions.rs, xtask/src/site.rs, xtask/src/main.rs and CHANGELOG.md all remain deliverables, with the original decisions-page criteria (verbatim style doc, GitHub source_url links, 120-word and no-recommendation refusals, not-recorded evidence, role-limit recommendations, index link) kept unchanged.
- Finding 1 (major, same fixture source) is fixed by giving each fixture a different source (e.g. Reference::path and Reference::job) and asserting each brief shows its own source line. Finding 2 (Pages checks out only main) is fixed by editing .github/workflows/pages.yml to fetch athena/records into refs/heads/athena/records (RECORDS_REF in crates/athena-git/src/records_branch.rs) before 'cargo run -p xtask -- site', plus an xtask test reading the workflow file and asserting the fetch precedes the render step.
- Other findings of 20261103-092929-s7ts (duplicated role_limit, build() failing whole site on one bad proposal, hand-built record path, word count excluding Record/command lines) are minor/nit; the producer should fix the cheap ones (derive path from canonical_path or the tree entry, count all displayed prose) and need not change athena-core, which stays out of scope.
- Code survey: read crates/athena-git/src/records_branch.rs (resolve_tip returns Missing when neither refs/heads/athena/records nor refs/remotes/origin/athena/records exists), .github/workflows/pages.yml (default checkout, then xtask site), and xtask/src/ (site.rs, main.rs). Records are read through athena-data check_record_file/AnyRecord; no new record format or data file is added, so no allow-list change.
- The new refusals (over-long brief, missing recommendation) apply only inside the new decisions renderer: expected 0% of newly refused readings.
- pages.yml is a workflow file and is a deliverable named in a criterion; athena.toml and defaults.toml do not change.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
2. [content] CHANGELOG.md has a new entry describing the owner decision briefs page and the Pages records-branch fetch.
3. [content] The diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) with no new file format and no new dependency beyond what xtask already has.
4. [behavior] Run `cargo test -p xtask`: a test builds briefs from two fixture proposals that differ in title, rung, source (different kinds or paths) and evidence, and asserts each brief's title, rung, source line and evidence line match its own proposal and differ from the other brief, so a hard-coded source line would fail; briefs contain no fixed cost, risk or 'Approve'.
5. [behavior] Run `cargo test -p xtask`: with role fixtures, an in-limit and an above-limit proposal at the same rung give exact, different recommendation lines (the in-limit one not 'Decide yourself'), and a proposal with an unrecorded role says the role is not recorded.
6. [behavior] Run `cargo test -p xtask`: every record link begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
7. [behavior] Run `cargo test -p xtask`: a brief over about 120 words of prose (counting every displayed prose line) is refused with an error, and one with no recommendation line is refused; expected 0% of newly refused readings outside the new renderer.
8. [behavior] Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so.
9. [behavior] Run `cargo test -p xtask`: the index links to the decisions page, build() writes decisions.html next to index.html, and the existing study page tests pass unchanged.
10. [content] .github/workflows/pages.yml fetches the athena/records branch (into refs/heads/athena/records or the tracking ref) in a step before 'cargo run --locked -p xtask -- site', with the existing checkout pins and permissions otherwise unchanged.
11. [behavior] Run `cargo test -p xtask`: a test reads .github/workflows/pages.yml and fails if the records-branch fetch is absent or comes after the site render step.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/decisions.rs (create)
- xtask/src/site.rs (modify)
- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- .github/workflows/pages.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Parts 2 and 3 of the owner decision briefs series
- Approving or rejecting proposals from the page
- Cost or risk figures not present in the proposal's fields
- Changes to athena.toml or defaults.toml
- Any new record or file format
- Changes to athena-core, such as moving role_limit into athena-data or edits to split_open
- Repository-wide fmt, clippy or test runs as their own criteria

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust in xtask.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, changelog and comments written for an outside reader.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests; fmt, clippy and test stay green but are not listed as separate criteria, per the request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered briefs and the workflow content by test, not file existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The published decisions page is made able to list proposals, checked by the workflow-fetch test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the previous deliverables plus pages.yml change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261103-092929-s7ts raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/decisions.rs tests::item (source: Reference::path("docs/design/x.md")) and tests::fixtures): Criterion 4 is unmet: the two fixture proposals have the same source, so no test shows the brief's source line comes from the proposal. item() hard-codes the source for every fixture. The test asserts title, rung and evidence differ but never the source. A renderer that printed a fixed source would still pass. Fix: give each fixture its own source (for example Reference::job(...) on one) and assert each brief shows its own source line.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
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

26 more qualifying classes are left out.

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
CHANGELOG.md has a new entry describing the owner decision briefs page and the Pages records-branch fetch.
The diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) with no new file format and no new dependency beyond what xtask already has.
Run `cargo test -p xtask`: a test builds briefs from two fixture proposals that differ in title, rung, source (different kinds or paths) and evidence, and asserts each brief's title, rung, source line and evidence line match its own proposal and differ from the other brief, so a hard-coded source line would fail; briefs contain no fixed cost, risk or 'Approve'.
Run `cargo test -p xtask`: with role fixtures, an in-limit and an above-limit proposal at the same rung give exact, different recommendation lines (the in-limit one not 'Decide yourself'), and a proposal with an unrecorded role says the role is not recorded.
Run `cargo test -p xtask`: every record link begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
Run `cargo test -p xtask`: a brief over about 120 words of prose (counting every displayed prose line) is refused with an error, and one with no recommendation line is refused; expected 0% of newly refused readings outside the new renderer.
Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so.
Run `cargo test -p xtask`: the index links to the decisions page, build() writes decisions.html next to index.html, and the existing study page tests pass unchanged.
.github/workflows/pages.yml fetches the athena/records branch (into refs/heads/athena/records or the tracking ref) in a step before 'cargo run --locked -p xtask -- site', with the existing checkout pins and permissions otherwise unchanged.
Run `cargo test -p xtask`: a test reads .github/workflows/pages.yml and fails if the records-branch fetch is absent or comes after the site render step.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
