# Changelog

All notable changes to Athena are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
