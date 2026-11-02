# Job 20261027-054251-33v7: Check ontology terms against the governing sources they summarize

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The job's checks covered the file's structure (the structure test) but not whether each term's conte
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: seeded-content-contradicts-governing-source-caught-only-at-final-verdict (20261017-182305-f62n)
> Cause: The job's checks covered the file's structure (the structure test) but not whether each term's content is right against the sources it summarizes. A term with several senses, here Hermes as both a tool and a named role, was defined from one...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-182254-my85 (job 20261017-173508-0wtc); prevention it proposed: Add a check, run before handover, that for each ontology term searches the governing docs (the Olympus brief and the other named sources) for the term's name. It fails unless the entry lists every sense found there, or records why a hit is ...
> Introspection 20261025-230247-cm9r (job 20261025-214509-y179); prevention it proposed: Add a measure-first stage to speed-target jobs. Before the producer edits anything, it records a baseline wall time for each binary and the total, using one fixed method. The plan must then give each binary a time budget that adds up to the...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261017-173508-0wtc and 20261025-214509-y179): checks covered structure or activity, not whether seeded or claimed content is right against the sources it summarizes. Fix it in the ontology test: for each term, search the governing docs for its name and fail unless the entry lists every sense found or records why a hit is not one. This is a retry of 0wtc's lesson on its landed files; y179's speed work is out of scope.

Assumptions:
- Code survey: read crates/athena-cli/tests/ontology.rs (problems() checks parse, empty fields, source existence, relation targets, not_part_of sources; it never reads the sources' content) and the term list in docs/ontology.toml (planning, job, Athena, Hermes, product versus system, landed, Sri). No records/ or check_record_file/AnyRecord is touched: docs/ontology.toml is not under crates/, and no new file is added under crates/.
- Job 20261017-173508-0wtc: its Hermes finding (BRIEF.md names Hermes as Orchestration god) is already fixed in the tree: the Hermes term has both senses and the test asserts them. Its files (docs/ontology.toml, crates/athena-cli/tests/ontology.rs, docs/decisions.md, CHANGELOG.md) are kept and extended, not dropped. What remains is the class: nothing checks a term against the source text, so the next multi-sense term would again fail only at the final verdict.
- Job 20261025-214509-y179: its findings were a missed speed goal, unmade test cuts and causes inferred rather than traced. It shares the class of a check that covers activity rather than the outcome. Its changed files (secrets.rs, checks.rs, its decisions.md and CHANGELOG entries) are kept untouched; the speed work and the measure-first proposal are out of scope here and need their own job.
- The new check is a test in crates/athena-cli/tests/ontology.rs. Governing sources are BRIEF.md plus each term's own listed source files. A term name is matched as a whole word or phrase. The term entry gains an optional field (for example senses or ignored_hits) with a reason for each hit it does not treat as a sense.
- Audience: Athena agents and Sri, who read the ontology; the verifier runs cargo test.

## Acceptance criteria

1. [content] The change description names the root cause: term content was checked for structure, not against the governing sources it summarizes, so a multi-sense term was defined from one source. It cites jobs 20261017-173508-0wtc and 20261025-214509-y179.
2. [behavior] Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and the term's sources for each term name in docs/ontology.toml and passes on the real file. It fails if a sense found there is neither listed on the entry nor excused with a recorded reason.
3. [behavior] Run `cargo test -p athena-cli --test ontology`: a unit test on in-memory input and a scratch source (the Hermes case: one entry defining only the tool while the source also names a role) is flagged. The same input with both senses listed passes. The test fails when the new check is removed.
4. [content] The check is generic over all terms, not special-cased to Hermes. The test's header or docs/ontology.toml's [about] states the new rule and the optional field for senses or excused hits.
5. [content] docs/decisions.md has a new decision stating the cause, the alternatives considered (a rule only in prose; a measure-first stage for speed jobs) and why the check was placed in the ontology test. CHANGELOG.md has an entry.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] No change to athena.toml, defaults.toml, workflows or the speed-work files (crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs). Verify with `git diff --stat`.

## Expected deliverables

- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speed-target work from job 20261025-214509-y179 (measure-first stage, test-side cuts, the secrets.rs memo, run_check polling); those files are kept as landed and not changed
- Adding new ontology terms not caused by a misread
- Changing any study's concept model or files under studies/
- Changes to athena.toml, defaults.toml or workflows
- Wiring the ontology into agent contexts or prompt budgets

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check is a Rust test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests, with cargo test, clippy and fmt passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a check on content against sources, not a patch to one term.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test and a failing in-memory case.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a test, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test file, the data file and the notes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and test header are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 16 cases in 14 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Give Athena one disk lifecycle. Before the low-disk check and before any hold, the writer's regenerable build is removed once the writer has finished and its output is committed or recorded. The guard then measures free space after that removal. Where checks come from GitHub, the guard does not coun...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

24 more qualifying classes are left out.

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

The change description names the root cause: term content was checked for structure, not against the governing sources it summarizes, so a multi-sense term was defined from one source. It cites jobs 20261017-173508-0wtc and 20261025-214509-y179.
Run `cargo test -p athena-cli --test ontology`: a new test scans BRIEF.md and the term's sources for each term name in docs/ontology.toml and passes on the real file. It fails if a sense found there is neither listed on the entry nor excused with a recorded reason.
Run `cargo test -p athena-cli --test ontology`: a unit test on in-memory input and a scratch source (the Hermes case: one entry defining only the tool while the source also names a role) is flagged. The same input with both senses listed passes. The test fails when the new check is removed.
The check is generic over all terms, not special-cased to Hermes. The test's header or docs/ontology.toml's [about] states the new rule and the optional field for senses or excused hits.
docs/decisions.md has a new decision stating the cause, the alternatives considered (a rule only in prose; a measure-first stage for speed jobs) and why the check was placed in the ontology test. CHANGELOG.md has an entry.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
No change to athena.toml, defaults.toml, workflows or the speed-work files (crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs). Verify with `git diff --stat`.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
