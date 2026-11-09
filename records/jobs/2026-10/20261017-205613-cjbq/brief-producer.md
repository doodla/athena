# Job 20261017-205613-cjbq: Validate each study against its own pin; tighten study admission

## Request (verbatim)

> Validate every study against its own pinned source, so a second study can be checked at all. Today the kept validation test in crates/athena-cap-study/src/rules.rs hard-codes firstmate (STUDY_SHA, study_dir(), mounted_tree()), and study checks run cargo test with the job's pinned tree mounted, so a study of any other system would fail its own test check and nothing would validate it against its tree. The capability's conventions (enforcement tags, no placeholders, a "none" guard is never checked by code) exist only as firstmate tests, not in crates/athena-cap-study/prompts/producer.md. The boundary test in xtask/tests/policy.rs would also fail on a studied system's own words (Hermes Agent's work-handling source uses three of the words that test bans; read the list in xtask/tests/policy.rs and do not write those words anywhere outside a claim's quote).
> Plan: (1) The kept test checks the conventions for every studies/*/ directory, and runs full validation against the tree only for the study whose pin matches ATHENA_PIN_URL and ATHENA_PIN_SHA (set by the pinning change in the previous study job); with no tree mounted it checks conventions only. (2) State the conventions in the study producer prompt. (3) Admission refuses a study job's change that touches more than one studies/<id>/ directory, or anything outside it and CHANGELOG.md. (4) In the boundary test, exempt only text that quotes a studied system's source (the quote fields of a study's claims), never a study's own prose, and keep scanning machine-path fragments everywhere. Plant banned words in tests the way the_boundary_scanner_flags_a_planted_term does (split strings joined at run time), so no test file contains them.
> Acceptance:
> - cargo test: with no tree mounted, every study's conventions are checked and the repository passes; with firstmate mounted, the computed validation equals studies/firstmate-work-handling/validation.json.
> - cargo test on temporary copies: an untagged claim, a "none" guard checked by code, a source SHA that differs from the pin, and a pin.json for the mounted URL with a different SHA each fail and name the offending id.
> - cargo test: admission refuses a study change spanning two study directories, and one touching a file outside its directory and CHANGELOG.md.
> - cargo test: the boundary scan still flags a planted banned word in a study's own prose and in any file outside studies/, and a machine-path fragment inside studies/, but not a banned word inside a claim's quote of studied source.
> - clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry; no file under studies/firstmate-work-handling/ changes.
> Out of scope: hiding one study from another's jobs, athena.toml, and the site.

## Reading (accepted by the user)

Generalize the kept study test in athena-cap-study so conventions are checked for every studies/*/ and full validation runs only for the study matching ATHENA_PIN_URL/SHA. Document conventions in the study producer prompt, make study admission refuse multi-directory or out-of-directory changes, and exempt only claim quotes in the xtask boundary test. Goal: a second study can be validated; audience is Athena's study jobs and verifiers.

Assumptions:
- Code survey: read crates/athena-cap-study/src/rules.rs (validate(), per-rule fns, kept test consts STUDY_SHA at ~1400 and study_dir() at ~1434, FixtureRepo, mounted tree reading) and crates/athena-cap-study/src/lib.rs (Capability::admit at line 112, the entry point for study admission), crates/athena-core/src/capability.rs (admit trait at line 121, ChangeList/PolicyViolation). Study records are data under studies/<id>/ (claims, pin.json, validation.json), not new files under crates/, so no new AnyRecord or check_record_file entry is needed; the existing validation.json comparison is the round-trip check. Validation is measured directly by running validate() on the parsed study and comparing to validation.json.
- Existing code: extend crates/athena-cap-study/src/rules.rs tests, crates/athena-cap-study/src/lib.rs admit(), crates/athena-cap-study/prompts/producer.md and xtask/tests/policy.rs; none are created by this job.
- Term Hermes: in this request, Hermes Agent is an example third-party studied system whose work-handling source contains words the xtask/tests/policy.rs boundary test bans; it is only a motivating example, not something this job adds or reads, and the ban list is not to be written anywhere outside a claim quote.
- Environment: no CI workflow, push or PR is touched; tests run locally with cargo, reading ATHENA_PIN_URL and ATHENA_PIN_SHA from the process environment, with no network.
- Consumers: none found for config; athena.toml and defaults.toml are not changed and are out of scope.
- Env-var matching uses studies/<id>/pin.json url and sha; with the variables unset or not matching any pin, only conventions are checked. A pin.json whose URL equals ATHENA_PIN_URL but with a different SHA fails naming the study id.
- Admission treats allowed paths as studies/<id>/** for exactly one id plus CHANGELOG.md; the expected newly refused share of past study readings is small.
- The boundary test plants banned terms as split strings joined at run time so no test file contains a banned word.

## Acceptance criteria

1. [behavior] Run cargo test: with no ATHENA_PIN_* set, the kept test in crates/athena-cap-study/src/rules.rs checks conventions for every studies/*/ directory and passes; with firstmate's tree mounted and the matching ATHENA_PIN_URL/ATHENA_PIN_SHA, the computed validation equals studies/firstmate-work-handling/validation.json.
2. [behavior] Run cargo test: tests on temporary copies of a study show an untagged claim, a "none" guard checked by code, a source SHA differing from the pin, and a pin.json for the mounted URL with a different SHA each fail, and the failure message names the offending id.
3. [behavior] Run cargo test: admit() in crates/athena-cap-study/src/lib.rs returns a violation for a change spanning two studies/<id>/ directories and for one touching a file outside its study directory and CHANGELOG.md, and none for a change inside one directory plus CHANGELOG.md. This refuses more study changes and the higher refusal rate is intended: expected 5% of newly refused readings.
4. [behavior] Run cargo test (xtask/tests/policy.rs): the boundary scan flags a planted banned word in a study's own prose and in any file outside studies/, and flags a machine-path fragment inside studies/, but not a banned word inside a claim's quote field; planted words are split strings joined at run time (Grep of xtask/tests/policy.rs finds no banned word).
5. [content] crates/athena-cap-study/prompts/producer.md states the conventions: enforcement tags, no placeholders, and that a "none" guard is never checked by code; the study producer prompt stays within its token budget (182 tokens of headroom).
6. [behavior] Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.
7. [content] CHANGELOG.md has a new entry describing the change; git diff shows no file under studies/firstmate-work-handling/ changed; athena.toml is unchanged.

## Expected deliverables

- crates/athena-cap-study/src/rules.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cap-study/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Hiding one study from another study's jobs
- athena.toml
- The site
- Any change under studies/firstmate-work-handling/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the tests' behavior (failures naming ids, refusals) rather than existence of code.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Cargo test, clippy -D warnings and fmt --check criteria included.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Conventions and single-directory rule are enforced by tests and admission code, not prose only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261017-205613-cjbq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205613-cjbq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test: with no ATHENA_PIN_* set, the kept test in crates/athena-cap-study/src/rules.rs checks conventions for every studies/*/ directory and passes; with firstmate's tree mounted and the matching ATHENA_PIN_URL/ATHENA_PIN_SHA, the computed validation equals studies/firstmate-work-handling/validation.json.
Run cargo test: tests on temporary copies of a study show an untagged claim, a "none" guard checked by code, a source SHA differing from the pin, and a pin.json for the mounted URL with a different SHA each fail, and the failure message names the offending id.
Run cargo test: admit() in crates/athena-cap-study/src/lib.rs returns a violation for a change spanning two studies/<id>/ directories and for one touching a file outside its study directory and CHANGELOG.md, and none for a change inside one directory plus CHANGELOG.md. This refuses more study changes and the higher refusal rate is intended: expected 5% of newly refused readings.
Run cargo test (xtask/tests/policy.rs): the boundary scan flags a planted banned word in a study's own prose and in any file outside studies/, and flags a machine-path fragment inside studies/, but not a banned word inside a claim's quote field; planted words are split strings joined at run time (Grep of xtask/tests/policy.rs finds no banned word).
crates/athena-cap-study/prompts/producer.md states the conventions: enforcement tags, no placeholders, and that a "none" guard is never checked by code; the study producer prompt stays within its token budget (182 tokens of headroom).
Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.
CHANGELOG.md has a new entry describing the change; git diff shows no file under studies/firstmate-work-handling/ changed; athena.toml is unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
