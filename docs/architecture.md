# Athena's architecture

This page is the map of the code. The design and its reasons are in
[`design/athena-v0.md`](design/athena-v0.md), and every significant choice is in
[`decisions.md`](decisions.md).

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
| `athena-cli` | The `athena` binary: commands, the interactive prompt, displays, the git and tool hook entry points | core, cap-change, data, gate, agent, git |
| `xtask` | Setup and CI tasks, and the repository-policy tests | data |

## Where things live

| Location | Contents | Lifetime |
|---|---|---|
| The repository | code, `athena.toml`, `defaults.toml`, `records/`, docs | permanent, public |
| State directory (`$ATHENA_STATE_DIR`, default `${XDG_STATE_HOME:-~/.local/state}/athena/<repo>-<key>/`) | snapshots, live copies of records not yet committed, running usage of invocations, the process-group registry, locks, `STOP`, per-job build directories | one machine; disposable once a job's records are committed |
| Install directory (`$ATHENA_HOME`, default `${XDG_DATA_HOME:-~/.local/share}/athena/`) | the adopted `athena` binary, the pinned gitleaks, a private Claude Code when the pinned version is not on `PATH`, and `config.toml` naming it | one machine |

## Seams

- **Capabilities.** A kind of work implements `athena_core::capability::Capability` in a crate of its own
  and is registered in `athena_cli::wiring::capabilities`. The core does not change.
- **Drivers.** Agents run through `athena_agent::AgentDriver`. The Claude Code driver is the real one; the
  fake driver runs every test of the lifecycle and every nested run.
- **The gate.** `athena_gate::gate` is the one decision point for commits: the hooks, the core's commit
  path, and `athena check --history` all call it.
- **Records.** `athena_data::records::AnyRecord` parses every record with unknown fields denied, and
  `canonical_path` says where each one lives. [`records.md`](records.md) documents them.
