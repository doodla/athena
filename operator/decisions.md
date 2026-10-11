# Operator decision log

Decisions about how the experiment is operated. Decisions about Athena's own design live in
Athena's decision log, not here. Newest last.

## OP-1. Two zones: Athena, and the operator's corner (2026-10-10)

- Decision: everything in this repository is Athena except `operator/`, `BRIEF.md`, and the
  routing lines in the root `CLAUDE.md`. Athena's code, prompts, skills, and records never refer to
  the operator's zone, cloud sessions, or this sandbox.
- Why: the brief requires that Athena know nothing about cloud sessions or the handoff, and that it
  can be lifted out and adopted. A path boundary is easy to see and easy to check.
- Alternatives: a separate operator repository (rejected: the brief wants the lease and handoff in
  this repo); keeping operator notes outside the repo (rejected: lost on restart).

## OP-2. The root `CLAUDE.md` is a front door, not Athena's contract (2026-10-10)

- Decision: the root `CLAUDE.md` routes whoever opens the repository: to Athena's docs, and the
  operator to `operator/HANDOFF.md`. Athena composes its own agents' context and launches them so
  they do not load the root `CLAUDE.md`.
- Why: a session started with only `continue` loads the root `CLAUDE.md` first, so the route to the
  handoff must be there. Athena's always-loaded contract must not mention operators, so it cannot
  be that file.
- Evidence that the separation can hold: `claude -p --setting-sources user` skips the project
  `CLAUDE.md` (canary test in `operator/preflight.md`).
- Alternatives: putting Athena's contract in the root `CLAUDE.md` with an operator line (rejected:
  Athena's agents would read about the operator); a SessionStart hook that prints the handoff
  (rejected: hooks in project settings would also fire for Athena's agents unless suppressed, and
  a file is simpler to find).

## OP-3. `main` is the shared state; work reaches it only by pull request after CI (2026-10-10)

- Decision: each session works on its own branch. Work lands on `main` only through a pull request
  merged with a merge commit after CI passes, using the GitHub REST API.
- Why: a new session may start in a fresh environment and checks out `main`, so `main` is the only
  state a successor can count on. Merge commits keep each commit's identity and trailers, and
  never rewrite published history. Server-side branch protection is unavailable (blocker B3), so the
  merge step checks CI itself.
- Alternatives: pushing to `main` directly (rejected: bypasses verification, and this session is
  only cleared to push its own branch); squash merges (rejected: lose per-commit traceability).

## OP-4. The lease lives on `main` and gates every push (2026-10-10)

- Decision: `operator/LEASE.toml` on `origin/main` is the lease of record, managed by
  `operator/opctl` (Rust, standard library only). A pre-push hook linked to `opctl` refuses pushes
  from a session that does not hold a live lease. Default expiry is 4 hours, renewed when work
  lands. A departing session releases the lease and lands the release before starting its
  successor.
- Why: the brief requires that two operators never push at once. Releasing before starting the
  successor removes the overlap window; the hook gives the rule teeth instead of relying on memory.
- Alternatives: a lease on a dedicated branch (rejected: this session is cleared to push only its
  own branch); a lease without a hook (rejected: prose-only).

## OP-5. The bootstrap runs as sub-agents of the operator session (2026-10-10)

- Decision: the automation engineer, and the independent reviewer of its v0 design, run as
  sub-agents of the first operator session. Their work is recorded as bootstrap work, outside
  Athena, with their measured usage.
- Why: Athena does not exist yet, and the brief makes the bootstrap the only work done outside it.

## OP-6. Lease timing, learned from the bootstrap (2026-10-10)

- What happened: the first lease (6 hours) expired during the bootstrap's long build, and the
  pre-push gate refused the next push, as designed. The lease was renewed and the push went through.
- Decision: renew the lease whenever work lands on `main`, and release it only while this session
  is still the holder of record on `main`. The gate lets the holder of record push a release; a
  session whose lease of record is missing or expired cannot push one.
- Also: landing is fast-forward only, so the operator never commits on a branch while one of
  Athena's jobs is in flight on it. Uncommitted operator edits wait, stashed, until the job lands.

## OP-7. Decision sheets for Sri have a fixed structure (2026-10-11)

- Decision: when Sri must decide, the operator writes a decision sheet as a doc. It opens with where
  things stand in a few sentences, then a "do now" checklist of actions, then the decisions. The
  decisions start with an at-a-glance table (number, decision, recommended option, what it blocks,
  a dropdown for Sri's choice). Each decision follows with the same layout: the question, why it
  matters, an options table (option, what happens, effort or cost, risk) with the recommendation
  listed first, and a one-line recommendation with its reason. The session's record comes last.
- Why: Sri asked for better-structured decision sheets after two that mixed context, options and
  asks in prose and a crowded single table.
