# Athena's architecture

This page is the map of the code. The design and its reasons are in
[`design/athena-v0.md`](design/athena-v0.md), and every significant choice is in
[`decisions.md`](decisions.md). The landing gate's rule for squash merges of a chain of jobs (D69) is in
[`design/squash-chain-landing.md`](design/squash-chain-landing.md); the gate implements it in
`crates/athena-gate/src/invariant.rs`. How `athena land --pr` picks the host's merge method, writes the squash
message and rebuilds the local branch after a squash (D88) is in
[`design/native-pr-landing.md`](design/native-pr-landing.md) §5.2a; the code is in
`crates/athena-core/src/gh.rs`, `crates/athena-core/src/land.rs` and `crates/athena-git/src/lib.rs`.

## The shape

Every command runs as one short-lived `athena` process: the **core**, deterministic Rust that holds no
model context. It starts three kinds of child process, each with an explicit argument list and an explicit
environment:

- `claude -p` (Claude Code, headless and hermetic), once per agent invocation, in one of three roles:
  - the **reader** turns a request into a reading with forecasts;
  - the **producer** makes the change;
  - the **verifier** judges the result.
- `git`, for snapshots, plumbing commits, fast-forwards, and attributes.
- `gitleaks`, for secret scanning.

Agents work only in **detached snapshots**: directories outside the repository that hold the files the
agent may see, with a throwaway git repository of their own. Only the core writes to the user's repository.
It writes through plumbing, gates every commit in process, and advances the branch by fast-forward only.

```text
athena ask ──▶ reader (plain snapshot of HEAD) ──▶ reading + forecasts, committed (proposal commit)
athena go  ──▶ job record + producer brief, committed (prediction commit)
           ──▶ producer (snapshot of the base, one-commit repository) ──▶ change list
           ──▶ candidate commit built in the user's repository ──▶ admission
           ──▶ checks, then verifier (snapshot of base and candidate, two-commit repository)
           ──▶ final verdict computed by code
           ──▶ pass: landing commit (the change plus every record), gated, fast-forwarded
               fail: closing commit with the findings
```

The core checks a reading before it accepts it, at no model cost
(`athena_core::reading::check_reading_acceptance`, for a first attempt and a retry alike). Among those checks,
`check_governance_coverage` applies admission's definition of a governance file to the reading: a request that
mentions a workflow, CI, deploy, GitHub Pages, routing, checks, classes, budgets or a configuration needs the
matching governance file (`.github/workflows/` or `athena.toml`), and a new configuration section in
`crates/athena-data/src/config.rs` needs `athena.toml`. The reading must list the file as a deliverable and name
it in an acceptance criterion, or name it in `out_of_scope`. Otherwise it is refused and the reader repairs it,
instead of the job failing at admission after the producer has finished.

## Crates and dependency direction

The workspace is split into crates so that Cargo enforces "no cycles". The table below is checked against
`cargo metadata` by `dependency_direction_matches_the_architecture` (xtask): an edge that is not listed
here fails the test. Beyond the table, `athena-core` never depends on a capability crate, no capability
depends on another, and `athena-data` and `athena-git` are leaves.

| Crate | Responsibility | Depends on (Athena crates) |
|---|---|---|
| `athena-data` | Every persisted shape: the twelve record types and their canonical paths, `athena.toml`, `defaults.toml`, `Quantity` with provenance, ids, machine-local paths | none |
| `athena-score` | Proper scoring rules (Brier, the summed deliverable score, log-scale WIS), scoring a job from its records, the reference forecast, aggregates | data |
| `athena-git` | A typed wrapper over the git CLI: detached snapshots, temporary indexes, attributes from committed rules, fingerprints, plumbing commits, merge-tree, fast-forwards | none |
| `athena-gate` | The commit gate: secrets (gitleaks), append-only and valid records, size, personal data, the governed-content invariant, prediction trailers, a job's records judged as a set (scores recomputed with the scorer), reconciliation, the full-history check, redaction | git, data, score |
| `athena-context` | Prompt assets, composition with hashes, the token estimator and budgets, briefs, and the typed outputs of the three roles with generated JSON Schemas | data |
| `athena-agent` | The driver interface, the Claude Code driver, the fake driver, agent environments, stream-json usage capture, process groups, the kill switch, and the tool policy behind the hook | data |
| `athena-core` | The job lifecycle, the `Capability` trait, admission, tool configuration from the base, checks, the final verdict, landing, records commits, scoring, status, nested runs | data, score, git, gate, context, agent |
| `athena-cap-change` | v0's one capability: a focused, independently verified change | core, data, context |
| `athena-cap-study` | The studying capability's reusable core: typed study records, the work-handling model, and the validation rules that gate a study. Registered in the CLI | core, data, context |
| `athena-cli` | The `athena` binary: commands, the interactive prompt, displays, the git and tool hook entry points | core, cap-change, cap-study, data, gate, agent, git |
| `xtask` | Setup and CI tasks, and the repository-policy tests | data, git |

## Where things live

| Location | Contents | Lifetime |
|---|---|---|
| The repository | code, `athena.toml`, `defaults.toml`, `records/`, docs, and the shared vocabulary `docs/ontology.toml` (D109), checked by `crates/athena-cli/tests/ontology.rs` | permanent, public |
| State directory (`<root>/<repo>-<key>/`, where the root is `$ATHENA_STATE_DIR`, by default `${XDG_STATE_HOME:-~/.local/state}/athena`; agents and checks get a root inside their job's own directory) | snapshots, live copies of records not yet committed, running usage of invocations, the process-group registry, locks, `STOP`, per-job build directories | one machine; disposable once a job's records are committed |
| Install directory (`$ATHENA_HOME`, default `${XDG_DATA_HOME:-~/.local/share}/athena/`) | the adopted `athena` binary, the pinned gitleaks, a private Claude Code when the pinned version is not on `PATH`, and `config.toml` naming it | one machine |

## Seams

- **Capabilities.** A kind of work implements `athena_core::capability::Capability` in a crate of its own
  and is registered in `athena_cli::wiring::capabilities`. The core does not change.
- **Drivers.** Agents run through `athena_agent::AgentDriver`. The Claude Code driver is the real one; the
  fake driver runs every test of the lifecycle and every nested run.
- **The gate.** `athena_gate::gate` is the one decision point for commits: the hooks, the core's commit
  path, and `athena check --history` all call it.
- **Planning.** `athena_core::plan` orders and groups plan entries and holds back incomplete ones;
  `athena_core::reading::plan_entry` builds an entry from a request and its reading, filling nothing by
  default (acceptance only from explicit bullets or a reading's criteria; the rung from a proposal's rung or
  from evidence, else flagged). Both are deterministic code and call no model. `athena_cli::drain` runs the
  pass (`drain::plan`, behind `athena plan`, and the start of `athena drain`): it reads unread requests whose
  own words leave the plan incomplete, so the reading can complete it, and refuses what stays incomplete.
  The plan-stage audit in `plan.rs` (`trace`, `GapLog`) is not yet wired to the records; that is follow-up.
- **Records.** `athena_data::records::AnyRecord` parses every record with unknown fields denied, and
  `canonical_path` says where each one lives. [`records.md`](records.md) documents them.
