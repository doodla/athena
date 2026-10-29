# Structure pass for `gh.rs` and `run.rs`

Status: proposed design. No code moves in the job that wrote this. Each move below becomes its own job once the
owner approves the plan. The decision entry is D138 in [`docs/decisions.md`](../decisions.md).

Requested by job 20261026-042445-ftnw. It also answers the owner's request to reconsider the design every few
jobs (job 20261017-205558-dn2e) and acts on the structural-pass rule of D132, which queues one such pass per file
over a code-health limit.

## 1. Why

Two files take most of the change. Both changed in about 10 of the last week's landings (the request's count; not
re-measured here, the records are not in the snapshot). This week's breakages (land refusing every pull request,
the cutover breaking pull-request checks, the early pull request's timing) were in them, and critics repeatedly
found callers that a change missed. The cause is shape: one file holds several concerns, so a change to one
concern cannot be checked against the others by reading the file's outline.

## 2. What the files look like now

Measured at the base commit of this job, by reading the files and counting lines with Grep.

| File | Total lines | Production lines | Tests start at |
| --- | --- | --- | --- |
| `crates/athena-core/src/gh.rs` | 5,478 | about 2,620 (lines 1-2404 and 2438-2654) | line 2405 (a test-only seam, `Guard`/`install`/`current`), 2655 (`after_merge_tests`), 2798 (`tests`) |
| `crates/athena-core/src/run.rs` | 3,164 | about 2,990 (lines 1-2990) | line 2991 |

The request gave 5,276 lines for `gh.rs`; the file now has 5,478. The production figure matches the request's
"about 2,600". The request's "251 functions" was not re-counted; this doc lists the production items it moves
instead. Production figures are estimates from line ranges, not from `code_health.rs`.

References to `gh` from other files: Grep for `gh::` and `crate::gh` finds 66 occurrences in 23 files (for
example `reconcile.rs` 8, `display.rs` 9, `run_pin_tests.rs` 7, `jev.rs` 5, `test_support.rs` 5, `main.rs` 4,
`external_calls.rs` 4). Each move below keeps these paths working, so none of those files changes.

### 2.1 `gh.rs`: concerns found

All line numbers are in `gh.rs` at the base commit.

| Concern | Items | Lines | Approx. size |
| --- | --- | --- | --- |
| Transport | `Runner`, `GhCli`, `GhFailure`, `parse_status`, `is_transient`, `transient_text`, `Retry`, `retry_transient`, `clock_entropy`, `HttpReply`, `HttpPost`, `CurlHttp`, `HostErrorLog` | 36-374 | 340 |
| Host API client | `Checks`, `NewPr`, `PrDraft`, `PrHost`, `CiRunState`, `parse_ci_runs`, `parse_check_run`, `failing_test_name`, `MergeMethod`, `MergeMethods`, `GhClient` and its `Host`, `PrHost`, `CiSource` impls, `parse_ci_run`, `classify_checks`, `parse_remote`, `parse_merged_pr`, the `CommitHost` impl | 377-605, 686-1040, 1234-1280, 1851-1885 | 665 |
| Git remote | `Remote`, `GitRemote` | 607-685 | 79 |
| Pages | `PagesRun`, `PagesHost`, its impl, `PagesReport`, `wait_pages_run`, `watch_pages` and the `PAGES_*` constants | 1042-1232 | 191 |
| Records-branch writes | `LandingRecords`, `LandingFacts`, `verification_id`, `history_method`, `GitLandingRecords` and its impls, `CutoverRun`, `cutover_job_for`, `run_cutover`, `cutover_catch_up`, the `ReconcileRecords` impl | 1427-1850 | 424 |
| Landing | `work_branch`, `landed_commit*`, `squash_text_for`, `Poll`, `LandPr`, `PrReport`, `land_pr`, `confirm_existing_merge`, `refusal_text`, `publish`, `Athena::land_pr`, `Athena::after_merge` | 1282-1425, 1886-2087, 2156-2209, 2499-2654 | 556 |
| Early pull request and CI wait | `EARLY_RUN_*` constants, `replace_early`, `wait_for_early_run`, `name_failure`, `Waited`, `wait_for_checks`, `EarlyPr`, `open_early_pr`, `https_github_repository`, `Athena::early_pr`, `Athena::early_pr_to` | 437-442, 2088-2155, 2203-2208, 2210-2404, 2438-2498 | 337 |

The sizes add up to about 2,590, and the test-only seam (lines 2405-2437) is the rest of the 2,620.

### 2.2 `run.rs`: concerns found

`produce_and_verify` runs from line 1549 to line 2293: 745 lines by declaration spacing (the request says 724;
the difference is the doc comment and the gap before the next item). Other long functions, measured the same way
(an upper bound, since blank lines and doc comments before the next item are included): `planning_round`
611-927 (317), `dispatch` 328-571 (244), `close_dispatched` 2392-2588 (197), `land_or_block` 2589-2770 (182).
By this measure only four, not five, are over 180; the fifth the request counts was not found, and the
code-health check (which uses `syn`) is the authority.

| Concern | Items | Lines | Approx. size |
| --- | --- | --- | --- |
| Lifecycle (stays in `mod.rs`) | `GoResult`, `Finished`, `Closing`, `change_status`, `short`, `introducing_commit`, `go`, `go_job`, `tree_seed`, `timeout_seed`, `dispatch`, `RunContext`, `TimeoutSeed` | 54-327, 328-571, 1198-1240 | about 600 |
| Planning | `stop_planning`, `plan_round`, `planning_round`, `end_round`, `RoundEnd`, `findings_line`, `plan_record`, `redacted`, `planning_grant` and `planning_grant_tests` | 572-990, 1166-1196, 1241-1322, 2982-3017 | about 560 |
| Pin and reread | `pin_repo`, `study_pin`, `reread_after_configuration_change` | 991-1165 | 175 |
| Send-back (pure text and report checks) | `unmet_criteria`, `left_to_athena`, `marked_left`, `is_athena_check_criterion`, `send_back_text`, `criterion_key`, and `left_to_athena_tests` | 1366-1478, 3018-3164 | 113 + 147 tests |
| Producer stage | the snapshot, the producer loop and its send-back; `keep_timed_out_work`, `redact_report` | 1586-1739 of `produce_and_verify`, 277-327, 2332-2391 | about 260 |
| Candidate and admission | the change list into a `Candidate`, the idle-producer failure, admission | 1741-1875 of `produce_and_verify` | about 135 |
| Checks | `take_baseline`, `baseline_checks`, `SnapExec`, routing the checks, marking pre-existing failures, the disk gate | 1323-1365, 1485-1548, 1563-1584, 1876-1892, 1923-2037 | about 290 |
| Verification | `verifier_snapshot`, the early pull request call, the verifier's inputs, the verifier run and its rerun on a moved snapshot, redaction of its text | 1893-1922, 2038-2293, 2294-2319 | about 310 |
| Close and land | `close_dispatched`, `land_or_block`, `retry_landing`, `closing_subject`, `landing_body`, `build_text`, small formatters | 2392-2981 | about 590 |

## 3. Proposed modules

Both files become a directory with a `mod.rs`. `mod.rs` re-exports every item that is public or `pub(crate)`
today, at the same path, so `athena_core::gh::LandPr`, `crate::gh::GhClient`, `crate::run::send_back_text` and
the rest keep working and no caller file changes.

### 3.1 `gh/`

| Module | Holds | Depends on |
| --- | --- | --- |
| `gh/mod.rs` | `use` lines, `HOST`, `pub use` of every submodule, the test-only seam, and the three test modules (until the last job) | all |
| `gh/transport.rs` | Transport row of 2.1 | nothing in `gh` |
| `gh/host.rs` | Host API client row | `transport` |
| `gh/remote.rs` | Git remote row | nothing in `gh` |
| `gh/pages.rs` | Pages row | `host`, `transport` |
| `gh/records.rs` | Records-branch writes row | `host` (for `CommitHost`, `ReconcileRecords`) |
| `gh/early.rs` | Early pull request and CI wait row | `host`, `transport` |
| `gh/landing.rs` | Landing row | `host`, `early`, `records`, `remote`, `pages` |

Direction: `transport` and `remote` at the bottom, then `host`, then `pages`, `records`, `early`, then `landing`.
No module in a lower row names a higher one. A circular `use` between two siblings is a sign a move was cut in
the wrong place and the job stops.

Two notes from job G6. First, `EARLY_RUN_WAIT` stays in `gh/client.rs` beside `PrHost::early_run_wait`, which
returns it, and `gh/mod.rs` re-exports it so `gh::EARLY_RUN_WAIT` holds. Moving it into `early.rs` would make
`client` name `early` while `early` names `client`, a cycle. Second, with the text kept unchanged, `early.rs`
also uses `remote` (`GitRemote`, `Remote`) and the `LandPr` type that stays in `mod.rs` until G7. G7 must
move `LandPr` to a place `early` can use without `early` naming `landing`, since `landing` depends on `early`
and a use in the other direction would be a cycle.

Note from job G7. `LandPr` moved with the rest of the Landing row into `gh/landing.rs`, text unchanged, and
`early.rs` still reaches `LandPr` and `work_branch` through the `mod.rs` re-export. So one two-way link remains
between `early` and `landing`: `landing` calls `replace_early` and `wait_for_checks`, and `early` names `LandPr`.
Splitting `LandPr` out to a lower module is a text change, not a move, so it is left to a later job.

### 3.2 `run/`

| Module | Holds | Depends on |
| --- | --- | --- |
| `run/mod.rs` | lifecycle row, `RunContext`, `Closing`, and `produce_and_verify` as a short driver | all |
| `run/planning.rs` | Planning row | `mod` types |
| `run/pin.rs` | Pin and reread row | `mod` types |
| `run/send_back.rs` | Send-back row with its tests | none |
| `run/producer.rs` | Producer stage | `send_back` |
| `run/candidate.rs` | Candidate and admission | `mod` types |
| `run/checks.rs` | Checks row | `mod` types |
| `run/verification.rs` | Verification row | `checks` |
| `run/close.rs` | Close and land row | `mod` types |

### 3.3 Seams in `produce_and_verify`

The function already marks its stages with comments. These are the seams, with their line numbers at the base
commit. Each stage becomes a method returning a small enum, so the early `return Ok(closing)` paths become
`Stage::Stop` and the driver returns when it sees one.

| Stage | Lines | Reads from the driver | Returns |
| --- | --- | --- | --- |
| S1 baseline | 1556-1584 | `ctx`, `seq`, `closing` | nothing; sets `closing.baseline` |
| S2 producer snapshot | 1586-1612 | `ctx`, base commit | the `Snapshot`, grant, composed brief, env |
| S3 producer loop with send-back | 1614-1739 | S2 output, `seq`, `closing`, `accepted` | `Stop`, or `Reported { report, report_gaps, report_kept_in, left_criteria }`; the timed-out and failure returns stay inside as `Stop` |
| S4 candidate | 1741-1828 | snapshot, report | `Stop` (idle producer), or `Built { candidate, changed }` |
| S5 admission | 1830-1875 | candidate | `Stop`, or admitted |
| S6 disk gate and early pull request | 1876-1896 | `ctx` | `Stop`, or continue |
| S7 verifier snapshot and checks | 1897-2037 | candidate, baseline | `Vec<CheckRun>` |
| S8 verifier inputs | 2038-2073 | report gaps, `left_criteria`, plan | the verifier brief |
| S9 verifier run and rerun | 2074-2204 | brief, snapshot | the verification |
| S10 redact and assemble | 2205-2293 | S9 output | the `Closing` |

Values that live across stages (`seq`, `closing`, `report_gaps`, `report_kept_in`, `left_criteria`) are the
reason for the `Reported` struct and for passing `&mut Closing` and `&mut u32`. Stage code is moved line for line;
only the local names that cross a seam change into struct fields. A stage whose body would need a changed
statement, not just a changed name, is split further and reported, not guessed at.

### 3.4 Alternatives considered

- **(a) Leave the files and rely on the code-health request alone.** Rejected: D132 only queues a request
  titled `Structural pass: <path>`; it does not say where the boundaries are, and this week's breakages show the
  cost of waiting.
- **(b) Split by type, not by concern** (all traits in one file, all structs in another). Rejected: a change to
  landing then touches three files, which is the missed-caller problem again.
- **(c) A new crate for the host client.** Rejected for now: `athena-core` items such as `Athena`, `Ui` and
  `LandPr` are used by the client, so a crate would force the dependency direction D20 checks to change. The
  directory split keeps that open.
- **(d) Move tests with the code at once.** Rejected as the first step: test names include the module path, so
  moving them changes `cargo test -- --list`, which is the cheapest proof of no change. Tests move in the last
  job of each file (G8, R9), where the list is compared after normalising the path prefix.
- **(e) Rewrite `produce_and_verify` as a state machine.** Rejected: that is a design change. The stage methods
  keep today's control flow.
- **(f) One big job per file.** Rejected: the producer deadline is 3,000 seconds (D124) and the full suite takes
  9 to 11 minutes, so one run can hold one move and one suite run.

## 4. Order of jobs

Each job is one producer run: one move, one full test run. Jobs run in this order. A job does not start until
its dependencies have landed. "Callers" counts are from Grep of the old paths at the base commit.

Visibility: a private item that another module now needs becomes `pub(super)`, and `mod.rs` re-exports it with
`pub(super) use` (or `pub(crate) use` where it was `pub(crate)`). That is the only edit to moved code. The
three test modules use `use super::*`, so a private item they reach must stay reachable from `mod.rs`.

### 4.1 `gh.rs`

| # | Job | Moves | Depends on | Size | Risk |
| --- | --- | --- | --- | --- | --- |
| G1 | Make `gh.rs` a directory | rename to `gh/mod.rs`; plus the two `paths` lists in `introspect.rs` and one constant in `crates/athena-core/tests/external_calls_guard.rs` (see 5.2) | none | rename | Medium. That guard test names the path `crates/athena-core/src/gh.rs` (its `WRAPPER` constant, line 8) and passes only for that exact file, so the rename fails it. Check also `include_str!`, `#[path]` and any text that names `gh.rs` |
| G2 | Transport | Transport row, about 340 lines | G1 | small | Low: a leaf. Used by every client call and by the retry tests |
| G3 | Remote and Pages | Git remote row and Pages row, about 270 lines | G1 | small | Low: few callers (Grep finds `PagesHost` only in `gh.rs`'s own `after_merge_tests` and `test_support.rs`-level fakes; the job re-greps) |
| G4 | Records-branch writes | Records row, about 424 lines | G1 | medium | Medium: callers are `reconcile.rs` (8 `gh::` occurrences), the cutover tests in `athena-cli/tests`, and `LandPr` |
| G5 | Host API client | Host API row, about 665 lines | G2 | medium | **High: caller-heavy.** Every fake host in the tests implements `PrHost`; Grep finds `GhClient`, `PagesHost`, `land_pr(` and `landed_commit` in 14 files outside `gh.rs` (49 occurrences, among them `main.rs`, `display.rs`, `reconcile.rs`, `trailer_stop.rs`, `external_calls.rs`) |
| G6 | Early pull request and CI wait | Early row, about 337 lines | G5 | medium | **High.** This week's timing breakage lives here; `wait_for_checks` and `wait_for_early_run` are called from `land_pr` and `publish`, which stay in `mod.rs` until G7 |
| G7 | Landing | Landing row, about 556 lines | G4, G5, G6 | large | **Highest.** `land_pr`, `LandPr`, `PrReport`, `Poll`, `landed_commit*` are used from `main.rs`, `drain.rs`, `run.rs`, `display.rs` and several CLI tests (see the Grep above). Do this job last so its diff is only a move |
| G8 | Move the three test modules to `gh/tests.rs`-style files | test text unchanged | G7 | medium | Low. Compare `--list` after normalising the path prefix |

G4 and G3 can run in either order, or one after the other, since both depend only on G1. G2 before G5 because
`GhClient` calls the `Runner`.

### 4.2 `run.rs`

| # | Job | Moves | Depends on | Size | Risk |
| --- | --- | --- | --- | --- | --- |
| R0 | Optional: one characterization test | Adds a test that runs a fake job through `produce_and_verify` and asserts the order of the `ui.line` output and the invocation numbers. This is the only job that adds a test; it edits no other test | none | small | Low. Without it the proof for R5-R8 rests on the existing tests only; the owner decides |
| R1 | Make `run.rs` a directory | rename to `run/mod.rs` | none | rename | Low. Same checks as G1 |
| R2 | Send-back | Send-back row with `left_to_athena_tests` | R1 | small | Low: pure functions. `unmet_criteria`, `left_to_athena`, `send_back_text` are `pub(crate)` and used outside `run` |
| R3 | Planning, then pin and reread | rows from 2.2 | R1 | medium | Medium: `planning_round` is 317 lines but a single move; `run_pin_tests.rs` (7 `gh::` hits) exercises pins |
| R4 | Close and land | Close and land row, about 590 lines | R1 | medium | **High.** `close_dispatched` and `land_or_block` call `gh::land_pr` and write the outcome records; `retry_landing` is reached from `main.rs` |
| R5 | Split `produce_and_verify`, step 1: S1-S3 into `run/producer.rs` | stage methods, each body unchanged | R2 | medium | **High.** First change that is not a pure move: early returns become `Stop` |
| R6 | Step 2: S4-S5 into `run/candidate.rs` | | R5 | medium | **High.** The idle-producer message depends on `report_gaps` and `report_kept_in` |
| R7 | Step 3: S6-S7 into `run/checks.rs` | with `take_baseline`, `baseline_checks`, `SnapExec` | R6 | medium | **High.** Pre-existing marking depends on the baseline's commit |
| R8 | Step 4: S8-S10 into `run/verification.rs` | with `verifier_snapshot` | R7 | medium | **High.** The rerun on a moved snapshot must run exactly once |
| R9 | Move the test modules | | R8 | small | Low |

After R8, `produce_and_verify` is a driver of about 80 lines in `mod.rs`. `dispatch` (244 lines) is not split
in this plan; a later pass can use the same method.

`gh` jobs and `run` jobs are independent until R4, which uses `gh::land_pr`: run R4 after G7 has landed or before
G1 starts, never between, so its caller check is made against one layout.

R4 is done. The Close and land row is in `run/close.rs`; `closing_subject` and `build_text` are re-exported
from `run/mod.rs`. The caller search found no call to `gh::land_pr` from the moved row (it is reached through
`Athena::land_pr` in `gh/landing.rs`), and `retry_landing` is called only from `go_job` in `run/mod.rs`.
`cost_text` stays in `run/mod.rs` because the producer, verifier and planning code also use it.

## 5. How each move is shown to change nothing

The rules below apply to every job. A job fails if any item is not met.

1. **Same tests, unedited.** `cargo test --locked` passes. `git diff -U0` over every test module, and over
   `crates/*/tests/`, shows no line removed or added except `use` lines (and, in G1 only, the guard constant
   described in 5.2). For G1-G7 and R1-R8 the test modules
   stay in `mod.rs`, so no test line changes at all.
2. **Same test list.** `cargo test --locked -p athena-core -- --list` gives the same names before and after (the
   job records the count in its report). A test that disappears or is renamed fails the job. Only G8 and R9
   change names, and they compare after removing the module prefix.
3. **Moved text is the same text.** `git diff -M --color-moved=zebra` (or `--color-moved-ws=ignore-all-space`
   for re-indented bodies) shows the moved blocks as moved. Any line not marked moved is listed in the job's
   report with its reason; the only expected ones are `mod`, `use`, `pub use` and visibility lines.
4. **Public paths unchanged.** `mod.rs` re-exports each moved public and `pub(crate)` item. No file outside the
   `gh/` or `run/` directory changes: `git diff --stat` lists only those paths, `CHANGELOG.md`, and the code-health
   or design docs a job is asked to update. Compilation of `athena-cli` and `athena-core` tests is the proof that
   a path still resolves.
5. **Caller check.** The job's report lists the Grep for each moved name's old path (for example
   `gh::GhClient`, `gh::land_pr`, `run::send_back_text`) over `crates/`, with the file count before and after.
   The counts must be equal, and every hit still compiles. For a moved name used by a trait impl in a test (the
   fake hosts implementing `PrHost`), the job names the test.
6. **Lints.** `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` are clean. No
   `#[allow]` is added.
7. **No other change.** No change to a config, workflow, default or documentation file other than a `CHANGELOG.md`
   line and a pointer in this design doc marking the job done.
8. **Behaviour, not wiring.** The proof is a test that exercises the moved code through its real entry point.
   For G6/G7 these are the existing `land_pr_*` and `early` tests (`land_pr_waits_for_the_early_run_before_pushing`,
   `land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request`, `land_pr_guards_the_merge_with_the_current_head_and_pushes_nothing_over_a_newer_one`).
   For G4 these are the records tests in `gh/tests` (`land_pr_writes_the_intent_then_the_landing_with_the_head_as_second_parent`,
   `cutover_land_imports_at_c0_and_writes_the_legacy_exempt_record`). For R2 these are `left_to_athena_tests`. For
   R5-R8 these are the `run`-level tests that drive a fake job; R0, if the owner approves it, adds the one
   missing check, on ordering.

For R5-R8, which change control flow, two more checks apply:

- The job's report places each early `return Ok(closing)` of the old function beside the `Stage::Stop` that
  replaced it, with the old and new line numbers. A missing pair fails the job.
- The job's report lists every local that crossed a seam and the field that carries it now.

### 5.2 Path literals that name `gh.rs`, and the one planned test edit that is not an import

Grep for `gh.rs` and `run.rs` in `*.rs`, `*.toml`, `*.yml` and `*.sh` files finds these uses of the path as data.
The other hits (`routing.rs` comment, and fixture strings in `code_health.rs`, `verify.rs`, `reading.rs`,
`display.rs` tests) only use the name as sample text and do not depend on the file existing.

- `crates/athena-core/src/introspect.rs` lines 148 and 191 list `crates/athena-core/src/gh.rs` as the `paths` of
  decision headers D87 and D88 (production code). G1 must make these name the directory
  (`crates/athena-core/src/gh/`) or add the new files, and the job must first read how `paths` is matched
  (exact or prefix) and show a test that a landing touching a moved file still selects D87 and D88. If the match is
  exact, this is a behaviour edge a rename can silently change, so it is the most important caller check of G1.
- `crates/athena-core/tests/external_calls_guard.rs`, described next.

`crates/athena-core/tests/external_calls_guard.rs` allows `Command::new("gh")` in exactly one file, by exact path
(`WRAPPER`, line 8, compared at line 25). The path is the thing that moves, so G1 must change that constant, or
the existing test fails. The proposed edit is the smallest one: compare with the directory prefix
`crates/athena-core/src/gh/` (and the old exact path removed). It is not an import line, so it breaks the
"imports only" rule on purpose, and it needs the owner's approval before G1 is queued. Its proof is behavioural,
not wiring: with the edit in place, the test still fails when a `Command::new("gh")` line is added to any file
outside `gh/` (the guard's own self-tests, which build a violating file list, cover that), and it passes with
the call in `gh/transport.rs`. The job's report shows both runs. No other test depends on the path
(see the list above; the job repeats the Grep to confirm). For R1, `run.rs` has no production path literal, only the
fixtures above.

### 5.1 Per-job proof summary

| Job | Specific test or grep proving it |
| --- | --- |
| G1, R1 | `cargo test --locked` passes; Grep `gh\.rs` and `run\.rs` over the repository finds only history and prose |
| G2 | the `retry_transient` and host-error tests (`a_500_twice_then_success_lands_with_two_host_errors_and_no_error_for_the_user`, `each_retry_is_logged_with_the_command_status_and_attempt_and_the_last_error_names_them`) pass; Grep `gh::(Runner|GhCli|GhFailure|retry_transient|HostErrorLog)` unchanged |
| G3 | `after_a_merge_the_branch_is_resynced_then_pages_watched_then_code_health_checked`; Grep `gh::(Pages|watch_pages|Remote|GitRemote)` unchanged |
| G4 | the records tests listed above; Grep `gh::(LandingRecords|GitLandingRecords|LandingFacts|cutover)` unchanged |
| G5 | `the_client_runs_only_gh_api_with_rest_paths`, `the_merge_method_is_chosen_from_what_the_host_allows`, `checks_are_classified_from_the_two_replies`; Grep `gh::(GhClient|PrHost|Checks|CiRunState|MergeMethod|parse_)` unchanged |
| G6 | the early and `wait_for_checks` tests above; Grep `gh::(EarlyPr|open_early_pr|EARLY_RUN_WAIT)` unchanged |
| G7 | every `land_pr_*` test; Grep `gh::(land_pr|LandPr|PrReport|Poll|landed_commit)` unchanged |
| G8, R9 | `--list` equal after prefix removal; moved test text identical under `--color-moved` |
| R0 | the new test fails when `closing.baseline` is set after the producer instead of before |
| R2 | `left_to_athena_tests`; Grep `run::(unmet_criteria|left_to_athena|send_back_text)` unchanged |
| R3, R4 | existing `run`-level tests and `run_pin_tests.rs`; Grep `run::` of the moved names unchanged |
| R5-R8 | all existing tests; R0's ordering test if approved; the early-return pairing described above |

## 6. Risks and what a job does about them

- **Visibility edits.** Moving a private function across files needs `pub(super)`. This is the reason for rule 3:
  the only unmoved lines are `mod`/`use`/visibility.
- **Missed callers** (the failure this plan is meant to stop). Rule 5 makes the Grep part of the job, and the
  path-preserving re-export means a missed caller is a compile error, not a silent change.
- **Conflicts with other landings.** Both files change weekly. A job rebases onto the latest base before it
  starts; if a landing touches the same functions, the job moves the new text, not the old. Queue the jobs
  with the owner's priority so ordinary work does not land between G6 and G7.
- **Producer deadline.** The moves are sized so that each fits one run (the deadline is 3,000 seconds, D124,
  and the suite is 9 to 11 minutes). Moved line counts per job: G5 about 665, G7 about 556 and R4 about 590 are
  the largest; a producer that cannot finish one of them reports it and the job is split by trait (for G5: the
  parsers first, then `GhClient`).
- **Stale numbers.** Line numbers in this doc are the base commit's. Each job re-measures before it moves.

## 7. What the owner decides

1. Approve the module names and order, or change them.
2. Whether R0 (the ordering test) is worth a job before R5.
3. Whether to queue all jobs now with the same priority, or one at a time, each after the last has landed.
4. Whether G1 may edit the one guard constant in 5.2, or the wrapper's `Command::new("gh")` must stay in a file
   at the old path `gh.rs` (the alternative: keep a small `gh.rs` that holds only the transport and declare
   `gh/` submodules beside it with `#[path]`-free `mod` lines; this keeps the guard unchanged but leaves a
   `gh.rs` plus `gh/` pair, which this plan prefers to avoid).
