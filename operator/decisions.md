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

## OP-7. Decision sheets for Sri are short and fixed in shape (2026-10-11)

- Decision: a decision sheet is a doc with, in order: one status line; a "do now" checklist of
  terse items; an at-a-glance table (number, decision, recommended option, a dropdown for Sri's
  choice); then one small options table per decision (option, cost or risk in a few words, the
  recommendation bold and first); and a one-line session record. No explanatory paragraphs.
- Mobile first: Sri reads on a phone. A decision sheet is an HTML artifact of cards, not a doc:
  a "Do now" card of tick buttons, then one card per decision with full-width option buttons (letter,
  what, cost in a few words, a "Recommended" chip). Taps save to the artifact's `db` capability
  (`answers/<decision>`, `todos/<id>`), which the operator reads with ArtifactData. No tables, no
  dropdowns. The template is the page at https://claude.ai/artifact/UoSLkWzLWMLxUJX7zRp9v3.
- Why: Sri asked for better structure, then said the structured version had too much text, then
  that tables and dropdowns were unfriendly on mobile (use cards and buttons).

## OP-8. Work lands on `main` by squash, through auto-merge (2026-10-11)

- Decision (Sri, from the decision sheet of 2026-10-11): PRs land by squash only. Sri set the
  repository to squash-only with auto-merge allowed, and a ruleset on `main` (no deletion, no force
  push, a PR with 0 approvals, required checks, linear history). The operator turns on auto-merge
  for a PR once its work is verified; GitHub merges when the required checks pass. This replaces
  OP-3's merge commits.
- Also decided: the Claude Code plugin requirement (brief addendum of 2026-10-10) comes after the
  first milestone; a stuck change gets one more attempt through Athena before any hand fix.
- Why: Sri wants `main` to read as one intent per PR; zero approvals keeps the operator able to
  land work, with CI and Athena's independent verifier as the gate.

## OP-9. One job per PR; a multi-job branch lands by rebase (2026-10-12)

- Decision (Sri, decision sheet of 2026-10-12 06:08, https://claude.ai/artifact/UoSLkWzLWMLxUJX7zRp9v3):
  future PRs carry one Athena job each, so a squash keeps one landing per commit on `main`. A branch
  that already holds several jobs lands once by rebase merge, then the repository goes back to squash.
- Why: `athena check --history` refuses a commit that closes more than one job, and every commit on
  `main` must trace to the work that produced it (BRIEF A5). A squash of several jobs breaks both.
- Lapse: the session that got this answer did not record it before restarting, so the next session
  squash-merged PR #8 with six jobs (`ee4b0dd`) and `main` went red at 08:28. Answers from a decision
  sheet are recorded here as soon as they are read. The repair is on the sheet at
  https://claude.ai/artifact/CCDeJxGckn7svF5QNioWJs.

## OP-10. The bar to reach Sri is high (2026-10-12)

- Decision (Sri, in chat, 2026-10-12 ~18:25): "I want the bar to reach me higher." The operator contacts
  Sri only for what nobody else can do: a setting only the repository owner can change, a change to the
  requirements, or a call only the owner can make. When any working path exists without Sri, take it, even
  if it is slower, and record the choice here or in Athena's records.
- Applied at once: the request to turn on rebase merging for PR #9 was withdrawn. PR #9 instead waits for
  Athena's squash-chain landing (D69) to be built, so its squash lands cleanly with no settings change.

## OP-11. PR #9 lands by squash and its squash is excused like ee4b0dd (2026-10-12)

- Decision: squash-merge PR #9, then add its squash commit to `[history] exempt` through one single-job
  Athena PR, the remedy Sri chose for ee4b0dd (1A on https://claude.ai/artifact/CCDeJxGckn7svF5QNioWJs).
- Why: a simulated squash of the branch still fails the history check (19 violations) after D69 was built.
  The chain rule links only jobs recorded as `athena.candidate/2`, and most of PR #9's jobs predate it.
  Scores re-derived after an intervention also do not reconcile inside a squash (to fix through Athena).
  Rebase merging would need Sri to change a setting, which OP-10 rules out while another path exists.
- Cost: main is red from the squash until the exemption lands, about one CI round.

## OP-12. The operator owns ordering and drives Athena toward accelerating itself (2026-10-13)

- Decision (Sri, in chat, 2026-10-13 ~04:55): "I want you to figure out ordering. In the end my goal is to get to
  what I've described in the brief. I'm okay to meander a bit but what I want is for Athena itself to accelerate."
  The operator owns the driving session: picks the order of work, runs it through Athena, tests hypotheses with
  measurements, and keeps tweaking Athena through Athena, until Athena can hold the ordering itself.
- How: each step states a hypothesis and a predicted time and cost; the result is measured against it. The
  ordering lives in Athena's roadmap once it exists (requirements map and strategy view are the next design), not
  in this file or the handoff.
- Also from the same chat: PRs that add records without landing a job are merged by hand with a squash message
  whose last paragraph names the job (`Athena-Job:`); a squash may name only one landed job. GitHub's auto-merge
  appends a co-author paragraph that hides those trailers. This lasts until native PR landing is built.

## OP-13. Course correction: an outcome milestone, lazy requests, every rescue logged (2026-10-14)

- Finding (BRIEF.md O4): work on Athena's machinery crowded out the study for a sustained stretch. On 2026-10-14
  `athena strategy` showed the study (G6) at 0 of 23 rows covered and the site (G7) at 0 of 15. Over the same
  stretch the operator's requests carried diagnosis, code locations and fixes (O2, O3), so the strategic judgment
  lived in the operator. Intervention 20261014-075229-5x3e records the unlogged detailed instructions.
- Decision: the next milestone is one bounded, externally useful outcome: a published Firstmate explainer slice,
  asked for in a line or two. Athena chooses and revises its own sequence of jobs. The operator logs every rescue,
  including diagnosis handed over, and fixes infrastructure only when it blocks the milestone or is a material
  safety risk (the verifier recording records-less runs as real results, audit finding H2, qualifies).
- Measures: total cost, elapsed time, operator interventions by category, and an independent rating of the
  slice's usefulness. Then repeat on a comparable slice and compare.
- Paused unless the milestone needs them: parallel jobs, decision-sets steps 1 and 3, the Socratic review build,
  the Jev data-quality fixes, the remaining defaults-loop steps.

## OP-14. The operator's job: relay, report, custody, and spot-checking the auditor and warden (2026-10-16)

- Decision (Sri, in chat, 2026-10-16): the operator's job was reworked outside Athena's loop. Grading inside Athena
  is fine when the yardstick is protected, so auditing becomes Athena's auditor and warden roles (BRIEF.md addendum
  "2026-10-16: An auditor and a warden"). Spend: up to $100 per day on Athena runs without asking Sri.
- Duties, none of which count as interventions:
  1. Relay: Sri's words go into BRIEF.md as dated addenda, word for word.
  2. Report: pages for Sri built from Athena's data.
  3. Custody: the lease, the harness version, the budget, the kill switch, and anything only the repository owner
     can change. Each item moves to Athena once Athena can do it.
  4. Spot-check the auditor and warden at each milestone: sample landed jobs against their claims, get an outside
     rating of any published outcome, and check that the protected files and checks held.
- Boundary: start Athena's work with one command, an `ask` or `drain --budget N` (stopped with `drain-stop`). No
  chain scripts, no ordering, no diagnosis in requests. Anything beyond that is a rescue: do it, log it with
  `athena intervene`, and file its gap in the same step. Rescues are the signal for Athena's intervention sensor.
- Retires OP-12 (the operator owns ordering): Athena's planning stage now orders the work.
- Unchanged: no hand edits to Athena's code, push only under the lease, Athena never references the operator, and
  the high bar for reaching Sri (OP-10).
