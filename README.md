# Athena

Athena is an agent orchestration system written in Rust. You give it a lazy, one- or two-line request.
It shows how it read the request and what it predicts, before any work starts. It runs the job on a
sub-agent in an isolated snapshot, has a different model verify the result independently, and lands a
commit you can trace back to the request, the prediction, the models, the cost, and the verdict. It records
every model call, scores its own predictions with proper scoring rules, and gates every commit with a
secret scan.

This is v0, the walking skeleton. Its design is [`docs/design/athena-v0.md`](docs/design/athena-v0.md).

## Quick start

```sh
./setup.sh                        # prerequisites, Rust, gitleaks, athena, git hooks, Claude Code, credentials
athena ask "add a changelog"      # shows the reading and predictions; Enter runs it
```

`./setup.sh` is idempotent and pinned: it reads every version from `athena.toml` and `rust-toolchain.toml`
(rustup-init, which runs before any Rust can, is pinned in `setup.sh` itself), checks downloads against pinned
SHA-256 sums, and reads credentials only to report whether Claude Code can
authenticate. It never stores them. `athena.toml` currently pins no `[tools.sccache]` (D84), so setup skips
that step; if a pin is added, setup builds it with `cargo install --locked` into the install directory (no
digest; this can take minutes). It needs git 2.40 or later, a C linker, curl, and either Claude Code
2.1.296 or npm to install it. `./setup.sh --ci` also skips any pinned sccache (CI does not use it) and stops
after building Athena.

`athena --version` prints the package version and the commit the binary was built from, for example
`athena 0.1.0 (commit 0123abc…, dirty)`. A build that names no commit shows `unknown`, and any build that
is not a clean commit shows `dirty`.

## What happens when you ask

```text
athena ask "<request>"
  reader (Sonnet, read-only snapshot)  → a reading: summary, criteria, deliverables, forecasts
                                         committed before any work (you can correct it in one line)
Enter, or athena go <job>
  prediction commit                    → the accepted reading's job record and the producer's brief
  producer (Haiku or Sonnet)           → edits in a detached snapshot with its own throwaway repository
  candidate                            → built from the change list; admitted or refused
  checks                               → fmt, clippy, tests, run by Athena under the base's configuration
  verifier (Opus, never a producer)    → judges the exact candidate; never sees the producer's report
  landing                              → the change plus every record, one commit, fast-forwarded
```

To retry a rejected or failed job, ask `athena ask "try again on <job>"` (the full id or a unique suffix; the form is
matched case-insensitively at the start of the request). The reader then sees that job's request and its final
verification's findings, redacted, in an "Earlier attempt" section. That section also shows the job's final
reading (summary, acceptance criteria, deliverables) and the corrections and answers the user gave, and it says the
attempt's changes were discarded, so the retry starts from the base. A failed job (for example one refused at
admission, which has no verification) also has its recorded reason carried there. Jobs that are landed, stale,
in flight, or unknown are refused.

Any request that names a job by its full id (like `20261012-075824-1x28`) anywhere in its text also shows the
reader that job's request and its final verification's findings, redacted, in a "Referenced jobs" section, whatever
the job's state. At most three jobs are carried; an id that matches no job is ignored. Unique suffixes are resolved
only in the "try again on" form.

Exit codes: 0 when the job landed, 2 when it did not (rejected, failed, stale, or blocked), 1 when Athena
itself erred.

| Command | What it does |
|---|---|
| `athena ask "<request>" [--yes \| --later]` | Read a request; on a terminal, Enter runs it, a typed line corrects it, `q` leaves it proposed. `--later` only queues the request, with no model call |
| `athena correct <job> "<line>"` | Correct the latest reading, or answer its question, and read again |
| `athena go <job>` | Run a proposed job, read a queued one, or retry the landing of a blocked one |
| `athena close <job> --why "..."` | Close a job that will not run or finish |
| `athena status [<job>] [--json]` | The state of the work, or one job's full trace |
| `athena intervene "<what>" --why ... --class ... --category ... [--job <job>] [--with-staged]` | Record something you did beyond what a lazy user would; `--with-staged` commits your staged hand changes with it |
| `athena restart --cost "..." [--minutes N] [--tokens N] [--measured] [--lost ...] [--via ...]` | Record a restart of your own context |
| `athena check [--history]` | Check records, reconciliation, and the commit gate, now or over the whole history |
| `athena stop` / `athena start` | The kill switch: terminate every running agent and check, and block new ones |

**Merge strategy.** Pull requests may be landed on your main branch by merge commit, squash merge, or
rebase merge; `athena check --history` accepts all three. It identifies a landing from the job records the
commit adds, and proves the prediction came before the work from those records, so it does not depend on
`Athena-*` trailers or on commit ancestry.

Wherever a command takes `<job>`, it accepts a full id, a unique suffix, or `latest`. Every command except
`stop` and `start` takes `--json`, so an agent can drive Athena through a shell as well as a person can.

## Accepting v0 from a clean clone

```sh
git clone <repository> athena && cd athena
./setup.sh
athena ask "<a small, useful one-line request>"    # read the reading and predictions, press Enter
git log -1 --format=%B                              # Athena-Job, Athena-Prediction, models, cost, verified content
athena status latest                                # request → prediction → owner → model → cost → verification → outcome → score
athena intervene "..." --why "..." --class docs --category other   # an intervention record, committed
athena restart --cost "..." --lost "..."                            # a restart record, committed
athena status                                       # the state of the work, from the repository alone
cargo test && cargo clippy --all-targets -- -D warnings
athena check --history                              # what CI runs over the full history
```

The tests that call real models are opt-in: `cargo test -p athena-cli --test live -- --ignored
--test-threads=1` (a few cents with Haiku). The last step of accepting v0 is closing the bootstrap range:
see "Closing the bootstrap" in [`docs/records.md`](docs/records.md).

## Picking up the work

Nothing about the state of the work lives in a hand-maintained file. A new context needs this page, then
`athena status` (or `athena status --json`), then the records themselves: plain JSON under `records/`,
documented in [`docs/records.md`](docs/records.md). Open work, including requests queued for later with
`athena ask --later`, is listed under `open`, each with the command that picks it up.

## Map

| Path | What it is |
|---|---|
| `crates/` | the Rust workspace; [`docs/architecture.md`](docs/architecture.md) maps the crates and their dependency direction |
| `xtask/` | setup, CI, and the repository-policy tests (`cargo run -p xtask -- setup`, `cargo run -p xtask -- ci`) |
| [`CHANGELOG.md`](CHANGELOG.md) | what changed in each version, in Keep a Changelog form |
| `athena.toml` | configuration: models, budgets, agent environment, the `change` capability's checks and allow-lists, classes, priors, pins |
| `defaults.toml` | the model of the user's default expectations |
| `records/` | every fact Athena has learned, as immutable JSON |
| [`docs/design/athena-v0.md`](docs/design/athena-v0.md) | the design, with the evidence behind it and the reviews that shaped it |
| [`docs/decisions.md`](docs/decisions.md) | every significant decision, with alternatives, tradeoffs, and provenance |
| [`docs/enforcement.md`](docs/enforcement.md) | every rule, its mechanism, and the tests that show what it blocks and allows |
| [`docs/non-rust-files.md`](docs/non-rust-files.md) | why each non-Rust file exists |
| [`docs/design/requirements-map.md`](docs/design/requirements-map.md) | every `BRIEF.md` requirement, marked covered, partial or not covered, with where it is answered; its [strategy view](docs/design/strategy-view.md) groups the rows by goal and its [roadmap](docs/design/roadmap.md) orders the next steps |

## Attribution

Athena's design adapts ideas from Ryan Lopopolo, *Harness Engineering*,
<https://github.com/lopopolo/harness-engineering>, licensed under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), read at commit
`226c8d35fb6ea3ed55467753dba6dea2b5fd5778`. Changes were made: Athena restates and adapts the bundle's
arguments for its own design and quotes only short phrases, with attribution. It reuses no third-party
material from the bundle's `sources/`. Where v0 follows the bundle and where it departs is in §15 of the
design.

## License

MIT; see [`LICENSE`](LICENSE).
