# Operator handoff

For the operator of the Athena build experiment: the Claude Code session that `BRIEF.md` calls
"you, the operator". If your prompt was `continue`, that is you. Athena never reads this file.

This file carries no project state, by rule. The state of the work comes from Athena.

## Your role

Sri is Athena's user (`BRIEF.md`, addendum 2026-10-17: Olympus). You pass Sri's words to Athena
unchanged, keep custody of what Athena cannot reach from inside the sandbox, and build reports for
Sri. Do not do Athena's work, and do not re-word Sri's intent. Your duties (relay, report, custody,
spot-checking the auditor and warden, a daily spend limit) are in `operator/decisions.md` OP-14 and
OP-16. Start work with one command (`ask` or
`drain --budget N`, halted with `drain-stop`); anything more is a rescue: log it and file its gap in
the same step.

## Where things live

- The spec: `BRIEF.md`. Read it at the start of every session.
- The intervention log: Athena's, in `records/interventions/` (one JSON record per intervention).
  Record each intervention as it happens with `athena intervene "<what>" --why "<why>" --class
  <class of work> --category <category>`. Athena records a one-line correction of its reading
  automatically.
- Your mistakes (Sri, 2026-10-23; OP-21): log each one as it happens with `athena intervene "Mistake: <what
  went wrong and what it cost>" --why "<what Athena would have had to know or check to prevent it>"
  --class <class> --category other`, so it becomes work for Athena.
- Sandbox facts and open blockers: `operator/preflight.md`.
- Your own decisions about operating: `operator/decisions.md`.
- Everything else, including what to do next: ask Athena.

## Starting a session

1. Lease: build the tool and look at the lease of record:
   `cargo build --release --manifest-path operator/opctl/Cargo.toml`, then
   `operator/opctl/target/release/opctl lease status`.
   If another session holds a live lease, do not push or merge; retry later. If its holder is
   archived or idle (check it with `get_session`), it is gone: take over and log that.
   Otherwise `opctl lease acquire`, `opctl install-hook`, commit the lease, and land it on `main`.
2. Environment: run Athena's one-command setup from clean (`./setup.sh`), and re-check the open
   blockers in `operator/preflight.md`. A hand fix to the environment is an intervention.
3. Read `BRIEF.md`. Then get the state of the work from Athena: `athena status`.
4. Log what the restart cost to get productive again, and anything lost:
   `athena restart --cost "<what it cost>" --lost "<what was lost>"`.

## Restarting

- When: right after v0 is accepted, at least once per milestone, and whenever your efficiency
  drops (a crowded context, re-reading the same material, contradicting earlier decisions).
- How:
  1. Land all work on `main`.
  2. `opctl lease release`, commit it, and land it on `main`.
  3. Start the next cloud session in the same environment with the prompt `continue` and nothing
     more.
  4. Stop: no further pushes or merges from this session.
- If the new session fails to start, record the blocker in `operator/preflight.md`, keep working,
  and retry. Never skip a required restart silently.

## Lease

`operator/LEASE.toml` on `main` is the lease of record: which session operates, since when, and
until when. `opctl install-hook` links `.git/hooks/pre-push` to `opctl`, so a push from a session
that does not hold a live lease is refused. Renew with `opctl lease renew` when you land work on
`main`; leases expire after 4 hours by default.
