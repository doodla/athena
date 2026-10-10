# Athena v0: independent acceptance verification 2 (re-check)

**Verdict: accept with fixes.** The two majors of [verification 1](athena-v0-acceptance-1.md) are resolved.
Round 1's forged landing, forged re-scores, and forged outcomes are now refused by the hooks and by the
full-history check. The residual the builder states, that a job fabricated record by record still passes
because records are unsigned, is honestly stated and acceptable for v0, with two precision notes below. Every
finding of verification 1 is either fixed, queued as open work that `athena status` shows, or closed by a
recorded decision; none was dropped silently. One new major remains, and its fix is small: an entry of one
character in `[history] exempt` hides later hand changes from the history check.

| | |
|---|---|
| Under test | Branch `claude/happy-goodall-j46p5t` at `11dcd2b6e3868e8898d711c93aab5ed4aa48b896`, which contains the fix round `00087eb`..`e09483a`, cloned fresh from GitHub |
| Scope | Three items, set by the user: M1, M2, and whether every open finding is visible. Anything new of major severity or above is reported too. |
| Verifier | The author of verification 1: an agent with no part in Athena's design or build. It judged by exercising Athena in throwaway clones; it read code only to aim probes. |
| Spend | $0.17 of this round's $1.00 cap (see [Spend](#spend)) |
| Date | 2026-10-10 |

## Status of the three items

| Item | Status | Evidence |
|---|---|---|
| **M1.** A hand change passed every gate as a verified landing | **Resolved**, with an acceptable residual | I closed the bootstrap range in a throwaway clone, then re-ran round 1's probe: a hand edit, a copied `verification.json`, and two trailers. `commit-msg` refused it: "job.json is not on the parent commit, so no dispatched job lands here". Committed with `--no-verify`, the same change was flagged by `athena check --history`. A landing now has to carry its whole job. The landing commit must name, in `Athena-Prediction`, the ancestor that added the accepted reading, and the job's `job.json` must be on its parent. The commit itself must add the candidate, the verification (both holding the landed fingerprint), a successful verifier invocation of the job's verifier model that reconciles, and a landed outcome. A real job confirms that a genuine landing still passes the new rule, and that its intervention re-score passes the recomputation. |
| **M2.** Hand-written records could rewrite a score or an outcome | **Resolved**, with the same residual | Each of these was refused by the hooks: a re-score that names an intervention which does not exist; a re-score with tampered numbers, even with a hand-written intervention for the job in the same commit ("does not follow from job …'s records"); `outcome-landed.json` for a rejected job ("follows a blocked outcome, but … is not blocked"); a landed `outcome.json` with no landing; and a `score.json` for a queued request. |
| **Open work is visible** | **Pass** | All 17 findings of verification 1 are accounted for from the repository alone (table below). Eight are fixed: I re-checked six of those fixes by exercising them, and read the other two. Eight are queued requests that `athena status` and `athena status --json` list as open work, each citing its finding and verification 1. One, n7, was closed by the user's decision, and the reason is recorded in its outcome. Half of n5 was fixed and its other half requeued, with the reason recorded. |

### The residual, and two precision notes

The design (§16.2) and `docs/enforcement.md` now say that records are not signed: "a deliberate fabrication of
every record of a job, across two commits and with usage that reconciles, still passes". I confirmed this. In
the throwaway clone I wrote a request, reading, reader invocation, and `job.json` in one records-only commit.
A second commit added the hand change, with a candidate, a verification, a verifier invocation copied from a
real job, and a landed outcome. Both commits passed the hooks and `athena check --history`, and `athena status`
listed the forged job as landed. I judge the residual acceptable for v0. Defeating it needs deliberate
fabrication of several consistent records, which no honest workflow produces. The forged job is visible in
`status` as a job nobody asked for. And no local mechanism can tell such records from Athena's own, because
Athena writes them with the user's own rights. The statement understates the residual in two ways:

- **One forged commit is enough for a job that was dispatched but never closed.** I asked a real request (one
  reader call), let Athena dispatch it, and stopped `athena go` while its stand-in producer waited. That left
  the job interrupted, which is what any crash leaves. A single forged commit then landed it, with a hand
  change and four fabricated closing records. The hooks and the history check accepted it. This is not "every
  record" nor "two commits": the request, reading, and dispatch were Athena's own. A user who wants to
  forge can make such a job with one request, by killing `athena go` while it runs (I used `kill -9`).
- **The residual also reaches calibration, without any landing.** One hand-made proposal commit holding a
  reading with a confident forecast, closed by Athena's own `athena close`, moved the `docs` class headline
  from reader 0.061 / reference 0.076 to reader 0.046 / reference 0.174, and the history check stayed clean.
  The statement speaks only of landings.

Smaller: a landed outcome with `"totals": null` skips the outcome's reconciliation (the gate's
`check_outcome` returns early). Athena itself always writes totals for a request job.

*Suggested, not required for v0:* say all three in the prose-only entry, and require `totals` on any outcome
of a request job.

### The 17 findings of verification 1

| Finding | Disposition | Checked here |
|---|---|---|
| M1, M2 | fixed (D55, D56) | yes: the probes above |
| m1 verifier escape | recorded as an intervention; the verifier now checks documented instructions against the repository's own rules (D59) | read the prompt change |
| m2 test counts | fixed: one shared output file; earlier records listed as inaccurate in `docs/records.md` | yes: a real job's check recorded 206 tests, and `cargo test` runs 206 |
| m3 boundary scan | fixed: all four subjects | yes: the planted sentence from round 1 now fails the test |
| m4 agents used the user's Athena state | fixed (D57) | yes: a running agent's `ATHENA_STATE_DIR` is inside its job's directory; setup refuses under the nested-run marker; no Claude Code folder was left under the user's home |
| m5 `CARGO_TARGET_DIR` | fixed (D57) | yes: setup on a fresh clone with `CARGO_TARGET_DIR` set now succeeds |
| n1 doc statements | corrected | yes: README and `docs/architecture.md` |
| n5, first half | fixed; its request dropped with the reason | — |
| m6, m7, m8, n2, n3, n4, n5 second half, n6 | queued requests, open work in `athena status` | yes: all eight listed, with their next commands |
| n7 | queued, then closed by the user's decision, with the reason in the job's outcome | yes: `athena status <id>` shows it |

---

## New finding

**N1 (major). A one-character entry in `[history] exempt` hides later hand changes from the history check.**
The invariant skips any commit whose id starts with any entry of `athena.toml`'s `[history] exempt`
(`invariant::check`). Entries are not validated, so a short entry matches future commits as well as the one
commit that predates Athena. In the throwaway clone, with the bootstrap range closed:

1. A declared intervention added `"0"` to the list (`athena intervene … --with-staged`). This is visible once,
   in the intervention log and the diff.
2. A hand edit of `README.md`, committed normally, was refused by `commit-msg`, because a hook sees no commit
   id yet.
3. The same edit, committed with `--no-verify` and retried until its id began with `0` (11 tries), passed
   `athena check --history` with 0 violations.

Every later hand change can repeat step 3 for the price of a few retries, and nothing records it: no
intervention, no job, and no line in `athena status`. The hooks remain a backstop only on machines that run
them, and CI's history check, the stated backstop for skipped hooks, is blinded.
*Fix:* accept only full 40-character ids in `[history] exempt`, each naming a commit that is an ancestor of the
commit that adds the entry, so an exemption can only name the past. Refuse anything else as a configuration
error, and add the probe as a test of what M6 blocks.

---

## Regression check

| Check | Result |
|---|---|
| `cargo test` at the head | 206 passed, 0 failed, 4 ignored (the three live tests and one helper) |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `athena check --history` | 0 violations at the head (69 commits); 0 after the real job and its re-score (73 commits) |
| A real job end to end under the new rules | "add one sentence to docs/records.md saying that record ids sort by time": reading, dispatch, Haiku producer, checks (206 tests), Opus verifier pass, landed. An `athena intervene --job` re-score was accepted by the recomputation. Every recorded figure equals Claude Code's own result events. |

## Spend

| Run | Invocations | Cost (USD) |
|---|---|---:|
| The interrupted-job probe | 1 reader (Sonnet) | 0.0424 |
| The real regression job | reader (Sonnet), producer (Haiku), verifier (Opus) | 0.1298 |
| Every other probe | none: they made no model call, and stand-ins for Claude Code refused or waited | 0.0000 |
| **Total** | | **0.1722** |
