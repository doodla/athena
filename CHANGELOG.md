# Changelog

All notable changes to Athena are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- The history check no longer refuses an intervention's re-score in a squash as not following from the job's
  records (D71). It recomputed the closing score from the whole squashed tree, so it counted interventions
  made after the closing score and resolved `user_intervenes` to true early; the recorded score differed in
  that event's `outcome`, `provisional` and `brier`. It now counts only interventions made at or before the
  score's own time. A forged re-score is still refused.

### Added

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
