# Job 20261104-153450-y72x: Amend D08 so docs match read-only commit history access

## Request (verbatim)

> try again on 20261010-225741-yvk0

## Reading (accepted by the user)

Retry of job 20261010-225741-yvk0: its only major finding was that docs/decisions.md D08 (line 233, "agents lose `git log` of the real history. That is the point.") still contradicts the new read-only commit view. Audience is the verifier and later readers of the decision log. This retry is docs-only: amend D08, mark it "Amended by D155", and ensure D155 lists alternatives and tradeoffs.

Assumptions:
- Job 20261010-225741-yvk0 passed fmt, clippy and test and failed only on one unmet content criterion (D08), so I assume the producer starts from its candidate (code, tests, CHANGELOG, enforcement.md and the D155 hunk already present). The repository copy I read has no D155 and D08 line 233 is unchanged, so if the producer instead starts from base, it must also write D155 (the new decision) in docs/decisions.md; the criteria cover both cases.
- Findings of 20261010-225741-yvk0: (major) D08 not amended: fixed by rewording line 233 and adding 'Amended by D155'. (minor) run/mod.rs and run_pin_tests.rs changed outside deliverables: no longer applies, they are kept as-is and excluded in out_of_scope as needed wiring. (nit) hashes taken from request text only: no longer applies, it is a documented limit in D155 and CHANGELOG and stays.
- Other docs saying agents never see real history: Grep found D08 line 233 as the clear one; docs/design/athena-v0.md:87 and :449 refer to D08 and should be read and reworded only if they claim no history access. docs/enforcement.md and docs/design/*.md are touched only if such a sentence exists.

## Acceptance criteria

1. [content] docs/decisions.md D08 Tradeoff line (around line 233) no longer says agents lose the real history; it says agents can read the commits a job names through a filtered read-only view and the rest of history stays hidden, and D08 contains 'Amended by D155' (as at 'Amended by D63 and D115'). Check with Grep for 'That is the point' and 'Amended by D155' in docs/decisions.md.
2. [content] docs/decisions.md has a D155 entry that lists alternatives (no history access, full history access, generated file only) with the tradeoff of each, and states the request-text-only hash limit. Check by reading the entry.
3. [content] Grep of docs/ for sentences saying agents never see or lose the real history finds none left unreworded or unmarked; a reader checking D08 can follow it to D155.
4. [content] Only files under docs/ differ in this retry's diff versus the candidate; no file under crates/, athena.toml, defaults.toml or workflows changes.
5. [behavior] Run `cargo test` and `cargo clippy` and `cargo fmt --check`; all still pass, showing the docs edit broke nothing (docs-lint or doc-reference tests included).

## Expected deliverables

- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any change to the candidate's code: crates/athena-git/src/snapshot.rs, crates/athena-git/src/lib.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-context/src/briefs.rs are kept as the earlier job left them, not edited here
- Extracting hashes from brief text as well as request text
- Making full history available to every agent, or any git write access
- Edits to athena.toml, defaults.toml, or workflows
- Building the proposed decision-to-claim consistency check (a separate job)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: D155 and the D08 amendment cite the exact hidden-path rules and tests in the candidate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision text is written for an outside reader, with alternatives and tradeoffs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change, code left untouched.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the sentence is gone and the pointer exists via Grep, not just that a file changed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Searches all docs for the same stale claim, not only D08.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261010-225741-yvk0

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> docs/decisions.md D08 (or a new decision) records the change with alternatives and tradeoffs, and no longer says agents never see real history.: New D155 records the change with alternatives (a) to (f) and their tradeoffs. However, the decisions.md diff is a single hunk at line 4231, and D08 is untouched. docs/decisions.md:233 still reads "Each snapshot is a copy, and agents lose `git log` of the real history. That is the point.", with no amendment note. D155 itself describes this as "D08's rule that agents never see the real history". The planned demonstration said the old sentence would be reworded or removed.

## Findings you must address

The verifier of job 20261010-225741-yvk0 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/decisions.md:233): D08 still says agents lose the real history and was neither reworded nor marked as amended, so criterion 7 is unmet. `git diff HEAD~1 -- docs/decisions.md` has only the appended D155 hunk. docs/decisions.md:233 is unchanged: "agents lose `git log` of the real history. That is the point." Elsewhere the repo marks amended decisions in place (for example "Amended by D63 and D115" at line 982, and "Superseded by D88" at line 1909). Fix: reword the D08 Tradeoff line, or add "Amended by D155" there.

All of the verification's findings:

> major (docs/decisions.md:233): D08 still says agents lose the real history and was neither reworded nor marked as amended, so criterion 7 is unmet. `git diff HEAD~1 -- docs/decisions.md` has only the appended D155 hunk. docs/decisions.md:233 is unchanged: "agents lose `git log` of the real history. That is the point." Elsewhere the repo marks amended decisions in place (for example "Amended by D63 and D115" at line 982, and "Superseded by D88" at line 1909). Fix: reword the D08 Tradeoff line, or add "Amended by D155" there.
> minor (crates/athena-core/src/run/mod.rs:1502): Two changed files are outside the expected deliverables list. crates/athena-core/src/run/mod.rs (a one-line switch to verifier_snapshot_for) and crates/athena-core/src/run_pin_tests.rs (the wiring test) are not in the deliverables list. Both were in the accepted plan, both are needed, and neither adds behavior beyond the request.
> nit (crates/athena-core/src/run/producer.rs:43): Hashes are extracted only from the request text, not from the brief text as well. `requested_commits` reads only `ctx.job.request.text`. The reading said "request/brief text". D155 and the CHANGELOG document "request text only", so this is a documented limit, not a defect.

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

The work starts from the rejected candidate of job 20261010-225741-yvk0: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/decisions.md D08 Tradeoff line (around line 233) no longer says agents lose the real history; it says agents can read the commits a job names through a filtered read-only view and the rest of history stays hidden, and D08 contains 'Amended by D155' (as at 'Amended by D63 and D115'). Check with Grep for 'That is the point' and 'Amended by D155' in docs/decisions.md.
docs/decisions.md has a D155 entry that lists alternatives (no history access, full history access, generated file only) with the tradeoff of each, and states the request-text-only hash limit. Check by reading the entry.
Grep of docs/ for sentences saying agents never see or lose the real history finds none left unreworded or unmarked; a reader checking D08 can follow it to D155.
Only files under docs/ differ in this retry's diff versus the candidate; no file under crates/, athena.toml, defaults.toml or workflows changes.
Run `cargo test` and `cargo clippy` and `cargo fmt --check`; all still pass, showing the docs edit broke nothing (docs-lint or doc-reference tests included).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
