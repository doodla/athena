# Changelog

All notable changes to Athena are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The full `cargo test --locked` run is faster, with no test removed, ignored or weakened. Text scans by gitleaks
  (`Gitleaks::scan_text`, used to redact every recorded text) are remembered for the life of the process, keyed
  by a digest of the binary's content, the generated configuration and the text, so a replaced binary or a
  changed configuration is scanned afresh and a failed scan is never remembered. `run_check` looks at a running
  check after 5 ms, doubling up to 200 ms, instead of a fixed 200 ms, so a quick check no longer waits out a
  pause; deadline and kill-switch handling are unchanged. Measured times per test binary are in decision D126.

### Added

- A producer that reaches its deadline no longer loses its work. The invocation now ends `timed_out` (new
  `InvocationOutcome::TimedOut`; the kill switch and a dead parent stay `killed`) and the job closes with the
  new outcome state `timed_out` (`OutcomeState::TimedOut`), not `failed`. The outcome reason and the invocation
  detail keep the deadline text, such as "deadline of 1800s reached". Athena moves the job's work tree to
  `kept/<job>/work` in the state directory before it removes the job directory, and commits a progress note,
  `records/jobs/<month>/<job>/progress-note.md`, that lists the changed files against the base. `athena retry`
  and "try again on <job>" accept a timed out job; the new producer starts from the kept files (a copy laid over
  a fresh base snapshot) and its brief gains a "Resume" section quoting the note. When the visible base changed
  since the timed out job, or the job was a pinned study, the retry starts from the base as before. The kept
  work is removed when the retry closes. `athena status` and the job lists show `timed_out` apart from
  `failed`, and `athena drain` retries a timed out job as it does a rejected or failed one. Records written
  before this change (`killed` invocations, `failed` outcomes) read unchanged. Only the producer stage is
  covered; verifier deadlines and the deadline length are unchanged.
- `docs/system-facts.md`: a short, sourced list of facts about how Athena runs that the code cannot show (host
  merge rules, what each role's copy holds, prompt budgets and current sizes, the request format `athena plan`
  parses, what `land` lands, where records live, remotes, CI caches, the early pull request). The planner and
  critic briefs carry it whole in a "System facts" section (`SYSTEM_FACTS` in `athena-context`). The budgets
  test `system_facts_budgets_match_computed` fails when a budget or size recorded in the file differs from the
  computed one, and `system_facts_drift_is_rejected` shows a doctored value is refused. Prompt budgets are
  unchanged.
- `athena land --pr` waits for the early commit's CI run to finish before it replaces the early head with the
  landing commit, so the landing run finds the cached pass. The race it closes: on PR #180 the early run passed
  at 10:46:49 but the landing run had started at 10:45:15 and ran the full 16 minutes. The wait looks at the
  commit's workflow runs every 20 seconds, for at most 10 minutes (`PrHost::early_run_wait`), then pushes
  anyway. A failed run, no runs, or an unreadable reply stop the wait at once and the push and merge go on as
  before. The wait covers all runs of the commit (the host reply carries no workflow name). The report gains
  one warning line saying how many seconds it waited and why it stopped. The landing time after this change is
  still to be measured: it is recorded here, with its PR number and run timestamps, once a real landing has
  been observed.
- An `urgent` priority above `high` in plan order, a new `athena priority <job> <level>` command and a guard on
  `docs/decisions.md` (D122). The command appends an `athena.priority/1` record (`priority-<n>.json`) for a queued
  request; the request is not rewritten, and plan and status both prefer the latest priority record over a
  `Priority:` line in its words (`Priority: urgent` now parses). A dependency still comes before its dependent
  whatever the priority. The reader may mark a request urgent, with a required `urgent_reason`, when doing it makes
  Athena faster or cuts wasted time; the rule is the field's description in the reading schema, so `reader.md`
  does not grow. Readings gain an optional `urgent_reason`. A test in `decisions.rs` parses the log, checks every
  entry has a Provenance bullet and ids ascend, and pins the earlier bytes so entries are only appended.
- Verification start in `athena go` opens or updates the job's pull request, so CI caches by the candidate's
  fingerprint while the verifier works. The candidate is pushed to the job's own `athena/<job>` branch and the
  pull request is opened (or, when one is already linked, its branch moved) through the host seam. It never
  merges. A missing origin, or one that is not an https URL on github.com, is skipped quietly with no output;
  a failure on a GitHub remote is non-fatal and prints one `warning:` line, and the job's outcome is the same
  as without the step. `athena land --pr` reuses the early pull request (no second one is opened): it replaces
  the early head, and only that head, with the landing commit through a pinned replace
  (`Remote::replace`, behind `check_replace`), then judges the head and merges guarded with the landing commit
  as before. The early facts are kept beside the job's directory, which `go` removes when it finishes.
  `Athena::go` and the CLI call sites are unchanged.
- The 'fmt, clippy, tests' CI job reuses its own earlier pass for unchanged governed content (D121).
  `cargo run -p xtask -- fingerprint` prints athena-git's governed-content fingerprint of `HEAD` (it ignores
  `records/` and owner paths), and `xtask ci-cache check|mark|report` hold the decisions. `ci.yml` looks up
  `ci-pass-<fingerprint>` with `actions/cache` before setup; a marker naming the fingerprint skips setup and the
  checks and reports the fingerprint and matched key in the log and step summary; otherwise the full job runs and
  the marker is saved only after the checks pass, on pull-request branches too (GitHub keeps those entries on
  their branch; the shared toolchain, registry and target caches are still saved only from the default branch).
  `xtask` now depends on `athena-git` (`docs/architecture.md` updated). Landing median before: 15.4 minutes over 33
  jobs, 2026-10-23 to 25; the figure after is not yet measured.
- A shared test fixture for the records-history cutover (D111, D120): `athena_core::test_support`, compiled
  only for tests and the `test-support` feature of `athena-core`. `CutoverFixture::build` makes a scratch
  repository with a bare remote and this history, all through Athena's own writers: a bootstrap commit; one job
  read, dispatched, verified and landed the old way by `ask` and `go` (with its trailers and no landing record);
  then the cutover job, whose change sets `[history] cutover_job`, landed through `gh::land_pr` with the real
  records sink, so the landing intent, landing record, import with manifest and legacy-exempt record are on
  `athena/records`. Only the host is faked. `athena-cli`'s `tests/common/mod.rs` has `cutover_fixture()` and
  `athena_check()`; `tests/cutover_fixture.rs` pins the shape and runs `athena check --history` and
  `athena check --attribution` on it. The trailer job and job 7 part 2 reuse it from `athena-cli` tests with
  those helpers, and from `athena-core` tests by passing their own capabilities and scripts in a `CutoverSpec`.
  The module text explains how. No production behavior changes.
- The records-history cutover (job 7, part 1, of `docs/design/records-history.md`, sections 4 and 10, D111,
  D119). `athena.toml` `[history]` gains an optional `cutover_job`; `C0` is the host commit of that job's landing
  record on `athena/records`. When `athena land --pr` lands that job, after the landing record it imports
  `records/` at `C0` (one commit with `import-manifest.json`, checked by a self-check) and writes the
  legacy-exempt record from `[history] exempt`. The reconcile step that `athena land` and `athena status` run
  first repeats this when the landing is on the branch but the import is not, and writes nothing the second time.
  `athena check --history` judges commits up to and including `C0` by the old rules and later commits only by the
  attribution check, once the branch shows the import; before that it behaves as it did and prints a note that the
  cutover is pending. The required CI job keeps its name, "the full history", and fetches `athena/records` before
  the check (not on pull requests). Commit-message trailers are still written.
- Request priority. A line `Priority: high`, `Priority: normal` or `Priority: low` (any case) in a request's
  words sets its priority; no such line means normal. `athena plan` and `athena drain` order by dependencies
  first, then priority, then rung, then age, so a newer high-priority request runs before older normal ones
  (D118). A high request's prerequisites are pulled forward with it. `athena plan` prints a `priority` line for
  every item, and `athena status` marks a request that is not normal with `priority high` or `priority low`.
  Requests with no priority line keep today's order.
- Incremental attribution checking with checkpoint records (job 6 of `docs/design/records-history.md`, section
  9, D111). A passing `athena check --attribution` (on the main branch, with `CI` unset and no
  `--no-checkpoint`) and `athena land` after its landing record append a `checkpoint` record to the local
  `athena/records` with the records writer. It names the last passing main commit, the records tip it was
  judged against, and the check version (`CHECK_VERSION`). It is never pushed by the check. A later check
  judges only main's commits and the records added since the newest checkpoint (newest in records-branch
  first-parent order); intents and landings are still looked up across the whole records tip, because a
  checkpoint can fall between a landing's intent and its landing. It runs a full replay, and says why, when
  there is no checkpoint, the check version differs, or the checkpoint names a commit that is not on main.
  `--full-replay` ignores checkpoints, judges everything and fails if the newest checkpoint names a commit
  the replay fails or one that is not on main. A failing check, or one with a pending commit, writes no
  checkpoint. CI never writes one: the workflow token stays `contents: read` and the check writes nothing
  when `CI` is set; its attribution job runs the incremental check on pushes and `--full-replay` on the
  weekly schedule. Every run prints the mode, seconds taken, and the commits and records judged (text and
  `--json`). New in `athena-git`: `audit_history_since`, `newest_added_under`, `files_added_between`.
  Timing, **measured on the synthetic fixture** of 200 landings (one commit, one intent record and one
  landing record each; test `attribution::tests::timing_on_synthetic_200_landings`, debug build, one run on
  this machine): full replay 1.731 s (200 commits, 400 records); incremental run after one more landing
  0.145 s (1 commit, 2 records). Seconds vary by machine and are printed, never asserted. The timing on this
  repository itself is taken after landing, outside this change.
- The legacy rule of the attribution check (job 5, part 2 of `docs/design/records-history.md`, sections 4
  and 10, D111): `athena_gate::legacy::check_attribution_with_legacy`. For main's commits up to and
  including a cutover commit `C0`, the existing history check judges them, with its exempt list read from
  `legacy-exempt.json` on `athena/records` instead of `athena.toml`'s `[history] exempt`
  (`check_history_with_exempt`; `check_history` is unchanged). Commits after `C0` keep the records-only rule.
  `C0` is a caller-supplied parameter; nothing chooses it, and no command or CI job calls the rule yet.
  `legacy_exempt_record` builds the record from `athena.toml`'s seven exempt entries and the reasons in its
  comments, and `legacy_exempt_change` the change that would write it; nothing is written to the branch. A
  records history in which `legacy-exempt.json` changed is a finding, and the records writer refuses an
  added entry.
- `athena check --attribution`, the records-only attribution check on main (job 5, part 1 of
  `docs/design/records-history.md`, sections 5.2, 6 and 10, D111). It reads only `athena/records` (the local
  branch, else `origin/athena/records`) and main's first-parent commits, never a commit message. Starting at
  the commit the earliest landing-intent expects as its base (the anchor, not itself judged), each later
  commit must be named as `host_commit` by a landing whose intent matches it (head tree equals the commit's
  tree, expected base equals its parent). A commit with a matching intent and no landing passes as pending
  for 15 minutes from the intent and then fails; any other commit fails as unattributed, and its sha is
  printed. A missing records branch, no landing-intent, an unparseable record, diverged local and remote
  records refs, and a records commit that drops or changes a file of its predecessor (not a fast-forward)
  are findings too. Exit 2 on a finding. It conflicts with `--history` and `--pr-head`. A new `attribution`
  job in `.github/workflows/ci.yml` runs it on pushes to main and on schedule after
  `git fetch origin athena/records:athena/records`; it does not block merging. Code:
  `athena_gate::attribution`, `athena_git::records_branch::{resolve_tip, audit_history}`.
- `athena check --pr-head <base> <head>`, the pre-merge check on a pull request (job 4, part 1 of
  `docs/design/records-history.md`). It passes only when every commit between the base and the head is a
  verified job landing whose prediction (the accepted reading) was recorded before the work, a declared
  intervention made before the last landing, or a records-only commit, and the head's governed content equals
  what the last landing verified. Otherwise it exits 2 and prints each offending commit's sha and the reason;
  all offending commits are listed, not only the first. A declared intervention after the last landing fails,
  because a squash of it is not a valid landing until the cutover. A new `pr-head` job in
  `.github/workflows/ci.yml` runs it on pull requests; it does not block merging yet and becomes required at
  the cutover (job 7). Classification is `land::classify_commit`, the per-commit step of `land::pr_commits`
  (whose results and messages are unchanged); `athena_gate::invariant::prediction_problem` is now public so the
  check applies the same prediction-order rule as the gate. Tests: `crates/athena-core/src/pr_check.rs`.
- A retry starts the producer from the rejected candidate, not the base, when every check of the rejected job
  passed and every unmet criterion is a content criterion; the producer is sent only the verifier's findings.
  Any other rejection (a failed check, an unmet or unverified behavior criterion, no candidate tree, a failed
  or interrupted job, a reading mismatch, admission violations) still starts from the base, and so does a
  retry after other work landed (a path outside `records/` differs between the rejected job's base and the
  retry's prediction commit), because the old candidate would undo that work. Root cause: the retry always
  rebuilt the producer's snapshot from the base, redoing code that had passed every check.
  The new candidate is still its change list against the original base, then admission, fingerprint, the full
  checks and the verifier, so landing is unchanged. A seeded producer that changes nothing fails as "the
  producer made no change". Earlier CHANGELOG or decision text that says a retry always starts from the base
  is superseded by this entry and by D117 in `docs/decisions.md`. Callers and writers touched, with the test
  that covers each:
  - `qualifies_for_tree_retry` and `tree_retry_seed` (new, `crates/athena-core/src/reading.rs`):
    `only_a_rejection_with_every_check_passed_and_content_criteria_unmet_qualifies` and
    `the_seed_is_found_from_the_request_and_carries_redacted_findings`.
  - `Athena::tree_seed` and `Athena::dispatch` (`run.rs`, the seed and the brief):
    `a_content_only_rejection_retries_from_the_rejected_tree` and `a_changed_base_retries_from_the_base`.
  - `Athena::produce_and_verify` (`run.rs`, the snapshot and the empty-candidate test):
    `a_content_only_rejection_retries_from_the_rejected_tree` (full checks, verifier, landing) and
    `a_seeded_retry_that_changes_nothing_fails_as_an_empty_candidate`; the base-start cases are
    `a_failed_check_retries_from_the_base`, `an_unmet_behavior_criterion_retries_from_the_base` and
    `a_failed_job_retries_from_the_base` (all in `crates/athena-cli/tests/retry.rs`).
  - `Snapshot::create` with the new `SnapshotRepo::BaseThenTree`, and `Snapshot::unchanged_since_seed`
    (`crates/athena-git/src/snapshot.rs`): `a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed`
    and `a_snapshot_without_a_seed_is_never_unchanged_since_seed`.
  - `producer_brief` (a retry section and a working-directory paragraph) and `reader_brief` (the base-start
    sentence is now conditional), in `crates/athena-context/src/briefs.rs`:
    `a_seeded_producer_brief_carries_the_findings_and_says_the_work_starts_from_the_candidate`,
    `an_unseeded_producer_brief_is_a_base_snapshot_without_a_retry_section` and
    `reader_briefs_carry_the_earlier_reading_corrections_and_the_discard_notice`.
  - Doc text only: the retry paragraph of `README.md` and the retry item of `crates/athena-cli/src/drain.rs`.
  - Commands run: `cargo test --locked -p athena-git seed`, `cargo test --locked -p athena-context`,
    `cargo test --locked -p athena-core --lib -- only_a_rejection the_seed_is_found`,
    `cargo test --locked -p athena-cli --test retry`, `cargo fmt --all --check` and
    `cargo clippy --all-targets --locked -- -D warnings` (all passed), and `cargo test --locked --no-fail-fast`
    on the whole workspace. That run had two failures, both caused by this change and both fixed: the backtest
    of `docs/decisions.md` (a bare "supersedes" phrase in the new decision) and
    `retrying_a_retry_carries_the_original_request_redacted` in `crates/athena-cli/tests/lifecycle.rs`, whose
    second attempt wrote the same file content as the seeded tree and so, correctly, failed as an empty
    candidate; its second attempt now writes different content. After the fixes,
    `cargo test --locked -p athena-cli --test backtest` and the lifecycle test named above passed; the whole
    suite was not run a second time.
- `LandingCommit` in `crates/athena-data/src/history.rs` has a required `kind` (`job_landing`,
  `declared_intervention`, `records_only`). `job` and `verification` are now optional and omitted when absent;
  `Landing::validate` requires both for `job_landing` and refuses them for the other two kinds. `GitLandingRecords`
  sets the kind of each commit from its content (see "Changed" below).
- The reconcile step of `docs/design/records-history.md` §8.1 (D111, job 3 part 2), in
  `crates/athena-core/src/reconcile.rs`. `athena land` and `athena status` run it first. It walks main's
  first-parent commits after the later of the last landing record and the first landing intent's expected base,
  and for each commit that no landing record or declared intervention names: (1) a landing intent whose head
  tree equals the commit's tree and whose expected base is the commit's parent gives the landing record, with no
  host call (the repair of a crash between the host's merge and the landing record); (2) else the host is asked
  which pull request merged the commit, and the landing is written only if a landing intent records that pull
  request and head; (3) else the commit is reported as unattributed and nothing is written. A host that is
  unavailable, or names no pull request, gives unattributed; Athena never guesses. With no records branch, or
  no landing intent on it, it does nothing and says so. It never fails the command: `athena status` shows the
  findings (and in `--json`), `athena land` prints them before it goes on. Known gap: a crashed landing that a
  later landing has already passed lies before the bound and is not repaired. A pull request merged with no
  landing intent stays unattributed. A rebase that replays several commits leaves the other commits
  unattributed. Declared interventions are read from `declared-interventions/` on the records branch, which
  nothing writes yet.
- `athena land --pr` now writes the records of `docs/design/records-history.md` §5.2, §5.3 and §6.1 (D111, job 3
  part 1). `gh::land_pr` writes a landing-intent record (PR number, head sha and tree, the remote default
  branch's tip as expected base, job ids) to `athena/records` on the remote **before** asking the host to merge,
  and a landing record (host commit, merge method, each commit with tree, job and verification, intent id)
  **after** the merge is confirmed. The landing's records commit takes the pull request's head as second
  parent. Every commit between the remote base and the PR head is classified from its content (see "Changed"
  below); one that no record covers is an error naming its sha, and nothing is recorded. A merge the host did not confirm writes no
  landing and adds a warning to the report that names the intent; reconciliation (§8.1, job 3 part 2)
  repairs it, see the entry above. `GitLandingRecords::for_remote` builds the real sink and `Athena::land_pr` always uses it.
  `RecordsWriter` gained `append_to_remote` (built on the remote tip, pushed with an expected-tip guard,
  local branch moved only after the push, bounded retry) and `sync_from` (reset the local branch to the
  remote). `PrHost::merge` now returns the merge method used, and `PrReport` has a `warnings` list.
- `records_import` in `athena-core` (`crates/athena-core/src/records_import.rs`): the importer of
  `docs/design/records-history.md` §4 and §10 item 2 (D111, job 2). It copies every file under `records/` at a
  commit `C0` into one commit on `athena/records` with `import-manifest.json` (each path, its blob id and
  SHA-256), then re-reads the commit and fails on a missing, changed or extra file or a count mismatch. A
  second import is refused. It exists and is tested on scratch repositories; it is not wired into the
  pipeline, the CLI or CI, and has not yet been run against main.
- `host_rules` in `athena-core`: a read-only check of the host rules that `docs/design/records-history.md` §3
  requires, as the confirmation named in §10 item 1 (D111, part 3 of job 1). It reads
  `repos/{owner}/{repo}/rules/branches/{branch}` with a plain GET for `athena/records` and `main`, and reports
  force-push forbidden, deletion forbidden and up-to-date required as in force, not in force, or unknown (a
  failed call or unreadable reply). It sees rulesets only; classic branch protection is not visible through
  that view. It changes no host setting and is not yet wired into a command.

### Fixed

- The wait before the landing push now waits only for the early candidate's `fmt, clippy, tests` check run.
  It used to stop at the first failed workflow run, but the candidate's history and pull-request-head jobs
  always fail (a candidate is not a landing), so it never waited for the job the cache serves and the landing
  ran the full suite again. That job completing ends the wait; that job failing, or being absent, pushes at
  once; the other jobs' results are ignored. The step and the bound are unchanged.
- `athena check --history` after the cutover no longer fails every pull request. On a head that main does not
  hold, a pull request's own commits after `C0` (the range from the merge base with main to the head) are judged
  by the pre-merge rule (records only, a declared intervention, or a job landing whose verification is in its
  tree), and only main's commits are judged by the attribution rule, which needs a landing record. Before this,
  the whole range went to attribution, so PR #175 failed the required "the full history" check with 50
  violations, each "no landing record names it and no landing-intent matches its tree". A head that is main or
  part of it, and a checkout with no main or no usable merge base (a note says so), are judged as before. The
  pre-merge rule now lives once, in `athena_gate::invariant::classify_pre_merge`; `land::pr_commits` calls it
  and its messages are unchanged. The CI history job fetches `main` and `athena/records` on pull requests too.

### Changed

- The producer deadline `producer_timeout_s` in `athena.toml` rises from 1800 to 3000 seconds. Six producer runs
  since 2026-10-24 hit the 1800 s deadline and lost all their work, several while doing legitimate work whose
  targeted test builds take 5 to 10 minutes (as reported). Splitting jobs, a faster suite and keeping work on
  timeout are queued. See D124.
- CI (`.github/workflows/ci.yml`) runs a push only for `main`, not for every branch, so a pull request branch gets one
  run per commit: the `pull_request` run, in the pull request's own cache scope. Before, each commit also ran a push
  run that missed the cached pass; on PR #185 the two push runs each took the full 16 minutes and landing waited
  17m51s (timings from the request). Pushes to other branches, including `athena/records`, no longer start CI. The
  `pull_request`, `workflow_dispatch` and weekly schedule triggers are unchanged. See D125.
- `land::pr_commits` classifies each commit of a pull request (`remote_base..head`) from its content, not from
  an `Athena-Job` trailer (`docs/design/records-history.md` §5.3). Each commit is judged against its first
  parent under the first parent's exclusion rules, by the first rule that applies: it adds a job's closing
  records, as `athena_gate::invariant::closing_jobs` finds them, so it is a `job_landing` carrying that job and
  its verification id (the job's `verification.json` must be in the tree; the closing records of more than one
  job are refused); else it adds `records/interventions/<month>/<id>.json`, so it is a `declared_intervention`;
  else, if the governed fingerprint changed, it is refused; else it is `records_only`. The error names the
  commit's sha. `PrCommit` now carries `kind`, `job` and `verification`; `LandingIntent` still needs at least one
  job, so a pull request with no `job_landing` commit gets an error and no intent. Root cause: the old rule
  trusted a line in the commit message that nothing checked, and a records-only commit (reading, dispatch,
  introspection, owner path, or a blocked job's records) carries none, so a range holding one could never be
  recorded. The gate already decides which commits land a job from the records they add, and
  `closing_jobs` (now `pub`) is the single source of truth for that, so there is no second rule to drift.
  `athena_gate::invariant::added_interventions` is the content-only twin of the gate's trailer check for
  declared interventions; the gate's own check is unchanged.

  Callers of `pr_commits` and writers of landing records, each with what covers it:
  - `gh.rs` `GitLandingRecords::commits` (the only caller of `pr_commits`; `intent` and `landing` both use it):
    now maps `PrCommit` to `LandingCommit` and validates each. Covered by `land::tests::*` (classification and
    refusals), `gh::tests::records_flow::commits_carry_their_kind_job_and_verification_and_pass_validation`,
    `records_flow::a_pull_request_that_lands_no_job_has_no_intent`,
    `records_flow::a_governed_commit_no_record_covers_is_named_and_nothing_is_recorded_or_merged`, and
    `records_flow::land_pr_writes_the_intent_then_the_landing_with_the_head_as_second_parent`.
  - `reconcile.rs`, which writes landing records through `GitLandingRecords`: no production change; its test
    fixture `World::pr_head` now builds a real job-landing commit, one job per pull request. Covered by
    `reconcile::tests::*`.
  - `crates/athena-cli/tests/reconcile.rs`: the fixture's pull request head was a commit with only a trailer; it
    is now one shared helper, `pr_head`, that builds a real job-landing commit. Covered by the two tests in that
    file.
  - `run.rs` `retry_landing`: no change. It does not call `pr_commits` and writes no landing record; it retries
    the local landing of a blocked job and adds `outcome-landed.json`. The commit shape it produces is the one
    `land::tests::a_blocked_outcome_is_records_only_and_the_later_landed_outcome_is_the_landing` covers.
  - `crates/athena-cli/src/drain.rs` `PrLander`: no change, for the same reason; it lands through
    `Athena::land_pr`, which reaches `pr_commits` only through `GitLandingRecords::commits`.
  - `athena-gate` `invariant.rs`: `closing_jobs` made `pub`, `added_interventions` added; covered by
    `invariant::tests::added_interventions_lists_the_records_a_commit_adds_whatever_its_message_says` and the
    existing gate landing tests.

  Commands run: `cargo test -p athena-core land::`, `cargo test -p athena-core gh::`,
  `cargo test -p athena-core reconcile::`, `cargo test -p athena-core`, `cargo test -p athena-cli --test
  reconcile`, `cargo test -p athena-gate`, `cargo clippy --all-targets --locked -- -D warnings`,
  `cargo fmt --all --check`, and `cargo test --locked`.
- The producer brief gains a "criteria_check: copy the line" section listing every accepted criterion as a bare
  line, with the instruction to report one entry per criterion, in order, with evidence, including criteria
  already satisfied (D116). First reports retyped criteria from memory and drifted from the wording
  `unmet_criteria` matches, so most producers were sent back once. The send-back text is now `send_back_text` in
  `athena-core` and says to copy the line; the shared producer prompt line is shorter. The send-back rate before
  the change could not be measured (records absent from the producer's copy), and no job has run since.

- The change verifier prompt now says each recorded fmt, clippy or test check it cites goes in that criterion's
  commands list, exactly as shown under "Checks Athena ran on this tree", and that quoting it only in evidence
  text is not enough. Reason: job 20261024-033935-c4w4 quoted its checks in evidence text with an empty commands
  list, and D43 (`verify.rs`) reads only the commands list, so the criterion was rejected. Some unrelated
  wording in the same prompt was shortened to stay inside the verifier's token budget. No code change.
- A cited command now counts as seen running if the verifier ran it, or Athena ran and recorded it on the same
  candidate tree and it passed (D115, amending D43). A failed record, a record for another tree, and a
  placeholder with no exit code still leave the command unseen. Reason: job 20261024-033933-ta60 was rejected on
  "cargo test passes for the workspace" while Athena's own test run on that candidate had passed. `judge` takes
  `candidate_tree` and `checks_tree`; unit tests cover the passing, failed and other-tree cases.
- The producer is no longer required to run the full suite (D113, amending D95 and D103). The producer prompt
  says to run targeted tests (`cargo test -p <crate> <filter>`) while working, that Athena runs fmt, clippy and
  the full suite on the candidate, and that the producer runs the full suite at most once. `produce_and_verify`
  sends a report back only for unmet or unreported acceptance criteria; the `producer-report-gaps` note no longer
  tells the verifier to run the suite. Reason: a run takes 9 to 11 minutes, and job nvbb ran the suite six times
  and hit its 1800 s deadline. Athena's own checks on the candidate and the verdict logic are unchanged. The fake
  producer reports no checks, and the lifecycle and job_plan tests cover the new behavior.

### Added

- `athena-git` gains `records_branch`, the writer of the `athena/records` branch (§8.2, D111, build job 1 of
  `docs/design/records-history.md` section 10). It appends opaque files keyed by path as one commit, built on
  the tip it read and published with `git update-ref <ref> <new> <expected>`, a compare-and-swap, so every
  update is a fast-forward. A rejected swap re-reads the tip, re-checks the files and retries, up to 8
  attempts, then returns a `Contended` error. The first commit is an orphan. The branch is append-only: a
  changed, deleted or renamed path is refused, and the same bytes at an existing path are a no-op that returns
  the current tip. Nothing calls the writer yet. Tests cover two threads writing at once, a deterministic stale
  tip with replay, the refusals, orphan history and the attempt bound.
- `athena-data` gains the six record types of the records branch, schema and validation only (D111, build job 1
  of `docs/design/records-history.md` section 10): landing-intent, landing, intervention (declared), checkpoint,
  legacy-exempt and import-manifest, in the new `history` module. Each has a schema tag, unknown fields denied,
  and a `validate` that rejects empty required fields and any sha that is not 40 lowercase hex. The declared
  intervention uses `athena.declared-intervention/1` so it does not collide with the existing
  `athena.intervention/1`. The writer and the host-rules check are later jobs; nothing reads or writes the
  records branch yet, and `records/` on main and the history check are unchanged.

- `docs/spec.toml`, the first layer of the cohesive spec (D114): sourced requirements for BRIEF.md "Project
  Olympus" sections 1 to 6, each with verbatim quotes under their headings, and `xtask/tests/spec.rs`, which
  fails on a quote not found under its heading, a duplicate id, a missing `supersedes` target, a requirement
  without a source, or an uncovered R1 to R8 or section 1 to 5 (enforcement row M33). Sections 7 to 10, the
  governing philosophy and the 2026-10-17 addenda follow in a second job.

- The change verifier prompt now tells verifiers to cite the recorded fmt, clippy and test results under "Checks
  Athena ran on this tree" instead of rerunning the full suite, and to run a check only if its record is missing or
  failed. On 2026-10-18 verifiers rerunning the full suite (about 11 minutes) timed out.
- A `mistake` intervention category (`athena intervene --category mistake`). `athena status` and `athena audit`
  (and their JSON) count mistakes per class of work and per cause, in a `mistakes` field apart from
  `interventions_by_category`, and list the prevention (a check, knowledge, or a design change) proposed for
  each cause; mistakes with the same cause share one proposal. Introspecting a mistake asks the model for the
  check or knowledge that would have prevented it. Records with category `other` whose text starts `Mistake:`
  count as mistakes too and are not rewritten. `athena audit` prints the mistake report even when no job audit
  is produced.
- Closing a job deletes its directory, build cache included (11 to 12 GB each), on every closing path (landed,
  failed, rejected, abandoned, a close of an interrupted job, a close while blocked), and prints the space
  reclaimed, for example `reclaimed 11.4 GB`. A removal that fails prints `warning: could not remove <path>`
  and no reclaimed line, instead of passing silently; the close still succeeds. A running or blocked job keeps
  its directory. Sharing one target directory across jobs was considered and not adopted (decision D112).
- `docs/design/records-history.md` and decision D111: a design (no code) for moving Athena's records onto
  their own history, a separate-root branch in the repository, with migration by copy, attribution of main's
  commits from records alone, and an eight-job build plan.
- D111 accepted under Sri's delegation (`BRIEF.md` addenda of 2026-10-23), with three changes: the history is
  the branch `athena/records` with its own root, not the custom ref `refs/athena/records`, so the host can forbid
  force-pushes and deletion and a plain clone carries it; a new section lists the readers of `records/` and how
  each reads from the branch after the cutover (job 7); and the pre-merge status is stated to depend on the host
  requiring branches to be up to date before merging. Open points filled: pending window 15 minutes, cutover
  after build job 6, two weeks between job 7 and job 8.
- Before the producer starts, Athena runs the capability's configured checks on the base commit and records
  each one's name, pass or fail, exit code and bounded redacted output tail in a new per-job record,
  `athena.baseline/1` (`baseline.json`). A check that fails, times out or cannot start is recorded as failed
  and never stops the job. The baseline is recorded only: verification judges as before, and part 2 will use
  the record at verification.
- Verification now uses the baseline. A check that already failed on the job's base commit and fails again is
  marked pre-existing in its note in `verification.json`; it no longer rejects the job. A check that passed
  on the base, or has no baseline entry, still rejects it, and a missing, empty or could-not-run baseline
  excuses nothing.

- `athena land --pr` no longer gives up at the first red check. It waits until the failed workflow runs
  complete (GitHub refuses to re-run a run in progress), re-runs the failed jobs once with
  `rerun-failed-jobs`, and waits again, within about twice the old wait. Green then merges as before when
  `merge_via_rest` is on; red or pending after the re-run merges nothing and is never re-run again. The
  report names the failing test (from the failed job's log), in the text and in `--json` (`failing_test`,
  `reruns`), whether or not a re-run happened. See `docs/design/native-pr-landing.md` §5.8.

### Fixed

- `athena land --pr` now judges the checks of the pull request's current head and guards the merge with
  that head. Before, it judged the job's own commit and force-replaced a different remote head with it, so
  after a fix was pushed to PR #149 (job 20261018-145008-k7fq) land kept reporting the old commit's failure
  though the new head was green. The replace path is removed: nothing can overwrite a newer head. Until the
  records-history build (D111) lands, a squash of a landing plus any later commit is not a valid landing
  (squash 02837e0 broke the history check), so land refuses when the head is anything other than the job's
  landing commit, declared intervention or not. The refusal names the head's check state, each extra commit
  and why, merges nothing and exits nonzero.

- When one check fails, the verifier now still runs, so every acceptance criterion gets its own result in the
  verification record instead of none. The final verdict still fails on that check.

- Job planning no longer crashes `athena go` when a plan lists a directory. The first planned job
  (20261017-204355-358k) listed `docs/`; `Git::show_file` checked existence with `git cat-file -e`, which
  succeeds for a tree, and then read it as a blob, failing with `git cat-file blob <base>:docs/ failed`.
  `show_file` now returns `None` for anything that is not a blob, so no caller can crash on a directory.
  Athena's plan checks also give a major finding, which sends the plan back to the planner, for a planned path
  that is empty, absolute, contains `..`, ends in `/`, or is a directory in the base. `[job_plan]` is
  enabled again in `athena.toml`; it had been turned off by hand after the crash.

### Changed

- Job planning is on for this repository (job 5b of `docs/design/job-planning.md`). `athena.toml` now has
  `[job_plan]` with `enabled = true`, planner and critic caps of $0.75 each, 600 s timeouts and at most 3
  rounds, so a change job gets a planner and a critic before its producer. Set `enabled = false` to turn it
  off. Decision D110 in `docs/decisions.md` records the alternatives, the keep rule and the reversal. Tests
  that need the unplanned path strip the section (test-only edits in `crates/athena-cli/tests/common/mod.rs`,
  `crates/athena-data/src/config.rs` and `crates/athena-core/src/run_pin_tests.rs`).
- The firstmate explainer page answers the open newcomer findings. "Agent" is firstmate's own deciding AI and
  "worker" is the separate AI tool that does a task, and the page says that "decided by agent" means the
  former; no agent-decided trigger names another decider's event (the t4 trigger now only says the agent runs
  teardown, and the landing is a code-checked guard). The t10 card explains why a yolo merge is drawn Ready to
  Ready while t4 reaches Done, citing claims c12, c13 and c29. The `terminal-success` tag shows as "ends the
  task" and the states lead-in explains Reopened. Finding f2 is removed and the `model-limit` and
  `could-not-run` finding tags get plain labels. "In scope" is defined in the start terms and the glossary
  (the cited instruction gives no test for it, and the page says so). A "Next step" link near the top points
  to the Next steps section. `xtask/src/site.rs` has tests for each.

### Added

- Job planning in `athena status`, and a recovery fix (job 5a of `docs/design/job-planning.md`). A job that went
  through planning shows "plan accepted r<n>" or "plan stopped r<n>" on its status line, the summary shows how
  many planned jobs the first round accepted and stopped, `athena status <job>` lists each round with its
  verdict and findings, and `status <job> --json` carries the plans and critiques. A job that was not planned
  prints and serializes exactly as before. `athena close` no longer adds a "missing producer" invocation for a
  job whose process died during planning: it records the planner and critic invocations it has and nothing for
  the producer, which never started. A job with no trace of any invocation still gets the missing producer.
- The revise and stop paths of job planning, and Athena's own checks on a plan (job 4 of
  `docs/design/job-planning.md`). Planning now runs up to `[job_plan] max_rounds` rounds (default 3). A revise
  sends the planner a revision brief that quotes the previous plan and every finding. Reject, a critic that
  judges the reading misread the request, or a revise in the last round stops the job: it fails through
  `failure()` with the reason "planning stopped before any code was written:" and the findings, with every
  plan and critique committed and no producer or verifier run. A job stopped in planning scores its
  `verification_passes` event as unknown, not as a failed verification (review finding F11); the scoring rule
  is in `athena-score`. A retry's carry quotes the findings of the last critique that stopped the job, not
  only the outcome reason (F11). Before each critic call, `admission::plan_findings` checks the plan at no
  model cost: one demonstration per criterion; for a behavior criterion, `fails_without` and a command the
  verifier's grant allows (`athena_agent::policy::evaluate`); files to modify exist and files to create do
  not; a governance file must be listed and named (`governance_change_is_listed`) or the finding is major;
  any other path outside the reading's deliverables is a minor finding. A major finding writes an Athena
  critique with no critic invocation and sends the plan back. A critic that accepts while holding a major
  finding is treated as a revise, as `verify::judge` does. `crates/athena-cli/tests/job_plan.rs` covers each
  path with scripted planner and critic turns.
- The verifier receives an accepted plan's demonstrations and planned files (job 3b of
  `docs/design/job-planning.md`). When a job has an accepted plan, `produce_and_verify` builds the verifier's
  brief with `verifier_brief_with_plan` from `VerifierPlan::from_plan`, for both the stored
  `brief-verifier.md` and the live brief, and `verification.json` lists "planned demonstrations and files"
  among its inputs. The plan's approach, decision reasons and rejected options never reach the verifier (review
  finding F5). Without a plan the brief and inputs are unchanged. `crates/athena-cli/tests/job_plan.rs` now
  proves the accept path end to end: roles run reader, planner, critic, producer, verifier; the producer's
  prompt holds the planned files and demonstration commands, with its section hash matching the plan record;
  and the verifier's brief holds the demonstrations and files but none of the reasoning.
- The planning round, threaded through dispatch behind `[job_plan] enabled = true` (job 3a of
  `docs/design/job-planning.md`). For a capability that does not pin a repository, `dispatch` runs one planner
  and one critic invocation after the prediction commit and before the producer, accept only: a critic that
  revises or rejects fails the job with "planning stopped before any code was written" (revising is job 4). The
  round seeds the `Closing` that `produce_and_verify` now continues from, so the invocations are numbered
  reader, planner, critic, producer, verifier, and the landing commit holds `job-plan-1.json`,
  `critique-1.json` and every invocation record (review finding F1). `spent` is recomputed after planning, so
  the producer-and-verifier budget check counts what planning cost, and a job left too little stops before
  the producer naming the budget. The producer's prompt is `brief-producer.md` plus the rendered plan section
  (`producer_prompt_with_plan`), the send-back is built from that same prompt, and the plan record's
  `producer_section_sha256` is the hash of that section (F6). With `[job_plan]` absent or off, and for a
  study, roles and records are unchanged.
- The planner and critic prompts, schemas and briefs (job 2 of `docs/design/job-planning.md`), as pure functions
  that nothing calls yet, so behavior does not change. `prompts/planner.md` and `prompts/critic.md` are
  registered as `PLANNER_FRAME` and `CRITIC_FRAME`. `outputs.rs` gains `PlanOutput` and `CriticOutput`
  (`plan_schema`, `critic_schema`), which forbid extra properties at every level; the critic's verdict is
  `accept`, `revise` or `reject`. `briefs.rs` gains `planner_brief` (a revision quotes the previous plan and
  every finding), `critic_brief` (the demonstrations and Athena's findings, nothing of the planner's cost or
  commands), `render_plan_section` (the producer's deterministic "Job plan (accepted, round r)" section) and
  `verifier_brief_with_plan`, which adds only the demonstrations and the planned file list, never the approach,
  reasons or rejected options (review finding F5). `verifier_brief` is unchanged and a test pins its output.
- The schema for job planning (job 1 of `docs/design/job-planning.md`), with no change in behavior. Records
  `athena.job-plan/1` (`<job>/job-plan-<round>.json`) and `athena.critique/1` (`<job>/critique-<round>.json`),
  documented in `docs/records.md`; roles `planner` and `critic` for invocation records; `FindingSource::Critic`
  (review finding F10); and an optional `[job_plan]` section of `athena.toml` (`enabled`, false by default,
  `max_rounds`, `planner_usd`, `critic_usd` and two timeouts) that nothing reads yet. A round-0 record, and a
  critique that accepts while holding a blocker or major finding, are refused. The section is left out of the
  serialized configuration when absent and reset in `reading_inputs`, and a test pins the digest of this
  repository's `athena.toml` computed before the change, so no reading's `config_sha256` moves (F16). The fake
  driver gained a planner that gives one demonstration per criterion and a critic that accepts, and the
  exhaustive role matches in `retry.rs` and `common/mod.rs` gained arms (F15). Where the code differs from the
  design: the design says Athena decides the critique's verdict, so the record keeps both `agent_verdict` and
  `final_verdict`; a critique's `invocation_seq` may be null for a round Athena's own checks ended; and
  `Records::load` files plans and critiques under `JobRecords.plans` and `JobRecords.critiques`, which the
  design did not name. `athena.toml` and `defaults.toml` are unchanged.
- An independent review of `docs/design/job-planning.md`, `docs/design/reviews/job-planning-review-1.md`. It
  checks the design's code claims against the source, the order and size of its five build jobs, and the keep
  rule, and ends with the verdict "build after the listed fixes" (0 High, 11 Medium, 10 Low findings).
- A small shared vocabulary, `docs/ontology.toml`, with seven terms that caused misreads (planning, job,
  Athena, Hermes, product versus system, landed, Sri). Each has a definition, the misread behind it, sources
  and relations. `crates/athena-cli/tests/ontology.rs` checks that sources exist, relation targets are defined
  and terms are unique. See D109 in `docs/decisions.md`.

### Changed

- An interrupted job can be retried and is drained. A job that was dispatched, has no closing record and has
  no process working on it (a machine restart killed it) is accepted by `athena retry`, which carries its
  original request without needing a verification or outcome record; a job whose process is still running is
  refused with "is running". `athena drain` lists such a job as work (on the drained channels only), retries
  it as its first attempt and runs the retry, and does not list it again once a retry names it. The
  `athena status` next step and the `athena go` message for an interrupted job name `athena retry` beside
  `athena close`.
- Generated commit subjects and job titles are cut at word boundaries, not mid-word. The intervention
  commit subject, a reading's title and the title fallback from the request share one helper,
  `truncate_at_word`, still at 72 characters; a single word longer than that is hard-cut. This fixes n4 of
  `docs/design/reviews/athena-v0-acceptance-1.md`.
- Scoring rules are versioned. A score's `rules` string names its version, `v1` (a cost known only as a bound
  is never provisional) or `v2` (it is, the current version); new scores are written under `v2`. The commit
  gate recomputes each score under the version it records instead of demanding the latest rules, and still
  refuses a score whose rules string is no known version. This lets `athena check --history` pass on scores
  written before the bound-only cost rule, with no record edited. See D108 in `docs/decisions.md`.
- A producer stopped by the kill switch is recorded as `killed`, not `error`, even when the child died
  before the driver's own check; and a `cost_usd` quantity score whose execution cost is only a bound is
  marked `provisional` (new `provisional` field, defaulting to false for older `score.json` records), left
  out of aggregates and out of the reference's history. This fixes m8 of
  `docs/design/reviews/athena-v0-acceptance-1.md`.
- The explainer page, after a newcomer's 4/5 rating. "Start here" now has a "Words used first" list that
  defines captain, yolo, green, red, local-only landing and home before they are used; the Glossary gains
  green, red and home and is sorted alphabetically by the generator; each design-choice card (ra1 to ra6)
  has a short title (`rationale_titles` in the study's `intro.json`); the lede explains the study id; claims
  are listed in numeric order and a claim lists each source location once. The sorting and de-duplication are
  in `xtask/src/site.rs`, so they hold for any study, and generator tests cover them.

### Fixed

- `athena audit` of a landed job whose candidate commit is missing (candidate commits are not in a fresh
  clone) read nothing: the auditor got an empty `outcome/` folder, yet the model call was paid for and an
  audit was recorded. The auditor now reads the changed files from the candidate commit if it exists, else
  from the commit on the current branch whose message has the trailer `Athena-Job: <job id>` (the squash
  commit that landed the job). If the job changed files and none can be read, `athena audit` stops before any
  model call with an error naming the job and the commits tried, and records nothing. `athena-git` gains
  `Git::commit_exists`; `Git::show_file` is unchanged.

### Added

- The auditor role and `athena audit [JOB]`. A model judges a job's real outcome (its request, how it ended,
  and the files it changed as they stood in its commit) against `BRIEF.md` and the owner's goals, never
  against the job's own acceptance criteria, which it is not given. It reads with `Read`, `Grep` and `Glob`
  only. The report is a summary and findings, each citing its evidence; a finding with no evidence refuses
  the report, and the paid invocation is still recorded. A good report is recorded as `athena.audit/1` at
  `records/audits/<YYYY-MM>/<id>.json`, beside the auditor's invocation and role definition; `athena audit`
  prints the findings and the invocation record's path, and `athena status` shows the latest audit. The
  auditor's prompt is protected from jobs only by being a fixed constant (`AUDITOR_PROMPT`); the warden
  request will enforce that by mechanism, and until then the protection is prose-only.
- Routing of the checks by the plan's job kind (item 5 of `docs/design/ci-routing.md` section 10), in
  `athena-core::routing`, called from the job run. The path rule `skip_when_only` still decides what is
  skipped, so a docs, design or study job that changes Rust, `Cargo.toml` or `Cargo.lock` still runs `fmt` and
  `clippy`; a `config` job, or one with a missing or unrecognised kind, never has a check skipped; `test`
  always runs. Kinds other than `build` read a CI result for the candidate SHA through `resolve` (a result for
  another SHA is ignored, a red one fails without a local rerun); a `build` job never reuses a result and is
  flagged as needing the workflow's own run. The run path has no CI result to read yet, so checks still run
  locally; no speedup is claimed.
- CI result reuse keyed by commit SHA (item 4 of `docs/design/ci-routing.md` section 10), in
  `athena-core::ci_result`: a record of the run address, full SHA, per-check conclusions and duration; a result
  for another SHA is ignored; a missing, unfinished, cancelled, skipped or timed-out run, or a green run with a
  workflow file changed, falls back to running the three checks locally once; a red run is a failure and is not
  hidden by the fallback; with no result from either, the outcome is blocked with a named reason. Athena reads
  the record from the platform API (`gh.rs`); the verifier does not. Not yet called from a job run; no
  speedup is claimed.

### Fixed

- Main's "full history" check passes again. Commit `39962ed` (the squash of PR #130) carried two jobs'
  landings, `20261010-154004-60r4` (its PR #129 was closed unmerged) and `20261017-121143-sg0j`, and the chain
  between them is broken, so the check failed on it. `39962ed` is added to `[history] exempt` in `athena.toml`
  with a comment, the same remedy as `ee4b0dd` (D66) and `f2a8453` (D70). Its content skips the invariant; each
  job's own records still get the per-record checks. The chain-landing check itself is unchanged.

- The verifier is told that `records/`, private files and history are absent from its snapshot by design, and
  that their absence is not a finding. Root cause: the verifier prompt named the absent paths but did not say
  their absence was expected, and the verifier brief did not mention them at all, so a verifier could report a
  missing `records/` as a defect. Both now say it, worded as the producer and reader prompts word it. A unit
  test in `crates/athena-context/src/briefs.rs` asserts the brief's statement.

- Drain works its queue after an `athena.toml` change that does not bear on a reading. Root cause, two faults:
  a reading was valid only for the raw bytes of `athena.toml`, so changing `[parallel] max_concurrency` read
  every proposed item again; and when that reading needed accepting, drain recorded "no outcome" and then ran
  `athena retry` on a job that was still proposed ("only a rejected or failed job can be tried again"), which
  counted as a second failure and stopped the drain as "repeated failure". Now a reading is valid for the
  parsed settings that bear on it (`Config::reading_inputs`: everything but `[parallel]` and `[landing]`) and
  the bytes of `defaults.toml`; when a relevant change forces a new reading, `athena go` with `--yes` and drain
  accept it and run the job if its capability, class of work and deliverables are unchanged
  (`reading::same_scope`); a changed scope leaves the item queued as "needs accepting" with the reason, not as a
  failure; and drain tries `athena retry` only after a rejected or failed run. Readings made before this
  change are read again once, because the stored configuration hash now covers the parsed inputs.

- A request's ladder rung comes from what it serves in BRIEF.md and the owner's addenda, not a word list.
  Root cause: `reading::infer_rung` took any touched path, the `change` capability or a word such as "code" or
  "ci" as Implementation, so nearly every request got that rung and age alone ordered the plan, which put the
  2026-10-16 speed priorities last. Now `Lookup` carries the brief's sections (`reading::brief_topics`); a request
  sharing at least two significant words with a requirement heading or an addendum ranks as a brief requirement,
  and a word such as "fix a bug" is no rung alone. Alternatives rejected: a model-chosen rung, and keeping the
  word list. Decision D107.

- Plan grouping ignores the shared append-only files `CHANGELOG.md`, `docs/decisions.md` and `README.md`
  (`docs/design/parallel-jobs.md` section 9). Root cause: `plan.rs` `disjoint` counted them as touched paths, and
  almost every item touches `CHANGELOG.md`, so every parallel group was one item. An item whose only paths are
  shared files is still not grouped. The landing check is unchanged. Decision D106.

### Added

- A bounded parallel drain. `athena drain` runs the independent items of one plan group (no dependency, disjoint
  touched paths) at once, up to the new `[parallel] max_concurrency` in `athena.toml`. The default is 1, which
  is the previous one-job-at-a-time drain, with the same order and retry rules; the schema ceiling is 8. The
  `[parallel]` section in `athena.toml` and its `Parallel` field in `crates/athena-data/src/config.rs` land in
  this one change, and a test (`the_real_athena_toml_parses_with_its_parallel_section`) parses the repository's
  real `athena.toml` through `Config`; the section is set to 1, so it changes no behavior until the user raises
  it. Landing and rating stay one at a time; a failure or refusal stops no sibling already in flight; a stop
  starts no further job; model-call caps in flight are held against the budget. A worker panic is not isolated.
  **Trait changes:** `jev::Evaluate` and `gh::HttpPost` now require `Send + Sync`, so the Athena handle can be
  shared by workers; `RealSteps` takes `Sync` UI and lander references. Two tests that used `[parallel]` as an
  example of an unknown section now use another name. Not built: multi-job `athena go` and the `--pr` phase of
  `docs/design/parallel-jobs.md`.
- CI cache layers and a wall-time baseline command (`docs/design/ci-routing.md` build-plan items 2 and 3).
  `cargo run --locked -p xtask -- walltime` reads job records and prints the median and p90 of
  dispatch-to-outcome time per job kind, with failed and rejected jobs reported separately, a caveat when fewer
  than 20 jobs exist, and an explicit `UNAVAILABLE` when the records lack timestamps (tests in
  `xtask/tests/walltime.rs`). `docs/design/ci-routing-baseline.md` records the "before" figures with each marked
  measured, given or estimate, and how to compute "after". **Governance path changed: `.github/workflows/ci.yml`**
  now restores toolchain, registry and target caches keyed on `rust-toolchain.toml` and `Cargo.lock`
  (`actions/cache` pinned by SHA), saves them only on the default branch, and reports exact, restore key or
  miss; the jobs, their commands and the permissions are unchanged. The design is still not accepted and its
  review is unchanged. Not built: the SHA-keyed result reuse and local fallback (item 4), per-kind routing (5),
  the after comparison (6), the README and decisions entry (7), per-phase timestamps, and `timeout-minutes`.

### Changed

- The landing gate and admission judge a changed `athena.toml` or `defaults.toml` by the candidate's schema
  without building candidate code on the host. Root cause: the landing gate parsed the file with the running
  Athena's compiled schema, so a commit adding a section together with its schema field was refused (job
  20261016-200605-xh54), and the earlier admission fix compiled the candidate's `athena-data` on the host. Now
  `crates/athena-data/tests/repo_config.rs` parses the repository's two files with that crate's own schema; it
  runs in the candidate's own `cargo test`. When a commit changes `crates/athena-data/`, both checks skip
  their parse and rely on that test (one shared rule, `athena_gate::defers_to_schema_test`); otherwise they
  parse with the running schema, which is then the parent's. The host-side builder in admission is removed.
  Design note in `docs/design/roles-and-ladder.md`. Tests in `athena-gate` (`content.rs`) and `athena-core`
  (`admission.rs`).
- Admission judges (superseded by the entry above: the host build was removed) a changed `athena.toml` or `defaults.toml` by the candidate's own schema. Root cause: the
  `configuration` rule parsed the file with the running Athena's compiled `Config`, so a job that added a
  section and its schema field together was always refused with `unknown field` (jobs 20261016-071202-q0xy and
  20261016-185517-qcrz, `[parallel]`). When the candidate changes `crates/athena-data/`, admission now writes
  that crate, the root `Cargo.toml` and `Cargo.lock` from the candidate commit to a temporary directory,
  builds a small checker against it offline and runs it on the file; a checker that cannot be built or run is a
  `configuration` violation carrying the real message. Without a schema change the running schema still
  judges. Landing (`athena_gate::config_at`) still parses with the running binary, so a landed schema change
  takes effect for the next run only once Athena is rebuilt. Tests in `admission.rs`.
- A request that `athena plan` has read stays in the plan and in `athena drain`'s queue. Root cause:
  `steward::queued_work` listed only requests in state queued, so reading one (state proposed) dropped it from
  the plan, from the next `athena plan` ("nothing is open") and from drain. It now lists the open requests of
  the drained channels in state queued or proposed, marked read when proposed; drain runs a read one without
  reading it again and lists a job once when a work decision also approved it. A declined, running,
  dispatched or closed request, and one on another channel, stay out. Also, `athena plan --budget` now
  reserves only the reader's cap for a read, not the read plus a run that plan never makes, so the default
  `--budget 2` reads with the shipped caps; drain still reserves the read plus the run.
- Planning no longer fills fields by default. Root cause: plan entries took the first line of an unread request
  as its acceptance and defaulted the rung to Implementation, so the refusal gate almost never fired. Now
  acceptance comes only from explicit bullets or a reading's criteria, the rung from a proposal or from
  evidence in the request or reading (else flagged), and open questions are filled by lookup (a touched path
  that does not exist, a referenced job not in the records). A flagged item names what is missing and why.
  Drain's planning pass reads an unread request whose own words leave the plan incomplete (within the budget
  and job limit) and plans it again from the reading, so a vague request is refused with a reason and a
  specifiable one runs, instead of unread requests being refused and left queued. The pass is deterministic
  code, not a model call; `plan::trace` and `GapLog` are still not wired to the records. Touched outside
  `plan.rs`, `reading.rs` and `drain.rs`: `display.rs` (refusal reason, `athena plan` output) and `main.rs`
  (the `plan` command and drain's help). See D105.
- `athena drain` plans before it works: the open requests are fleshed out (goal, ladder rung, acceptance, open
  questions, dependencies, touched paths) and planned together, and drain works them in plan order (higher rung
  first, a dependent after what it needs, age as the tiebreak) instead of oldest first. The report shows the
  plan's parallel groups; drain still runs one job at a time. A request whose plan lacks a goal, rung or
  acceptance is refused and reported. Why: the owner's 2026-10-16 addendum asks for planning first and for
  open initiatives to be planned together. See D105.
- `athena drain` now also works requests deferred with `athena ask --later` (channel `cli`), not only the
  steward's. Why: those requests were listed as open work but no command that works the queue took them. The
  selection is one channel-set filter in `steward::queued_work`; ordering and retry are unchanged. See D104.

### Added

- `athena plan [--budget USD] [--json]` prints the coordinated plan without working anything: order, parallel
  groups, each item's goal, rung, acceptance, open questions, dependencies and touched paths, then the flagged
  items with reasons; it says so when nothing is open. It runs the same planning pass as drain, including the
  reading of underspecified unread requests, capped by `--budget` (default $2; 0 reads nothing).
- `docs/design/ci-routing.md`: a design, planned first, for moving checks to CI with per-job routing, warm
  caches, one check run per commit SHA reused by the verifier, unchanged verification strength, and wall time
  per verified result measured before and after. Design only; review 1 is requested
  (`docs/design/reviews/ci-routing-review-1.md`) and not done. Why: the owner's 2026-10-16 addendum asks to move
  compute to CI with intelligent routing.
- `athena drain-stop` asks a running drain to stop at the next job boundary: the job in progress is finished,
  landed and rated, no further job starts, and drain ends with the new stop reason `stop requested` (exit code
  2). Why: the only way to end a drain was to kill it mid-job. See D105.
- The plan-stage audit, first slice (`athena_core::plan`): each logged gap is traced to a stage (reading,
  decomposition, assumptions, criteria, outcome check) and a check says whether later gaps of a stage fell after
  it was changed. Reading gaps from records automatically is follow-up.
- `athena status` lists interventions that call for follow-up under open work (`open_interventions` in the JSON,
  a `follow-up` line in text, and a command in `next`), each until a later request job's text names its record
  id. Why: the acceptance review (m6) found that status only counted interventions, so a fresh reader had to
  open records to find them. Interventions Athena observed itself are not listed.
- The drain's page source check now judges the study's model against the pinned code. The checker traces each
  state, transition, counter and rule of `model.json` through the source and compares the page's rules with
  each other, and reports a new problem kind `behavior` that fails the check and holds back "target met". Why:
  the check scored the page 4 against an independent reader's 3 four times, and the reader found a reopened
  item shown as finished (the source returns it to Queued), a counter that missed a failed close freeing the
  place, and a red-PR rule contradicting a waiver. Prompt `source-check.prompt` is now version 2. See D100.
- The firstmate study (`studies/firstmate-work-handling`) now records, each with a cited path and quote from the
  pinned source: teardown reopening a backlog item that is still an open captain call instead of closing it;
  firstmate merging green work itself when yolo is on, and not when it is off; that only a recorded PR or a
  cleanup frees a worker place; and the separate handling of local-only ships, scouts, relaunches and
  secondmates. The explainer page built from it shows three new transitions and a sixth state.
- `athena close <job> --why "..." --allow-legacy-usage` closes an interrupted job whose already committed
  invocation record fails `usage records reconcile` (for example, one written before the usage fix below), which
  used to leave the job impossible to close or retry. The closing commit is gated under `Scope::CommitLegacyUsage`,
  which excuses only invocation records unchanged from the parent commit; new invocation records and all outcomes
  are still reconciled. The closing outcome's totals are summed from the job's real invocation records, the
  legacy record is not modified, and the command prints each record it let through. Without the flag the refusal
  message names the flag. See decision D101. Tests: `closing_a_job_with_a_legacy_usage_record_needs_the_flag_and_names_the_record`
  (`crates/athena-cli/tests/lifecycle.rs`) and `legacy_invocations_are_excused_but_new_records_are_not`
  (`crates/athena-gate/src/reconcile.rs`).

### Fixed

- A producer whose report is still incomplete after the one send-back no longer fails the job when it left
  changes: the change goes to the verifier, which is told which criteria were unmet or unreported and whether the
  full test result was missing, and judges them itself. Only a run that leaves no change fails. The root cause
  was the send-back's give-up policy (`produce_and_verify` in `crates/athena-core/src/run.rs`), which failed
  finished work over a report gap: in job 20261016-005953-582g the producer had done the work, but its second
  report covered 3 of 5 criteria. The number of send-backs is unchanged. See decision D103. Tests are in
  `crates/athena-cli/tests/lifecycle.rs` and `crates/athena-core/src/run_pin_tests.rs`.
- A study job now reads exactly the commit its study's `pin.json` names. The rule: an existing study's
  `studies/<id>/pin.json` overrides `repo_ref`; athena.toml's `repo_ref` is used only for a new study (no
  pin.json for the repository). The cause: the core's pinning step (`pin_repo` in `crates/athena-core/src/run.rs`)
  resolved only the configured `repo_ref` (`main`), so job 20261015-201650-cyvb mounted fc8972cd while the
  study pin said 6407052f, and the producer cited newer code under the old label. With several studies for one
  repository the request must name one study id, otherwise the job stops; a pin.json sha that is not a full
  40-hex id, or a pin that does not resolve to that sha, stops the job before dispatch naming the study. The
  existing study is not re-pinned. See decision D102. Tests are in `crates/athena-core/src/run_pin_tests.rs`.
- Locking a snapshot no longer fails intermittently with `Spawn(NotFound)`. The root cause: the recursive
  permission walk in `crates/athena-git/src/snapshot.rs` (`set_tree_writable`) listed a directory and then
  examined each entry, and a transient file git had just written under `.git` could vanish in between; the
  walk now treats a vanished entry as already handled (other errors still propagate). Snapshot git commands
  also set `gc.auto=0`, `gc.autoDetach=false` and `maintenance.auto=false` so no background housekeeping runs.
  The regression test `a_walk_survives_entries_removed_after_the_directory_was_listed` removes entries
  mid-walk deterministically. The race itself was not reproduced.
- An invocation's recorded tokens are now the requested model's `modelUsage` tokens, the same quantity the
  gate's `usage records reconcile` rule compares. Before, the record took the result event's top-level
  `usage`, which also counts subagents, so a verified job could be refused at landing. The root cause was two
  definitions of what an invocation used. `athena_agent::claude::output_from_result` builds the output (top-level
  usage is the fallback only when `modelUsage` has no entry for the model). End-to-end tests in
  `crates/athena-core/src/invocation.rs` run a result event through parsing, the record and `check_invocation`:
  one with a subagent passes, and
  `a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end` builds its record from an event whose
  reported cost disagrees with `modelUsage` and asserts a Reconciliation violation (it fails with the cost check
  disabled). The gate had no legacy exception to remove. The `modelUsage` definition: an invocation's tokens
  are the requested model's `modelUsage` entry. So the end-to-end negative test covers only the cost
  disagreement (`total_cost_usd` against the `costUSD` sum); a token disagreement cannot arise through the
  parser, and it is not claimed to be covered end to end.

### Added

- Three standing defaults in `defaults.toml`, applying to every class: `outcome-check` (the outcome is the check
  behind the work; a fresh model reader rates audience-facing results as a newcomer, and below 4 of 5 blocks
  completion), `no-empty-success` (an empty queue or finished list is never success alone) and `thrash-stop`
  (the same item failing twice stops the loop and reports thrashing). The reader prompt now tells the reader
  to record applicable ones as acceptance criteria without asking, unless the request says otherwise. Tests in
  `crates/athena-data/src/defaults.rs` and `crates/athena-context/src/briefs.rs` show an explainer-page request
  receives them. Prose-only: no loop code computes the rating or detects thrashing yet.
- A `behavior-evidence` default in `defaults.toml`: acceptance criteria check the requested behavior happening,
  not that types, files or wiring exist, and a protection or denial test shows the action succeeding as root
  before the protection, then refused, and fails rather than skips if the environment cannot enforce it. A unit
  test in `crates/athena-data/src/defaults.rs` pins the rendered line, and `docs/design/defaults-loop.md` §3.7
  gives two worked examples (a read-only request, and a role that calls a model). The reader sees the rule's
  first sentence, which pushed its set over 1,930 tokens, so the reader budget in
  `crates/athena-context/src/lib.rs` is now 2,000 (`docs/decisions.md` D25 records it). The reader prompt
  itself is unchanged.

### Changed

- `athena drain` now counts only outcome-changing landings and checks all page text against the source before
  it says "target met" (`docs/decisions.md` D100). Root cause: the earlier source check read only `model.json`
  for self-consistency, so a wrong statement in `claims.json`, the intro, its transition or a glossary entry
  never held drain back, and landings that touched only other files could still end a run on a rating of a page
  that had not changed. Now "target met" needs (1) a landing in the run whose changed paths include `studies/`,
  the site generator (`xtask/`) or the publish workflow, else the report says the landing did not change the
  outcome; and (2) a model-backed check of every page text file in every study (all but `pin.json` and
  `validation.json`, so a text file added later is covered) against the pinned source, which fails on a
  contradicted or omitted statement and names it (`Athena::check_study_text` in `crates/athena-core/src/pin.rs`,
  run as the verifier role). The self-consistency check stays as a floor. The source check is a model call
  per study: the verifier's cap times the study count must fit `--budget`, or drain stops as "budget spent",
  and its cost is added to the spend. `Stop::TargetMet`, the module docs, `athena drain --help` and the README
  state both rules.

- `athena drain` no longer stops on "target met" from the audience rating alone, and the places-held counter in
  the firstmate study is corrected. Root cause: the stop depended on a rating by a reader who sees only the
  page, who cannot detect a wrong fact; the page said so wrongly for the work-slot counter (`changed_by` listed
  the spawn and the ready-PR report but not cleanup, while the retry on t2 and claim c3 say a place is freed
  when a worker records its ready PR or is cleaned up). Now a rating at the target ends the run only when a
  model-free check of the study models also passes (`model_contradictions` in
  `crates/athena-cli/src/drain.rs`: a counter named in a retry's changes must list a transition for taking a
  place and one for each way it is freed). When the check fails, drain keeps running and its report says the
  check held it back. `model.json` now lists t1, t3 and t4, and `intro.json` says a place is freed only by a
  recorded PR or a cleanup (citing c3). The check is a floor: it finds a model that contradicts itself, not
  every wrong fact; the pinned source stays the authority.

- The explainer site now introduces Athena and firstmate to a newcomer. Root cause: the generator
  (`xtask/src/site.rs`) and `intro.json` assumed a reader who already knew firstmate and studies, so the page
  never named Athena, put the glossary first, printed sources as plain text, and showed an internal field name
  in a finding. Now the title and opening say what Athena is, what a study is and why this one was made; the
  firstmate definition says its workers are AI coding agents and who uses it; the glossary follows the main
  sections; the repository and each claim source link to the pinned commit on GitHub; a Next steps section
  links to Athena and firstmate; and finding f2 is in plain language.

- The verifier's snapshot now refuses writes even from root. Root cause: the verifier ran in a snapshot that
  only its tool policy protected, so a command it ran as root could change HEAD or the work tree, and the
  change was only detected afterwards and counted as a blocker against the work. Now the grant's write deny
  (`crates/athena-agent/src/policy.rs`, tested for every write tool and command) is joined by a lock the
  core takes around the verifier's run (`Snapshot::lock`: write bits removed everywhere, plus the immutable
  attribute through `chattr` where the process may set it). The lock is probed and fails loudly if a write
  still works. If the verifier still changed its snapshot, the core discards that verdict, reruns the verifier
  once on a freshly created snapshot, and records a system fault (a minor finding) instead of a blocker
  against the work. If the budget cannot pay for the rerun, the job's reason reports the budget; if the rerun
  changes the snapshot too, the verifier is recorded as not having completed.

### Fixed

- `athena drain` now approves in-limit proposals and works approved jobs. Root cause: the drain's queue view
  listed every open proposal as waiting for a person, including ones on a rung the raising role may change
  itself (such as the implementation rung), and it never looked at the job that `approve-work` creates, which
  stayed in state proposed with nobody to run it. Now `work_decisions::split_open` separates proposals by the
  raising role's recorded limit, `steward::approved_work` lists approved jobs still in state proposed, and
  drain approves the first kind (`Athena::approve_in_limit`, guarded by the budget and the job limit), then
  runs and lands those jobs without reading them twice. Proposals above the limit, or from an unknown role,
  still wait for a person, and drain never rejects. The module docs, `athena drain --help`, the README and
  `docs/design/roles-and-ladder.md` say so.
- The producer self-check no longer fails producers on wording, and keeps the report that failed it. Root
  cause: a criterion counted as reported only if its text equalled the accepted text exactly, so a leading
  number, a `[behavior]` tag, different case, extra spaces or a trailing full stop failed even a plain
  "cargo test passes", and the failing report was recorded nowhere, so jobs failed with no way to tell
  whether the producer or the check was wrong. Now `unmet_criteria` (`crates/athena-core/src/run.rs`) ignores
  those differences but still lists a criterion that is missing, not marked met, or worded differently. The
  redacted report is kept in the `detail` of the attempt's invocation record, for the first attempt and the
  send-back, and the failure reason names that record.
- The firstmate explainer page now states the failure and limit behaviour the claims already record, and drops
  wording no claim supports. No new claim was added; every fact below reuses a claim in
  `studies/firstmate-work-handling/claims.json`, whose quote is in the pinned firstmate source at the path and
  lines given. A new "When things go wrong, and what is not limited" list in Start here (`intro.json` `limits`)
  and the transition summaries say:
  - a spawn that cannot take the project lock exits 1 and does not wait (c7, `bin/fm-spawn.sh` 3186-3192);
  - an unreadable capacity declaration exits 1 (c4, `bin/fm-spawn.sh` 3173-3176), while a full capacity exits 75
    (c3, `bin/fm-spawn.sh` 194-197 and 3201-3203);
  - a failed relaunch restores the prior durable record (c19, `docs/agent-control.md` 150);
  - no relaunch bound was found (c20, `bin/fm-control.sh` 134) and no bound on deferrals (c9, `AGENTS.md` 369,
    `bin/fm-spawn.sh` 3202); no retry limit is stated for a failed close (c14, `bin/fm-teardown.sh` 22-23);
  - the retry of a failed close is the next session start (c14, c15, `bin/fm-backlog-transition-lib.sh` 19), and
    the cited text names no other, so a close is not replayed until a session starts (an inference from those
    two claims, worded as such);
  - merge authority is the configured merge authority, and with yolo off the captain approves every PR merge and
    local-only landing (c11, `AGENTS.md` 229-230 and 234), with no deadline stated for the captain.
  Glossary: `exit 1` is added and says only what c4 and c7 support; `ship` now says what c10
  (`docs/architecture.md` 371-372) and c12 (`docs/architecture.md` 416) state; `captain` no longer carries words
  no claim states. Transitions t5, t7 and t8 have guard `none` with a placeholder evaluator `agent`; the renderer
  now shows "No guard applies" and names no checker. Corrected in `intro.json`, `model.json` and `findings.json`
  (f2), and the renderer in `xtask/src/site.rs`.
- Limit of the check: the study's source-quote test runs only when `pinned-source/` is mounted. In a checkout
  without it, the test is skipped, so it does not always check the quotes against the pinned commit.

- Building the explainer page no longer writes into the work tree, so a verifier's snapshot stays unchanged.
  `cargo run -p xtask -- site` now writes to `site/` in the cargo target directory (`$CARGO_TARGET_DIR`, else
  `target/site`), and refuses an `--out` inside the repository root other than under the target directory.
  The Pages workflow uploads `target/site`.

### Added

- `athena drain --budget <usd> [--rating-threshold N] [--job-limit N] [--json]`. It works the steward's queue
  itself, so the queue no longer has to be run, landed and rated by hand: for each queued steward request it
  reads and runs the job, lands it through the pull-request path of `athena land --pr` (only a merged, verified
  pull request counts as landed), and then rates the live explainer page. A rating counts toward the target only
  when it was taken after a landing in this run; a run that landed nothing says the page is unchanged and never
  reports `target met`. Stop reasons: `target met`, `budget spent`, `repeated failure` (the same item failed
  twice; it is named), `queue empty` (not success) and `job limit`. Before every model call, including the
  re-reading of a retry, the money spent so far plus the call's cap must fit the budget. Open proposals that
  need a person are listed as waiting and are never accepted, approved or landed. The budget, threshold and job
  limit are command-line flags; there is no new `athena.toml` key (D98). The report, as text or `--json`,
  gives the stop reason, each job with its outcome, cost and landing, the total spent, the latest rating
  against the target and whether it followed a landing, and what is still queued or waiting.
- A Pages watch after a merge. When `athena land --pr` lands a pull request, it finds the `pages.yml` run for the
  merged commit, waits for it for about ten minutes, and re-runs it once through the `gh` wrapper if it timed
  out or was cancelled. Each step is reported; a second failure is reported and not re-run, and a Pages problem
  never undoes the landing (D97).
- A criteria check before verification. The producer's report gains a required `criteria_check` list: one entry
  per accepted acceptance criterion, with the criterion's exact text, a status of `met` or `unmet`, and its
  evidence. In the producer loop, a criterion marked `unmet` or missing from the list goes back to the
  producer, in the same working directory, with every such criterion named in one brief; the send-back budget
  is the one the full-test check already uses, so one round covers both problems. A criterion still unmet or
  missing after it fails the job before the verifier runs. Athena checks that the list is complete and has
  no unmet entry; it does not judge whether the evidence is true. A pinned study is checked too when it has
  criteria. Records made earlier have no `criteria_check` and still read. Lifecycle tests cover an unmet
  criterion that is sent back and fixed before the verifier's first run, one still unmet after the send-back,
  and one missing from the list.
- An `audience` role beside the steward, and `athena audience` to run it. It fetches the live explainer page
  (or `--url`), has a model read it as a newcomer (or `--reader`), and records an `athena.audience-rating/1`
  record at `records/audience-ratings/<YYYY-MM>/<id>.json`: a rating from 1 to 5 and findings that each cite
  their evidence. A rating outside 1 to 5 and a finding with no source are refused. The role only reads and
  records. The steward's brief and `athena status` show the latest rating and its findings, and a rating of 2
  or lower queues a proposal work item that cites the rating. The role is defined in code
  (`athena_data::roles::audience`); `athena.toml` and the agent tool policy are unchanged. Tests cover the
  role, the record, and a run with the fake agent over a stub page. See `docs/design/audience-role.md`.
- `athena approve-work <id>` and `athena reject-work <id> --why "..."` decide a steward proposal (a work item
  of kind `proposal`) by its id. `athena status` now lists open proposals with their ids. Approving was
  impossible before: the work item only cites its evidence by reference into `records/`, which readers cannot
  see, so asking Athena to do the work item gave a blind reading. The approved request is now built from the
  work item: its title, rung and source, and the content of each cited job, file, directory or commit copied
  into the text. The decision is recorded as an append-only `athena.work-decision/1` record, since work items
  are never edited; an item with a decision is no longer open, and a second decision, an unknown id and an
  empty reason are refused. Tests cover the record's round trip, the inlined request, both decisions and the
  refusals.

### Changed

- The introspection of a rejected or failed job now includes why the verification failed. Before, the model saw
  only the outcome's one-line reason, so rejections were grouped by that line and not by cause. The event's
  `what` now lists each unmet acceptance criterion, each blocker or major finding (severity, summary,
  location), and, under a separate "Failed checks" heading, each failed check with the end of its output.
  Lists are capped (8 entries, 400 characters per item, 600 per output tail); a job with no verification or a
  passing one is unchanged, and a failed verdict with no detail says so. The text is read from the job's stored
  `verification.json` and is not redacted again (the check output was redacted when stored). The system prompt
  now says criteria and checks differ. Tests call `event_of_job` on stored records, and a fake-model test checks
  the prompt for a rejected job.
- Agents now receive `GH_TOKEN`, `GITHUB_TOKEN`, `GH_HOST` and `GH_ENTERPRISE_TOKEN` when set (D96). The
  steward's `gh` failed because an agent's environment held no GitHub variable. The owner should export a
  read-only token before running Athena; without one the `github` blind spot is still reported. A test in
  `env.rs` covers set, unset and empty values.
- The reader frame now has the reader check whether the outcome needs `athena.toml`, `defaults.toml` or a
  workflow file, and list and name it, so admission does not refuse the reading later. The producer frame tells
  the producer to stop and report, not edit, a governance file the reading does not list. Both are guidance;
  admission stays the enforcement. Tests cover the frame wording and a workflow path under
  `governance_change_is_listed`.
- Candidate admission now parses a changed `athena.toml` and `defaults.toml` with the gate's own parsers
  (`Config::parse`, `DefaultsModel::parse`). A file with an unknown field fails the job with a
  `configuration` finding before the landing commit is built, instead of only at landing. Tests cover an
  unknown field in each file, parsing files, and a candidate that touches neither.
- The producer must run the full `cargo test --locked` before it finishes, for every change including docs,
  TOML and data-only ones (D95). Policy and config tests read tracked non-Rust files, so such a change can
  fail them. The producer frame says so and no longer says the producer "may" run the checks. In code,
  `produce_and_verify` sends a report whose `checks_run` lacks that command, or shows it `not-run`, back to the
  producer once, with a "Sent back" section in its brief; a second miss fails the job before verification. A
  pinned study is exempt. The producer token budget rises from 1,100 to 1,300 because the study and change
  producers share the frame. The fake producer now reports a passing `cargo test --locked`, and the lifecycle
  tests cover the send-back, the bound, a missing check, and a `fail` result going on to verification.
  Superseded by D113: the producer is no longer required to run the full suite, and a missing result is no
  longer sent back.
- The steward now calls a model (row A9-1 stays Partial). `steward::run` starts an agent with the stored
  `STEWARD_PROMPT` as its system prompt, in a read-only workspace of the code, a copy of `records/`, and a
  `GIT-LOG.txt` of the last 200 commit subjects, and records the invocation (role, usage, cost) as a
  `steward` role record kept apart from jobs. The agent reports structured work items; Athena validates them
  and routes every item through `decide()`: a rung at or below the limit queues a request, a higher one only
  a proposal with its evidence. Each unreadable source, GitHub included, is a blind spot keyed on the
  source's stable name, never on error text, so repeats are not queued twice; a key already in the records is
  skipped. The old failure counting is now an input to the agent. The steward role is version 2. A run that
  queues nothing makes no commit, and a nested run makes no model call. The tool policy gains an allow-list
  for `gh` reads: `-f`/`-F` in any form (`-ftitle=x`), `--field`, `--raw-field`, `--input`, any method but
  GET, graphql, and every unlisted flag are denied, with tests. `AgentDriver` is now `Send + Sync` and
  `Athena.driver` an `Arc`, so the background thread can use it. Limits: the agent sees recent commit
  subjects and a one-commit snapshot, not the full history; it borrows the verifier's model and budget; and
  it has not been run against the real driver.
- A job may not mark its own requirement rows Covered, and the verifier checks cited documents (D94, from job
  20261014-174815-7bf8). `judge()` now compares `docs/design/requirements-map.md` at the base with the
  candidate and adds a major, Athena-sourced finding for each row that moved to Covered or was added Covered,
  so the verdict is not pass. The producer prompt states the rule. The verifier prompt says to read a document
  the request cites and compare the change with it; falling short is a request-reading mismatch. That second
  rule is prose-only, because the repository cannot tell which documents a request cites. Rows A9-1 (the
  steward never calls a model) and A10-1 (cost and yield tracking, reshaping and retiring are not built) are
  back to Partial, and the strategy view counts follow.
- A request's goal now survives to the verdict. The reader prompt names the request's audience, requires one
  acceptance criterion that audience can check, forbids listing the request's own goal as out of scope, and
  tells the reader to read the output a request is about first. The producer prompt treats the criteria as a
  floor and serves the goal. The verifier prompt judges user-facing output as its audience would. A non-empty
  `request_reading_mismatch` now adds a major finding in `judge()`, so the verdict is not pass. The reader
  token budget rises from 1,800 to 1,930 (measured 1,927; D93).

### Added

- Roles and a first role, the steward (rows A9-1 and A10-1). `athena.role/1` records a role: name, version,
  goal, prompt, cadence in finished tasks, and the highest rung it may change itself. `athena.work-item/1`
  records work a role queued, naming its source and citing evidence by reference, with a kind (including
  `blind-spot`) and the requirement rung it concerns. `Ladder` orders five rungs (owner goals, brief
  requirements, design decisions, defaults and conventions, implementation), and `may_change` lets a role
  change a rung itself only at or below its limit and never in the top two. The steward starts in the
  background when the fifth finished task (landed or rejected) is counted, reads the records in place, queues
  a work item for each repeated failure and for each thing it could not read, applies a change on a rung it
  may change by queueing a request, and queues anything above its limit as a proposal with evidence. The
  design note is `docs/design/roles-and-ladder.md`.
- The explainer page is easier for a newcomer to follow: a cited plain-language lead-in under States,
  Counters, Transitions and Why it is built this way; a one-line summary at the top of each transition card;
  an explanation of the two deciders, agent and code; and glossary entries for the state names, the
  places-held counter, exit 75, PR and session start. The text is in `studies/firstmate-work-handling/intro.json`
  (`leadins`, `deciders`, `summaries`), not in `model.json`, whose schema rejects unknown fields. Each new
  sentence cites claim ids, and the renderer rejects an uncited one or a transition with no summary.
- The explainer page opens with a "Start here" section for readers new to firstmate: what it is, a walk-through
  of one piece of work, and a glossary of the page's terms. The text is in
  `studies/firstmate-work-handling/intro.json`; each sentence cites claim ids, and the renderer rejects an
  uncited one.
- `cargo run -p xtask -- site [--out <dir>]` renders the explainer page for the firstmate work-handling
  study to `site/index.html` (gitignored): one self-contained HTML file with inline CSS, a responsive layout
  and a dark theme. All text comes from the study JSON, is HTML-escaped, and cites claim ids; the page states
  the pinned firstmate commit. A new workflow, `.github/workflows/pages.yml`, builds it on pushes to `main`
  and deploys it to GitHub Pages. The repository owner must set Pages source to GitHub Actions once. The
  design note is `docs/design/explainer-site.md`.
- `docs/design/socratic-review.md`: a design (no code) for a plan review between reading and dispatch, for the
  `BRIEF.md` addendum "2026-10-14: Socratic questions". Six fixed question kinds, an answer ladder (facts, then
  Jev, then at most two user questions), recording, defaults-loop feedback, failure origin, honest acceptance,
  alternatives and cost. It is grounded in D87, 9gwv, 16zf, nem8, 8qkc, m1bs, the four reader-budget retries and
  the PR #59 claim. It is for review before anything is built.
- `athena strategy` (read-only, `--json`): coverage per goal (rows, covered, partial, not covered, and the
  not-covered rows) computed from `docs/design/requirements-map.md`, with goal membership read from the goal
  lists of `docs/design/strategy-view.md`. An ID in a goal but not in the map, or a map row in no goal, is an
  error (exit 2). A test fails when the view's "Coverage at a glance" table differs from the computed counts,
  so the strategic view cannot go stale unnoticed. The view's intro no longer calls the counts hand-counted.

### Changed

- The pinned Claude Code version is 2.1.299, up from 2.1.296. The pin is `claude_version` in `athena.toml`;
  the README, the design's setup and pin table, and D03 name the new pin. Root cause: the version was written
  into several current-state docs and a fixture, so each had to change with the pin. Historical review records
  keep 2.1.296 as the version they tested.
- The verifier prompt now says its working directory is a snapshot without records/, owner files or history, so
  anything it ran there is a result about the snapshot only. It must say so in evidence and findings when a
  command depends on absent paths, and never present such a run as a result on the real repository (the false
  claim that reached main in PR #59). Root cause: the prompt never told the verifier what its snapshot lacked.
  A unit test pins the wording; the rule itself stays prose.
- The requirements map, strategy view and roadmap are brought up to date. The map gains four rows for the
  addenda of 2026-10-13 and 2026-10-14 (AD-12 root causes, AD-13 introspection on every correction, AD-14
  decision sets and knowing when a decision is not working, AD-15 common sense over rigid rules), all partial,
  making 128 rows. Five rows move from not covered to partial against what landed (A2-4, A5-5, A6-4, AD-4,
  AD-5), each citing the changelog entry and file that justify it; other rows were re-read and their evidence
  cells updated. The strategy view's coverage table is recounted from the map's tables (20 covered, 67 partial,
  41 not covered) and its "What to do next" is now a numbered list of steps with what blocks each. The roadmap
  counts and step rows agree (108 partial and uncovered rows; AD-12 to AD-14 in RM-4, AD-15 in RM-13). Docs only.

### Added

- `athena review <introspection-id>`, the writer of class labels that `athena score-jev` needs (it counts only
  introspections a reviewer labeled, and no command recorded a review). It shows the introspection (what
  happened, class, cause, prevention, Jev's shadow answer, or that Jev was not asked or failed) and commits an
  `athena.class-label/1` record in the same call: `--right` (the recorded class), `--class <id>` (another
  registry class) or `--new-class <name> --description "..."` (also writes a proposed `athena.failure-class/1`
  record in the same commit), with an optional one-line `--note` and `--json`. `ClassLabel` gains an optional
  `note` (absent from the JSON when not given; older records still parse). `labeled_by` is `person`. A review
  never edits the introspection; reviewing again adds a newer label, which the newest-wins rule already handles,
  and the command says so. An unknown or ambiguous id, an unknown class, no verdict or two, `--right` on an
  introspection with no class, and a multi-line note are errors that write nothing. `athena status` shows the
  number of unreviewed introspections and, when it is above zero, a `next` line that points at `athena review`;
  the count and `labeled::build` share one definition (`labeled::unreviewed`).
- The labeled set and Jev scoring, step 6 of the build order in `docs/design/defaults-loop.md` section 12
  (sections 10.2 to 10.4), as a read-only command: `athena score-jev [--cut <time>] [--json]`. A new record,
  `athena.class-label/1`, holds a reviewer's class for one introspection (a registry id or none, who labeled it:
  `person` or `stronger-model`, and optionally whether it repeats); `athena review` (below) writes it. `labeled.rs` in
  `athena-core` builds the labeled set from introspections that have a label and a stored Jev answer, counting
  those left out (unreviewed, no Jev answer, Jev call failed). It scores the stored failure-class probabilities
  by Brier score against two references (always `none`, word overlap), reports skill over the better one, top-1
  accuracy and the false-match rate on `none` labels, splits by time, and gives the 95% interval only when the
  test size meets the section 10.4 rule, otherwise "no bound was computed". The promotion verdict covers items 1
  to 3; items 4 to 7 are shown as not evaluated. With fewer than 10 test cases (an estimate) it says "too few
  labeled cases" and gives no score and no verdict. The module never opens a file for writing (a test asserts
  it). Not built: set membership, signal match, cluster, confidence, prose and cost scoring, and the live
  shadow run. The run over the real `records/` follows landing.
- The Jev client and a shape-faithful fake, step 5 of the build order in `docs/design/defaults-loop.md` section
  12 (decision D92). `crates/athena-core/src/jev.rs` builds `POST /v1/evaluate` with `{model:
  "typesafe-ai/jev", state, questions}`, several questions per call (`choice` with described options, or
  `boolean`), and parses the answers (`choice`, `probabilities`, `confidence`; or the `probability` of true),
  `usage` and the gateway's cost string. A refusal from the chat endpoint is a build error and is not retried;
  auth and other 4xx errors are not retried; 500, 502, 503, 504, timeouts and resets are, through the retry
  loop of `gh.rs` (now `retry_transient`) over a new `HttpPost` transport (`CurlHttp`, the `curl` program, no new
  crate). The key comes only from `AI_GATEWAY_API_KEY` and is never printed or recorded. `FakeJev` returns the
  same typed reply and is the only Jev used outside the ignored live test. After a rejected or failed job, a
  correction or an intervention, Athena asks Jev the failure-class question (a choice over the registry ids plus
  `new`) and the is-repeat question (a boolean) in one call, in shadow, when `Athena::jev` is set (tests; the
  command line only with `ATHENA_JEV_SHADOW=1`). Both answers, the usage and the cost are stored in the optional
  `jev` field of `athena.introspection/1`, which older records lack; a failed call is stored as an error. Jev
  never changes the recorded class, status or any outcome. `athena status` shows the answers. Live facts are not
  checked here (no network); the live test is ignored by default.
- Defaults proposals, step 3 of the build order in `docs/design/defaults-loop.md` section 12. A new record,
  `athena.defaults-proposal/1`, holds a proposed default: id, `kind` (`add` or `amend`), statement, `applies_to`,
  the version bump (`from`, and `to` = `from + 1`), the introspections it cites with their jobs, a `status`
  (`pending`, `accepted` with the job, `rejected` with a reason, or `withdrawn`) and the hash of the candidate
  `defaults.toml`. A status change is a new record. `athena propose` groups the recorded introspections by failure
  class and writes a pending proposal when a class spans at least 3 distinct jobs (`REPEAT_THRESHOLD`, a constant
  in code; repeats within one job count once). Code drafts the statement from the class and its cases' cause text;
  no language model and no Jev is used, so assumptions, signals and probabilities are not recorded. Each proposal
  is checked before it is written: the candidate `defaults.toml` is built in memory and must parse with
  `DefaultsModel::parse`, the id is unique among the model and the pending proposals, the version is the base plus
  one, and every cited record is later than the decision's start (the evidence-after-decision rule). `athena
  status` and `--json` list open proposals with id, kind, statement, version bump and source records, and omit
  rejected and accepted ones. `athena accept-proposal <id>` submits the ordinary job "apply defaults proposal
  <id>", which lists `defaults.toml` as a deliverable and is read and verified like any request; Athena never
  writes `defaults.toml`, and a test asserts the module's code has no write call. `athena check-proposal <id>
  --candidate <file>` is the apply check: id and statement equal the proposal, the version is the base plus one,
  no other entry changed, and the file parses. `athena reject-proposal <id> --why "..."` records the reason and
  suppresses the class until 3 more distinct jobs have cases newer than the rejection.
- `athena retry <job>` (`docs/design/defaults-loop.md` section 3.5, build order step 2). It retries a rejected or
  failed job from that job's introspection record: the reader's brief gains a "Why it happened" section with the
  cause, the prevention and the proposed next request, and lists the earlier cases of the same class (id, job,
  one line on what happened, and the prevention each proposed, shown as already tried). The original request and
  the accepted corrections are carried as before. The reading is shown as `ask` shows it, with `--yes` and
  `--json`, and nothing runs until the user accepts. A job with no usable introspection falls back to
  `athena ask "try again on <job>"` and says so on standard error. The earlier cases are the ids the
  introspection stored; no class matching is repeated. The section also appears for a typed "try again on <job>",
  since the two share one code path.
- One bounded-retry wrapper for every call to GitHub. The audit found a single call site, `GhCli` in
  `crates/athena-core/src/gh.rs` (the only place that runs `gh`; no HTTP client is used in `crates/`), and the
  retry loop already in `GhClient` (the 5xx retry) is now the wrapper. A call is made at most 3 times. Transient
  means an HTTP 5xx of 500, 502, 503 or 504, a timeout, or a connection reset (the last two read from `gh`'s
  text, since it gives no status for them). A 4xx, an auth error (401, 403, "not logged in") and anything
  else fails on the first attempt. The wait doubles with each failure and adds up to half again as jitter
  taken from the clock. Each retry prints one line on standard error with the call, the status or failure
  kind and the attempt number; the last failure's error names the call, the status and the attempt count. The
  stored `HostError` record is unchanged. A test (`crates/athena-core/tests/external_calls_guard.rs`) scans
  `crates/` and fails when code outside the wrapper runs `gh` or names an HTTP client. Tests in
  `crates/athena-cli/tests/external_calls.rs` inject a one-off and a persistent 500 into each GitHub-calling
  operation and into the `land --pr` flow. Git, the `claude` binary and the tool downloads in `xtask` are not
  covered.
- Introspection, step 1 of the build order in `docs/design/defaults-loop.md` section 12. After every job that
  ends `rejected` or `failed`, every correction of a reading (or answer to its question) and every other
  intervention, Athena records one `athena.introspection/1` without being asked: what happened, the failure
  class, earlier cases in the same class, the cause beyond this instance, what would have prevented the class,
  and a proposed next request. A language model writes the prose and no Jev is called, so the class and the
  prevention kind are marked as not from Jev. New classes are added to the class registry (a store of
  `athena.failure-class/1` records, never a repository file) as `proposed`, with an id assigned by code.
  The reading code passes the model only the stored fields section 3.3 lists, narrows earlier cases by class of
  work and category, caps them at 5, and drops and logs any returned id that was not on the list it passed in. A
  cause that restates what happened (equal after normalising) is flagged and the record rated failed. A failed
  introspection is recorded as failed, never blocks the job, and is not itself introspected. `athena status
  <job>` prints the introspection and `--json` includes it. The records reader, the typed decision headers, the
  evidence-after-decision rule and the citation validation moved from `athena backtest` to
  `athena_core::introspect` and exist once; the backtest re-exports them and its output is unchanged. Step 1 is
  built; the run over the real `records/` follows landing, because the agents' copies leave `records/` out.
  Host-error and landing triggers, retry-from-record and proposals are later steps.

- Fix: the backtest's introspection record was tied to one kind of evidence (the fired run's intervention
  events), so D87 under review through the derived host-error record of ar1x built none. It is now built from the
  earliest valid citation of whichever signal fired: an intervention, or for a derived host-error record its
  source intervention. Observed host-error records have no intervention and are not sources. A decision under
  review with no introspection record is now a finding (condition `d`) naming the decision. Tests copy the
  ar1x intervention and its derived record.

- Backtest evidence now postdates the decision and each citation is validated. A decision starts at the first
  commit that introduced its id heading (`## D87 `), found with `git log -S`, not its current full heading text.
  `athena backtest` counts only records later than that start and fails (condition `e`) on any cited record that
  does not exist, does not match the signal, or is not later than the start. `derive_host_error` now derives only
  when the HTTP code is tied to the refusal of the merge itself ("GitHub refused a merge commit (HTTP 405)"); a
  code from another failure ("the push failed with HTTP 422") derives nothing, and a full stop inside a number
  or version no longer cuts the text. Tests use fixture records with the text and times of interventions kn3q,
  qk2d and ar1x and show D87 under review citing only ar1x.

- Derived host-error records. The 405 that falsified D87 (PR #41, job 9h2j) predates host-error records and
  exists only in the text of intervention `20261013-192156-ar1x`. `athena.host-error/1` gains `derived` and
  `source_intervention` (absent on observed records; validation rejects a derived record without a source).
  `derive_host_error` in `crates/athena-core/src/watcher.rs` derives a record from an intervention whose text has
  `HTTP <code>` (100 to 599) and the word merge, and a job. `athena backtest` holds them in memory (nothing is
  written under `records/`), lists them apart from observed records, labels them derived and not observed by the
  client, and cites the source intervention when one fires D87.

- Fix: D87's intervention signal in `athena backtest` matched class `docs`; it now matches class `code`, as
  `docs/design/decision-sets.md` §9 revised it.
- Decision-sets design §8 step 4, without the brief component and set selection. The root cause of two problems
  seen on 2026-10-13 and 2026-10-14 was that the host client threw away the HTTP status of a refused call:
  - `athena.host-error/1` (status code, operation, phase, a closed message class; no host text), written by the
    host client in `crates/athena-core/src/gh.rs` for every refused call. A 5xx of 500, 502, 503 or 504 is
    repeated up to three calls in all, each refusal recorded, and only the last one reaches the user (the `athena
    land --pr` run of job w4cj that needed a manual rerun after an HTTP 500). Other statuses, 405 and 404
    included, are recorded once and not repeated.
  - `crates/athena-core/src/watcher.rs`: the landing trigger and exact matching. D87's hard signal (operation
    `merge`, status 405 or 422, phase `landing`, one job) is loaded and checked against the record vocabulary;
    `athena land --pr` runs the watcher at the end of every run, so the HTTP 405 of job 9h2j (PR #41) puts D87
    under review on its first occurrence. The watcher reads host-error records only: a record cited by an earlier
    state record is not evaluated twice and an introspection is never an input.
  - `athena.decision-state/1` (append-only, folded to the current state) and `athena.default-expectations/1`
    (record type, writer and reader only; `defaults.toml` is unchanged).
  - `athena backtest` reads host-error records and reports D87 under review, with the record id cited, when one
    shows a refused merge; the decision view gains `under_review`.
  - Held back: the brief component for the decision-state fold (it needs step 3 set selection), and clearing a
    review. `docs/records.md` and the design §8 describe what was built.
- A typed, read-only decision view (`crates/athena-cli/src/decisions.rs`) of every entry of `docs/decisions.md`
  (id, status, `superseded_by`, `amended_by`), and `docs/decision-links.toml`, the committed list of the log's 24
  supersession and amend hits with their confirmed status (D11 by D39 at line 322 is `amended`). Decision-sets
  design §8 step 2 is marked built. No brief, default or decision text changes.

### Changed

- `athena backtest` condition (a) compares the parsed hits with the confirmed list in both directions instead of
  a hard-coded pair list, so it passes on the real log.
- The non-Rust file check (M20) now aims at its intent, that all code is Rust: it flags code in another language
  by extension or `#!` line and allows data and documents anywhere, so a JSON test fixture no longer needs a
  ledger entry. A named allow-list holds `setup.sh` and the CI workflow with their reasons. `docs/non-rust-files.md`
  describes the new rule; the test is renamed `every_tracked_file_is_rust_or_data_or_an_allowed_exception`, with
  `a_stray_script_is_flagged` as its blocking test.
- Decision-sets design records the first `athena backtest` result (failed): the supersession parse had 24 hits and
  one unexpected partial revision (D11 by D39), now decided as `amended_by`; D87's hand-change signal found 0 jobs
  because it named class `docs` and is revised to class `code` (job 9h2j, PR #41, HTTP 405). New design rules: a
  host error that falsifies an assumption puts a decision under review on first occurrence while soft signals need
  repeats, signal values are validated against the record vocabulary, and a rule that rejects work meeting its
  intent is a failure signal (`BRIEF.md` addendum 2026-10-14). Review 1 gets a matching addendum. Design only.
- Decision-sets and defaults-loop designs revised to apply the eight changes of section 6 of review 1: sets reach
  the reader only through capped code-side narrowing, with a runtime per-job set-size check named as a build item;
  over-budget refusal is described as a test; a default's assumptions and signals get a typed record keyed by
  default id; host errors get a record with a status field and landing becomes a watcher trigger; one evidence
  rule and a corrected D87 example; a closed class vocabulary, `decisions_matched`, a no-recursion rule and the
  under-review state as a hashed brief component; one boolean question per decision with the probability of the true
  option as threshold; labeled set, scoring and promotion for set membership and signal match; and the small
  corrections. D90 gets a dated note for its stale section reference. Design only; no code changes.

### Added

- `athena backtest`: the read-only backtest of decisions D87 and D88 (review 1, section 5). It defines the typed
  header for those two decisions, parses the supersession phrases of `docs/decisions.md`, reads job outcomes and
  interventions from `records/` in Athena code, runs D87's exact-match hand-change signal, builds one
  introspection record for the earliest matching event, and reports pass or a finding with its reason (exit 2).
  Its tests write their `records/` fixtures into a temporary directory and remove it; no fixture files are
  committed.
- Review 1 of the decision-sets design together with the introspection step of the defaults-loop design
  (`docs/design/reviews/decision-sets-review-1.md`): fit across introspection records, the defaults loop, typed
  decisions and failure signals; claims checked against code; Jev usage checked against `POST /v1/evaluate`; the
  smallest first build; verdict accept with changes. Review only; neither design is edited.
- Decision-sets design (`docs/design/decision-sets.md`, D91): typed decision fields (kind, scope, status,
  source, enforcement), sets computed by scope and applied to role briefs within their token budgets, per-decision
  assumptions and failure signals that put a decision under review, the D87 merge refusal as the worked example,
  the fit with the defaults loop and Jev, and a migration plan. Design only; nothing is built.
- Every change reading now lists `CHANGELOG.md` as a deliverable, added in code by `ensure_changelog` in
  `athena-core`'s `reading.rs` together with a `changelog` default that cites BRIEF.md's "tagged releases and a
  changelog". Nothing is added when the file is already listed or when the reading is a study or a decline. The
  verifier brief gains a scope section: the deliverables list is the single statement of scope, and a changed file
  outside it is the scope finding, judged against the list and not against how a criterion is worded. No text of
  the acceptance criteria is matched.

### Fixed

- Piping input to a child process no longer deadlocks once the child's output outgrows the pipe buffer. Writing
  all of stdin before reading stdout blocked both sides; `athena_git::output_with_stdin` now writes stdin on its
  own thread while output is drained, and `Git::exit` and the gitleaks runner use it. The repository-policy test
  helper in `xtask` does the same locally, since `xtask` may not depend on `athena-git`. A broken pipe from a
  child that exits early is tolerated.

### Changed

- The defaults-loop design (`docs/design/defaults-loop.md`) is revised to apply review 1: Jev only answers
  typed questions and a language model drafts, R2 is a counter-example and the loop is gated on records, the
  `POST /v1/evaluate` shape and the probability and confidence signals are defined, the reader budget counts
  the first sentence and entry count, and the promotion criterion computes its test size. Introspection on
  every correction, intervention, rejection and failure is added as a first-class step, with Athena code
  reading past records and passing a filtered summary. D90 and D26's count are updated to match. Design only.
- Review 2 of the revised parallel-jobs design (`docs/design/reviews/parallel-jobs-review-2.md`) confirms the
  answers to review 1's F1-F8 as resolved against the source and the native pull-request landing design, and
  finds one High problem (the launcher's landing queue does not match the children's local landing) and five
  smaller ones. Verdict: accept with changes. The review does not edit the design; the next entry records the
  revision that answers it.
- `docs/design/parallel-jobs.md` is revised again to answer review 2 (N1-N6), N1 first: local landing stays in
  each child process and is ordered by the repository lock, and `land --pr` becomes a second phase run per
  landed job, so the earlier launcher landing queue and the claim that a rebuild precedes the next landing
  check are removed. Section 9 rules 1 and 2 take effect with the build, and serializing jobs that name the
  same file is the working rule until then. Children report `locked`, `refused`, `error` and `reread` in a
  `result` field instead of the launcher parsing message text, and the `--json` text now gives the `reread`
  fields. An open pull request gives exit 2 in a multi-job run, on purpose. The `--all` default warns of the
  cost of likely-stale jobs and asks for confirmation, and declared paths are read from
  `Reading.deliverables`. Design only; the build is a later job.
- `docs/design/parallel-jobs.md` is revised to answer review 1 (F1-F8): jobs run as one child process each,
  each lands through its own pull request by squash with the local branch rebuilt after each landing, a job
  whose base is behind is stale until re-verified on request, duplicate refs are rejected up front, the
  worst-case spend is printed, and shared append-only files are written only by a docs job that runs alone.
  Design only; the build is a later job.

### Added

- `athena land --pr` works on repositories that allow only squash merges (D88, which supersedes the
  merge-commit decision of D87 at the owner's, Sri's, choice). The REST merge reads `allow_merge_commit`,
  `allow_squash_merge` and `allow_rebase_merge` from the host and requests the first allowed method in the
  order merge, squash, rebase; with none allowed it fails with an error naming the flags. A squash carries a
  message built from the job's landed commit: its title, and the trailers in the last paragraph with exactly
  one `Athena-Job`, so `athena check --history` passes on main. After a squash (or rebase) is seen merged,
  Athena rebuilds the local branch on the merged commit by replaying the commits after the job's landed
  commit (`Git::replay`, `Athena::resync_onto_merge`), so later landed jobs, records and hand work are kept
  and the next pull request opens clean. A conflict stops the rebuild and leaves the branch unchanged.
- A `root-cause` default in `defaults.toml`: fix the fault where it originates, name an out-of-scope cause
  and propose the fix, and describe the root cause in the change. The reader now sees a compact rendering of
  the defaults (the first sentence of each statement); producer and verifier briefs still get the full text.
- `athena land <job> --pr` (`athena-core::gh`): pushes the job's branch (through the push guard, never forced),
  opens a pull request, waits for green checks and stores the link in the job's state directory. The host
  client runs only `gh api` with REST paths. The new setting `[landing] merge_via_rest` (off when absent)
  decides whether Athena asks the host to merge once the checks are green; with it off the command stops at an
  open pull request, and running it again after the merge finds the merge by pull request number and lands the
  job on a matching fingerprint, or marks it `merged-unverified`. Red or pending checks never trigger a merge.
- `athena-core::pr`, slice 1 of native pull-request landing (D86): a pull request link store in the job's state
  directory, a push guard (the job's own branch only, fast-forward only, no force option), and a
  merged-content check that finds the merge by pull request number and returns "merged but not verified" when
  the fingerprint differs. A library only: no command and no host calls yet.

### Fixed

- `athena land --pr` run again after the merge no longer fails with `not a tree object` until a manual
  `git fetch`: the merged-content check fetches the merged commit from `origin` first (`Git::ensure_commit`;
  the object only, no branch or ref moves). When the job's pull request link exists, the host is asked about
  the merge before the landed commit is required on the current branch, so a merged job is confirmed even if
  its landed commit is not on this branch. A job with no link and no landed commit still reports `not landed
  on this branch`. (D87 also recorded merge commits as the landing shape; D88 replaces that.)
- `athena land --pr` now pushes the job's locally landed commit (the one with the `Athena-Job` trailer and
  the records) instead of the bare candidate or a synthetic records commit, and says so and pushes nothing
  when the job is not landed locally. A rerun on an open pull request whose stored head differs replaces
  `athena/<job>` with one atomic conditional-force `git push` (`Git::push_if_remote_at`) that holds only if
  the remote still has the head Athena just read; the pull request is reused and the stored head updated.
  A remote head missing, unknown locally, or an ancestry check that errors is reported and nothing is pushed;
  no other branch can be replaced. The design is in `docs/design/native-pr-landing.md`.
- `athena land --pr` opens the pull request against the repository's default branch as the host reports it,
  not the local branch (a local branch missing on the remote made the host answer HTTP 422, and a detached
  HEAD stopped the command). The pushed `athena/<job>` branch is now the verified candidate plus a commit
  that adds the job's records from the local history, so merging it lands the verification and outcome too.
  The checks are read on, and the link stores, that pushed commit. The merge opt-in is unchanged.
- Trying again on a failed job that was itself a retry now carries the accepted corrections of every job in
  the retry chain (oldest first, redacted) into the reader brief, as it does for a rejected job.

### Changed

- This repository's `athena.toml` now sets `[landing] merge_via_rest = true`, so `athena land --pr` asks the
  host to merge once the checks are green. The default in code stays off; the test for it now runs on a
  config with the landing table removed and separately asserts that this repository turns the merge on.
- The native pull-request landing design is revised again for review 3, findings N1-N4 (D86): the exact
  merged-content comparison and the `merged-unverified` state, the link kept in the state directory in both
  modes, the stealth body, branch name and leak check, and the per-job checks for a stealth squash.
- The native pull-request landing design (`docs/design/native-pr-landing.md`) is revised to answer review 2,
  findings F1-F8 (D85): a landed job is found from records and a stored pull request number, not from
  trailers; one pull request per job; a pre-report merged-content check; no model `Co-Authored-By` in stealth
  mode; four new open questions with the assumptions taken. Docs only; the roadmap is not edited.
- sccache is removed from `athena.toml` (D84, reversing D80). The user reported 0 hits over 4 jobs; the cause
  could not be reproduced because the producer cannot run `sccache --show-stats`, so no hit count was
  measured. The `[tools.sccache]` pin and the `RUSTC_WRAPPER`, `SCCACHE_DIR` and `CARGO_INCREMENTAL` keys are
  gone from the change and study capabilities; builds are cold again. Optional pin support in setup stays.
  Tests in `config.rs` pin the absence.
- The roadmap (`docs/design/roadmap.md`) is reordered so acceleration comes first (D83). A new step RM-0
  follows the 2026-10-13 addendum "Athena owns acceleration" (`BRIEF.md:494-496`) and covers the sccache
  cache (the user reports 0 hits; not measured here), concurrent jobs including collisions on `CHANGELOG.md`
  and `docs/decisions.md`, native pull-request landing, and timings fed back. The old RM-8 rows and AD-11 moved
  into it, and RM-8 no longer exists. The firstmate study (RM-6) is stated to run alongside RM-0. The strategy
  view and requirements map point to the new order. Docs only; no code changed.
- `./setup.sh --ci` no longer builds sccache. CI sets no `RUSTC_WRAPPER`, so the build only added minutes to
  each CI job. Setup without `--ci` still installs the pinned sccache (D81).

### Added

- A retry now carries more into the reader's "Earlier attempt" section: the retried job's final reading
  (summary, acceptance criteria, deliverables), the corrections and answers the user gave to it, and a
  sentence saying the attempt's changes were discarded and are not in the repository the reader sees. All of it
  is redacted. A job with no reading or no corrections omits those parts. A retry of a retry still carries the
  original request (D82).
- A requirements map (`docs/design/requirements-map.md`), a strategy view (`docs/design/strategy-view.md`)
  and a roadmap (`docs/design/roadmap.md`). The map has a row, with an ID, for every requirement in
  `BRIEF.md` (124 rows) and marks each covered, partial or not covered. Each row either quotes the brief or
  paraphrases it, as the map's *Wording* section says. Only `BRIEF.md`, which this work does not edit, is
  cited by line; every file edited together with the map is cited by heading or quote, so no citation can
  drift. The strategy view groups the rows into eight goals with their coverage, and the roadmap orders the
  next steps for the partial and uncovered rows, each linked to row IDs. No code changed.
- Warm builds with sccache (D80). `athena.toml` now pins `[tools.sccache]` at `0.10.0`, and the change and
  study capabilities set `RUSTC_WRAPPER` to `{athena_home}/tools/sccache/0.10.0/bin/sccache`, `SCCACHE_DIR` to
  `{athena_home}/sccache` and `CARGO_INCREMENTAL` to `0`. `CARGO_TARGET_DIR` and `CARGO_TERM_COLOR` are
  unchanged. This is the step D79 deferred. The `config.rs` tests that assumed no pin now read the pin from
  `athena.toml` and strip it to test the optional case. The speedup is not measured.
- Optional sccache support in configuration and setup (D79). `[tools.sccache]` in `athena.toml` may hold a
  `version` and nothing else; it is absent today, so every existing `athena.toml` still parses. When it is
  present, `cargo run -p xtask -- setup` runs `cargo install --locked --version <v>` into
  `<athena_home>/tools/sccache/<v>`, skips that when the binary is already there, and stops with a message
  if the install fails. Environment values in a capability's `env` may use `{athena_home}`, and
  `paths::sccache_binary` derives the binary path from the one pinned version. `athena.toml` itself is not
  changed in this entry: it sets no pin and no `RUSTC_WRAPPER`, and a later job does that. Unit tests cover
  these; no real sccache install was run.
- Checks can be skipped for prose-only changes. A check in `athena.toml` may carry `skip_when_only`, a list
  of path patterns (`dir/`, `*.ext` at any depth, or an exact path). The runner skips the check, with a
  "not run" note in its record, only when the change is not empty and every changed path matches. Absent
  means the check always runs (row M31 of `docs/enforcement.md`). `athena.toml` now sets the key on the
  change capability's fmt and clippy checks only, with `docs/` and `*.md`, so a prose-only change skips
  them. The test check never skips: `xtask/tests/policy.rs` reads `docs/*.md` and scans every tracked
  file, so a prose change can fail a test.
- Study briefs carry the pin Athena resolved. The producer and verifier briefs of a job that pins a
  repository gain a "Pinned source" section with the url, ref and full SHA from the job record; other
  briefs are unchanged. The producer brief's rule against citing commit hashes now bars only hashes made
  during verification and says to cite the pinned instance as given.
- The first study, `studies/firstmate-work-handling/` (D78): firstmate's work handling at the commit the user
  pinned, as typed records (pin, claims, rationales, model, findings, validation). Every claim carries the
  full pinned SHA, a path and line range, and a verbatim quote, and is tagged `[mechanism]` or
  `[prose-only]`. Kept tests in `athena-cap-study` validate the study against the mounted `pinned-source/`
  tree, including the wrong-SHA and quote-mismatch rules; `docs/non-rust-files.md` lists the JSON files.
- A study job's repository is pinned by the core (D75). Before the prediction commit, `athena go` resolves
  the `repo_url` and `repo_ref` in `[capabilities.study]` to a full commit id through a `Pinner` (the real one
  uses `git`), records URL, ref and commit in a new optional `pin` field of the job record, and mounts the
  tree read-only (file permissions) at `pinned-source/` for the producer and the verifier. The change
  capability never pins. `athena.toml` gains the two keys as a commented example; set them to run a study.
- The study capability is registered in the `athena` binary, so the reader can pick `study` and the core can
  run it. Containment, the run step and publishing are not built.
- `athena.toml` sets `repo_url` and `repo_ref` in `[capabilities.study]` to the firstmate repository on
  `main`, the first study instance (D77). The pin tests' `config()` helper strips any existing keys before
  adding its own. This is a governance file change; it governs later jobs only.
- `athena.toml` gains a `[capabilities.study]` section with the same commands, environment and checks (fmt,
  clippy, test) as the change capability, so a study job's grants and gates come from configuration. This is a
  governance file change; the section governs later jobs only.

### Fixed

- The squash check forms its chain from landed jobs only (D74). A rejected job whose records shipped in the
  squash was counted as a link, so its landed retry looked like a fork ("both follow") and later jobs like
  invalid landings. A rejected job is now left out, and a chain head may name it as its previous job.
  Landed jobs that fork are still refused.
- The history check no longer refuses an intervention's re-score in a squash as not following from the job's
  records (D71). It recomputed the closing score from the whole squashed tree, so it counted interventions
  made after the closing score and resolved `user_intervenes` to true early; the recorded score differed in
  that event's `outcome`, `provisional` and `brier`. It now counts only interventions made at or before the
  score's own time. A forged re-score is still refused.
- The history check exempts the squash of PR #14, 5a73478, as decided in D76. It changes `records/` but names
  no job; the exemption is a configuration entry in `athena.toml`, with no code change.

### Added

- The producer brief lists the words the boundary scan (M19) rejects, in a section before the checks (D73).
  A test fails if that list differs from the scanner's.

- A design for native pull-request landing, including a stealth mode that keeps Athena's records out of a
  repository the user does not own (D72, `docs/design/native-pr-landing.md`). Design only; no code or rule
  changes.

- The gate lands a squash of a chain of verified jobs (D69, now accepted). A commit that adds the closing
  records of several jobs passes when their `athena.candidate/2` records form one chain: ordered by
  `previous_job`, the head starting from the parent's content, each job starting where the previous one ended,
  and the last job holding the landed content. Every job still passes every per-job landing check. The
  commit is refused if a job is `athena.candidate/1`, the chain forks or has a gap, a job changes a
  `.gitattributes`, or the commit has more than one parent. `jobs::check` accepts every job of a chain, the
  history walk records every one, and a single-job landing is as strict as before. `docs/enforcement.md`
  (new row M30), design §7.3, and the squash-chain design are updated; no `[history] exempt` entry was added.

- New `athena.candidate/2` record, and the core now writes it: `base_fingerprint` (the fingerprint of the base
  commit's tree) and `previous_job` (the latest job landed in the base commit's history, absent if none).
  `athena.candidate/1` still parses. This is the record and core half of the squash-chain landing design
  (D69); the gate's chain landing rule, the jobs and history changes, and the "implemented" markings are the
  next job, so the gate still refuses a commit that lands more than one job.

- The squash-chain landing design answers its review 2 (Appendix B): the core derives `previous_job` as the
  latest landed job in the base commit's history, a partly landed chain with one job left is judged by the
  single-job rule and a landed predecessor (even `/1`) is found at the parent, the merge-commit message is
  documented, and the schema work names `AnyRecord`, the schema-name table, and its consumers. D69 is amended
  and stays proposed. Documentation only; the build is a later job.

- Review 2 of the squash-chain landing design (`docs/design/reviews/squash-chain-landing-review-2.md`)
  confirms the answers to review 1 and reports one major and three minor findings, chiefly that the design does
  not say where `previous_job` comes from. Documentation only; the design is unchanged.

- The squash-chain landing design answers its review 1: the chain order is recorded (`previous_job`), the
  schema is `athena.candidate/2`, chains that change classification rules or land on a merge commit are
  refused, the security claim is narrowed to accidental mismatches, and a response appendix and a larger test
  list are added. D69 is amended. Documentation only; still a proposal.

- Design `docs/design/squash-chain-landing.md` and decision D69 propose how a squash merge of a chain of
  verified jobs can land without an exemption: a chain-aware landing rule linked by a new
  `base_fingerprint`. It is a proposal only; no code, configuration, or records changed.

- A commit on `[history] exempt` now also excuses the job records it adds (D68, after D66 and D67): the
  history check no longer reports a landed outcome or a score in that commit. Later commits are still judged.

- Decision D66 proposes a remedy for main's history check failing on a squash that carries several jobs:
  stop squashing multi-job pull requests, and excuse the one existing commit with a `[history] exempt`
  entry. It is a proposal only; no code or configuration changed.

- `athena ask "try again on <job>"` now also retries a job that failed (for example one refused at admission,
  which has no verification). Its recorded outcome reason is carried into the reader's "Earlier attempt"
  section as a finding (redacted), and the section says whether the job failed or was rejected. Other
  states (landed, stale, in flight) are still refused, now with a message saying only rejected or failed
  jobs can be tried again.

- `athena ask "try again on <job>"` retries a rejected job. The reader's brief gains an "Earlier attempt"
  section with that job's request and its final verification's findings (redacted); the new job's own
  request stays as typed. A job that is not rejected, or is unknown, is refused with an error naming it.

- "Try again on <job>" and job references now also carry each failed check's name, command, exit code and
  output tail (redacted, at most 1 KiB each, the end kept) in the reader's brief, after the findings and
  reasons, so a retry can see why a check failed.

- A request that names a job by its full id anywhere in its text now shows the reader that job's request and
  final verification findings (redacted, any job state) in a "Referenced jobs" section of the reader's brief.
  At most three jobs are carried, and an id matching no job is ignored. "try again on" is unchanged.

- Independent review 1 of the study capability M1 design
  (`docs/design/reviews/study-capability-m1-review-1.md`): approve with changes, 7 major findings, answered
  checks 1–8 of the design's §9. The design's status line links to it. No code changed.

- The study capability M1 design is revised to answer all 17 findings of review 1, with a response appendix
  and the review's eight decisions settled (D64). Evidence rules are non-waivable, citations and runs are
  tied to the pin, containment and redaction join the run step, and the model gains guards decided by agents,
  unhandled failures, and retry changes. A re-check is required before building. No code changed.

- Review 2 of the revised study capability M1 design
  (`docs/design/reviews/study-capability-m1-review-2.md`): checks every review-1 finding and decision against
  the revision and approves with named changes (3 minor, 1 nit). The design's status line and §9 link to it.
  No code changed.

- New crate `athena-cap-study`: the validation core of the study capability. It defines the typed
  `study.*/1` records and the `study.model/1` work-handling model, implements the eighteen validation rules of
  the design's §5.4 as pure functions (eight evidence rules that no waiver clears, ten model rules that only a
  user-written waiver citing a user intervention clears), looks up git through a trait with an in-memory
  fixture, and implements the `Capability` trait with its prompts. It is not registered in the CLI and runs no
  studied system. The design gains the review-2 changes (R2-m1 to R2-m3). Scope call recorded as D65.

### Fixed

- `athena ask "try again on <job>"` on a job that was itself a retry now shows the reader the original
  request (followed back through the chain of retries, redacted) plus the retried job's own findings. Before,
  the brief carried only the retry phrase as the earlier request. A broken or cyclic chain falls back to the
  last request reached. A bare "try again on" is refused with "say which job to try again on".

### Changed

- Behavior-criterion evidence (D43) now matches a cited command against whole simple commands inside the
  commands the verifier ran (split on `&&`, `||`, `;`, `|`, outside quotes, whitespace collapsed), so a
  command run in a chain or piped to `tail` counts as seen (D63). A different argument, filter, or package,
  or a command never run, is still unseen and leaves the criterion unmet.

- The study capability M1 design (`docs/design/study-capability-m1.md`) now says how to confirm its
  `BRIEF.md` line citations (by quoted passage, with no commit pinned). Citations were re-checked against
  `BRIEF.md`; its cross-references all resolve. No code changed.

- The governed-content invariant and `athena check --history` now accept a landing that was squash-merged
  or rebase-merged, as well as a merge commit. A landing is identified by the job records the commit adds,
  not by `Athena-*` trailers or by the prediction commit being an ancestor. "Prediction before work" is
  proved from the records: the accepted reading must be recorded before the dispatch and before every
  producer and verifier invocation. A trailer that contradicts the records still refuses the landing;
  `Athena-Prediction` is no longer checked, and the `prediction` rule no longer reports. A second landing
  of a job is found from the parent's records and the history walk, not from trailers. See design §7.3.

- `athena check --history` now reads `[history] exempt` from the tip's `athena.toml`, so an entry added
  after the commit it names takes effect. Before, each commit was judged by its parent's list, which could
  never hold such an entry. An entry still must be a full commit id already in history, so it cannot cover
  a later commit. Per-commit checks are unchanged.

- `[history] exempt` in `athena.toml` now excuses the squash commit ee4b0dd, which carried several jobs'
  landings and fails the landing check, so the history check no longer judges that commit's content. This is the user's
  choice of option (a) in D66, recorded as D67. Its content skips the invariant; its jobs' records still get
  the per-record checks. Stopping multi-job squashes (option c) is not yet adopted.

- `[history] exempt` in `athena.toml` now excuses f2a8453, the squash of PR #9, so the history check no longer
  judges that commit. Most of PR #9's jobs predate `athena.candidate/2`, so the chain landing rule (D69)
  cannot link them. Recorded as D70, following D67 and D68.

## [0.1.0]

The first version: the v0 walking skeleton. It takes one request, runs one job through a sub-agent,
verifies the result with a separate agent, and lands one traceable commit with secret scanning on.

### Added

- A Rust workspace split into focused crates, with the dependency direction documented in
  `docs/architecture.md` and checked by a test: `athena-data`, `athena-score`, `athena-git`,
  `athena-gate`, `athena-context`, `athena-agent`, `athena-core`, `athena-cap-change`, `athena-cli`,
  and `xtask`.
- The `athena` command: `ask`, `correct`, `go`, `close`, `status`, `intervene`, `restart`, `check`,
  `stop`, and `start`. Every command accepts `--json`. `athena --version` prints the package version
  and the commit the binary was built from.
- The request flow. A reader turns a request into a reading: a summary, acceptance criteria,
  deliverables, and forecasts. The user accepts it with Enter or corrects it in one line. A producer
  then makes the change in a detached snapshot, Athena runs the checks (fmt, clippy, tests), and a
  separate verifier judges the exact candidate.
- Landing. A change that passes is committed once, with its records, and the branch moves by
  fast-forward only. A change that fails is closed with its findings.
- The `change` capability, the only kind of work v0 can do.
- Records. Jobs, predictions, interventions, restarts, and scores are stored as plain JSON under
  `records/`, documented in `docs/records.md`. Unknown fields are rejected when records are read.
- A commit gate that every commit passes. It scans for secrets with gitleaks, checks that records are
  valid and append-only, and checks size and personal data.
- Scoring of predictions with proper scoring rules (Brier, log-scale weighted interval score,
  and a summed deliverable score), compared against a reference forecast.
- Cost and token capture for every model call made through Claude Code.
- `setup.sh`, an idempotent, version-pinned setup. It reads its versions from `athena.toml` and
  `rust-toolchain.toml`, checks downloads against pinned SHA-256 sums, and never stores credentials.
- Documentation: `README.md`, `BRIEF.md` (the requirements), `docs/architecture.md`,
  `docs/design/athena-v0.md`, `docs/decisions.md`, `docs/enforcement.md`, `docs/records.md`, and
  `docs/non-rust-files.md`.
