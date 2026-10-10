# Athena decision log

Every significant choice in Athena's design gets an entry here. Each entry contains:

- **Decision**: the choice that was made.
- **Alternatives**: the options that were considered.
- **Tradeoff**: what the choice gives up.
- **Reversal cost**: what it would take to undo it.
- **Provenance**: where the rationale comes from.
- **Bundle**: where Ryan Lopopolo's *Harness Engineering* bundle (CC BY 4.0) shaped the choice, or where
  Athena departs from it. This field appears only when relevant.

The bundle was read at commit `226c8d35fb6ea3ed55467753dba6dea2b5fd5778`.

Provenance takes one of three labels:

- **stated**: the requirement comes from the user's brief, and the entry cites where;
- **evidenced**: the rationale rests on a measurement or experiment, cited as E1 to E12 from Appendix A of
  [`docs/design/athena-v0.md`](design/athena-v0.md);
- **inferred**: the rationale is the designer's own reasoning, and is never presented as stated.

Entries are never rewritten once accepted. A later decision that changes an earlier one adds a new entry
that supersedes it, and names the entry it replaces.

Entries D01–D37 were written while designing v0, in the `bootstrap` job.

---

## D01 · Code owns the job lifecycle; agents own bounded roles

- **Decision:** The core is a deterministic Rust program. It sequences the job, isolates it, commits,
  gates, records, and scores. Model contexts are used only for bounded judgments: reading the request,
  producing the change, and verifying it. No model context orchestrates.
- **Alternatives:**
  - (a) An orchestrator agent (an LLM supervisor) that plans, dispatches, and closes the job.
  - (b) A single agent that does the whole job end to end and is checked afterward.
  - (c) Claude Code's own sub-agent mechanism, with a parent session spawning children.
- **Tradeoff:** The core cannot improvise around unforeseen situations. It fails with a precise error
  where an agent might have adapted. In return, the steps that must never be skipped (commit gates,
  verification before landing, recording every invocation) are mechanisms rather than requests, and the
  orchestrating context costs zero tokens.
- **Reversal cost:** Medium. The lifecycle sits behind `athena-core`'s job API, so an agent-driven
  planner could later sit above it, choosing among jobs, without replacing it.
- **Provenance:** Stated, for the mechanism over hope requirement (A8), orchestrating contexts as the
  scarcest resource (A6), deterministic work going to code (startup notes), and independent verification
  (A7). Inferred, for the conclusion that lifecycle closure is deterministic work.
- **Bundle:** Departs from *Give one agent the whole job*, where one primary trajectory owns closure.
  Follows the same thesis's exception: a separate context window earns its cost when it buys independent
  evidence, such as adversarial review. Each of Athena's three contexts is justified that way.

## D02 · The user talks to Athena through a command-line tool

- **Decision:** `athena ask "<request>"` shows the reading and predictions. On a terminal, the user then
  presses Enter to run the job or types a one-line correction. `athena correct`, `go`, and `drop` are the
  same steps as separate commands, for callers without a terminal. Every command offers `--json`.
- **Alternatives:**
  - (a) A Claude Code skill or plugin inside the user's own agent context.
  - (b) A long-running chat service or daemon.
  - (c) An MCP server exposing Athena's operations as tools.
  - (d) A web interface.
- **Tradeoff:** A CLI has no conversational memory and no rich UI. Neither is needed: the records hold the
  memory, and the reading fits in a terminal. Option (a) would load Athena's working details into the
  user's context, which the brief says should hold intent and verdicts only (O2). Options (b) and (d)
  add infrastructure that v0 does not need.
- **Reversal cost:** Low. A skill, an MCP server, or a UI can wrap the CLI's `--json` interface later.
- **Provenance:** Stated, for the lazy one-line request and correction in one line (A2, O2). Inferred,
  for the interface choice.
- **Bundle:** Shaped by *Make capabilities legible*: familiar CLIs carry strong model priors, which
  matters because Athena's user may itself be an agent driving a shell.

## D03 · Agents run as headless Claude Code processes, one process per invocation, at a pinned version

- **Decision:** Every agent invocation is a separate `claude -p` process. Its result comes back as JSON,
  with measured usage and reported cost. Claude Code is pinned at 2.1.296, and agents never spawn further
  agents.
- **Alternatives:**
  - (a) Call the Messages API directly from Rust and implement the tool loop (file editing, shell) in
    Athena.
  - (b) Embed the Claude Agent SDK, which runs under TypeScript or Python.
  - (c) Use the gateway's chat-completions route.
  - (d) Use `--bare` mode.
  - (e) Let one Claude Code process spawn sub-agents through its Agent tool.
- **Tradeoff:** Each invocation pays Claude Code's startup time and about 590 tokens of host-injected
  context (E3). Option (a) avoids both but means rebuilding a mature agent loop, which fails buy before
  build. Option (b) would put non-Rust code in Athena's runtime path. Option (c) has no tools. Option (d)
  failed authentication in the build environment and accepts only API keys. Option (e) would hide
  per-agent cost and context inside one process.
- **Reversal cost:** Medium. The `AgentDriver` trait isolates the runtime, so a direct-API driver could
  replace this one per role.
- **Provenance:** Evidenced: headless JSON output with usage, cost, and `modelUsage` (E2, E6), and
  authentication that needs only `HOME` and `PATH` (E7). Stated: buy before build, and all code in Rust.
  Inferred: process-per-invocation as the unit of cost capture.
- **Bundle:** Shaped by *Hold the worker constant*. The model plus its host is a black box, pinned for an
  adoption epoch and recorded in every invocation.

## D04 · Every agent gets a hermetic, explicitly composed context

- **Decision:** The core launches every agent through one function. It sets
  `--system-prompt-file <composed>` (replacing Claude Code's default system prompt), an empty
  `--setting-sources ""`, `--restricted`, `--strict-mcp-config`, an explicit `--tools` list,
  `--disable-slash-commands`, and `--no-session-persistence`. It runs the agent with an environment
  cleared and rebuilt from an allow-list.
- **Alternatives:**
  - (a) Append Athena's prompt to the default system prompt with `--append-system-prompt-file`.
  - (b) Rely on `--setting-sources user` alone.
  - (c) Use the defaults and instruct agents to ignore project files.
- **Tradeoff:** Replacing the default system prompt drops Claude Code's own behavioral guidance, so
  Athena's role frames must say what each role needs. Athena then owns, and can measure, everything except
  the host's injected text. Option (a) costs about 17k tokens per agent and changes with every Claude Code
  release (E2). Option (b) still loads user-level settings, plugins, and skills. Option (c) leaks the
  project's `CLAUDE.md` (E1).
- **Reversal cost:** Low. Switching is one flag in one function.
- **Provenance:** Stated: Athena owns and composes the context of every agent it runs, and the root
  `CLAUDE.md` must never reach Athena's agents. Evidenced: E1 (the leak with default flags, and its
  absence when hermetic), E2 (17,586 tokens with defaults against 680–5,303 when hermetic), and E7 (an
  environment of `HOME` and `PATH` suffices; a child process otherwise inherits its parent's identifying
  variables).
- **Bundle:** Departs from the bundle's routing through the repository's `AGENTS.md`. Athena does not
  let the host discover context. It composes context per role and routes with briefs instead. Shaped by
  *Route context just in time*: a small fixed map, with depth retrieved on demand.

## D05 · Host-injected context is measured and redacted, not owned

- **Decision:** Athena accepts that, even when launched hermetically, the host injects about 590 tokens
  of its own. These are an identity line, an account-context reminder containing the account's email
  address, an environment block, and the date. Athena records the host's baseline per invocation and
  redacts email addresses from agent free text before recording it.
- **Alternatives:**
  - (a) A direct-API driver that injects nothing.
  - (b) Ignore the injection.
- **Tradeoff:** v0 cannot remove text it does not control. Option (a) costs the agent loop (see D03).
  Option (b) risks personal data reaching public records whenever an agent echoes it.
- **Reversal cost:** Low.
- **Provenance:** Evidenced: E3. Stated: never commit personal data (Boundaries).

## D06 · Records are immutable JSON files, one fact per file, in per-job directories

- **Decision:** Each record is a pretty-printed JSON file with a `schema` field. It is written once and
  never modified or deleted. Each job writes only under `records/jobs/<id>/`. State is derived from which
  records exist. New knowledge about an old fact becomes a new record, such as a re-score or a resolution
  of a pending value.
- **Alternatives:**
  - (a) A SQLite database committed to the repository.
  - (b) Append-only JSON Lines ledgers, one per kind of record.
  - (c) One mutable `job.json` per job, updated in place.
  - (d) git notes.
  - (e) A separate records branch.
- **Tradeoff:** Many small files, and status computed rather than stored. In return, parallel jobs never
  touch the same path, so merges are trivially clean. Every change is an addition, so diffs are readable.
  The append-only rule can be enforced by a simple gate. Option (a) is binary and conflicts on every
  change. Option (b) conflicts when two branches append to the same file. Option (c) invites
  after-the-fact edits, including to predictions. Option (d) is invisible to most tools and to strangers.
  Option (e) separates records from the work they explain.
- **Reversal cost:** Medium, since a migration tool could fold the files into another store. Low for
  readers, because the files are plain JSON with published schemas.
- **Provenance:** Stated: records are machine-readable and live in the repository (A5); storage is
  diff-friendly, merge-friendly, and safe under parallel work (bootstrap brief). Inferred: immutability
  as the mechanism for all three.
- **Bundle:** Shaped by *Make the repository teach the agent*: one owner per concept, and parse at the
  boundary.

## D07 · Predictions are committed before dispatch

- **Decision:** At dispatch, a prediction commit lands on the user's branch. It contains the request,
  every reading and correction, the accepted job definition, and the producer brief. The closing commit
  follows at the terminal state.
- **Alternatives:**
  - (a) Keep predictions uncommitted until the job closes.
  - (b) Store predictions in git notes or a side ref.
  - (c) Rely on file timestamps.
- **Tradeoff:** Two commits per job instead of one, so the history is noisier. The trailers make it
  filterable. In return, the history itself proves the forecast came first. The append-only gate stops
  the forecast from being edited afterward, and a crash mid-job still leaves the prediction on record.
  Option (a) loses the ordering proof. Option (b) loses visibility. Option (c) is easily forged.
- **Reversal cost:** Low.
- **Provenance:** Stated: record predictions before dispatch (A2, bootstrap brief item 2). Inferred:
  commit order as tamper evidence.

## D08 · Agents work in git worktrees outside the checkout, one per role

- **Decision:** The reader works in a worktree at `HEAD`. The producer works in a worktree at the
  prediction commit, and the verifier in one at the candidate commit. All live under the state directory,
  outside the user's checkout, and all are removed when the job closes.
- **Alternatives:**
  - (a) Copy the repository per job.
  - (b) Use a container per job.
  - (c) Let agents work in the user's checkout.
  - (d) Nest worktrees under a hidden directory inside the checkout.
- **Tradeoff:** Worktrees share the object store, so they are cheap but not a security boundary.
  Containers are stronger but heavier, and Docker is not a dependency Athena should impose. Option (c)
  makes parallel work collide. Option (d) lets tools that walk up the directory tree (cargo workspaces,
  Claude Code's memory discovery, ripgrep) confuse the nested tree with the outer one.
- **Reversal cost:** Low. Creating a workspace sits behind one function in `athena-git`.
- **Provenance:** Stated: parallel work never corrupts other work (A5). Evidenced: worktree and sparse
  checkout behavior (E9). Inferred: placement outside the checkout.

## D09 · Owner-private paths are marked with a git attribute and kept off agents' disks

- **Decision:** Any path the repository marks `athena-exclude` in `.gitattributes` is left out of every
  agent worktree by sparse checkout, and so is `records/`. Athena reads the attribute generically and
  never names the paths itself.
- **Alternatives:**
  - (a) Deny reads of those paths in the tool hook.
  - (b) List the paths in `athena.toml`.
  - (c) Rely on hermetic launch only.
- **Tradeoff:** Repository owners must learn one attribute. Option (a) cannot stop Grep or Glob from
  surfacing a file's contents in search results. Option (b) would put the owner's private paths into
  Athena's own configuration. Option (c) prevents automatic loading but not an agent's own reads.
- **Reversal cost:** Low.
- **Provenance:** Stated: the root `CLAUDE.md` must never reach Athena's agents, and Athena must know
  nothing about the people or tools around it. Evidenced: sparse worktrees keep attributed paths off
  disk and intact in the tree (E9). Inferred: the attribute as the boundary.

## D10 · Landing is fast-forward only, gated in process, and keyed on a content fingerprint

- **Decision:**
  - Athena builds every commit as an object with git plumbing: a temporary index and `commit-tree`,
    signed with `-S` when the user's configuration asks for signing.
  - It gates the commit with the same function the hooks use.
  - It then advances the user's branch with `git merge --ff-only`.
  - A landing is allowed only if the landed content, meaning every path outside `records/` and the
    owner-private paths, has the same SHA-256 fingerprint as the content the verifier judged.
  - Records-only interleavings replay through `merge-tree`. Any other change to the base makes the job
    `stale`.
- **Alternatives:**
  - (a) Commit with porcelain in the user's checkout.
  - (b) Cherry-pick or merge, and trust the result.
  - (c) Open pull requests.
  - (d) Rebase onto the new base and verify again automatically.
- **Tradeoff:** Plumbing commits skip hooks, so Athena must gate its own commits (M2). A stale job must
  be asked again; v0 does not re-verify it automatically. In return, the bytes that land are provably
  the bytes that were verified, and the user's index and uncommitted work are never touched. Option (a)
  would mix Athena's commits with the user's staged changes. Option (b) breaks the link between
  verification and the landed artifact. Option (c) is unavailable in the build environment and adds a
  step that v0 does not need. Option (d) is left out until stale landings recur.
- **Reversal cost:** Medium.
- **Provenance:** Stated: nothing reaches the main branch without passing verification, and parallel
  work never corrupts other work (A5). Evidenced: `commit-tree` needs `-S` to sign, and `merge-tree`
  preserves the fingerprint (E9). Inferred: the fingerprint as the identity of what was verified.
- **Bundle:** Shaped by *Prove the outcome*, specifically release integrity: deliver the artifact that
  was tested, and do not rebuild it under different conditions.

## D11 · Every commit names its work in trailers, checked at commit time and in CI

- **Decision:** Every commit that touches anything other than owner-private paths must carry
  `Athena-Job: <id>` or `Athena-Record: <path>`, naming something that exists at `HEAD`. Landing commits
  also carry the class, capability, producer, verifier and verdict, cost, and verified-content
  fingerprint. Merge commits are exempt, as are the commits listed in `athena.toml`: the one commit that
  predates Athena.
- **Alternatives:**
  - (a) Put job ids in subjects only.
  - (b) Keep a commit-to-job index file.
  - (c) Use git notes.
- **Tradeoff:** People committing by hand must add a trailer, which is the point (A5). Option (a)
  clutters subjects and is hard to parse. Option (b) is a second source of truth that drifts. Option (c)
  is not visible by default.
- **Reversal cost:** Low.
- **Provenance:** Stated: every commit traces to the work that produced it (A5), and a commit must be
  followable from request to outcome (bootstrap brief item 7). Inferred: resolving trailers against
  `HEAD`, so that records may arrive after the commits they explain, as the bootstrap's do.

## D12 · The verifier is independent by type, process, model, and worktree

- **Decision:** The verifier runs as its own process, on a model that never produces (Opus 5.5 in v0),
  in a fresh worktree at the exact candidate commit. Its context is composed from `VerifierInputs`, a
  type with no field that can carry the producer's report. Before it runs, the core runs the capability's
  checks. The final verdict is computed by code from the agent's verdict, the criteria, the findings, and
  the checks.
- **Alternatives:**
  - (a) The same model as the producer, in a separate context.
  - (b) A verifier that sees the producer's report, for efficiency.
  - (c) Self-review by the producer.
  - (d) Deterministic checks only.
- **Tradeoff:** An Opus verdict costs more than a Sonnet one, and a fresh worktree means a fresh build.
  Option (a) is weaker on independence: the same model shares blind spots. Option (b) violates the brief.
  Option (c) is the producer verifying its own work. Option (d) cannot judge whether a change does what
  was asked.
- **Reversal cost:** Low for the choice of model, which is configuration. High for the independence
  property, which is the brief's.
- **Provenance:** Stated: verifiers never see the producer's reasoning, nothing verifies its own work,
  and high-stakes conclusions are verified at the highest tier (A7, bootstrap brief item 4). Inferred:
  model-level separation, and a fresh worktree at the candidate.
- **Bundle:** Departs from *Prove the outcome* where the worker produces its own proof. Athena lets the
  producer self-check, but counts only checks run by code and an independent verdict.

## D13 · One verification round, with severities and a verdict computed by code

- **Decision:** Findings carry a severity: `blocker`, `major`, `minor`, or `nit`. A job passes only if
  the agent says pass, every criterion is met, no finding is a blocker or major, and every check passed.
  v0 runs exactly one round. A fail closes the job, and the user can ask again.
- **Alternatives:**
  - (a) A bounded fix-and-reverify loop, for example two rounds.
  - (b) Unbounded iteration until the verifier passes.
- **Tradeoff:** Some fixable jobs are rejected that a loop would have rescued. In return, the stopping
  rule is trivial and the cost is bounded. Option (b) has no stopping rule and invites reviewer bullying,
  which the bundle documents. Option (a) is left out until the rework rate argues for it.
- **Reversal cost:** Low. A loop wraps produce and verify.
- **Provenance:** Stated: every loop has a stopping rule, and findings carry severity (A7). Inferred:
  that one round is enough for v0.
- **Bundle:** Shaped by *Keep review convergent*: severity thresholds, explicit outcomes, and a stopping
  rule.

## D14 · Brier score for events, weighted interval score on a log scale for quantities, components kept separate

- **Decision:** Events are scored with the Brier score. Cost and time are forecast as 10th, 50th, and
  90th percentiles and scored with the weighted interval score (WIS) on natural-log values. Each
  component is stored separately, together with skill against a reference forecast.
- **Alternatives:**
  - (a) The log score for events.
  - (b) The continuous ranked probability score (CRPS) over full distributions.
  - (c) Point predictions scored by absolute error.
  - (d) Interval coverage alone.
  - (e) One blended number per job.
- **Tradeoff:**
  - The Brier score is less harsh than the log score on confident misses. It is still strictly proper,
    and it stays finite with small samples while decomposing into calibration and resolution.
  - WIS needs only three numbers from a forecaster, unlike (b), and it penalizes width, so vague
    intervals score badly.
  - Option (c) does not reward honest uncertainty: it is not a proper score for a distribution.
  - Option (d) can be gamed with very wide intervals.
  - Option (e) hides tradeoffs between unlike units.
- **Reversal cost:** Low. Raw forecasts and outcomes are stored, so any rule can re-score history.
- **Provenance:** Stated: a proper scoring rule, so that vague predictions do not score well (A2,
  bootstrap brief item 6). Inferred: the specific rules, following Gneiting and Raftery (2007) on proper
  scores, and Bracher et al. (2021) on WIS.
- **Bundle:** Shaped by *Measure effectiveness*: keep the dimensions visible as separate measures.

## D15 · Every forecast is made by up to three forecasters, scored side by side

- **Decision:** The reader, Jev, and a code-computed reference forecast all forecast the same events.
  The reader and the reference also forecast cost and time. The reader sees the reference first, and may
  move away from it only with a stated reason. All three are scored with the same rules.
- **Alternatives:**
  - (a) The reader alone.
  - (b) The reference alone, computed by code.
  - (c) Jev alone.
- **Tradeoff:** Extra fields in every reading, and about a second of extra latency per reading. In
  return, from the first job Athena gathers evidence on whether the expensive forecaster adds skill over
  the cheap one, and over the base rate. That is exactly the evidence that routing revisions need (A6).
  With option (a) a vague reader could not be told apart from a sharp one, (b) has no judgment, and (c)
  has no calibration record yet.
- **Reversal cost:** Low.
- **Provenance:** Stated: Jev is the default for typed decisions, calibrated alongside use and shadowed
  by a stronger model (A6), and calibration is tracked (A2). Evidenced: Jev answers predicate questions
  with probabilities, cheaply (E12). Inferred: the reference forecast as the yardstick for skill.

## D16 · Every quantity carries its provenance

- **Decision:** Every number for tokens, time, or money is a `Quantity`:
  `{value, unit, provenance, basis}`. Provenance is one of `measured`, `reported`, `estimated`, or
  `pending`. Claude Code's cost is `reported`, with the basis "list price". Token counts reported by the
  API are `measured`. The bootstrap's own token cost is `pending`.
- **Alternatives:**
  - (a) Bare numbers, with a convention in the docs.
  - (b) A separate estimated flag on some fields only.
- **Tradeoff:** Records are wordier. In return, an unlabeled number cannot be written, because the
  schema refuses it.
- **Reversal cost:** Low.
- **Provenance:** Stated: numbers are measured or labeled as estimates (Standards), and tokens are
  measured or labeled (A6). Evidenced: Claude Code reports cost at list price (`costBasis: "list"`
  inside `modelUsage`).

## D17 · Secret scanning uses gitleaks, pinned and checksum-verified, in three places

- **Decision:** gitleaks 8.30.1 is installed by setup with its SHA-256 verified. Athena's single gate
  function runs it from the `pre-commit` hook, inside Athena's own commit path, and over the full history
  in CI. The gate fails closed when gitleaks is missing. Tests generate fake tokens at runtime, so no
  secret-shaped literal ever sits in the repository.
- **Alternatives:**
  - (a) TruffleHog. It is capable, but licensed AGPL-3.0, and its strength is verifying live
    credentials.
  - (b) ripsecrets. It is written in Rust, but has a smaller rule set and no git-history mode.
  - (c) detect-secrets, which is Python.
  - (d) The hosting service's secret scanning and push protection.
  - (e) A regex scanner written in Rust.
- **Tradeoff:** gitleaks is a Go binary, which makes it a non-Rust tool used through interop. Athena's
  logic (when to scan, what to block, how to report) stays in Rust. Option (d) cannot be configured from
  the build environment and acts only after a push. Option (e) fails buy before build.
- **Reversal cost:** Low. The scanner sits behind one function.
- **Provenance:** Stated: every commit gated, and CI scans the full history (bootstrap brief item 8; A5).
  Evidenced: release checksums are published per platform (E11). Inferred: the tool choice.

## D18 · Git hooks are symlinks to the `athena` binary, which dispatches on its own name

- **Decision:** Setup links `.git/hooks/pre-commit` and `.git/hooks/commit-msg` to the adopted `athena`
  binary. Invoked under a hook's name, the binary runs that hook's gate. Setup never overwrites a hook it
  did not create, and leaves other hooks, such as an existing `pre-push`, alone.
- **Alternatives:**
  - (a) Committed shell shims in `.githooks/`, with `core.hooksPath`.
  - (b) The `pre-commit` framework, which is Python.
  - (c) Hook scripts generated at setup.
- **Tradeoff:** Symlinks do not work on Windows, which v0 does not support. In return, the hooks contain
  no shell at all. Option (a) adds shell files and overrides the whole hooks directory. Option (b) adds
  a Python dependency.
- **Reversal cost:** Low.
- **Provenance:** Stated: all code is Rust. Evidenced: the repository already uses this pattern for an
  existing hook. Inferred: name-based dispatch.

## D19 · Athena drives git through its CLI

- **Decision:** `athena-git` wraps the `git` command (version 2.38 or later, for `merge-tree
  --write-tree`), with typed inputs and outputs.
- **Alternatives:**
  - (a) `git2`, the libgit2 bindings.
  - (b) `gix`, gitoxide in pure Rust.
- **Tradeoff:** Spawning a process per operation is slower, and output has to be parsed. In return,
  Athena gets complete support for worktrees, sparse checkout, attributes, signing, and `merge-tree`.
  Hooks and configuration behave exactly as they do for the user. Option (a) is a C dependency whose
  sparse and worktree support lags. Option (b) does not yet cover sparse worktrees and signing as
  completely.
- **Reversal cost:** Medium, behind the wrapper's typed API.
- **Provenance:** Evidenced: the needed mechanics work in git 2.43 (E9). Inferred: the comparison.

## D20 · Crates per concern, with the dependency direction checked against the documentation

- **Decision:** Ten crates ([§6 of the design](design/athena-v0.md#6-modules-and-dependency-direction)). Cargo forbids cycles between crates. A test
  compares `cargo metadata` against the table in `docs/architecture.md`, and also requires that the core
  never depends on a capability.
- **Alternatives:**
  - (a) One crate with modules.
  - (b) Fewer, larger crates.
- **Tradeoff:** More manifests to maintain. In return, "no cycles" and "documented direction" become
  mechanisms, and each crate is tested independently. With (a), cycles between modules go unchecked.
- **Reversal cost:** Low.
- **Provenance:** Stated: focused, independently tested modules, a documented dependency direction, and
  no cycles (A4). Inferred: crates as the enforcement unit.

## D21 · Core and capabilities meet at one trait; v0's capability is `change`

- **Decision:** `athena-core` defines `Capability`: the reader's catalog card, producer and verifier
  prompt sections, tool grants, checks, and admission policy. `athena-cap-change` implements it, and the
  CLI registers it.
- **Alternatives:**
  - (a) A capability as configuration only, with prompts and checks in TOML.
  - (b) Hard-code `change` in the core.
- **Tradeoff:** A new kind of work needs Rust code, not only configuration. In return, capabilities can
  carry typed checks and policies. Option (b) would make adding studying a rebuild of the core.
- **Reversal cost:** Low.
- **Provenance:** Stated: the core is separate from capabilities from day one, and a new kind of work
  means adding a capability (A4).

## D22 · Agent tool policy in three layers, with the hook written in Rust

- **Decision:** There are three layers of tool policy:
  - an explicit `--tools` list per role;
  - `--restricted` mode, with permission rules and `--permission-prompts none`;
  - a `PreToolUse` hook whose command is the `athena` binary itself.

  The hook denies paths outside the worktree, writes to `records/` and `.git`, git commands that change
  state, and any write at all by the reader or the verifier.
- **Alternatives:**
  - (a) Permissions only.
  - (b) A hook only.
  - (c) Bypass permissions inside a container.
- **Tradeoff:** Three layers to keep consistent. Inspecting Bash commands is coarse, and the layers are
  defense in depth rather than a containment boundary. With (a) or (b) alone, one gap opens everything. Option (c)
  needs container infrastructure.
- **Reversal cost:** Low.
- **Provenance:** Evidenced: a hook's deny overrides an allow rule; `--restricted` confines reads and
  writes to the working directory; denials are reported (E4, E5). Stated: rules have teeth, using Claude
  Code's hooks, settings, permissions, and tool restrictions (A8, bootstrap brief).
- **Bundle:** Shaped by *Maximize autonomy inside explicit authority*: broad capability in a reversible
  worktree, and no authority over history.

## D23 · Setup is a minimal POSIX bootstrap, then Rust

- **Decision:**
  - `./setup.sh`, about 30 lines, installs rustup only if `cargo` is missing, from a pinned
    `rustup-init` with its SHA-256 checked. It then hands off to `cargo run -p xtask -- setup`.
  - Rust installs gitleaks, builds and adopts the `athena` binary, links the hooks, and checks or
    privately installs the pinned Claude Code.
  - Downloads use `curl`, which honors proxies and CA settings. Rust verifies checksums and extracts
    archives.
  - Pins live in `athena.toml`, apart from rustup's, which lives in `setup.sh`, its only consumer.
- **Alternatives:**
  - (a) Require rustup to be preinstalled and use `cargo xtask setup` as the one command.
  - (b) A shell script for all of setup.
  - (c) Nix or a container image.
  - (d) An HTTP client in Rust for downloads.
- **Tradeoff:** One shell file exists, and it is listed as an exception. Option (a) fails "a clean machine
  with one command". Option (b) puts setup logic outside Rust. Option (c) imposes a heavy tool on
  adopters. Option (d) would have to handle TLS-intercepting proxies and custom CA bundles that `curl`
  already handles; for the one-time downloads in setup, `curl` is the clearly best tool. Jev's runtime
  calls, which carry a key, are made in-process instead (D31).
- **Reversal cost:** Low.
- **Provenance:** Stated: one command, committed, idempotent, version-pinned, credentials from the
  environment and never stored, and setup code in Rust with platform exceptions listed (A4, Settings,
  bootstrap brief). Evidenced: crates.io, rustup, and GitHub release downloads are reachable (E10).
- **Bundle:** Shaped by *Dependency ownership*: one source of truth per version, and policy that parses
  the authoritative manifest.

## D24 · CI is two thin GitHub Actions workflows that call Rust

- **Decision:** `ci.yml` runs `./setup.sh --ci` and then `cargo run -p xtask -- ci`, which covers fmt,
  clippy, tests, and `athena check --history`. `secrets.yml` runs a full-history gitleaks scan on every
  push and weekly. Both check out full history, run with read-only permissions, and pin actions by commit
  SHA.
- **Alternatives:**
  - (a) Logic in YAML.
  - (b) The official gitleaks action.
  - (c) One workflow.
- **Tradeoff:** Two YAML files, listed as exceptions. In return, local `cargo run -p xtask -- ci`
  reproduces CI exactly. Option (b) requires a license for organization accounts and hides the logic.
- **Reversal cost:** Low.
- **Provenance:** Stated: CI scans the full history, and YAML is acceptable where a platform requires it
  (bootstrap brief). Inferred: thin YAML.

## D25 · The always-loaded contract is compiled in, measured, and budgeted at 350 tokens

- **Decision:** The contract has eight rules and measures 294 tokens. It is compiled into the binary,
  hashed into every invocation record, and held to a fixed budget of 350 tokens. CI enforces the budget
  with an estimator that errs high, ⌈bytes/3.0⌉; the measured ratio is 3.3 bytes per token. An opt-in live
  test measures the real count.
- **Alternatives:**
  - (a) Load the contract from the repository at runtime.
  - (b) A larger budget, such as 1,000 tokens.
  - (c) No contract, with role frames only.
- **Tradeoff:** A small budget forces every addition to pay for itself. With option (a), a stray edit in
  the repository could change every agent's behavior without a new version of Athena. Option (c) would
  repeat the shared rules in every frame.
- **Reversal cost:** Low.
- **Provenance:** Stated: a fixed token budget from day one, and every addition paid for (A13, bootstrap
  brief). Evidenced: 294 tokens measured, identical on two models (E8).

## D26 · The defaults model is a versioned TOML file seeded from the standing spec

- **Decision:** `defaults.toml` holds eleven defaults, each with an id, a statement, what it applies to,
  and a source in `BRIEF.md`. The reader loads all of them and names the ones that apply. Only those
  reach the producer's and verifier's briefs.
- **Alternatives:**
  - (a) Put the defaults inside the contract.
  - (b) Learn defaults only from interventions, starting empty.
  - (c) Use JSON.
- **Tradeoff:** Seeding from the spec is the designer's reading of it, so every entry cites its source
  where it can be checked. Option (a) spends always-loaded budget on things only some jobs need. Option
  (b) wastes the written spec. Option (c) is harder for people to edit.
- **Reversal cost:** Low.
- **Provenance:** Stated: an explicit, versioned model of defaults, learned from the standing spec,
  requests, and interventions (A2). Inferred: the seed entries.

## D27 · Interventions and restarts are recorded by command; reading corrections are captured automatically

- **Decision:** `athena intervene` and `athena restart` write and commit records at once. A one-line
  correction of a reading is itself recorded as an intervention (category `corrected-output`, source
  `observed`), as is an answer to a question the reader asked.
- **Alternatives:**
  - (a) Record only interventions the user reports.
  - (b) Treat a reading correction as part of the normal flow, not an intervention.
- **Tradeoff:** Intervention counts include cheap corrections, which inflates the rate the user is
  judged on. The brief says correcting Athena's output is an intervention, and never suppressing one
  matters more than a low count. Option (b) would hide exactly the signal the defaults model should learn
  from.
- **Reversal cost:** Low.
- **Provenance:** Stated: what counts as an intervention, and never suppress one (O3). Inferred:
  capturing corrections by mechanism.

## D28 · Spend is bounded per invocation and per job, with timeouts and a kill switch

- **Decision:**
  - Every invocation passes `--max-budget-usd` and runs under a deadline, with a process-group kill on
    timeout.
  - The core refuses to start an invocation whose cap would exceed what remains of the job's budget.
  - `athena stop` creates a stop file that blocks new invocations and terminates running ones.
  - Every invocation record carries the caps that applied to it.
- **Alternatives:**
  - (a) Caps in prompts.
  - (b) A spending ledger checked after the fact.
- **Tradeoff:** Claude Code's budget check is approximate within a turn. The deadline backs it up.
- **Reversal cost:** Low.
- **Provenance:** Stated: spend is bounded with a cap and a kill switch, both recorded (Settings).

## D29 · No transcripts in v0; agent free text is redacted before it is recorded

- **Decision:** Claude Code runs with `--no-session-persistence`. Athena keeps only structured outputs.
  Free-text fields are redacted before writing: every email address outside an allow-list becomes
  `[email]`, and every gitleaks finding becomes `[secret:<rule>]`.
- **Alternatives:**
  - (a) Capture `stream-json` transcripts and redact them.
  - (b) Keep transcripts outside the repository.
- **Tradeoff:** Without trajectories, diagnosing a failed job is harder. Transcript capture is left out
  until a failure needs it. Option (a) has a large redaction surface and makes the repository grow
  quickly.
- **Reversal cost:** Low.
- **Provenance:** Stated: never commit transcripts unredacted, and never commit personal data
  (bootstrap brief, Boundaries). Evidenced: the host injects the account's email address into every
  agent's context (E3).

## D30 · Ids are time-sortable with a random suffix

- **Decision:** An id has the form `YYYYMMDD-HHMMSS-xxxx`: UTC time plus four Crockford base32
  characters. The id `bootstrap` is reserved.
- **Alternatives:**
  - (a) ULIDs.
  - (b) Sequential numbers.
  - (c) UUIDs.
- **Tradeoff:** Ids are a little longer than sequence numbers. Option (b) needs coordination between
  parallel jobs and branches. Options (a) and (c) are harder for people to read and type.
- **Reversal cost:** Low for new ids. Old ids stay valid.
- **Provenance:** Inferred.

## D31 · v0's model routing, with Jev as a shadow forecaster

- **Decision:**
  - The reader is Sonnet 5.5.
  - The reader picks the producer from {Haiku 5.5, Sonnet 5.5}, with Sonnet as the default.
  - The verifier is Opus 5.5, which never produces in v0.
  - Jev (`typesafe-ai/jev`), through the gateway's decisions API, answers one predicate question per
    forecast event on every reading. It costs about $0.00001 per question set. Its forecasts are
    recorded and scored beside the reader's and the reference's. Jev never gates or changes a job. A
    missing key or a failed call is recorded as skipped, and the job continues.
- **Alternatives:**
  - (a) Leave Jev out of v0, the default the user suggested.
  - (b) Make Jev the shown, primary forecaster at once.
  - (c) Route the reading itself through Jev.
  - (d) Use Opus for everything.
- **Tradeoff:** Jev adds an HTTP driver and a credential name to setup. Against option (a), the
  acceptance judgment asks for a calibration record behind Jev's use, and every job run without Jev is
  calibration data lost. Starting the record at the first job costs fractions of a cent and puts
  nothing at risk. Option (b) would trust an uncalibrated forecaster. Option (c) fails because a reading
  is generative and needs repository access, which Jev, a decision model, lacks. Option (d) costs several
  times more with no evidence that it pays.
- **Reversal cost:** Low. A configuration switch disables Jev.
- **Provenance:** Stated: the starting routing (startup notes), using Jev liberally with calibration
  alongside use (A6), and the gateway authenticating when v0 was designed. Evidenced: Jev answers
  predicates with probabilities, at measured usage and reported cost (E12). Inferred: the shadow role,
  and keeping Opus off the producer list so the verifier model always differs.

## D32 · The producer's self-report is kept with its run and never fed forward

- **Decision:** The producer's report includes mistakes, learnings, and desires (MLD). It is stored,
  redacted, in the job's `candidate.json`. It is never given to the verifier, and never injected into a
  later agent's context. Improvement work reads it as telemetry and corroborates it before acting on it.
- **Alternatives:**
  - (a) No self-report.
  - (b) Feed past self-reports into later briefs.
- **Tradeoff:** Low-trust telemetry is collected, at a little output cost per job. Option (b) turns
  unverified self-commentary into instructions.
- **Reversal cost:** Low.
- **Provenance:** Inferred.
- **Bundle:** Adopted from *MLD: telemetry for the harness builder*: keep raw telemetry out of agent
  context, and corroborate before promoting.

## D33 · Non-Rust files are inventoried, and the inventory is enforced

- **Decision:** `docs/non-rust-files.md` lists every kind of non-Rust file, with its reason. A test fails
  on any tracked file that is neither Rust nor listed. Owner-private paths are outside Athena and exempt.
- **Alternatives:** (a) List the files in prose only.
- **Tradeoff:** A table to maintain. In return, a new kind of file cannot slip in.
- **Reversal cost:** Low.
- **Provenance:** Stated: non-Rust files only where a platform requires them, each listed with its
  reason (A4).

## D34 · A boundary test keeps Athena free of references to who runs it and where

- **Decision:** A test scans Athena's own files for terms that would tie Athena to the people or machine
  around it. The files covered are its code, prompts, schemas, configuration, docs, setup, and workflows.
  The forbidden terms are assembled from fragments, so the test itself contains none of them.
  - Excluded from the scan: the user's verbatim spec (`BRIEF.md`), records (data, which may quote the
    user), the license, `.gitattributes` (where the repository owner declares private paths), and
    owner-private paths.
- **Alternatives:**
  - (a) Review by hand.
  - (b) A plain grep in CI.
- **Tradeoff:** Term lists produce false positives and miss paraphrases. The test is a tripwire, not
  proof.
- **Reversal cost:** Low.
- **Provenance:** Stated: nothing in Athena refers to the people or tools that run it, or to the
  particular machine it runs on, and a test enforces that (bootstrap brief).

## D35 · The state of the work is derived from records, never hand-maintained

- **Decision:** `athena status` and `athena show` compute everything from the records at `HEAD` and the
  state directory. No status file exists to keep current.
- **Alternatives:**
  - (a) A `STATUS.md` updated by each job.
  - (b) A state file in the state directory.
- **Tradeoff:** Reading the state needs the binary, or reading the JSON directly. In return, the view
  cannot drift from the truth. Option (a) drifts and causes merge conflicts. Option (b) is lost with the
  machine.
- **Reversal cost:** Low.
- **Provenance:** Stated: a fresh context gets the state of the work from the repository alone
  (bootstrap brief item 10, A11). Inferred: derived views.

## D36 · Buy before build: the crates Athena adopts, and what it builds

- **Decision:**
  - Adopted:
    - `clap` for the CLI;
    - `serde`, `serde_json`, and `toml` for serialization;
    - `schemars` for JSON Schemas generated from types;
    - `jiff` for timestamps;
    - `sha2` for hashing;
    - `getrandom` for id entropy;
    - `regex` for redaction and policy;
    - `anyhow` and `thiserror` for errors;
    - `tempfile` for temporary files;
    - `flate2` and `tar` for setup;
    - `libc` for process-group signals;
    - a small blocking HTTPS client for Jev's decisions API (`ureq`, with native certificate roots so
      that proxies and custom CA bundles work), chosen at build time after a connectivity test.
  - Built, each too small to justify a dependency: a 20-line Crockford base32 encoder, the token
    estimator (D25), and the scoring rules (D14). The scoring rules are formulas worth owning with
    tests, unlike a statistics crate that would bring many unrelated functions.
- **Alternatives:**
  - `chrono` or `time` instead of `jiff`.
  - `reqwest` instead of `ureq`, which pulls in an async runtime.
  - `tiktoken-rs` for token counting, which uses another vendor's tokenizer and so gives the wrong
    counts for Claude.
  - `proptest` for propriety tests; deterministic grids suffice.
- **Tradeoff:** Each dependency is supply-chain surface. All are widely used, and all are pinned in
  `Cargo.lock`.
- **Reversal cost:** Low per crate.
- **Provenance:** Stated: buy before build, recording each call and the options considered. Inferred:
  the selections.
- **Bundle:** Shaped by *Dependency ownership*: own narrow, central behavior, such as the scoring rules,
  and adopt specialist capabilities.

## D37 · Rust 1.97.0, edition 2024, pinned in `rust-toolchain.toml`

- **Decision:** The toolchain is pinned to Rust 1.97.0 with clippy and rustfmt, and the workspace uses
  edition 2024. Builds and CI use `--locked`.
- **Alternatives:**
  - (a) The stable channel, unpinned.
  - (b) An older minimum supported version.
- **Tradeoff:** Upgrading the toolchain is a deliberate change. That is the point.
- **Reversal cost:** Low.
- **Provenance:** Stated: version-pinned setup (A4). Evidenced: Rust 1.97.0 is current in the build
  environment.
