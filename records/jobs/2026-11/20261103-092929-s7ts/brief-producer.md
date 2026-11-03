# Job 20261103-092929-s7ts: Render owner decision briefs from open proposals, with role-limit

## Request (verbatim)

> try again on 20261103-075205-wbtm

## Reading (accepted by the user)

Retry of job 20261103-075205-wbtm (failed in planning, round 3): add the owner-style doc and a decisions page in xtask that renders one short brief per open Steward proposal for the owner. The recommendation now reads the raising role's limit from role records through athena-data, so in-limit proposals are not told 'Decide yourself'.

Assumptions:
- Goal and audience: the owner reviewing waiting Steward proposals on the published site; briefs are one screen, plain, and use only the proposal's own fields.
- Previous job 20261103-075205-wbtm produced no candidate (no files changed), so nothing is kept or dropped; every deliverable below is built fresh. Its findings are addressed as: (major) the recommendation reads the role limit from role records instead of guessing; (minor) the run command is plain `cargo test -p xtask` so every demonstrating test runs; (nit) the records-link pin is stated as Pin { base: ATHENA_URL, sha: records tip }, and the link test asserts that base.
- Code survey: read crates/athena-core/src/work_decisions.rs (split_open at line 73: open proposals within the raising role's limit are approved without a person), crates/athena-data/src/roles.rs (RoleDefinition, may_change at lines 57 and 106), crates/athena-git/src/records_branch.rs (resolve_tip, RecordsTip) and crates/athena-git/src/lib.rs (ls_tree_under, cat_blobs). Entry point is site::build in xtask; the records tip is resolved first, then records and role records are listed and read in one cat_blobs call, parsed with check_record_file into AnyRecord. No new record or file format is added, so the validation allow-list and the round-trip test are untouched. The recommendation is measured by asserting the exact rendered line per fixture.
- Existing code: crates/athena-core/src/work_decisions.rs, crates/athena-data/src/roles.rs, crates/athena-git/src/records_branch.rs, xtask/src/site.rs (source_url, ATHENA_URL). The recommendation reuses the same limit rule via athena-data's may_change and the role limit (newest version by name, then highest_rung), rather than a per-rung table.
- No new dependencies: xtask already depends on athena-data and athena-git, so xtask/Cargo.toml and Cargo.lock are listed only at low probability.
- Rung/limit sentence choice: role not recorded; rung above the role's limit (decide yourself); rung in the top two (no role may change it, decide yourself); rung within the limit (Athena approves it without you; check only if you disagree). Each brief keeps a first-evidence pointer and the approve-work/reject-work command.

## Acceptance criteria

1. [content] docs/conventions/owner-style.md exists and contains the owner's style paragraph verbatim, including 'Show me with big pictures and little words without witholding key details' and 'dont use weird metaphors'.
2. [content] CHANGELOG.md has a new entry describing the owner decision briefs page.
3. [content] Grep of the diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) and introduces no new file format and no new dependency.
4. [behavior] Run `cargo test -p xtask`: a test builds briefs from two fixture proposals with different title, rung, source and evidence and asserts the briefs differ, with no fixed cost, risk or 'Approve'.
5. [behavior] Run `cargo test -p xtask`: with role fixtures, one proposal within the role's limit and one above it at the same rung produce exact, different recommendation lines; the in-limit one does not say 'Decide yourself', and a proposal with an unrecorded role says the role is not recorded.
6. [behavior] Run `cargo test -p xtask`: a test asserts every record link in the briefs begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
7. [behavior] Run `cargo test -p xtask`: a brief over about 120 words of prose is refused with an error, and a brief with no recommendation line is refused with an error. Expected 0% of newly refused readings outside the new renderer.
8. [behavior] Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so rather than reporting success.
9. [behavior] Run `cargo test -p xtask`: the rendered index contains a link to the decisions page, build() writes the decisions page next to index.html, and existing study page tests still pass unchanged.

## Expected deliverables

- docs/conventions/owner-style.md (create)
- xtask/src/site.rs (modify)
- xtask/src/decisions.rs (create)
- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Parts 2 and 3 of the owner decision briefs series
- Approving or rejecting proposals from the page
- Cost or risk figures not present in the proposal's fields
- Changes to athena.toml, defaults.toml or workflows
- Any new record or file format
- Changes to athena-core, such as new edges or edits to split_open
- Repository-wide fmt, clippy or test runs as their own criteria

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests under cargo test -p xtask.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert rendered lines, links and refusals.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The recommendation derives from the role limit read from records, not a guess.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A missing records branch gives an explicit empty-state page, tested.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Doc and briefs carry no personal data.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses athena-data, athena-git and source_url; no new dependency.
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
CHANGELOG.md has a new entry describing the owner decision briefs page.
Grep of the diff shows xtask reads WorkItem proposals and role records through athena-data (check_record_file/AnyRecord) and introduces no new file format and no new dependency.
Run `cargo test -p xtask`: a test builds briefs from two fixture proposals with different title, rung, source and evidence and asserts the briefs differ, with no fixed cost, risk or 'Approve'.
Run `cargo test -p xtask`: with role fixtures, one proposal within the role's limit and one above it at the same rung produce exact, different recommendation lines; the in-limit one does not say 'Decide yourself', and a proposal with an unrecorded role says the role is not recorded.
Run `cargo test -p xtask`: a test asserts every record link in the briefs begins with the Athena GitHub base (ATHENA_URL) and contains the records tip sha, and no href is relative.
Run `cargo test -p xtask`: a brief over about 120 words of prose is refused with an error, and a brief with no recommendation line is refused with an error. Expected 0% of newly refused readings outside the new renderer.
Run `cargo test -p xtask`: a proposal with no evidence renders an explicit 'not recorded' statement, and a checkout without the records branch lists no proposals and the page says so rather than reporting success.
Run `cargo test -p xtask`: the rendered index contains a link to the decisions page, build() writes the decisions page next to index.html, and existing study page tests still pass unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
