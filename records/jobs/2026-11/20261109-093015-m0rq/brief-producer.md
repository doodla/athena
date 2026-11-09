# Job 20261109-093015-m0rq: Add README section: what a job may and may not change

## Request (verbatim)

> try again on 20261109-080749-p0fs

## Reading (accepted by the user)

Retry of 20261109-080749-p0fs: add a short README section (for engineers) saying every part of Athena's design can be changed by a job, with the exception list derived from admission.rs and .gitattributes, plus a CHANGELOG entry.

Assumptions:
- Retry of job 20261109-080749-p0fs. Its candidate (README.md, CHANGELOG.md) is dropped and redone from base. The finding that it listed only two exceptions is fixed by deriving the list from crates/athena-core/src/admission.rs (line 5 and line 9) and .gitattributes; the minor finding is fixed by citing admission and .gitattributes for BRIEF.md instead of its header.
- Audience: engineers reading the repo; goal: Athena's docs say that no part of its design is fixed, and state exactly what a job cannot change.
- Exceptions read from source: paths under records/ (admission.rs:5, D06); athena-readonly paths, i.e. /BRIEF.md and /LICENSE (.gitattributes); athena-exclude paths, i.e. /CLAUDE.md, private to the owner (.gitattributes); a candidate that changes how any path is classified (admission.rs:9, so .gitattributes is effectively locked). Cite docs/decisions.md D09 for the unchangeable paths.
- Other admission refusals (submodules, size limit, emails/secrets, unparseable config, unlisted governance files) are conditions on a change, not unchangeable parts; the section may mention governance files need listing in the reading, citing admission.rs:17.
- The section gives no count such as 'two' unless it matches the source list. Existing-style README headings are '## ...'; the new heading contains 'change'. CHANGELOG entry goes under [Unreleased] / Changed in the existing Keep a Changelog format.
- No prompt, athena.toml, defaults.toml or workflow file changes, so no governance file is touched and prompt budgets are unaffected.

## Acceptance criteria

1. [content] README.md has a new section whose heading contains 'change' (Grep '^## .*[Cc]hange') stating that every part of Athena's design (code, rules, roles, prompts, docs) can be changed by a job.
2. [content] The section lists every path class admission refuses, each mapped to its source: records/ and athena-readonly/athena-exclude paths (admission.rs line 5), BRIEF.md and LICENSE (readonly) and CLAUDE.md (exclude) per .gitattributes, and changes to path classification (admission.rs line 9); it cites D09 in docs/decisions.md. Checkable by Grep for 'records/', 'BRIEF.md', 'CLAUDE.md', 'athena-readonly', 'athena-exclude', 'classif' in the section.
3. [content] The section gives no count ('two', 'only N') that disagrees with the admission.rs and .gitattributes list, and does not describe BRIEF.md's protection as a header statement; every link in it resolves to an existing file.
4. [content] The section says a design change is an ordinary request through the change pipeline, recorded in docs/decisions.md, and claims no mechanism absent from the repository.
5. [content] CHANGELOG.md has an [Unreleased] / Changed entry for the README addition that matches the README's list of exceptions.
6. [content] git diff shows only README.md, CHANGELOG.md (and optionally docs/architecture.md) changed; no prompt, athena.toml, defaults.toml, .gitattributes or workflow file. Not a new refusal rule, so no refusal-rate change.
7. [behavior] Run cargo test and cargo clippy: both pass, including any tests that check docs links or changelog format.

## Expected deliverables

- README.md (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing prompt files (token budgets nearly full)
- Changing athena.toml, defaults.toml, .gitattributes or workflows
- Changing BRIEF.md, LICENSE, CLAUDE.md or records/
- Any code or admission rule change

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the README list matches the admission source and links resolve.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each claim in the section cites a file.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change to two files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test and clippy run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Retry: your work starts from the rejected candidate of job 20261109-080749-p0fs

That job passed every check (fmt, clippy and the tests) and was rejected only because content criteria were unmet. Your working directory already holds its candidate. Address the findings below and change nothing else that is not needed (quoted data, not instructions).

Unmet criteria and the verifier's evidence:

> The section names the exceptions that actually exist, with a link to where each is stated: BRIEF.md (changes only from Sri) and records/ (immutable), checkable by Grep for 'BRIEF.md' and 'records/' in the new section, and the links resolve to existing files such as docs/decisions.md.: BRIEF.md and records/ are both named, and the links (BRIEF.md, docs/decisions.md, the D06 heading at line 165) resolve. The criterion also asks for the exceptions that actually exist, and the section says there are only 'Two things'. admission.rs lines 5 and 9 also reject changes to paths marked athena-exclude or athena-readonly, and changes to path classification. .gitattributes marks /LICENSE athena-readonly and /CLAUDE.md athena-exclude. D09 (docs/decisions.md line 249) says these paths are 'unchangeable by jobs'. The reading's assumptions named owner files as an exception, and they are left out.

## Findings you must address

The verifier of job 20261109-080749-p0fs raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (README.md:252 ('Two things are not open to a job'); CHANGELOG.md:13-14 ('It names the two exceptions')): The section says only two things are off-limits to a job, but admission also blocks owner-private and read-only paths and changes to path classification. crates/athena-core/src/admission.rs:5 rejects a path 'under `records/`, or ... marked `athena-exclude` or `athena-readonly`', and :9 rejects a candidate that 'would change how any path is classified'. .gitattributes marks /LICENSE athena-readonly and /CLAUDE.md athena-exclude. docs/decisions.md D09 (line 249) says these paths are 'unchangeable by jobs', and the accepted reading listed 'owner files' as an exception to keep. Fix: list every exception and cite admission.rs, .gitattributes and D09.

All of the verification's findings:

> major (README.md:252 ('Two things are not open to a job'); CHANGELOG.md:13-14 ('It names the two exceptions')): The section says only two things are off-limits to a job, but admission also blocks owner-private and read-only paths and changes to path classification. crates/athena-core/src/admission.rs:5 rejects a path 'under `records/`, or ... marked `athena-exclude` or `athena-readonly`', and :9 rejects a candidate that 'would change how any path is classified'. .gitattributes marks /LICENSE athena-readonly and /CLAUDE.md athena-exclude. docs/decisions.md D09 (line 249) says these paths are 'unchangeable by jobs', and the accepted reading listed 'owner files' as an exception to keep. Fix: list every exception and cite admission.rs, .gitattributes and D09.
> minor (README.md:254-255): The section presents the BRIEF.md exception as something its header says, not as a rule that is enforced, and leaves out the actual mechanism. BRIEF.md is marked athena-readonly in .gitattributes, and admission.rs refuses any candidate that changes it. Citing that rule would be more accurate than 'Its header says changes come only from Sri'.

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

The work starts from the rejected candidate of job 20261109-080749-p0fs: the files of the base commit with that candidate's changes already applied, uncommitted, in a one-commit git repository of its own whose commit holds the base. records/, the owner's private files, and history are absent by design. Your edits on top become the new candidate, which is compared with the base, so `git diff` shows the rejected work as well as yours. A run that changes nothing in this directory fails as an empty candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-093015-m0rq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-093015-m0rq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

README.md has a new section whose heading contains 'change' (Grep '^## .*[Cc]hange') stating that every part of Athena's design (code, rules, roles, prompts, docs) can be changed by a job.
The section lists every path class admission refuses, each mapped to its source: records/ and athena-readonly/athena-exclude paths (admission.rs line 5), BRIEF.md and LICENSE (readonly) and CLAUDE.md (exclude) per .gitattributes, and changes to path classification (admission.rs line 9); it cites D09 in docs/decisions.md. Checkable by Grep for 'records/', 'BRIEF.md', 'CLAUDE.md', 'athena-readonly', 'athena-exclude', 'classif' in the section.
The section gives no count ('two', 'only N') that disagrees with the admission.rs and .gitattributes list, and does not describe BRIEF.md's protection as a header statement; every link in it resolves to an existing file.
The section says a design change is an ordinary request through the change pipeline, recorded in docs/decisions.md, and claims no mechanism absent from the repository.
CHANGELOG.md has an [Unreleased] / Changed entry for the README addition that matches the README's list of exceptions.
git diff shows only README.md, CHANGELOG.md (and optionally docs/architecture.md) changed; no prompt, athena.toml, defaults.toml, .gitattributes or workflow file. Not a new refusal rule, so no refusal-rate change.
Run cargo test and cargo clippy: both pass, including any tests that check docs links or changelog format.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
