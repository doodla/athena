# Bootstrap brief: the automation engineer designs and builds Athena v0

You are the automation engineer for Athena, an agent orchestration system written in Rust. This is
the bootstrap: the only work ever done outside Athena. Your job is to design, and then build,
Athena v0 as a walking skeleton in the git repository at `/home/user/athena` (current branch; do
not switch branches). You own the method. I am Athena's first user and I own the requirements.

## Read first

- `/home/user/athena/BRIEF.md`: the full requirements brief, which is also Athena's user's
  standing spec. Read "Goal", the Standards, Boundaries, Part 1 (A1–A13), and Part 3 D1 closely.
  v0 is a small slice of all this; everything else is grown later through Athena itself, pulled by
  real work. Passages addressed to "the operator" about cloud sessions, the sandbox, leases, or
  handoffs are not Athena's concern: Athena must know nothing about them. Do not read or depend
  on anything under `/home/user/athena/operator/`.
- The harness-engineering bundle, cloned at `/home/user/lopopolo/harness-engineering` (pinned at
  commit `226c8d35fb6ea3ed55467753dba6dea2b5fd5778`; start at its `AGENTS.md`). Use it as a working
  reference for Athena's design. Record where it shaped a decision and where Athena departs from it
  and why. It is CC BY 4.0: attribute it, and check its `COPYING.md` before reusing any
  third-party material it contains.

## What v0 must do

v0 is the smallest Athena that can:

1. take a request: a lazy, one- or two-line request from its user;
2. make its reading of the request visible before work starts, in a form the user can correct in
   one line, and record specific, checkable predictions before dispatch: how it read the request
   and which defaults it applied, what it will produce, the expected cost, time, and models, the
   expected verification outcome, and whether it expects the user to intervene;
3. dispatch one job to a sub-agent;
4. verify the result independently: the verifier never sees the producer's reasoning, and nothing
   verifies its own work;
5. record every model invocation with its job, model, tokens (measured, or labeled as estimates),
   time, cost, and outcome;
6. score the predictions against what actually happened with a proper scoring rule, so vague
   predictions do not score well;
7. land a commit that traces to its job, so request → prediction → owner → model → cost →
   verification → outcome can be followed from the commit;
8. with secret scanning on: every commit is gated, and CI scans the full history.

And, because Athena's user starts using it the moment v0 is accepted:

9. the user can record an intervention (what happened, why, the class of work, and the category)
   and a restart of their own context (what it cost to get productive again, and anything lost),
   as machine-readable records that Athena owns and will learn from;
10. a fresh context, with zero memory, can get the state of the work from the repo alone.

## How I will accept it

After you finish, I will check it myself from a clean clone:

- One command takes a clean machine to a working Athena. Setup is committed, idempotent, and
  version-pinned, reads credentials from the environment, and never stores them.
- I give Athena a one-line request. I see its reading and predictions before any work starts. The
  job runs on a sub-agent, isolated so parallel work could never corrupt other work. An
  independent verifier passes or fails it. On a pass, a traceable commit lands. The prediction is
  scored. Every record is machine-readable and lives in the repo.
- `cargo test` passes, clippy is clean, and the CI workflows pass.
- Nothing in Athena (code, prompts, record schemas, docs) refers to operators, cloud sessions, or
  this particular sandbox, and a test enforces that.

## Constraints

- All code we write is Rust, setup included. Non-Rust files appear only where a platform requires
  them (GitHub Actions YAML, for example), and each one is listed with its reason. A clearly-best
  non-Rust tool may be used through interop if our own logic stays in Rust; record the call.
- The core (how work runs) is separate from capabilities (what work Athena can do) from day one,
  even if v0 has only a trivial capability. Focused, independently tested modules with clear
  interfaces, documented dependency direction, and no cycles.
- Standalone and environment-agnostic: someone else could lift Athena out and adopt it.
- Buy before build: adopt good, maintained crates and tools; build only for needs they don't meet,
  and name those needs. Record each call and the options considered.
- Enforced by mechanism rather than hope: decide which of v0's rules get mechanisms (and tests that
  show each blocks what it should and allows what it should); mark the rest prose-only. Claude
  Code's hooks, settings, permissions, tool restrictions, sub-agent definitions, skills, and
  plugins are all available as mechanisms.
- Orchestrating contexts are the scarcest resource; Athena owns the context of every agent it runs
  and composes it explicitly. Its always-loaded contract gets a fixed token budget from day one.
- Everything committed is public. Never commit secrets or credentials. Never commit transcripts
  unredacted. Keep the repo's size healthy.
- Written for strangers: code, commits, docs, and records are written for an outside reader.
- Keep v0 small. Its size is a design choice, not a wish list. List what v0 leaves out and what
  evidence would pull each piece in. Unless you argue otherwise, v0 leaves out: specialists, the
  lab harness, the explainer site, Jev integration, richer memory and skills, multi-job planning,
  and the trained-contract machinery.

## Facts about the current environment (verified today)

- Claude Code 2.1.296 is installed as `claude` and runs headless. `claude -p "<prompt>" --model
  <id> --output-format json` returns JSON with `usage` (input, output, cache-read and
  cache-creation tokens), `total_cost_usd`, `duration_ms`, `session_id`, and `modelUsage`. Model
  ids: `claude-opus-5-5`, `claude-sonnet-5-5`, `claude-haiku-5-5`.
- Useful flags (see `claude --help`): `--system-prompt[-file]`, `--append-system-prompt[-file]`,
  `--setting-sources`, `--settings`, `--agents`, `--plugin-dir`, `--mcp-config`,
  `--strict-mcp-config`, `--tools`, `--allowedTools`, `--disallowedTools`, `--permission-mode`,
  `--json-schema` (validated structured output), `--max-budget-usd`, `--resume`, `--session-id`,
  `--no-session-persistence`, `--add-dir`, `--output-format stream-json`.
- `--setting-sources user` stops the project's `CLAUDE.md` from loading (tested with a canary).
  The repository root `CLAUDE.md` is not Athena's contract and must never reach Athena's agents;
  launch every Athena agent with explicitly composed context.
- `--bare` fails authentication here: it accepts only `ANTHROPIC_API_KEY` or an `apiKeyHelper`,
  and this environment authenticates Claude Code another way. Don't depend on it.
- A child `claude` process inherits its parent's environment variables, including ones that
  identify the parent's session (a test run reported the parent's session id). Give each agent run
  a clean, explicit environment.
- The Vercel AI Gateway, Jev's route, currently rejects authentication (HTTP 401). Don't build v0
  on it.
- GitHub: REST through `gh api repos/<owner>/<repo>/...` works; GraphQL is refused, so `gh pr`
  subcommands fail; branch protection cannot be configured. Don't push or merge: commit on the
  current branch, and I will land the work on `main`.
- Present today, but not to be relied on (setup pins its own): Rust 1.97.0 via rustup, Node 22,
  Python 3.13, uv, Docker, tmux, jq, Playwright's Chromium. crates.io and GitHub release-asset
  downloads are reachable.

## Process

1. Design first. Write v0's design document, plus a decision log with an entry per significant
   choice (alternatives considered, tradeoff, reversal cost, and provenance: stated, evidenced, or
   inferred). Cover at least: the architecture and process model; how the user talks to Athena;
   how context is composed for each role; the data model and its storage (diff-friendly,
   merge-friendly, safe under parallel work); module boundaries and dependency direction; the
   protocols (job briefs, verifier independence, landing and commit trailers, secret-scan gates);
   which rules v0 enforces by mechanism; the scoring rule; the model-invocation driver and how cost
   is captured; setup and CI; the always-loaded contract v0 and its token budget; and what v0
   leaves out. Then STOP: commit the design, and report back with a short summary and the paths.
   An independent reviewer will review the design before you build anything.
2. After the review, I will send you its findings. Address each (or argue against it with
   evidence), then build, test, and run the acceptance flow once yourself, end to end, with a real,
   small, useful request of your choosing.
3. Commit as you go. Every bootstrap commit carries the trailer `Athena-Job: bootstrap`, plus
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Do not push. Record the bootstrap as
   bootstrap work in Athena's records, including this brief verbatim. Leave the bootstrap's own
   token cost as a field marked pending; I will fill it from measured usage afterwards.
4. Report: what you built, how to run the acceptance test, what is out of scope, and open risks.

Take the tokens you need, and make it something to be proud of, not merely something that passes.
