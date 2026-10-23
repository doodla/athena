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
2.1.299 or npm to install it. `./setup.sh --ci` also skips any pinned sccache (CI does not use it) and stops
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

`athena retry <job>` is the short form: it does the same, and when the job has an introspection record it also shows
the reader that record's cause, prevention and proposed next request, and the earlier cases of the same class with
the preventions already tried, so a repeat is seen as a repeat. It takes `--yes` and `--json` like `ask`, and runs
nothing until the reading is accepted.

Any request that names a job by its full id (like `20261012-075824-1x28`) anywhere in its text also shows the
reader that job's request and its final verification's findings, redacted, in a "Referenced jobs" section, whatever
the job's state. At most three jobs are carried; an id that matches no job is ignored. Unique suffixes are resolved
only in the "try again on" form.

Exit codes: 0 when the job landed, 2 when it did not (rejected, failed, stale, or blocked), 1 when Athena
itself erred.

| Command | What it does |
|---|---|
| `athena ask "<request>" [--yes \| --later]` | Read a request; on a terminal, Enter runs it, a typed line corrects it, `q` leaves it proposed. `--later` only queues the request, with no model call |
| `athena retry <job> [--yes]` | Try a rejected or failed job again from its introspection record (proposed next request, cause, prevention, earlier cases of the same class); shown as `ask` shows a reading. Without an introspection record it is `athena ask "try again on <job>"` |
| `athena propose` | Write a pending defaults proposal for each failure class whose introspections span at least 3 distinct jobs. Proposals are records; `athena status` lists the open ones with their sources, and `defaults.toml` is never edited by Athena |
| `athena score-jev [--cut <time>] [--json]` | Read-only. Score Jev's recorded shadow answers against introspections a reviewer labeled (`athena.class-label/1` records): Brier skill over two references, sample size, interval or "no bound computed", and the promotion verdict for the items it can compute. With too few labeled cases it says so and gives no verdict |
| `athena review <introspection-id> (--right \| --class <class-id> \| --new-class <name> --description "...") [--note "..."] [--json]` | Show an introspection (what happened, class, cause, prevention, Jev's shadow answer) and record your verdict on its class in the same step, as an `athena.class-label/1` record: the recorded class is right, the right class is another registry class, or a new class (added as a proposed `athena.failure-class/1` record in the same commit). The note is one line. The introspection is never edited; reviewing again adds a newer label that replaces the earlier one. An unknown id or class writes nothing. `athena status` shows how many introspections are still unreviewed and its `next` lines point here |
| `athena accept-proposal <id>` / `athena reject-proposal <id> --why "..."` | Accept: submit the ordinary job "apply defaults proposal <id>", which lists `defaults.toml` as a deliverable. Reject: record the reason; the class is proposed again only after 3 more distinct jobs have newer cases |
| `athena approve-work <id>` / `athena reject-work <id> --why "..."` | Decide an open steward proposal (a work item above the steward's limit), by the id `athena status` lists. Approve: submit an ordinary job whose request copies the work item's title, rung and source and the content of each record it cites (readers cannot see `records/`), then read it like any request. Reject: record the reason. Either way the decision is an appended `athena.work-decision/1` record; a decided item is no longer open |
| `athena check-proposal <id> --candidate <file>` | Check a candidate `defaults.toml` against the proposal and the file at HEAD: id, statement, version base plus one, no other entry changed, parses |
| `athena correct <job> "<line>"` | Correct the latest reading, or answer its question, and read again |
| `athena go <job>` | Run a proposed job, read a queued one, or retry the landing of a blocked one |
| `athena land <job> --pr` | Push the job's locally landed commit as its branch (a rerun on an open pull request replaces the branch with one atomic conditional push, never a plain force), open a pull request with `gh api`, and wait for green checks. It stops at an open, green pull request unless `[landing] merge_via_rest = true` is set (off when absent). Run it again after the merge to land the job (exit 0 when landed, or open with green checks; 2 otherwise). Every GitHub call goes through one wrapper that tries at most 3 times, with exponential backoff and jitter, on a 5xx, a timeout or a connection reset, and prints a line for each retry; a 4xx or an auth error fails at once |
| `athena drain --budget <usd> [--rating-threshold N] [--job-limit N] [--json]` | Work the queue within your budget: the requests the steward queued and the ones you deferred with `athena ask --later`. For each queued request, it reads and runs the job, lands it through the pull-request path of `athena land --pr`, and only then rates the live explainer page as a newcomer would. It stops at the first of: `target met` (a rating taken after a landing in this run reached `--rating-threshold`, default 4), `budget spent`, `repeated failure` (one item failed twice; it is named), `queue empty` (not success), or `job limit` (`--job-limit`, default 5). Exit 0 only when the target was met; 2 for any other stop |
| `athena close <job> --why "..."` | Close a job that will not run or finish |
| `athena status [<job>] [--json]` | The state of the work, or one job's full trace |
| `athena intervene "<what>" --why ... --class ... --category ... [--job <job>] [--with-staged]` | Record something you did beyond what a lazy user would; `--with-staged` commits your staged hand changes with it |
| `athena restart --cost "..." [--minutes N] [--tokens N] [--measured] [--lost ...] [--via ...]` | Record a restart of your own context |
| `athena check [--history]` | Check records, reconciliation, and the commit gate, now or over the whole history |
| `athena stop` / `athena start` | The kill switch: terminate every running agent and check, and block new ones |

**Merge strategy.** Pull requests may be landed on your main branch by merge commit, squash merge, or
rebase merge; `athena check --history` accepts all three. It identifies a landing from the job records the
commit adds, and proves the prediction came before the work from those records, so it does not depend on
`Athena-*` trailers or on commit ancestry. When `athena land --pr` asks the host to merge, it requests a
method the repository allows (merge commit, else squash, else rebase); after a squash it rebuilds your local
branch on the squash commit, keeping later landed jobs and records, so the next pull request opens clean.

**Which commit `land --pr` judges.** It judges and guards the pull request's current head, read from the
job's branch on the remote, never the job's own commit once the branch has moved. It never replaces a
newer head with the job's commit. Until the records-history build (D111) lands, a squash of a job's
landing plus any later commit is not a valid landing, so `land --pr` refuses when the head is anything
other than the job's landing commit, even when the extra commit is a declared intervention. The refusal
names the head's check state, each extra commit (short id and subject), and this reason, merges nothing and
exits nonzero. If you see it, land the job's own commit, or wait for D111.

Wherever a command takes `<job>`, it accepts a full id, a unique suffix, or `latest`. Every command except
`stop` and `start` takes `--json`, so an agent can drive Athena through a shell as well as a person can.

## What it does not decide

`athena drain` runs, lands and rates the queue (the steward's requests and your `ask --later` ones) for you, inside limits you set on the command line.
You no longer run, land and rate each item yourself. What it leaves to you:

- **Proposals above a role's limit.** `drain` approves an open proposal on a rung the role that raised it may
  change itself, and works the job that approval makes. An open proposal on a rung that role may not change,
  or from a role with no recorded definition, is listed as *waiting* in the output. `drain` never rejects one
  and never approves one above the limit; `athena approve-work` and `athena reject-work` stay your decision.
- **How much to spend and what is good enough.** The budget, the rating threshold and the job limit are flags of
  the run. There is no `athena.toml` key for them, so a run's limits are never changed by a committed file.
- **Whether a landing merges.** `drain` lands through the same pull-request path as `athena land --pr`: it
  pushes the job's branch, opens the pull request, waits for green checks, and merges only when
  `[landing] merge_via_rest = true` is set in `athena.toml`. With the setting off, the pull request stays open,
  the item counts as not landed, and the run ends in `repeated failure` naming it.
- **Reading a rating as success.** A rating counts toward the target only when it was taken after a landing in
  this run. A run that landed nothing prints that the page is unchanged and shows the last recorded rating
  marked as not taken after a landing; it never says `target met`.
- **Reading `target met` as a good page.** It needs two more things. A landing in the run must have changed what
  the explainer is built from (a path under `studies/`, the site generator in `xtask/`, or the publish
  workflow); otherwise the report says the landing did not change the outcome. And all of the page's text
  (every study file but the pin and the rule results: claims, the intro with its transition and glossary,
  and the rest) must pass a model-backed check against the pinned source, which fails on a contradicted or
  omitted statement, or on a model whose behavior differs from the code or a page that contradicts itself,
  and names it. Each study model's self-consistency check runs first, as a floor.

**Planning comes first.** `drain` does not work the queue oldest first. Every open request is first fleshed out
in a plan: its goal, the requirements-ladder rung it serves, its acceptance, its open questions, what it depends
on and which paths it touches. All open requests are then planned together into one ordered plan: a higher rung
goes first, an item that depends on another comes after it, and age only breaks ties. The report prints the
plan's groups; the items of one group depend on nothing in each other and touch different paths, so they could
run in parallel: drain runs up to `[parallel] max_concurrency` of a group at once (`athena.toml`; the default 1
is one job at a time), while landing and rating stay one at a time. Nothing is filled in by default: acceptance comes
only from explicit bullets in the request or from a reading's criteria (the first line is never copied into
it), and the rung from the proposal that made the job or from evidence in the request or reading, otherwise
it is flagged. The planning pass is deterministic code, not a model call. A request whose own words leave the
plan incomplete is first read (a reading is a model call, guarded by the budget like any other) and planned
again from its reading, so a specifiable request runs and the queue does not fill with unread refused items.
One that is still missing a goal, rung or acceptance after that is not worked: the report lists it as
*refused* with the missing fields and a reason (it has been read; correct it with `athena correct`).
`athena plan` prints the same plan without working anything: the order, the parallel groups, each item's goal,
rung, acceptance, open questions (a touched path that does not exist, a referenced job that is not in the
records), dependencies and touched paths, and the flagged items with reasons. It says so when nothing is open.
`--budget` caps what it spends reading (default $2; 0 reads nothing). Planning runs nothing, so each read only
has to fit the reader's cap from `athena.toml`, not a run after it.

**A read request stays in the queue.** A request is open until it is run or closed. One that `athena plan` has
read waits in state proposed and is still in the plan and in `drain`'s queue: a second `athena plan` lists it
again, and `drain` runs it in plan order without reading it a second time. Only a request that no capability
fits (declined), one that is running or has ended, and one on a channel `drain` does not work are left out.

The plan-stage audit in `athena_core::plan` (`trace`, `GapLog`) is still not wired to the records; that is
follow-up.

**Stopping a drain.** Run `athena drain-stop` from another terminal. The drain finishes the job it is on (it is
landed and rated as usual), starts no further job, and ends with the stop reason `stop requested`, exit code 2.
Nothing is killed, and the rest stays queued. To kill running work instead, use `athena stop`. A stop asked for
before a drain starts belongs to an earlier run and is cleared when the next drain begins.

**Budget.** Before every model call (a reading, a run, the re-reading of a retry, a rating, the source check)
`drain` checks that the money already spent, as the calls actually cost, plus that call's cap from
`athena.toml` fits `--budget`. If it does not, nothing more starts, even when an earlier job cost more than
its cap. A job starts only when its reading and its run both fit, so no request is left read and never run for
want of money. The source check is one call per study, so its reserve is the verifier's cap times the number of
studies, and what it costs is added to the spend.

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

## The explainer page

A page explaining how firstmate handles work, rendered from `studies/firstmate-work-handling`. Every
statement on it cites a claim id, and it states the pinned firstmate commit. Build it locally with one command:

```sh
cargo run -p xtask -- site        # writes target/site/index.html; open it in a browser
```

The page goes to `site/` in the cargo target directory (`$CARGO_TARGET_DIR`, else `target/`), never into the
work tree, so building it cannot change a verifier's snapshot. `--out` inside the work tree is refused.
The page introduces Athena and the study first, then explains firstmate; claim sources link to the pinned
commit when the studied repository is on GitHub.

`.github/workflows/pages.yml` builds and deploys the same page on every push to `main`. The expected URL is
`https://<owner>.github.io/<repository>/`. The repository owner must set **Settings → Pages → Source** to
**GitHub Actions** once; until then the deploy fails. See [`docs/design/explainer-site.md`](docs/design/explainer-site.md).

## Map

| Path | What it is |
|---|---|
| `crates/` | the Rust workspace; [`docs/architecture.md`](docs/architecture.md) maps the crates and their dependency direction |
| `xtask/` | setup, CI, and the repository-policy tests (`cargo run -p xtask -- setup`, `cargo run -p xtask -- ci`, `cargo run -p xtask -- site`) |
| [`CHANGELOG.md`](CHANGELOG.md) | what changed in each version, in Keep a Changelog form |
| `athena.toml` | configuration: models, budgets, agent environment (export a read-only `GH_TOKEN` for the steward's `gh`), the `change` capability's checks and allow-lists, classes, priors, pins |
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
