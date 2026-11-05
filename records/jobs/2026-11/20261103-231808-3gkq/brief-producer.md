# Job 20261103-231808-3gkq: Fix decisions page to list only open, above-limit proposals

## Request (verbatim)

> try again on 20261103-201526-t4qv: its reading was refused because the reader assumed a design file (docs/design/outcome-check-overrates.md) that does not exist at the base commit. The request needs no design file: it changes only which steward proposals the decisions page lists (open and above the steward's limit), read from the approve-work and reject-work decision records.

## Reading (accepted by the user)

Retry of job 20261103-201526-t4qv: the owner reading decisions.html must see only proposals that are open (no approve-work or reject-work record names them) and above the raising role's limit; with none, one line 'Nothing waiting for you'. No design file is needed or assumed. The producer must trace and name why decided proposals reached the published page, and prove the fix with a test that fails on the base.

Assumptions:
- Goal and audience: the owner reading https://doodla.github.io/athena/decisions.html, who should be asked only about proposals that are theirs to decide.
- Previous job 20261103-201526-t4qv (rejected, verdict fail): its candidate changes to CHANGELOG.md and xtask/src/decisions.rs are DROPPED; this retry starts from the base. Its findings are addressed as follows. Major (root cause not named; only an existing filter was moved): the producer must trace and state the cause and fix it there, and the changelog line must say only what the diff does. Minor (in_limit copied into xtask): reuse one definition, or state in the code why xtask keeps its own rule. Minor (records-branch test also passes on old code): the new tests must fail on the base, with the failing run shown in the check output.
- Code survey: crates/athena-core/src/work_decisions.rs (role_limit, in_limit, split_open, approve_work, reject_work) and crates/athena-core/src/commits.rs (commit_records, Committer::advance), plus xtask/src/decisions.rs. read_snapshot() reads work items, work decisions and roles from the records branch tip, then also reads records/work-decisions from HEAD (MAIN_TREE), and drops decided ids with proposals.retain(). render() applies NO limit check, so a within-limit proposal such as ne91 is listed (a certain defect at base). The empty state is currently 'No proposals waiting'. approve_work and reject_work commit decisions through commit_records onto the records branch. xtask does not depend on athena-core. .github/workflows/pages.yml fetches athena/records with --depth=1 only on a push to main or manual dispatch.
- Root-cause leads for the producer to check (not confirmed; records/ is absent from the snapshot): (a) in-limit proposals are never filtered at base; (b) decisions are committed to the records branch, which may not be pushed or may differ in the fetched copy, while read_snapshot's HEAD read of MAIN_TREE assumes main (its comment says so); (c) a decision tied to a dropped job such as 20261030-081751-edsg may exist only where the page never reads; (d) the page is stale because pages.yml runs only on a push to main. The producer must pick the supported cause by reading code and building fixtures, and report it. If the cause lies in workflow or push behavior outside scope, name it in the changelog and report, and still make the renderer correct for every data source it reads.
- Code check: records validation uses check_record_file and AnyRecord in athena-data; the change adds no record or data file, so no allow-list change is needed. The rule is measured directly by rendering Snapshot/Records values and by running locate() on a temp git repo that holds real WorkItem and WorkDecision records.
- A role with no recorded definition counts as above the limit, matching split_open in athena-core.
- This change adds no refusal rule. It removes entries from a rendered page only.

## Acceptance criteria

1. [behavior] A new test in xtask/src/decisions.rs builds an approved, a rejected, and a within-limit open proposal, using the real WorkItem and WorkDecision shapes. It asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'waiting</h2>' count heading. Run: cargo test -p xtask decisions.
2. [behavior] A new test with approved, rejected, open-above-limit and open-below-limit proposals together asserts render() shows exactly one '<article' card, for the open above-limit one, with heading '1 waiting'. Run: cargo test -p xtask decisions.
3. [behavior] A test through locate() on a temp git repo (records branch plus decision records in the place(s) where approve-work and reject-work really write them) shows the decided proposals absent from the page. The recorded check output shows this test failing on the old code and passing on the new. Run: cargo test -p xtask decisions.
4. [content] The CHANGELOG.md entry names where decided or within-limit proposals entered the page and why they were not excluded, and says only what the diff changes; nearby 'Root cause:' entries are the model. Check with git diff CHANGELOG.md.
5. [content] The empty state in render() of xtask/src/decisions.rs is one line reading 'Nothing waiting for you', replacing 'No proposals waiting'. The missing-records-branch message ('No proposals listed') stays distinct. The module doc comment states that only open, above-limit proposals are listed. Conflicting existing tests are updated. Check with Grep and git diff.
6. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass at the repository root.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing records/ files, or recording decisions for work items 20261016-055247-qyd9, 20261017-204324-jtd2 or 20261018-000648-ne91
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows (including pages.yml; if the cause is there, name it in the report)
- Job 20261030-081751-edsg and its docs and design work (docs/design/outcome-check-overrates.md, docs/decisions.md, docs/design/audience-role.md): not continued, nothing kept
- Changing approve-work or reject-work behavior or the record schemas in crates/athena-data and crates/athena-core
- Redesigning the card layout or the 160-word brief limit
- Kept from job 20261103-201526-t4qv: nothing; its candidate edits to xtask/src/decisions.rs and CHANGELOG.md are dropped and redone

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The reading requires naming where decided and within-limit proposals enter the page and fixing it there, not moving an existing filter.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria use fixtures of real record shapes, require tests that fail on the base, and check the rendered output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy --all-targets and fmt --check must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that no decided or within-limit proposal appears on the rendered page.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The changelog and doc comment are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the renderer, its tests and the changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo commands and reads the diff.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261103-201526-t4qv raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/decisions.rs read_snapshot()/render(); CHANGELOG.md): The root cause of approved and rejected proposals showing on the published page is neither found, fixed nor named; for decided proposals the change only moves an existing filter. In the old code, read_snapshot already ran `proposals.retain(|p| !decided.contains(&p.item.id));` (the line removed in the diff), and the old test the_page_links_every_record_to_github_at_the_tip_and_lists_only_open_proposals already checked it. Moving the check into render() changes nothing for decided proposals. So whatever made qyd9 (approved) and jtd2 (rejected) show on 3 Nov is untouched. Reading the code suggests a stale page: crates/athena-core/src/work_decisions.rs approve_work/reject_work commit the decision to the records branch at once, but .github/workflows/pages.yml rebuilds decisions.html only on `push: branches: [main]`. The other possibility is that the decision records are missing from the branch. Neither can be checked in this snapshot, which has no records/. The workflow is out of scope, so the root-cause rule required naming the cause and proposing a fix. The CHANGELOG line names no cause, while nearby entries ('Root cause: ...') do. The reading's assumptions also asked the producer for this.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261103-231808-3gkq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261103-231808-3gkq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in xtask/src/decisions.rs builds an approved, a rejected, and a within-limit open proposal, using the real WorkItem and WorkDecision shapes. It asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'waiting</h2>' count heading. Run: cargo test -p xtask decisions.
A new test with approved, rejected, open-above-limit and open-below-limit proposals together asserts render() shows exactly one '<article' card, for the open above-limit one, with heading '1 waiting'. Run: cargo test -p xtask decisions.
A test through locate() on a temp git repo (records branch plus decision records in the place(s) where approve-work and reject-work really write them) shows the decided proposals absent from the page. The recorded check output shows this test failing on the old code and passing on the new. Run: cargo test -p xtask decisions.
The CHANGELOG.md entry names where decided or within-limit proposals entered the page and why they were not excluded, and says only what the diff changes; nearby 'Root cause:' entries are the model. Check with git diff CHANGELOG.md.
The empty state in render() of xtask/src/decisions.rs is one line reading 'Nothing waiting for you', replacing 'No proposals waiting'. The missing-records-branch message ('No proposals listed') stays distinct. The module doc comment states that only open, above-limit proposals are listed. Conflicting existing tests are updated. Check with Grep and git diff.
cargo test, cargo clippy --all-targets and cargo fmt --check all pass at the repository root.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
