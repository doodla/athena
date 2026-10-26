# Job 20261026-194248-xw3g: Turn job planning off per the D110 keep rule and record the decision

## Request (verbatim)

> Priority: urgent
> Apply the job-planning keep rule (docs/design/job-planning.md section 9, D110) now that it has decided. athena status reports 58 of 105 planned jobs landed (55%), at or below the 64% baseline at the 100-job point, with cost per landed job $3.43 above the $2.50 guard: the rule says turn planning off. Set `enabled = false` under [job_plan] in athena.toml. Add a decision entry in docs/decisions.md recording the result with the numbers from athena status, that the rule (not a person) decided it, that the decision was taken by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and that the earlier keep-planning-on override (intervention 20261025-033755-b01h) was taken at 51 jobs, below the decision point. Note what is kept: the planning code, the resume-from-kept-plan path and the status measures stay, so planning can be turned back on for a new measured trial. Update section 9 of job-planning.md with the outcome.
> Acceptance:
> - athena.toml has enabled = false under [job_plan] and no other change.
> - docs/decisions.md has the new entry with the measured numbers and the decider; job-planning.md section 9 states the outcome.
> - A test or the existing config tests show the config parses with planning off and a change job runs with no planner or critic invocation.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, docs/decisions.md, docs/design/job-planning.md, CHANGELOG.md

## Reading (accepted by the user)

Turn job planning off per the decided keep rule: set enabled = false in [job_plan], record the decision with its numbers and decider in decisions.md, update section 9 of job-planning.md, and add a CHANGELOG entry. The existing repo-config test asserts enabled is true, so it is updated to expect planning off.

Assumptions:
- Goal: apply the keep rule's outcome (planning off); audience is Sri and later maintainers reading the decision log and design doc.
- The existing test the_repository_config_turns_job_planning_on_with_the_designs_caps asserts enabled is true and will fail, so it must be updated and renamed; this makes crates/athena-data/src/config.rs a needed deliverable beyond the listed files.
- A no-planner-invocation test for the disabled path probably already exists in crates/athena-cli/tests/job_plan.rs; if not, one is added there.
- Numbers are taken from the request as given, since athena status cannot be run by the verifier.

## Acceptance criteria

1. [content] athena.toml has `enabled = false` under [job_plan]; git diff shows that one line as the only change to the file (caps and timeouts untouched).
2. [content] docs/decisions.md has a new entry with the athena status numbers (58 of 105 landed, 55%, 64% baseline, $3.43 per landed job vs $2.50 guard), states the rule decided and not a person, the brain acted under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26), the override 20261025-033755-b01h was taken at 51 jobs below the decision point, and what is kept (planning code, resume-from-kept-plan path, status measures) for a new measured trial.
3. [content] docs/design/job-planning.md section 9 states the outcome: planning turned off by the keep rule, with the numbers, and what remains.
4. [content] CHANGELOG.md has an entry for turning job planning off.
5. [behavior] Run `cargo test`: the repo config test in athena-data/src/config.rs is updated to assert the repository athena.toml parses with job_plan present and enabled == false (caps kept), and passes; an existing or added test shows a change job with planning off makes no planner or critic invocation (e.g. athena-cli tests/job_plan.rs disabled-path test), and passes.
6. [behavior] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean and the full `cargo test` passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Removing or changing planning code, the resume-from-kept-plan path, or the status measures
- Changing planner/critic caps or timeouts in [job_plan]
- Starting a new planning trial
- Re-measuring or re-deriving the athena status numbers

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Config test updated; fmt, clippy and test suite must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites the measured athena status numbers and the intervention id.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check parsed config value and no planner/critic invocation when run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The keep rule is applied via the config flag, enforced by code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and doc text written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change with the minimal test edit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

athena.toml has `enabled = false` under [job_plan]; git diff shows that one line as the only change to the file (caps and timeouts untouched).
docs/decisions.md has a new entry with the athena status numbers (58 of 105 landed, 55%, 64% baseline, $3.43 per landed job vs $2.50 guard), states the rule decided and not a person, the brain acted under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26), the override 20261025-033755-b01h was taken at 51 jobs below the decision point, and what is kept (planning code, resume-from-kept-plan path, status measures) for a new measured trial.
docs/design/job-planning.md section 9 states the outcome: planning turned off by the keep rule, with the numbers, and what remains.
CHANGELOG.md has an entry for turning job planning off.
Run `cargo test`: the repo config test in athena-data/src/config.rs is updated to assert the repository athena.toml parses with job_plan present and enabled == false (caps kept), and passes; an existing or added test shows a change job with planning off makes no planner or critic invocation (e.g. athena-cli tests/job_plan.rs disabled-path test), and passes.
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean and the full `cargo test` passes.

## Report

End with the producer report as your structured output.
