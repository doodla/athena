# Job 20261112-023817-9at7: Enforce Opus/Sonnet/Haiku roles in config and Jev gateway for SET-02

## Request (verbatim)

> try again on 20261030-001211-6mzd

## Reading (accepted by the user)

Retry of 20261030-001211-6mzd (rejected: the hard-coded ROSTER had no reader). Instead of a copied list, Config::validate in athena-data now refuses an athena.toml whose [models] roles lack an Opus, a Sonnet or a Haiku model. A test checks the real athena.toml plus the Jev gateway constants, and OLY-B-SET-02 records that as mechanism and measure. Audience: Athena's owner and the spec gap check.

Assumptions:
- Prior job 20261030-001211-6mzd: finding 1 (major, ROSTER in jev.rs read by nothing, test checks only constants) is fixed by dropping ROSTER and jev.rs edits entirely and enforcing the roles through Config::validate, which every Config::parse calls. Finding 2 (minor, xtask/shard-timings.txt) is kept as a deliverable because the new test file needs a line there (the estimate is 5.00 s). Finding 3 (nit, lib.rs unchanged) no longer applies: lib.rs is dropped from scope.
- Prior candidate files: CHANGELOG.md kept (rewritten for the new mechanism); crates/athena-core/src/jev.rs dropped (reverted); crates/athena-core/tests/model_roster.rs kept but rewritten to load the repository athena.toml; docs/spec.toml kept (re-edited); xtask/shard-timings.txt kept.
- Code survey: read crates/athena-data/src/config.rs (Models struct at ~314, ConfigError at ~567, Config::parse and validate at ~616-680, which already check verifier not in producer_allowed and default producer allowed), crates/athena-core/src/reading.rs (reads config.models.producer_allowed at ~617, ~797 to choose and check the producer model), and crates/athena-core/src/jev.rs (ENDPOINT, MODEL constants at lines 31-33). Consumers of the roles: reading.rs and the briefs. No record or data file is added under crates/, so check_record_file and AnyRecord do not apply. The producer must check existing config tests and fixtures that build Models with other model names (e.g. crates/athena-context/src/briefs.rs ~3550, config.rs tests ~1329) still pass, and adjust fixtures if the new rule refuses them.
- Existing code: crates/athena-data/src/config.rs (Config::validate, ConfigError), crates/athena-core/src/jev.rs (ENDPOINT, MODEL, reused not duplicated), docs/spec.toml (OLY-B-SET-02 entry), athena.toml (read only, current [models] already has claude-sonnet-5-5, claude-haiku-5-5 and verifier claude-opus-5-5).
- Family match is by substring on the model name (opus, sonnet, haiku) across reader, producer_default, producer_allowed and verifier. athena.toml is not changed because it already satisfies the rule; Consumers: none found for any config file change.
- Jev stays shadow-only; the requirement's Jev clause is shown by a test that the Jev endpoint is on ai-gateway.vercel.sh and the model is typesafe-ai/jev, not by making Jev decide real work.
- The measure ends with `status: not shown` plus the test command, since no `athena status` label covers this.

## Acceptance criteria

1. [content] docs/spec.toml OLY-B-SET-02 has a mechanism other than "none" naming Config::validate in crates/athena-data/src/config.rs and the test; owner Athena; and a measure ending with `status: not shown` and the test command.
2. [content] Consumer named: crates/athena-data/src/config.rs Config::validate (called by Config::parse for every load) is the non-test reader. It returns a new ConfigError variant when [models] roles lack an Opus, a Sonnet or a Haiku model. No hard-coded ROSTER list is added, and jev.rs is unchanged.
3. [behavior] Run `cargo test -p athena-data`: new unit tests build a Config with the Haiku, Sonnet or Opus model removed from the roles and assert Config::parse returns the new error for each. The same text with all three present parses. These tests fail if the validation is removed.
4. [behavior] Run `cargo test -p athena-core --test model_roster`: it parses the repository athena.toml through the real loader and asserts the roles cover Opus, Sonnet and Haiku. It also asserts jev::ENDPOINT contains ai-gateway.vercel.sh and jev::MODEL is the Jev model, using the existing constants.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test spec` and xtask ci_shards (xtask/shard-timings.txt has a model_roster line).
6. [content] CHANGELOG.md has an entry for the model-family check; no secrets are written; athena.toml, defaults.toml and workflows are unchanged.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/tests/model_roster.rs (create)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Making Jev decide real work (it stays shadow-only)
- Changing athena.toml, defaults.toml or workflows
- Other requirements such as OLY-B-SET-04
- Adding a ROSTER constant or any change to crates/athena-core/src/jev.rs and lib.rs (prior candidate dropped)

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the real config loader and show the refusal when a family is removed.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that configured roles cover the three families, not that a list exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-001211-6mzd raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/jev.rs ROSTER; crates/athena-core/tests/model_roster.rs): Soundness: the roster is a hard-coded list that nothing reads, and its test only checks that constants exist. It does not show that work uses Opus, Sonnet and Haiku. A grep for ROSTER/roster_entry/RosterEntry finds only jev.rs, the new test, spec.toml and CHANGELOG.md. No code path uses it. The models work actually runs with come from athena.toml [models] (reader/producer_default = claude-sonnet-5-5, producer_allowed includes claude-haiku-5-5, verifier = claude-opus-5-5), loaded by crates/athena-data/src/config.rs. ROSTER copies those strings by hand. If athena.toml dropped Haiku or moved to a new model version, this test would still pass and ROSTER would silently go stale. That breaks the behavior-evidence default ("check the requested behavior happening, not that types, files or wiring exist"). It also departs from the accepted reading's "Opus, Sonnet, Haiku via configured roles". Simpler, more correct alternative: check the configured roles. For example, a test or config validation that loads the repository's athena.toml through athena-data's config loader and asserts its [models] roles cover an Opus, a Sonnet and a Haiku model, alongside the existing Jev ENDPOINT/MODEL check. Putting the Claude model list inside the Jev client module is also an odd home.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 19 cases in 19 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a check before the producer runs that compares the accepted reading with the request's own wording. Require any new rule or check, and any measure text claiming what status shows, to be shown on real data: the rule must fail on a known-bad input and pass on current main, and it must not duplicat...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261112-023817-9at7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-023817-9at7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml OLY-B-SET-02 has a mechanism other than "none" naming Config::validate in crates/athena-data/src/config.rs and the test; owner Athena; and a measure ending with `status: not shown` and the test command.
Consumer named: crates/athena-data/src/config.rs Config::validate (called by Config::parse for every load) is the non-test reader. It returns a new ConfigError variant when [models] roles lack an Opus, a Sonnet or a Haiku model. No hard-coded ROSTER list is added, and jev.rs is unchanged.
Run `cargo test -p athena-data`: new unit tests build a Config with the Haiku, Sonnet or Opus model removed from the roles and assert Config::parse returns the new error for each. The same text with all three present parses. These tests fail if the validation is removed.
Run `cargo test -p athena-core --test model_roster`: it parses the repository athena.toml through the real loader and asserts the roles cover Opus, Sonnet and Haiku. It also asserts jev::ENDPOINT contains ai-gateway.vercel.sh and jev::MODEL is the Jev model, using the existing constants.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test spec` and xtask ci_shards (xtask/shard-timings.txt has a model_roster line).
CHANGELOG.md has an entry for the model-family check; no secrets are written; athena.toml, defaults.toml and workflows are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
