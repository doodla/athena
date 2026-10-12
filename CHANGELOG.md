# Changelog

All notable changes to Athena are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
