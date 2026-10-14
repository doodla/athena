# Athena's records

Everything Athena learns is a JSON file under `records/`, written once and never changed. The records are
plain JSON, so they can be read without the `athena` binary; `athena status` and `athena status <job>`
are views over them. The Rust types in `crates/athena-data/src/records.rs` are the authority: every record
parses into exactly one of them, with unknown fields refused, and must sit at the one path its content
names.

## Principles

1. **One fact, one file, written once.** New knowledge about an old fact is a new record that references
   it: a re-score, a later outcome, or a resolution. The commit gate refuses any change to, deletion of, or
   rename of a committed record.
2. **Per-job directories, sharded by month.** A job writes only under `records/jobs/<YYYY-MM>/<job id>/`.
3. **State is derived.** No record holds a mutable status; a job's state follows from which records exist:
   only `request.json` is a queued request (`athena ask --later`); a reading makes it proposed; `job.json`
   dispatched; an outcome closed, or blocked.
4. **Every number carries its provenance.** Tokens, time, and money are quantities:
   `{"value", "unit", "provenance", "basis"}` with optional `"lower"` and `"upper"`; `unit` is `usd`,
   `tokens`, `seconds`, or `minutes`. `provenance` is
   `measured` (counted or timed), `reported` (computed by a tool, such as Claude Code's cost at list price),
   `estimated` (a judgment or a bound), or `pending` (not known yet; `value` is null).
5. **Committed before the process exits.** Every command that creates records commits them before it
   exits; a job that dies mid-run is closed with `athena close`, which commits what it left.

## Layout

```text
records/
  jobs/<YYYY-MM>/<job id>/
    request.json                 athena.request/1       the request, verbatim (redacted)
    reading-<n>.json             athena.reading/1       a reading with both forecasters' forecasts
    correction-<n>.json          athena.correction/1    a one-line correction of reading n, or an answer
    job.json                     athena.job/1           the dispatched job: accepted reading, owner, models, caps
    brief-producer.md            the producer brief, verbatim
    invocations/<seq>-<role>.json athena.invocation/1   one model invocation
    candidate.json               athena.candidate/2     the change list, fingerprint, admission, producer report, chain links (/1 for older jobs)
    brief-verifier.md            the verifier brief, with the diff elided
    verification.json            athena.verification/1  checks, criteria, findings, the final verdict
    outcome.json                 athena.outcome/1       where the job ended, and its totals
    outcome-landed.json          athena.outcome/1       a blocked job that landed on retry
    outcome-abandoned.json       athena.outcome/1       a blocked job closed instead
    score.json                   athena.score/1         every forecast scored
    score-<intervention id>.json athena.score/1         a re-score after a late intervention
  jobs/bootstrap/                the one job done outside Athena: brief.md, brief-update-<n>.md, job.json
  interventions/<YYYY-MM>/<id>.json  athena.intervention/1
  restarts/<YYYY-MM>/<id>.json       athena.restart/1
  resolutions/<YYYY-MM>/<id>.json    athena.resolution/1
  host-errors/<YYYY-MM>/<id>.json    athena.host-error/1
  decision-states/<YYYY-MM>/<id>.json  athena.decision-state/1
  default-expectations/<YYYY-MM>/<id>.json  athena.default-expectations/1
  defaults-proposals/<YYYY-MM>/<record>.json  athena.defaults-proposal/1
```

Ids are `YYYYMMDD-HHMMSS-xxxx`: UTC time, then four Crockford base32 characters. They sort by time and give
their month shard. `bootstrap` is reserved.

## The record types

Every record has a `schema` field naming its type and version.

| Schema | What it holds |
|---|---|
| `athena.request/1` | `job`, `text` (verbatim, with private addresses and secrets redacted), `received_at`, `channel` |
| `athena.reading/1` | `n`; the reader's `invocation_seq`; the `base_commit` and the hashes of `athena.toml` and `defaults.toml` it was read under; `title`, `summary`, `class_of_work`, `capability` or `decline_reason`, `producer_model`, `verifier_model`, `defaults_applied`, `assumptions`, `question`, `acceptance_criteria` (each `content` or `behavior`), `deliverables` (path, action, probability), `out_of_scope`; `forecasts` by the `reader` and the `reference`; `departures` |
| `athena.correction/1` | `n` (the reading it corrects), `kind` (`correction` or `answer`), `text`, and the `intervention` recorded for it |
| `athena.job/1` | `accepted_reading`, `accepted_by` (`user`, or `flag` for `--yes`), `owner` (role, capability, model), `verifier_model`, `caps`, `base_commit`, configuration hashes, the `athena` build and `driver` that ran it, `brief_sha256`, `dispatched_at`; for the bootstrap, `request_path` and its pending `cost` and `tokens` |
| `athena.invocation/1` | `seq`, `role`, `driver`, `athena` build, `model`; `context` (components with ids, versions, hashes, and estimated tokens, plus the host's baseline); `caps`; `started_at`, `ended_at`, `wall_time`; `tokens` (input, output, cache read, cache write); `models_used` (per model, with cost); `cost`; `outcome` (`success`, `error`, `killed`, `missing`); `turns`; `commands_run` and `policy_denials` (redacted) |
| `athena.candidate/1` | `base_commit` (the prediction commit), `candidate_commit` and `candidate_tree`, `changes` (path, status, mode, bytes), `fingerprint`, `violations` from admission, and the redacted `producer_report`. Records written before chain links existed stay `/1` and are still read |
| `athena.candidate/2` | everything in `/1`, plus two fields that link the job to the one before it. `base_fingerprint` (required): the content fingerprint of the base commit's tree, computed under the base commit's own classification rules. `previous_job` (optional, absent rather than null): the id of the latest job that had landed in the base commit's history, found by the core and never supplied by the user; absent when no job had landed. Hand changes and records-only commits add no landed outcome, so they do not break the link. A `/1` record may not carry either field. The core writes `/2` for every new candidate. The gate reads the two fields only when one commit lands several jobs (a squashed chain): every such job needs `/2`, and `previous_job` gives the order |
| `athena.verification/1` | `verified_fingerprint`; `tool_config_from_base`; `checks` (command, exit code, tests run, duration, a redacted output tail of at most 4 KiB); `check_cleanup`; the `verifier_invocation`; `criteria` with evidence, cited commands, and whether every cited command was seen running; `findings` by severity and source; `request_reading_mismatch`; `agent_verdict`; `final_verdict`; `reasons`; the `inputs` the verifier was given, by name |
| `athena.outcome/1` | `state` (`landed`, `blocked`, `rejected`, `failed`, `stale`, `dropped`, `declined`, `abandoned`, or `closed` for the bootstrap), `reason`, `at`, `totals` (all-in cost, execution cost, execution wall time, tokens by model, invocation count; required for every job but the bootstrap), the branch, base, and candidate; `after_blocked`; for the bootstrap, `through_commit` |
| `athena.score/1` | the `rules` and their version; per event (`reading_accepted` for every reading, `verification_passes`, `user_intervenes`), each forecaster's probability, the outcome, each Brier score, and whether it is provisional; `deliverables` (the scoring set, each term, the sum); per quantity (`cost_usd`, `wall_time_s`), each forecaster's quantiles, the actual, the WIS, and whether it fell inside the 80% interval; the unscored `models` check; `cause` for a re-score. A score is derived data: the gate recomputes every score a commit adds from the job's records in that commit and refuses one that differs, and a re-score must come in the commit that adds its intervention (D56) |
| `athena.intervention/1` | `what`, `why`, `class_of_work`, `category`, `job`, `source` (`user` or `observed`), `with_staged_changes` |
| `athena.restart/1` | `cost` (free text), `minutes` and `tokens` (estimated unless measured), `lost`, `via` |
| `athena.resolution/1` | `target_path`, `target_field`, `value`, `note`: supplies a pending quantity |
| `athena.host-error/1` | A call to the code host was refused. `id`, `at`, `status_code` (an integer from 100 to 599, or absent when the call failed before an HTTP answer), `operation` (`merge`, `create-pr`, `read-repo`, `read-pr`, `read-checks`), `job` (the job the call was made for), `phase` (`reading`, `producing`, `verifying`, `landing`), `message_class` (`server-error`, `method-not-allowed`, `unprocessable`, `not-found`, `not-authorized`, `rate-limited`, `other-status`, `no-answer`; it follows from `status_code` and the gate refuses one that does not). The client's own text is **not** a field and is never stored. The host client writes one for every refused call, including each repeat of a transient failure (500, 502, 503, 504, repeated up to three calls in all); `athena land --pr` commits them under an `Athena-Record` trailer |
| `athena.decision-state/1` | Append-only: one change of a decision's state. `id`, `at`, `decision` (such as `D87`), `state` (`under-review`, or `cleared`, which nothing writes yet), `signal` (the name of the signal that fired), `firing_records` (the ids that fired it; required for `under-review`), `decision_job` (for `cleared`). The current state is the fold of all of them: the latest by `(at, id)` per decision. The watcher writes `under-review` when a signal fires; `docs/decisions.md` is never edited |
| `athena.default-expectations/1` | What a default in `defaults.toml` assumes and how to tell it failed, keyed by the default's id (`default_id`), so `defaults.toml` itself stays as it is (D26). `id`, `at`, `assumptions`, `signals` (each with `source`, `strength` `hard` or `soft`, `text`, `threshold_jobs`: at least 1 for hard and at least 2 for soft), `defaults_version` (the `defaults.toml` version it was written against), `proposal` (the defaults proposal it came from, if any). A newer record for the same default replaces an older one for readers; neither is edited. Only the type, a writer and a reader exist; no migration of the 12 defaults has been done |
| `athena.class-label/1` | A reviewer's failure class for one introspection. `id`, `at`, `introspection`, `class` (a registry class id, or `null` for "none"), `labeled_by` (`person` or `stronger-model`), `is_repeat` (optional), `note` (optional one line; absent when not given). `athena review` writes one for a person, in the same commit as an `athena.failure-class/1` record when the class is new, and never edits the introspection. The newest label per introspection (by `at`, then `id`) counts; an introspection with none is unreviewed, and `athena status` counts those |
| `athena.defaults-proposal/1` | Append-only: a proposed change to `defaults.toml`, which is a record and never an edit. `record` (this record's id, which names the file), `at`, `id` (the default's id), `kind` (`add` or `amend`), `statement`, `applies_to`, `version` (`from`, and `to` = `from + 1`), `sources` (each an `introspection` id with its `intervention`, `job` and `at`; at least three distinct jobs when code writes it), `failure_class`, `status` (`pending`, `accepted` with the `job` that applies it, `rejected` with a `reason`, or `withdrawn`), `candidate_sha256` (the hash of the full candidate `defaults.toml` that passed `DefaultsModel::parse`), `notes`. A status change is a new record with the same `id`; the latest by `(at, record)` is current. Assumptions, signals and evidence probabilities need Jev and are not fields yet |

An abridged invocation record:

```json
{
  "schema": "athena.invocation/1",
  "job": "20261010-153012-k7q2",
  "seq": 2,
  "role": "producer",
  "driver": { "name": "claude-code", "version": "2.1.296" },
  "model": "claude-sonnet-5-5",
  "wall_time": { "value": 192.4, "unit": "seconds", "provenance": "measured", "basis": "athena monotonic clock" },
  "tokens": {
    "output": { "value": 5120, "unit": "tokens", "provenance": "measured", "basis": "Claude Code result.usage" }
  },
  "cost": { "value": 0.41, "unit": "usd", "provenance": "reported", "basis": "Claude Code total_cost_usd, at list price" },
  "outcome": "success"
}
```

A killed invocation records bounds instead: tokens `estimated` with the basis "lower bound: usage before
termination", and a cost with `value` null, `lower` 0, and `upper` the invocation's cap.

## Following a job from a commit

Every commit Athena makes carries an `Athena-Job` trailer. A landing commit also carries:

```text
Athena-Job: <job id>                       → records/jobs/<YYYY-MM>/<job id>/
Athena-Prediction: <commit>                the commit that added the accepted reading and its forecasts
Athena-Class: docs
Athena-Capability: change
Athena-Producer: <model>
Athena-Verifier: <model> pass
Athena-Cost-USD: <all-in cost> reported
Athena-Verified-Content: sha256:…          recomputable: the fingerprint of the landed tree
Co-Authored-By: <producer's name> <noreply@anthropic.com>
```

The trace runs: `request.json` → `reading-<n>.json` (the prediction) → `job.json` (the owner and models)
→ `invocations/` (tokens, cost, time) → `verification.json` → `outcome.json` → `score.json`.

A commit that changes governed content passes the gate as a landing only with its whole job behind it: the
job's `job.json` and its accepted reading in the landed tree, the reading recorded before the dispatch and
the producer and verifier invocations, and, in the commit itself, the candidate, the verification, a
successful verifier invocation that reconciles, and the landed outcome (D55). The gate reads these records,
not the trailers, so a merge, squash, or rebase merge lands the same way. A squash may carry the whole jobs of
a chain: the gate then also requires that their candidates link, from the parent's content to the landed
content (D69).
`git log --grep "Athena-Job: <id>"` lists a job's commits; `athena status <id>` shows the same trace.

## Resolving a pending quantity

The bootstrap's own token cost and dollar cost are `pending` in `records/jobs/bootstrap/job.json`. To
resolve one, add `records/resolutions/<YYYY-MM>/<id>.json`:

```json
{
  "schema": "athena.resolution/1",
  "id": "20261011-090000-abcd",
  "at": "2026-10-11T09:00:00Z",
  "target_path": "records/jobs/bootstrap/job.json",
  "target_field": "tokens",
  "value": { "value": 12345678, "unit": "tokens", "provenance": "measured", "basis": "how it was measured" },
  "note": null
}
```

and commit it with the trailer `Athena-Record: records/resolutions/<YYYY-MM>/<id>.json`. `athena check`
verifies that the target exists, that the field is pending, and that only one resolution targets it.
`athena status` lists pending quantities and the resolutions that settle them.

## Closing the bootstrap

Until `records/jobs/bootstrap/outcome.json` exists, a commit labeled `Athena-Job: bootstrap` may change
governed content without verification, and `athena status` says the range is open. The bootstrap closes it
as the last step of accepting v0, with a records-only commit that adds:

```json
{
  "schema": "athena.outcome/1",
  "job": "bootstrap",
  "state": "closed",
  "reason": "v0 accepted",
  "at": "<time>",
  "totals": null,
  "branch": "<branch>",
  "base_commit": null,
  "candidate_commit": null,
  "after_blocked": false,
  "through_commit": "<the last bootstrap commit>"
}
```

From then on, governed content changes only through verified landings and declared interventions, and a
later commit labeled `Athena-Job: bootstrap` that changes governed content is refused.

## Known inaccuracies in earlier records

Records are never edited, so an error in one is corrected here, not in the file.

- `tests_ran` in the `test` checks of jobs `20261010-131714-adcd`, `20261010-132713-ezm8`, and
  `20261010-135737-y9gg` undercounts the tests that ran, and their `output_tail` may have lost lines. The
  check's standard output and standard error were written through two separate opens of one file, which
  overwrote each other. Checks recorded from the fix round after acceptance verification 1 on share one open
  file.

## What is never recorded

No transcripts. Claude Code runs with `--no-session-persistence`, and the driver keeps only usage figures,
tool names, Bash command lines, denials, and structured outputs. Agent free text, commands, and check output
are redacted before they are recorded: private email addresses become `[email]`, gitleaks findings become
`[secret:<rule>]`, and this machine's paths become placeholders (`<job>`, `<state>`, and `~` for the home
directory).
