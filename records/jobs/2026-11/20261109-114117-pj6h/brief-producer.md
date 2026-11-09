# Job 20261109-114117-pj6h: Make the steward decide itself; ask the owner only for env or token

## Request (verbatim)

> try again on 20261109-080220-zkhy

## Reading (accepted by the user)

Retry of 20261109-080220-zkhy. The steward steps back across repeated failures and blind spots and queues the fix itself. The owner gets a five-field brief only for an item that needs an environment change or a token Athena lacks. Code in steward.rs and work_decisions.rs enforces that routing, not only the prompt. The audience is the owner (Sri), who judges by how few decisions reach them.

Assumptions:
- Previous result, job 20261109-080220-zkhy (rejected, 10 findings): its prompt-only approach is dropped and rebuilt. Its candidate's six changed files (CHANGELOG.md, steward.rs, records.rs, roles.rs, docs/decisions.md, docs/design/roles-and-ladder.md) are kept as deliverables, redone from base. Findings addressed: (1) env/token never reached the owner: decide() now routes by the presence of the brief. (2) over-limit proposals reached the owner without a brief: open_proposals now lists only briefed proposals. (3) 600-token budget unmeasured on the real prompt: a new test composes the real STEWARD_PROMPT. (4) missing brief tests: added in steward.rs and crates/athena-cli/tests/steward.rs. (5) records.rs edit out of scope: it is now a deliverable, a path update only. (6) misread request: the routing is fixed as above.
- Code survey: read crates/athena-core/src/steward.rs. decide() (about line 693) applies a change when role.may_change(rung) and keeps change.brief only for Decision::Proposed. blind_spot_change() (about line 739) is on Ladder::Implementation with brief None. crates/athena-core/src/work_decisions.rs open_proposals() (about line 43) lists every kind Proposal item with no decision. Plan: a change carrying a brief means the steward judged it needs an environment or token the owner must supply, so decide() records it as Decision::Proposed with the brief whatever the rung. A change without a brief is applied if within the limit and otherwise recorded without reaching the owner. open_proposals lists only items with a brief. A missing-token blind spot gets a briefed constructor. The measured check is the item's brief field and open_proposals output in tests. Records validation: crates/athena-data/src/records.rs check_record_file returns AnyRecord and a round-trip test covers the role file. The steward role bumps to version 3 (steward-3.json), so the path asserted at records.rs:2265 and roles.rs:306 (steward-2.json) is updated. check_record_file and the round-trip test accept the new role record unchanged. Prompt budget: the Steward budget is 600 tokens, checked by check_budget in crates/athena-context/src/lib.rs, which today composes a placeholder. Add a test on the real STEWARD_PROMPT.
- Term ladder: the ordered rungs of change (Ladder::Implementation lowest, up to design). A role may change itself only up to its highest_rung; above it the change is only proposed.
- The steward prompt is a model prompt, so the step-back and decide-yourself wording is prose-only. The routing of briefs and owner decisions is enforced by code and tests.
- The change adds no refusal rule. It does not touch athena.toml, defaults.toml, workflows or records/, so no refusal-rate figure applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A new steward.rs test shows a change with no env/token need and no brief, at a rung above the limit, produces a work item with brief None that open_proposals does not list.
2. [behavior] Run `cargo test -p athena-core -p athena-cli`. A test shows a change with a five-field brief (an env or token need, e.g. a missing GH_TOKEN) is recorded as Proposed with its brief at any rung, and open_proposals lists it. A crates/athena-cli/tests/steward.rs test exercises this end to end.
3. [content] Grep/Read of the diff: crates/athena-core/src/work_decisions.rs open_proposals and crates/athena-core/src/steward.rs decide() carry the routing rule above. blind_spot_change or a new constructor gives a missing-token source a brief.
4. [content] Grep/Read of crates/athena-data/src/roles.rs: STEWARD_PROMPT tells the steward to step back across repeated failures and blind spots, to decide and queue a request itself, and to write a brief only for an environment change or a token or credential. A roles.rs test asserts this wording.
5. [behavior] Run `cargo test -p athena-data -p athena-context`. A test composes the real STEWARD_PROMPT with the contract and asserts the estimated tokens are at most 600. The steward role is version 3, and the steward-3.json path is updated in roles.rs and records.rs tests.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass clean.
7. [content] CHANGELOG.md has an entry for the new steward behaviour. docs/design/roles-and-ladder.md and docs/decisions.md no longer describe the old behaviour (briefs for every over-limit proposal). The docs mark the prompt rule prose-only and the routing as code-enforced. The change does not touch athena.toml, defaults.toml, .github/workflows or records/. Check with git diff.

## Expected deliverables

- crates/athena-data/src/roles.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the steward's highest_rung or cadence
- Adding a code check that refuses briefs
- Changes to athena.toml, defaults.toml, .github/workflows or records/
- Changing the drain, audience or auditor roles

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the owner-routing behavior (briefs and open_proposals), not just that the wording exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover each routing case; cargo test, clippy and fmt must be clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Routing is enforced in code; the prompt wording is marked prose-only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261109-080220-zkhy raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/steward.rs:708-712, 739-749): The env/token exception, the one case the owner wants to be asked about, never reaches the owner as a brief `decide()` keeps `change.brief` only for `Decision::Proposed` (rung above the limit) and drops it for `Applied`. The prompt says to write a brief for an environment change or a token, but never tells the steward to rank such an item above its limit. The prompt's own example, a missing GH_TOKEN, shows up as an unreadable `gh` source. `blind_spot_change` hard-codes that at `Ladder::Implementation` with `brief: None`, so it becomes a queued request and the owner is never asked. A sounder approach makes env/token needs a distinct item kind, or routes them to the owner whatever the rung.
> major (crates/athena-core/src/work_decisions.rs:43-51): Proposals above the limit still go to the owner, now without a brief `open_proposals` lists every Proposal work item that has no decision, whether or not it has a brief. After this change, a design-rung proposal the steward raises still lands on the owner as an open decision. The prompt now forbids the brief, so the owner gets the same decision with less to go on. That is the opposite of "I don't want decisions". The reading kept this route, but the result does not cut the decisions that reach the owner.
> major (crates/athena-data/src/roles.rs:120-129; crates/athena-context/src/lib.rs:430-440): The new steward prompt likely exceeds the 600-token Steward budget, and no test checks the real prompt The budget test composes a placeholder prompt, not STEWARD_PROMPT. Estimate from counting bytes by hand: contract about 928 bytes plus prompt about 971 bytes plus separators, about 1,902 bytes, about 634 tokens at ceil(bytes/3), over the budget of 600. Not measured. Criterion 4 assumed the existing test covers this; it does not.
> major (crates/athena-core/src/steward.rs tests; crates/athena-cli/tests/steward.rs): The required test for briefs with and without an env/token need is missing The only new steward.rs test lines assert prose substrings. No test shows that an item with no env/token need gets no brief while one with such a need keeps its five-field brief. crates/athena-cli/tests/steward.rs is a listed deliverable but was not modified.
> major: The reading does not match the request, so passing criteria do not show the request was met The owner wants to be asked only about environment changes or tokens. The reading keeps the existing route, where every proposal above the steward's rung becomes an open proposal for the owner, and changes only whether that proposal has a brief. So the owner is still sent non-env/token decisions; they just arrive without the plain-language brief. The reverse case also fails. The code keeps a brief only on items above the steward's limit, and a missing-GH_TOKEN blind spot sits on the lowest rung, so the one kind of decision the owner wants never reaches them as a brief.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 13 cases in 13 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a reading's deliverable states a limit, exception or "only N" claim about Athena's behavior, require the reading and plan to derive the list from the enforcing source (the admission path classification code and config) and cite it. Add a criterion that each listed item maps to that source and t...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

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

- path: <state>/jobs/20261109-114117-pj6h/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-114117-pj6h/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`. A new steward.rs test shows a change with no env/token need and no brief, at a rung above the limit, produces a work item with brief None that open_proposals does not list.
Run `cargo test -p athena-core -p athena-cli`. A test shows a change with a five-field brief (an env or token need, e.g. a missing GH_TOKEN) is recorded as Proposed with its brief at any rung, and open_proposals lists it. A crates/athena-cli/tests/steward.rs test exercises this end to end.
Grep/Read of the diff: crates/athena-core/src/work_decisions.rs open_proposals and crates/athena-core/src/steward.rs decide() carry the routing rule above. blind_spot_change or a new constructor gives a missing-token source a brief.
Grep/Read of crates/athena-data/src/roles.rs: STEWARD_PROMPT tells the steward to step back across repeated failures and blind spots, to decide and queue a request itself, and to write a brief only for an environment change or a token or credential. A roles.rs test asserts this wording.
Run `cargo test -p athena-data -p athena-context`. A test composes the real STEWARD_PROMPT with the contract and asserts the estimated tokens are at most 600. The steward role is version 3, and the steward-3.json path is updated in roles.rs and records.rs tests.
Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass clean.
CHANGELOG.md has an entry for the new steward behaviour. docs/design/roles-and-ladder.md and docs/decisions.md no longer describe the old behaviour (briefs for every over-limit proposal). The docs mark the prompt rule prose-only and the routing as code-enforced. The change does not touch athena.toml, defaults.toml, .github/workflows or records/. Check with git diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
