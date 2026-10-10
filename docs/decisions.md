# Athena decision log

Every significant choice in Athena's design gets an entry here. Each entry contains:

- **Decision:** the choice that was made.
- **Alternatives:** the options that were considered.
- **Tradeoff:** what the choice gives up.
- **Reversal cost:** what it would take to undo it.
- **Provenance:** where the rationale comes from, labeled as one of the following:
  - **stated:** the requirement comes from the user's brief, and the entry cites where;
  - **evidenced:** the rationale rests on a measurement or experiment, cited as E1–E21 from Appendix A of
    [`docs/design/athena-v0.md`](design/athena-v0.md), or as R1–R10 from the appendix of
    [review 1](design/reviews/athena-v0-review-1.md);
  - **inferred:** the rationale is the designer's own reasoning, and is never presented as stated.
- **Bundle:** where Ryan Lopopolo's *Harness Engineering* bundle shaped the choice, or where Athena departs
  from it. The bundle is licensed CC BY 4.0 and was read at commit
  `226c8d35fb6ea3ed55467753dba6dea2b5fd5778`. This field appears only when relevant.

**How entries change.** Until the v0 design is accepted, an entry may be revised. Each revision adds a
**Revised** line that says what changed and which review finding prompted it, so the history stays
readable. After acceptance, an entry is never rewritten: a new entry supersedes it and names the entry it
replaces.

Entries D01–D37 were written in revision 1 of the v0 design, and D38–D45 were added in revision 2 after
review 1. The re-check (review 2) left five decisions to the user. They are folded into D22, D38, and D39,
and added as D46 and D47, with the provenance "user decision after review 2". With them the design was
accepted, so from here on entries are superseded, not revised. D48–D54 were added while building v0, and
amend earlier entries without rewriting them. All of these entries belong to the `bootstrap` job.

---

## D01 · Code owns the job lifecycle; agents own bounded roles

- **Decision:**
  - The core is a deterministic Rust program. It sequences, isolates, commits, gates, records, and scores.
  - Model contexts are used only for bounded judgments: reading the request, producing the change, and
    verifying it. No model context orchestrates.
  - The reader is a separate agent from the producer, for two reasons. The forecasts must not become the
    executor's targets, and the reader must stay read-only.
- **Alternatives:**
  - (a) An orchestrator agent (an LLM supervisor) that plans, dispatches, and closes the job.
  - (b) One agent for the whole job: read, pause for the user's acceptance, then produce.
  - (c) Claude Code's own sub-agent mechanism, with a parent session spawning children.
- **Tradeoff:**
  - The core cannot improvise around unforeseen situations: it fails with a precise error where an agent
    might have adapted. In return, the steps that must never be skipped are mechanisms rather than
    requests, and the orchestrating context costs zero tokens.
  - Option (b) would give the same ordering more cheaply, since the producer re-reads what the reader read.
    But it would put the forecasts, such as the predicted cost and the predicted verdict, in the executor's
    context, where they could become targets. It would also give the reader Bash.
- **Reversal cost:** Medium. The lifecycle sits behind `athena-core`'s job API, so a planner could later
  sit above it without replacing it.
- **Provenance:**
  - Stated: mechanism over hope (A8); orchestrating contexts as the scarcest resource (A6); deterministic
    work goes to code (startup notes); independent verification (A7).
  - Inferred: that lifecycle closure is deterministic work, and that forecasts should be kept from the
    executor.
- **Bundle:** Two departures from *Give one agent the whole job*:
  - code, not a trajectory, owns closure;
  - the reader/producer split, which ordering alone would not require.

  The verifier follows the bundle's own exception: a separate context earns its cost when it buys
  independent evidence, such as adversarial review.
- **Revised** (review 1, m12): the reader/producer split is now justified by forecast isolation and a
  read-only reader, rather than by ordering, and recorded as a departure from the bundle.

## D02 · The user talks to Athena through a command-line tool with ten commands

- **Decision:** The commands are `ask`, `correct`, `go`, `close`, `status [<job>]`, `intervene`,
  `restart`, `check`, `stop`, and `start`.
  - On a terminal, `ask` waits after showing the reading: Enter runs the job, `q` leaves it proposed, and
    any other line is a correction.
  - Without a terminal, `ask` commits its proposal and exits with the follow-up commands.
  - `go` also retries the landing of a blocked job.
  - `close` replaces drop and abandon.
  - `status <job>` replaces a separate trace command.
  - Every command offers `--json`.
- **Alternatives:**
  - (a) A Claude Code skill or plugin inside the user's own agent context.
  - (b) A chat service or daemon.
  - (c) An MCP server.
  - (d) A web interface.
  - (e) Revision 1's fifteen commands.
- **Tradeoff:** A CLI has no conversational memory, but the records hold the memory. Option (a) would load
  Athena's details into the user's context (O2). Options (b) and (d) add infrastructure. Option (e) carried
  commands that fold into others.
- **Reversal cost:** Low. Other front ends can wrap the `--json` interface.
- **Provenance:** Stated: a lazy one-line request, correctable in one line (A2, O2). Inferred: the
  interface.
- **Bundle:** Shaped by *Make capabilities legible*: familiar CLIs carry strong model priors, which
  matters because the user may itself be an agent.
- **Revised** (review 1, M5): fifteen commands became ten; `show`, `calibration`, `drop`, `abandon`, `land`,
  and `doctor` were folded into others or removed.

## D03 · Agents run as headless Claude Code processes, one per invocation, at a pinned version

- **Decision:** Every agent invocation is a separate `claude -p` process with `--output-format stream-json`.
  Claude Code is pinned at 2.1.296, and agents never spawn further agents.
- **Alternatives:**
  - (a) Call the Messages API directly and implement the tool loop in Athena.
  - (b) Embed the Claude Agent SDK, which runs under TypeScript or Python.
  - (c) Use a gateway's chat-completions route.
  - (d) Use `--bare` mode.
  - (e) Let one Claude Code process spawn sub-agents.
- **Tradeoff:** Each invocation pays Claude Code's startup time and about 600 tokens of host-injected
  context (E3).
  - (a) means rebuilding a mature agent loop.
  - (b) puts non-Rust code in Athena's runtime path.
  - (c) has no tools.
  - (d) failed authentication in the build environment.
  - (e) hides per-agent cost and context.
- **Reversal cost:** Medium, behind the `AgentDriver` trait.
- **Provenance:** Evidenced: headless output with usage, cost, and per-model usage (E2, E6, E16), and
  authentication needing only `HOME` and `PATH` (E7). Stated: buy before build, and Rust.
- **Bundle:** Shaped by *Hold the worker constant*: the model plus its host is a black box, pinned and
  recorded.
- **Revised** (review 1, M3): `json` output became `stream-json`, so usage is captured as it accrues (D40).

## D04 · Every agent gets a hermetic, explicitly composed context

- **Decision:** One function launches every agent with these flags: `--system-prompt-file <composed>`
  (replacing Claude Code's default system prompt), `--restricted`, `--strict-mcp-config`, an explicit
  `--tools`, `--disable-slash-commands`, and `--no-session-persistence`. It also sets the environment:
  - cleared, then rebuilt from an allow-list;
  - `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`;
  - git configuration and discovery fenced to the snapshot.
- **Alternatives:**
  - (a) Append to the default system prompt.
  - (b) `--setting-sources ""` instead of `--restricted`.
  - (c) Default flags, with instructions to ignore project files.
- **Tradeoff:**
  - Replacing the default system prompt saves about 2,300 tokens per invocation, measured with each role's
    own tools (E18). It also gives Athena ownership of, and measurements for, everything except the host's
    injected text. But it drops Claude Code's own behavioral guidance, whose effect on the producer's
    quality is unmeasured, and is listed for evaluation in §14 of the design.
  - Option (b) blocks a project `CLAUDE.md` but not auto-memory (E19).
  - Option (c) leaks both (E1, E19).
- **Reversal cost:** Low. The prompt mode is one flag.
- **Provenance:** Stated: Athena owns and composes every agent's context, and the root `CLAUDE.md` must
  never reach its agents. Evidenced: E1, E2, E7, E18, E19.
- **Bundle:** Departs from routing through the repository's `AGENTS.md`. Shaped by *Route context just in
  time*: a small fixed map, with depth retrieved on demand.
- **Revised** (review 1, m8 and n4):
  - The cost comparison now uses E18's 2,328 tokens, not E2's total, which included 25 tools.
  - `--setting-sources ""` was dropped in favor of `--restricted`, because E19 showed only the latter
    blocks auto-memory.
  - The auto-memory variable was added.

## D05 · Host-injected context is measured and contained, not owned

- **Decision:** Athena accepts that the host injects about 600 tokens of its own (E3), including the
  account's email address. Athena takes three measures:
  - it records the host's baseline per invocation;
  - it redacts email addresses from agent free text before recording it;
  - its gate refuses non-allow-listed email addresses in added lines.
- **Alternatives:**
  - (a) A direct-API driver.
  - (b) Ignore the injection.
- **Tradeoff:** v0 cannot remove text it does not control. Option (a) costs the agent loop. Option (b)
  risks personal data reaching public records.
- **Reversal cost:** Low.
- **Provenance:** Evidenced: E3. Stated: never commit personal data (Boundaries).
- **Revised** (review 1, m5): the personal-data gate on added lines was added.

## D06 · Records are immutable JSON files, one fact per file, in per-job directories sharded by month

- **Decision:**
  - Each record is a pretty-printed JSON file with a `schema` field, written once and never modified or
    deleted.
  - A job writes only under `records/jobs/<YYYY-MM>/<id>/`.
  - State is derived from which records exist.
  - New knowledge about an old fact is a new record: a re-score named after the intervention that caused
    it, a later outcome, or a resolution.
- **Alternatives:**
  - (a) SQLite.
  - (b) Append-only JSON Lines ledgers.
  - (c) A mutable `job.json`.
  - (d) git notes.
  - (e) A records branch.
  - (f) One flat jobs directory.
- **Tradeoff:** Many small files, and status computed rather than stored.
  - (a) is binary and conflicts.
  - (b) conflicts on concurrent appends.
  - (c) invites after-the-fact edits.
  - (d) is invisible by default.
  - (e) separates records from the work they explain.
  - (f) rewrites an ever-growing tree object on every commit.
- **Reversal cost:** Medium, since a migration tool could fold the files into another store. Low for
  readers.
- **Provenance:** Stated: machine-readable records in the repository (A5); diff-friendly, merge-friendly,
  and safe under parallel work (bootstrap brief). Inferred: immutability as the mechanism.
- **Bundle:** Shaped by *Make the repository teach the agent*: one owner per concept, and parse at the
  boundary.
- **Revised** (review 1, m6, m13): month shards were added, and re-scores are named after their
  intervention.

## D07 · Forecasts are committed before dispatch, and the closing commit points back to them

- **Decision:**
  - Readings, with their forecasts, are committed no later than dispatch: in a proposal commit if the
    command exits first, otherwise in the prediction commit.
  - The closing commit carries `Athena-Prediction: <commit>`.
  - CI checks that the named commit is an ancestor and added the job's accepted reading, which the
    append-only gate then protects.
- **Alternatives:**
  - (a) Commit predictions only at closing.
  - (b) git notes, or a side ref.
  - (c) File timestamps.
- **Tradeoff:** Extra commits per job. Option (a) loses the ordering proof. Option (b) is not fetched by
  default, and a side ref can be force-moved without a trace (review 1, Q1). Option (c) is easily forged.
- **Reversal cost:** Low.
- **Provenance:** Stated: predictions recorded before dispatch (A2; bootstrap brief, item 2). Inferred:
  commit order as tamper evidence, strong once the commit is shared.
- **Revised** (review 1, Q1 and M3): proposal commits and the `Athena-Prediction` trailer and check were
  added.

## D08 · Agents work in detached snapshots, each with its own throwaway repository

- **Decision:**
  - For each agent, the core writes the visible files of the source commit into a fresh directory outside
    the checkout, using a temporary index, `read-tree`, and `checkout-index`. The visible files exclude
    `records/` and `athena-exclude` paths, under the source commit's committed rules.
  - The reader gets a plain directory. The producer gets a repository with one commit, base. The verifier
    gets a repository with two commits, base and candidate.
  - The core turns the producer's edits into a candidate commit in the user's repository from the change
    list, with `--no-renames`, `hash-object -w`, a temporary index over the base's full tree, and
    `commit-tree`.
- **Alternatives:**
  - (a) Linked git worktrees with sparse checkout, which was revision 1's design.
  - (b) Full clones per job.
  - (c) Containers.
  - (d) `git archive` exports.
- **Tradeoff:** Each snapshot is a copy, and agents lose `git log` of the real history. That is the point.
  - (a) shares objects, refs, and configuration with the user's repository: `git show` reaches excluded
    content, `update-ref` moves branches, and `sparse-checkout set` rewrites shared configuration (review
    1, R1 and R2).
  - (b) copies history, including excluded files.
  - (c) imposes Docker.
  - (d) drops `export-ignore` paths and rewrites `export-subst` files.
- **Reversal cost:** Low, behind `athena-git`'s workspace function.
- **Provenance:** Stated: parallel work never corrupts other work (A5), and the root `CLAUDE.md` never
  reaches agents. Evidenced: E14 and E15 (isolation, candidate transfer, and a byte-identical
  configuration), and review 1's R1 and R2 (the worktree escapes).
- **Revised** (review 1, B1): linked worktrees were replaced by detached snapshots.

## D09 · Owner-private paths are declared with a git attribute, read from committed rules at the base commit

- **Decision:**
  - A path marked `athena-exclude` in a committed `.gitattributes` file is absent from every snapshot,
    outside the governed content, and unchangeable by jobs.
  - The rules are evaluated with `check-attr --source=<base>`, `core.attributesFile=/dev/null`, and
    `GIT_ATTR_NOSYSTEM=1`.
  - Athena refuses to run if `info/attributes` mentions an `athena-*` attribute.
  - No candidate may change any path's classification.
  - In this repository, the owner's tooling directory declares itself from inside its own zone, and the
    root `.gitattributes` marks `/CLAUDE.md`.
- **Alternatives:**
  - (a) Deny reads in the tool hook.
  - (b) List the paths in `athena.toml`.
  - (c) Rely on hermetic launch alone.
  - (d) Evaluate the attributes with git's defaults.
- **Tradeoff:** Owners learn one attribute (two, with D44).
  - (a) cannot stop Grep or Glob from surfacing contents.
  - (b) puts owner paths into Athena's own files.
  - (c) stops automatic loading but not an agent's own reads.
  - (d) lets files outside the repository, such as the per-user and system attribute files and
    `info/attributes`, change the fingerprint and the agents' view (review 1, R7; E14).
- **Reversal cost:** Low.
- **Provenance:** Stated: the root `CLAUDE.md` must never reach Athena's agents, and Athena knows nothing
  about the people and tools around it. Evidenced: E14; R7.
- **Revised** (review 1, B1, M1, m1, Q4):
  - Enforcement now rests on snapshots, not sparse checkout.
  - The rules come from committed files only, at the base commit.
  - Candidates cannot change classifications.

## D10 · Landing is fast-forward only, gated in process, and keyed on a content fingerprint under the base's rules

- **Decision:**
  - The core builds every commit with plumbing, signed with `-S` when the user's configuration asks for
    signing, and gates it with the hooks' own function.
  - It then advances the branch with `merge --ff-only`.
  - A landing requires all of the following:
    - the base commit P is an ancestor of the branch tip H;
    - every path's classification is the same under P's and H's rules;
    - the landed governed content has the fingerprint that was verified, computed under P's rules.
  - Interleavings of records or owner paths replay through `merge-tree`. Any other change makes the job
    `stale`.
- **Alternatives:**
  - (a) Porcelain commits in the user's checkout.
  - (b) A cherry-pick or merge, trusted as it comes.
  - (c) Pull requests.
  - (d) Automatic re-verification on the new base.
- **Tradeoff:** Plumbing skips hooks, so Athena gates itself (M2). Stale jobs must be asked again.
  - (a) mixes Athena's commits with the user's staged work.
  - (b) breaks the link between verification and the artifact.
  - (c) is unavailable here and adds a step.
  - (d) is left out until stale landings recur.
- **Reversal cost:** Medium.
- **Provenance:** Stated: nothing reaches main without passing verification, and parallel work never
  corrupts other work (A5). Evidenced: signing needs `-S`, and `merge-tree` preserves the fingerprint (E9,
  E15).
- **Bundle:** Shaped by *Prove the outcome* (release integrity): deliver exactly the artifact that was
  verified.
- **Revised** (review 1, B1, m1): the candidate is built from the snapshot's change list; fingerprints use
  the base's rules; the ancestry and classification checks were added.

## D11 · Records-only commits name their work in a trailer

- **Decision:** A commit that leaves the governed content unchanged needs an `Athena-Job:`,
  `Athena-Intervention:`, or `Athena-Record:` trailer naming something that exists at `HEAD`. A commit that
  touches only owner paths needs no trailer. Commits that change governed content fall under D39.
- **Alternatives:**
  - (a) Job ids in subjects.
  - (b) An index file mapping commits to jobs.
  - (c) git notes.
- **Tradeoff:** Hand commits need a trailer. Option (a) is hard to parse. Option (b) is a second source of
  truth. Option (c) is invisible by default.
- **Reversal cost:** Low.
- **Provenance:** Stated: every commit traces to the work that produced it (A5). Inferred: resolving
  trailers against `HEAD`, so records may arrive after the commits they explain.
- **Revised** (review 1, M2): for commits that change governed content, the trailer-only rule is
  superseded by D39's invariant.

## D12 · The verifier is independent by type, process, model, and snapshot

- **Decision:**
  - The verifier runs as its own process, on Opus 5.5, which is outside the producer's allowed set.
  - It works in its own snapshot repository, holding base and the exact candidate.
  - Its context is composed from `VerifierInputs`, which cannot carry the producer's report or commands.
  - The capability's checks run first, by code.
  - The final verdict is computed by code.
- **Alternatives:**
  - (a) The same model as the producer.
  - (b) A verifier that sees the producer's report.
  - (c) Self-review by the producer.
  - (d) Deterministic checks only.
- **Tradeoff:** An Opus verdict costs more, and every snapshot starts with a cold build.
  - (a) shares blind spots.
  - (b) breaks the brief.
  - (c) is the producer verifying its own work.
  - (d) cannot judge fitness to the request.
- **Reversal cost:** Low for the choice of model. High for the independence property, which is the brief's
  own.
- **Provenance:** Stated: verifiers never see the producer's reasoning, nothing verifies its own work, and
  the highest tier verifies high-stakes conclusions (A7; bootstrap brief, item 4). Inferred: model-level
  separation. Review 1 (Q2) concurred.
- **Bundle:** Follows the bundle: its whole-job closure includes adversarial review in a separate context.
- **Revised** (review 1, B1, m2): the verifier works in a snapshot, and its exercise is made mechanical
  (D43).

## D13 · One verification round, with severities and a verdict computed by code

- **Decision:**
  - Findings are `blocker`, `major`, `minor`, or `nit`.
  - A job passes only if the agent says pass, every criterion is met, no finding is a blocker or major, and
    every check passed.
  - v0 runs exactly one round. A fail closes the job, and the user can ask again.
- **Alternatives:**
  - (a) A bounded fix-and-reverify loop.
  - (b) Unbounded iteration.
- **Tradeoff:** Some fixable jobs are rejected. Option (b) has no stopping rule.
- **Reversal cost:** Low.
- **Provenance:** Stated: a stopping rule for every loop, and findings carry severity (A7).
- **Bundle:** Shaped by *Keep review convergent*.

## D14 · Brier for events, a summed Brier over an unchosen set for deliverables, log-scale WIS for quantities

- **Decision:**
  - Events are scored with the Brier score.
  - Deliverables are scored with the *sum* of Brier terms over the union of the listed paths and the paths
    that changed. Unlisted paths count as p = 0.
  - Cost and time are 10th, 50th, and 90th percentiles, scored with WIS on natural-log values. Values are
    floored first, at $0.001 and 1 s, and out-of-order quantiles are rejected.
  - Components are stored raw and separately. Skill against the reference is computed only for aggregates
    of 10 or more jobs.
  - The models forecast is checked, not scored.
- **Alternatives:**
  - (a) The log score.
  - (b) CRPS over full distributions.
  - (c) Point predictions with absolute error.
  - (d) Interval coverage alone.
  - (e) One blended number.
  - (f) A mean Brier score over the listed deliverables, which was revision 1's design.
  - (g) Skill per job.
- **Tradeoff:**
  - (a) is harsh with small samples.
  - (b) needs whole distributions.
  - (c) is not a proper score for distributions.
  - (d) can be gamed with wide intervals.
  - (e) hides tradeoffs.
  - (f) can be gamed by padding: nine padded paths turn a complete miss of 0.81 into 0.08 (review 1, M4).
  - (g) is unstable when the reference's score is near zero.
- **Reversal cost:** Low. Raw forecasts and outcomes are stored, so history can be re-scored.
- **Provenance:** Stated: a proper scoring rule under which vague predictions do not score well (A2;
  bootstrap brief, item 6). Inferred: the rules, after Gneiting and Raftery (2007) and Bracher et al.
  (2021).
- **Bundle:** Shaped by *Measure effectiveness*: keep the dimensions separate.
- **Revised** (review 1, M4):
  - The deliverable score became a sum over an unchosen set.
  - `only_listed_paths` was dropped.
  - Floors and quantile ordering were added.
  - Skill moved to aggregates only.
  - Models are checked, not scored.

## D15 · Two forecasters: the reader, and a reference forecast computed by code

- **Decision:** The reader and the reference forecast every event and quantity, except deliverables, which
  only the reader forecasts. The reader sees the reference first, and departs from it only with a stated
  reason.
- **Alternatives:**
  - (a) The reader alone.
  - (b) The reference alone.
  - (c) Add Jev as a third forecaster, which was revision 1's design.
- **Tradeoff:** With (a), vagueness would be invisible. With (b), there is no judgment. Option (c) is moot:
  Jev's forecasts can be computed retroactively (D31).
- **Reversal cost:** Low.
- **Provenance:** Stated: calibration is tracked (A2). Inferred: the reference as the yardstick for skill.
- **Revised** (review 1, M5): three forecasters became two.

## D16 · Every quantity carries its provenance, and bounds when it is not measured

- **Decision:** Every number for tokens, time, or money is a `Quantity`:
  `{value, unit, provenance, basis, lower?, upper?}`. Provenance is one of `measured`, `reported`,
  `estimated`, or `pending`. Killed invocations record lower-bound tokens, and cost bounded by the cap.
- **Alternatives:**
  - (a) Bare numbers.
  - (b) An estimated flag on some fields only.
- **Tradeoff:** Records are wordier, but an unlabeled number cannot be written.
- **Reversal cost:** Low.
- **Provenance:** Stated: numbers are measured or labeled as estimates (Standards, A6). Evidenced: cost is
  reported at list price, and partial usage is a lower bound (E16).
- **Revised** (review 1, M3): the bounds were added.

## D17 · Secret scanning uses gitleaks, pinned and verified, and always run with Athena's own configuration

- **Decision:**
  - gitleaks 8.30.1 is installed by setup, checked against its SHA-256.
  - One gate function runs it in the `pre-commit` hook, in Athena's own commit path, and over the full
    history in CI.
  - It always gets four settings:
    - a generated `--config` that extends gitleaks' default rules with `athena.toml`'s `[secrets]`
      allow-list, taken from the parent commit, or from `HEAD` for the history scan;
    - an empty `--gitleaks-ignore-path`;
    - `--ignore-gitleaks-allow`;
    - `--max-decode-depth=5`.
  - The gate fails closed when gitleaks is missing.
  - Tests generate fake tokens at runtime.
- **Alternatives:**
  - (a) TruffleHog (AGPL-3.0, focused on verifying live credentials).
  - (b) ripsecrets (Rust, a smaller rule set, no history mode).
  - (c) detect-secrets (Python).
  - (d) The hosting service's own secret scanning.
  - (e) A regex scanner written in Rust.
  - (f) gitleaks with its defaults, which honor a `.gitleaks.toml`, a `.gitleaksignore`, and inline markers
    found in the scanned content.
- **Tradeoff:**
  - gitleaks is a Go binary used through interop; Athena's policy stays in Rust.
  - (d) cannot be configured from here and acts only after a push.
  - (e) fails buy before build.
  - (f) lets a commit carry its own exemption (E17; review 1, R5).
- **Reversal cost:** Low.
- **Provenance:** Stated: every commit gated, and CI scans the full history (bootstrap brief, item 8; A5).
  Evidenced: E11, E17.
- **Revised** (review 1, M1, n3): the scanned content can no longer relax the scan, and the allow-list
  moved into `athena.toml`, so there is no `.gitleaks.toml`.

## D18 · Git hooks are symlinks to the `athena` binary, which dispatches on its own name

- **Decision:** Setup links `pre-commit` and `commit-msg` in the common git directory to the adopted
  `athena` binary. It never overwrites a hook it did not create, and leaves other hooks alone.
- **Alternatives:**
  - (a) Committed shell shims with `core.hooksPath`.
  - (b) The `pre-commit` framework (Python).
  - (c) Hook scripts generated at setup.
- **Tradeoff:** Symlinks do not work on Windows, which v0 does not support. In return, there is no shell in
  the hooks.
- **Reversal cost:** Low.
- **Provenance:** Stated: all code is Rust. Evidenced: E13 (git invokes a symlinked hook with `argv[0]` set
  to the hook's path, also from linked worktrees).
- **Revised** (review 1, n2): the evidence is now experiment E13.

## D19 · Athena drives git through its CLI, version 2.40 or later

- **Decision:** `athena-git` wraps the `git` command with typed inputs and outputs, and setup checks for git
  2.40 or later, which `check-attr --source` and `merge-tree --merge-base` need.
- **Alternatives:**
  - (a) `git2`, the libgit2 bindings.
  - (b) `gix`, gitoxide in pure Rust.
- **Tradeoff:** Spawning a process per operation is slower, and its output must be parsed. In return,
  attributes, signing, plumbing, and configuration behave exactly as they do for the user.
- **Reversal cost:** Medium.
- **Provenance:** Evidenced: E9, E14, E15. Inferred: the comparison.
- **Revised** (review 1, m1): the minimum version was corrected from 2.38 to 2.40.

## D20 · Crates per concern, with the dependency direction checked against the documentation

- **Decision:** Ten crates ([§6 of the design](design/athena-v0.md#6-modules-and-dependency-direction)).
  Cargo forbids cycles between them. A test compares `cargo metadata` with the table in
  `docs/architecture.md`, and also requires that the core never depends on a capability.
- **Alternatives:**
  - (a) One crate with modules.
  - (b) Fewer, larger crates.
- **Tradeoff:** More manifests. In return, acyclicity and the documented direction become mechanisms.
- **Reversal cost:** Low.
- **Provenance:** Stated: focused modules, a documented dependency direction, and no cycles (A4).

## D21 · Core and capabilities meet at one trait, configured from the base commit

- **Decision:** `athena-core` defines `Capability`: a catalog card, prompt sections, tool grants, checks,
  and admission. Each method receives a `JobConfig` parsed from the job's base commit. v0's capability is
  `change`.
- **Alternatives:**
  - (a) Capabilities as configuration only.
  - (b) Hard-coding the capability in the core.
- **Tradeoff:** A new kind of work needs Rust, but it can carry typed checks and policies.
- **Reversal cost:** Low.
- **Provenance:** Stated: core separate from capabilities from day one (A4).
- **Revised** (review 1, M1): configuration comes from the base commit, never from the candidate.

## D22 · Agent tool policy in four layers, with an allow-list hook written in Rust

- **Decision:** Four layers:
  - the detached snapshot;
  - an explicit `--tools` list;
  - `--restricted`, with permission rules and `--permission-prompts none`;
  - a `PreToolUse` hook (the `athena` binary itself) whose Bash policy is an allow-list.

  The hook splits commands into segments and denies what it cannot classify. It allows `cargo` with listed
  subcommands, `ls`, and git `status`, `diff`, `log`, and `show` only. It denies `-c`, `-C`, `--git-dir`,
  `--work-tree`, and `GIT_*` overrides, paths outside the snapshot, and, for the reader and verifier, every
  write.
- **Alternatives:**
  - (a) Permissions only.
  - (b) A hook only.
  - (c) A git denylist, which was revision 1's design.
  - (d) Claude Code's built-in confinement of Bash commands.
  - (e) Containers.
- **Tradeoff:** Four layers to keep consistent, and command parsing that fails closed.
  - (a) and (b) each leave one gap open.
  - (c) missed `sparse-checkout`, `update-ref`, and `-c` (review 1, R1).
  - (d) confines the agent's Bash, but needs bubblewrap on Linux, which is not installed in the build
    environment and which setup cannot install pinned and portably. It also would not cover the checks the
    core runs (R4).
  - (e) imposes Docker.

  Options (d) and (e) are the candidates for containing executed code, which is left out of v0 with its
  pull-in evidence named.
- **Reversal cost:** Low.
- **Provenance:** Evidenced: a hook deny overrides an allow rule, and `--restricted` confines file tools
  (E4, E5); the escapes (R1). Stated: rules have teeth, using Claude Code's mechanisms (A8).
- **Bundle:** Shaped by *Maximize autonomy inside explicit authority*.
- **Revised** (review 1, B1, m11): the denylist became an allow-list, snapshots were added as a layer, and
  Claude Code's Bash confinement was evaluated and deferred.
- **Revised** (user decision after review 2, N3 and N2):
  - The producer may also run `rm`, `mv`, and `chmod`, with every path argument confined to its snapshot.
    Without them, a reading could promise a deletion that no agent could perform.
  - Cargo's `--ignored` and `--include-ignored` are denied to every role, so agents cannot start the live
    tests.

## D23 · Setup is a minimal POSIX bootstrap, then Rust

- **Decision:**
  - `./setup.sh`, about 30 lines, installs rustup only if `cargo` is missing, from a pinned `rustup-init`
    with its SHA-256 checked, and then hands off to `cargo run -p xtask -- setup`.
  - The Rust half checks the prerequisites (git 2.40 or later, a C linker, `curl`) and prefetches crates.
    It installs gitleaks, builds and adopts `athena` (D45), and links the hooks. It also checks, or
    privately installs, the pinned Claude Code, and confirms credentials with `claude auth status`.
  - Downloads use `curl`; Rust verifies checksums and extracts the archives.
- **Alternatives:**
  - (a) Require rustup to be preinstalled.
  - (b) A shell script for all of setup.
  - (c) Nix or a container image.
  - (d) An HTTP client in Rust for downloads.
  - (e) A live model call to check credentials.
- **Tradeoff:** One shell file exists as a listed exception.
  - (a) fails "a clean machine with one command".
  - (b) puts setup logic outside Rust.
  - (c) imposes a heavy tool.
  - (d) re-solves the proxy and CA handling that `curl` already does.
  - (e) is a model call outside any job.
- **Reversal cost:** Low.
- **Provenance:** Stated: one command, idempotent, pinned, credentials from the environment and never
  stored, setup in Rust (A4, bootstrap brief). Evidenced: E10, E20.
- **Bundle:** Shaped by *Dependency ownership*: one source of truth per version.
- **Revised** (review 1, M3, m1, m9, m10): prerequisite checks, crate prefetching, `claude auth status`, and
  build provenance were added. The gateway key is no longer checked, since Jev is out.

## D24 · CI is one thin GitHub Actions workflow with two jobs

- **Decision:** `ci.yml` runs on every push and pull request, and weekly. Both of its jobs check out the
  full history and run `./setup.sh --ci`:
  - `checks` runs `cargo run -p xtask -- ci`, which covers fmt, clippy, and tests;
  - `history` runs `athena check --history`, which covers secrets over every commit and the history
    invariants.

  Actions are pinned by commit SHA, the workflow has read-only permissions, and nothing is deployed.
- **Alternatives:**
  - (a) Logic in YAML.
  - (b) The official gitleaks action.
  - (c) Two workflows, which was revision 1's design.
- **Tradeoff:** One YAML file, listed as an exception. Option (b) needs a license for organization accounts
  and hides logic. Option (c) adds a file for no gain.
- **Reversal cost:** Low.
- **Provenance:** Stated: CI scans the full history; YAML only where a platform requires it. Inferred:
  thin YAML.
- **Revised** (review 1, M5): two workflows became one.

## D25 · Fixed token budgets for the contract and for each role's whole always-loaded set

- **Decision:**
  - The contract has eight rules, measured at 294 tokens, and a budget of 350.
  - Each role's always-loaded set also has a fixed budget: reader 1,800, producer 1,100, and verifier
    1,300.
  - The live measurement, by the delta method, is the authority. CI uses ⌈bytes/3.0⌉ as a tripwire for
    prose.
  - Every asset is compiled in and hashed into each invocation record.
- **Alternatives:**
  - (a) Load prompts at runtime from the repository.
  - (b) Budget only the contract, which was revision 1's design.
  - (c) Larger budgets.
- **Tradeoff:** Small budgets force every addition to be paid for.
  - (a) would let a stray edit change every agent without a new version.
  - (b) lets rules escape into frames (review 1, m4).
  - (c) blunts the discipline.
- **Reversal cost:** Low.
- **Provenance:** Stated: a fixed budget from day one, every addition paid for, and the defaults model held
  to the same discipline (A13; bootstrap brief). Evidenced: E8.
- **Revised** (review 1, m4, n5, Q5): budgets for each role's whole set were added, and the estimator's
  domain was stated.

## D26 · The defaults model is a versioned TOML file, read from the base commit

- **Decision:**
  - `defaults.toml` holds eleven defaults, each with an id, a statement, scope, and its source in `BRIEF.md`.
  - The reader loads all of them, within its set budget, and names the defaults that apply. Only those reach
    the briefs.
  - The file is a governance file: changing it is a job that lists it as a deliverable.
- **Alternatives:**
  - (a) Put the defaults in the contract.
  - (b) Start empty and learn only from interventions.
  - (c) JSON.
- **Tradeoff:** The seed is the designer's reading of the spec, so every entry cites its source.
  - (a) spends always-loaded budget on every job.
  - (b) wastes the written spec.
  - (c) is harder for people to edit.
- **Reversal cost:** Low.
- **Provenance:** Stated: an explicit, versioned defaults model (A2). Inferred: the seed entries.
- **Revised** (review 1, M1, m4): the model is read from the base commit and counted inside the reader's
  budget.

## D27 · Interventions are captured by mechanism wherever Athena can see them

- **Decision:** Three cases:
  - A one-line correction of a reading, or an answer to the reader's question, is recorded automatically
    as an intervention (`corrected-output` or `answered-question`, source `observed`).
  - Hand changes to governed content must be declared with an intervention record in the same commit
    (D39).
  - Everything else is recorded with `athena intervene`.
- **Alternatives:**
  - (a) Only user-reported interventions.
  - (b) Treat reading corrections as part of the normal flow.
- **Tradeoff:** Counts include cheap corrections. Never suppressing an intervention matters more than a
  low count, and the user has confirmed that a correction is an intervention.
- **Reversal cost:** Low.
- **Provenance:** Stated: what counts as an intervention, and never suppress one (O3). The user answered
  review 1's Q3.
- **Revised** (review 1, M2, m7): answered questions and declared hand work were added.

## D28 · Spend is bounded per invocation and per job, with a kill switch that reaches orphans

- **Decision:**
  - Every invocation passes `--max-budget-usd` and runs under a deadline. Termination is SIGTERM to its
    process group, then SIGKILL after 5 seconds.
  - Children get the parent-death signal on Linux.
  - Every process group is recorded in a registry. `athena stop` terminates all recorded groups, and any
    command reaps groups whose owner has died.
  - The core refuses an invocation whose cap would exceed the job's remaining budget, and refuses a sixth
    reading.
- **Alternatives:**
  - (a) Caps in prompts.
  - (b) A ledger checked after the fact.
  - (c) Polling for the kill switch only inside the running process, which was revision 1's design.
- **Tradeoff:** Claude Code's budget check is approximate within a turn, so the deadline backs it up.
  Option (c) cannot reach an agent whose parent died (review 1, M3).
- **Reversal cost:** Low.
- **Provenance:** Stated: spend is bounded, with a cap and a kill switch, both recorded (Settings).
  Evidenced: E16 (a group SIGTERM ended a run within 5 s).
- **Revised** (review 1, M3): the parent-death signal, the registry, and a `stop` that reaches every group
  were added.

## D29 · No transcripts; agent free text is redacted; personal data is gated

- **Decision:**
  - From the agents' stream, Athena keeps only usage, tool names, Bash command lines, and structured
    outputs.
  - Free text is redacted before recording: email addresses outside the allow-list, and gitleaks findings.
  - The gate refuses non-allow-listed email addresses in added lines.
- **Alternatives:**
  - (a) Redacted full transcripts.
  - (b) Transcripts kept outside the repository.
- **Tradeoff:** Without full trajectories, diagnosis is harder; transcript capture is left out until a
  failure needs it. Option (a) has a large redaction surface and grows the repository quickly.
- **Reversal cost:** Low.
- **Provenance:** Stated: never commit transcripts unredacted, or personal data. Evidenced: E3.
- **Revised** (review 1, m2, m5): Bash command lines are kept as evidence, and the added-line gate was
  added.

## D30 · Ids are time-sortable with a random suffix

- **Decision:** `YYYYMMDD-HHMMSS-xxxx`: UTC time, plus four Crockford base32 characters. The id gives the
  month shard. `bootstrap` is reserved.
- **Alternatives:**
  - (a) ULIDs.
  - (b) Sequence numbers.
  - (c) UUIDs.
- **Tradeoff:** Ids are slightly longer than sequence numbers. Option (b) needs coordination. Options (a)
  and (c) are hard to read.
- **Reversal cost:** Low.
- **Provenance:** Inferred.

## D31 · v0's model routing; Jev is left out of v0

- **Decision:**
  - The reader is Sonnet 5.5.
  - The reader picks the producer from {Haiku 5.5, Sonnet 5.5}, defaulting to Sonnet.
  - The verifier is Opus 5.5, which never produces in v0.
  - Jev is **not** part of v0.
- **Alternatives:**
  - (a) Jev as a shadow forecaster on every reading, which was revision 1's design.
  - (b) Jev as the primary forecaster.
  - (c) Opus for everything.
- **Tradeoff:** Revision 1 argued that every job run without Jev is calibration data lost. Review 1 showed
  that argument fails on the design's own data model:
  - Jev's input is built only from fields the reading record stores;
  - Jev never sees outcomes;
  - so a later job can compute Jev's forecasts for every v0 job, labeled as retroactive.

  Against that, Jev would add an HTTPS client, a credential, a driver and its fake, refusal handling, and a
  forecaster column. Option (b) would trust an uncalibrated forecaster. Option (c) costs several times more
  with no evidence that it pays.
- **Reversal cost:** Low. The retroactive forecasts are the pull-in plan: at the first calibration review,
  or with the first typed decision in the flow.
- **Provenance:** Stated: the starting routing (startup notes), and the user's default to leave Jev out
  (bootstrap brief, and its update). Evidenced: E12 (Jev works) and the review's argument. Inferred: the
  pull-in plan.
- **Revised** (review 1, M5, Q6): reversed, from including Jev as a shadow forecaster to leaving it out.

## D32 · The producer's self-report stays with its run and is never fed forward

- **Decision:** The producer's report includes mistakes, learnings, and desires (MLD). It is stored redacted
  in `candidate.json`, never given to the verifier, and never injected into a later context.
- **Alternatives:**
  - (a) No self-report.
  - (b) Feed past self-reports into later briefs.
- **Tradeoff:** Low-trust telemetry at a small output cost. Option (b) turns unverified self-commentary into
  instructions.
- **Reversal cost:** Low.
- **Provenance:** Inferred.
- **Bundle:** Adopted from *MLD: telemetry for the harness builder*.

## D33 · Non-Rust files are inventoried, and the inventory is enforced

- **Decision:** `docs/non-rust-files.md` lists every kind of non-Rust file with its reason, and a test fails
  on any tracked file that is neither Rust nor listed. Owner paths are outside Athena and exempt.
- **Alternatives:** (a) List the files in prose only.
- **Tradeoff:** A table to maintain, but no new kind of file slips in.
- **Reversal cost:** Low.
- **Provenance:** Stated: non-Rust files only where a platform requires them, each with its reason (A4).
- **Revised** (review 1, n3, M5): there is no `.gitleaks.toml` and no `schemas/` directory.

## D34 · A boundary test keeps Athena free of references to who runs it and where

- **Decision:** A test scans Athena's own files for terms that would tie Athena to the people or machine
  around it. It covers code, prompts, configuration, docs, setup, and workflows. The terms are assembled
  from fragments, so the test itself contains none of them. The scan excludes the user's verbatim spec,
  records, the license, `.gitattributes` declarations, and owner paths.
- **Alternatives:**
  - (a) Review by hand.
  - (b) A plain grep in CI.
- **Tradeoff:** A term list is a tripwire, not proof.
- **Reversal cost:** Low.
- **Provenance:** Stated: nothing in Athena refers to the people or tools that run it, or to the particular
  machine it runs on, and a test enforces that (bootstrap brief).

## D35 · The state of the work is derived from records, never hand-maintained

- **Decision:** `athena status [<job>]` computes everything from the records at `HEAD` and the state
  directory.
- **Alternatives:**
  - (a) A `STATUS.md` updated by jobs.
  - (b) A state file in the state directory.
- **Tradeoff:** Reading the state needs the binary, or reading the JSON directly. In return, the view
  cannot drift from the records.
- **Reversal cost:** Low.
- **Provenance:** Stated: a fresh context gets the state from the repository alone (bootstrap brief, item
  10; A11).

## D36 · Buy before build: the crates Athena adopts, and what it builds

- **Decision:**
  - Adopted:
    - `clap`;
    - `serde`, `serde_json`, and `toml`;
    - `schemars`, to generate the role output schemas at run time;
    - `jiff`;
    - `sha2`;
    - `getrandom`;
    - `regex`;
    - `anyhow` and `thiserror`;
    - `tempfile`;
    - `flate2` and `tar`, for setup;
    - `libc`, for process groups, signals, and `PR_SET_PDEATHSIG`;
    - `shell-words`, to tokenize commands in the hook, confirmed at build time.
  - Built, each too small for a dependency: the Crockford base32 encoder, the token estimator, the scoring
    rules, and the hook's command classifier on top of the tokenizer.
- **Alternatives:**
  - `chrono` or `time` instead of `jiff`.
  - `reqwest` or `ureq`, which are no longer needed now that Jev is out.
  - `tiktoken-rs`, which counts with another vendor's tokenizer.
  - `proptest`; deterministic grids suffice.
- **Tradeoff:** Each dependency is supply-chain surface. All are widely used, and all are pinned in
  `Cargo.lock`.
- **Reversal cost:** Low per crate.
- **Provenance:** Stated: buy before build, recording each call. Inferred: the selections.
- **Bundle:** Shaped by *Dependency ownership*.
- **Revised** (review 1, M5, m11): the HTTP client was removed, schemars is limited to run-time role
  schemas, and the shell tokenizer was added.

## D37 · Rust 1.97.0, edition 2024, pinned in `rust-toolchain.toml`

- **Decision:** The toolchain is pinned with clippy and rustfmt, the workspace uses edition 2024, and every
  build uses `--locked`.
- **Alternatives:**
  - (a) An unpinned stable channel.
  - (b) An older minimum supported version.
- **Tradeoff:** Upgrading is a deliberate change.
- **Reversal cost:** Low.
- **Provenance:** Stated: version-pinned setup (A4).

---

## D38 · No commit is judged by rules it changes

- **Decision:**
  - A job's configuration comes from its base commit. That covers the checks, caps, models, verifier,
    allow-lists, `[secrets]` allow-list, defaults model, and attribute rules.
  - `job.json` records the hashes of `athena.toml` and `defaults.toml`, and `go` re-reads if they changed
    since the reading.
  - Commit gates judge each commit by its parent's rules; the history secret scan uses `HEAD`'s.
  - Admission refuses a candidate that would change any path's `athena-*` classification. It also refuses
    a candidate that changes a governance file (`athena.toml`, `defaults.toml`, any `.gitattributes`,
    `.github/workflows/*`, `setup.sh`, `rust-toolchain.toml`) unless the accepted reading lists that file as
    a deliverable and an acceptance criterion names it.
- **Alternatives:**
  - (a) Read configuration from the working tree.
  - (b) Read it from the candidate.
  - (c) Leave it unspecified, as revision 1 did.
- **Tradeoff:** A legitimate change to the rules governs only later work. A gitleaks false positive is
  handled by committing its allow-list entry first.
  - (a) lets uncommitted edits decide, unrecorded.
  - (b) lets the producer choose its own checks.
  - (c) leaves both open (review 1, M1).
- **Reversal cost:** Low.
- **Provenance:** Stated: enforcement by mechanism (A8); verification before anything lands (A5).
  Evidenced: E17 (gitleaks relaxed by content); R5.
- **Bundle:** Shaped by *Maximize autonomy inside explicit authority*: one decision point, and authority
  that the worker cannot rewrite.
- **Revised** (user decision after review 2, M1):
  - The governance files now include the check tools' configuration: any `.cargo/config.toml` or
    `.cargo/config`, `rustfmt.toml` or `.rustfmt.toml`, `clippy.toml` or `.clippy.toml`, and the `[lints]`
    and `[workspace.lints]` tables of every `Cargo.toml`.
  - These are judged from the base commit. In the verifier's snapshot the working tree holds the base's
    versions, and a manifest new in the candidate inherits the workspace lints. The checks and the
    verifier's own commands therefore run under the rules in force at dispatch.
  - A test check that ran zero tests fails.
  - Evidence: review 2's R13, where each of the three files switched a failing check to a pass.

## D39 · Governed content changes only through a verified landing, a declared intervention, or the closed bootstrap range

- **Decision:** For every commit X, compare its governed-content fingerprint with its first parent's, under
  the parent's rules. If they differ, X must be one of the following:
  - (a) a landing commit, whose `Athena-Verified-Content` and `verification.json` (a pass) both match;
  - (b) a declared intervention, which carries `Athena-Intervention: <id>` and adds that record in the same
    commit, with any category;
  - (c) a bootstrap commit inside the range closed by `records/jobs/bootstrap/outcome.json`.

  A merge passes if its fingerprint equals one parent's; otherwise it needs (a) or (b). The rule is checked
  in the `commit-msg` hook, in the in-process gate, and over the history in CI. `athena intervene
  --with-staged` makes declaring hand work one command.
- **Alternatives:**
  - (a) Revision 1's trailer rule, which any existing job id satisfied.
  - (b) Branch protection, unavailable here.
  - (c) Restrict declared interventions to `did-work-themselves` and `fixed-environment`, as the review
    proposed.
- **Tradeoff:** Hand work by the user now takes one extra command. Without branch protection, CI detects a
  violating push but cannot prevent it. Option (c) would leave a hand fix of Athena's output
  (`corrected-output`) unable to declare itself, though it is the most valuable lesson to log.
- **Reversal cost:** Medium. History written under the rule stays valid.
- **Provenance:** Stated: nothing reaches main without passing verification, and every commit traces to
  its work (A5); doing work yourself is an intervention, and every intervention is logged (O3); every commit
  on main traces to accepted work (How I'll judge it). Evidenced: today's history satisfies the rule (R9,
  and every commit after the first carries `Athena-Job: bootstrap`).
- **Revised** (user decision after review 2, M2):
  - Writing `records/jobs/bootstrap/outcome.json`, which names the last bootstrap commit and carries the
    bootstrap's measured usage, is the final step of acceptance. It is written when the user asks, after
    their acceptance check.
  - Until then, `athena status` flags the open range.

## D40 · Usage is captured as it accrues, records are committed before their process exits, and costs reconcile

- **Decision:**
  - The driver reads stream-json and keeps per-message usage, de-duplicated by message id. It persists the
    running sums, so a killed invocation records lower-bound tokens and cost bounded by its cap.
  - Every command that creates records commits them before it exits: proposal, prediction, closing
    (including blocked jobs), and re-score commits.
  - `athena close` commits what an interrupted job left behind.
  - No model call happens outside a job.
  - `athena check` reconciles per-model costs with totals, requested-model tokens with usage, and job totals
    with invocation sums.
- **Alternatives:**
  - (a) Read usage only from the final result, which was revision 1's design.
  - (b) Keep uncommitted records in the state directory.
  - (c) Record calls outside jobs as a maintenance job.
- **Tradeoff:** More commits, and a stream parser to maintain. Option (a) loses the expensive, killed
  invocations. Option (b) loses records with a fresh environment. Option (c) is unnecessary once setup
  checks credentials without a model call (E20).
- **Reversal cost:** Low.
- **Provenance:** Stated: every model invocation recorded (bootstrap brief, item 5); the token records are
  complete and reconcile (How I'll judge it). Evidenced: E16, E20.

## D41 · v0 is cut to what its ten capabilities need

- **Decision:** Ten commands, two forecasters, and one workflow. The following are left out, each listed in
  §14 of the design with the evidence that would pull it back:
  - Jev;
  - the calibration view;
  - committed JSON Schemas and their freshness test;
  - the decision-log completeness test.
- **Alternatives:** Revision 1's larger surface: fifteen commands, three forecasters, two workflows, and
  committed schemas.
- **Tradeoff:** A few conveniences wait for evidence. The spine (three roles, prediction commits,
  fingerprinted landing, three-place gates, hermetic launch, append-only records, the reference forecaster,
  and the crate split) is unchanged.
- **Reversal cost:** Low per item.
- **Provenance:** Stated: keep v0 small; its size is a design choice (bootstrap brief). Inferred: review 1's
  analysis (M5), weighed and accepted by the method owner.

## D42 · Classes of work are a closed list in configuration

- **Decision:** `athena.toml` lists the classes: `code`, `tests`, `docs`, `prompts`, `config`, `build`,
  `design`, and `other`. The reader and `athena intervene` must choose from the list, and adding a class is
  a job.
- **Alternatives:** Free text, as in revision 1.
- **Tradeoff:** A new kind of work lands in `other` until a job adds its class. In return, synonyms cannot
  fragment the per-class metrics that immutable records would keep forever.
- **Reversal cost:** Low going forward. Already-recorded classes never change.
- **Provenance:** Stated: calibration and intervention trends per class of work (A2, O3, A13). Inferred
  from review 1 (m3).

## D43 · Behavior criteria must cite commands the verifier actually ran

- **Decision:**
  - The reader marks each acceptance criterion `content` or `behavior`.
  - The verifier may build and run the candidate in its snapshot.
  - Each `behavior` criterion's evidence lists exact commands. The core checks them against the Bash
    commands it saw the verifier run, and a criterion citing an unrun command counts as unmet.
- **Alternatives:**
  - (a) Trust the verifier's account.
  - (b) Rely on the capability's checks alone.
- **Tradeoff:** Exact matching may fail a verifier that ran an equivalent command; that rate is visible in
  the findings. Option (b) runs tests the producer may have written, which is not independent evidence.
- **Reversal cost:** Low.
- **Provenance:** Stated: user-facing behavior is verified by exercising it (A7). Evidenced: tool calls,
  with their commands, arrive in the stream (E16). Inferred from review 1 (m2).

## D44 · Owner-maintained paths: the `athena-readonly` attribute

- **Decision:** Paths marked `athena-readonly` in committed `.gitattributes` are visible in snapshots, so
  agents read them. They are outside the governed content, and no job may change them. Edits by the owner
  need no intervention record. In this repository, `BRIEF.md` and `LICENSE` are marked this way.
- **Alternatives:**
  - (a) Treat the user's spec as governed content, so that the owner's edits must be declared as
    interventions.
  - (b) Mark it `athena-exclude`, which hides it from the reader.
- **Tradeoff:** One more attribute for owners to learn.
  - (a) would count requirement changes as interventions and let a job edit the spec, which `BRIEF.md` says
    only its owner changes.
  - (b) would hide the standing spec that Athena is meant to read.
- **Reversal cost:** Low.
- **Provenance:** Stated: "requirement changes come only from" the owner (`BRIEF.md` header), and Athena
  may read the brief as its user's standing spec. Inferred: the attribute, while answering review 1's M2.

## D45 · The adopted binary names its source commit, and dirty builds are not adopted by default

- **Decision:** Setup builds the binary with its source commit and a dirty flag embedded. It refuses to
  adopt a build from a dirty tree unless `--allow-dirty` is given. Every invocation and job record carries
  the version, the commit, and the flag.
- **Alternatives:** (a) Adopt whatever the checkout holds, as revision 1 did.
- **Tradeoff:** Adopting a local experiment takes one flag. In return, every agent's context can be
  reconstructed from the records and the source tree they name.
- **Reversal cost:** Low.
- **Provenance:** Stated: versions are adopted deliberately (A3). Inferred from review 1 (m9).

## D46 · Each job builds in its own directories

- **Decision:** The capability's environment points `CARGO_TARGET_DIR` at directories inside the job's own
  state directory, one per side: `jobs/<id>/target/produce` and `jobs/<id>/target/verify`.
  - Nothing is shared across jobs.
  - The directories are removed when the job closes.
  - Setup prefetches dependency sources with `cargo fetch --locked`.
- **Alternatives:**
  - (a) Persistent caches per side, shared across jobs, which was revision 2's design.
  - (b) One cache for everything.
  - (c) A content-addressed compiler cache such as sccache.
- **Tradeoff:** Every job compiles its workspace and dependencies cold, so jobs take longer.
  - (a) and (b) let one job's checks run another job's binaries, because cargo judges freshness by
    modification times. Review 2's R14 showed a job's `cargo test` recompiling nothing and running the
    other job's tests.
  - (c) keys outputs by content and would be safe and fast, but it is a new dependency. It is left out until
    cold builds are shown to dominate job time or cost.
- **Reversal cost:** Low.
- **Provenance:** User decision after review 2 (N1): correctness over speed. Evidenced: review 2's R14.

## D47 · Nested runs use the fake driver and a scratch repository

- **Decision:** The core sets `ATHENA_NESTED=<outer job id>` in every agent environment and every check
  environment. When an `athena` process sees the marker, three things change:
  - it uses the fake agent driver, and the real Claude Code driver refuses to start;
  - it clones the repository it was started in into a scratch directory, and works there with a scratch
    state directory, never committing into the original;
  - the live tests refuse to run.

  Separately, the hook denies cargo's `--ignored` and `--include-ignored` to every agent.
- **Alternatives:**
  - (a) Record the path as a residual: narrow "every invocation is recorded" to Athena's own invocations,
    and have the reader mark criteria about model-calling commands `content`.
  - (b) Strip credentials from the agent environment.
  - (c) Forbid running `athena` inside a job.
- **Tradeoff:** A nested run exercises the candidate's real command-line behavior, but with fake agents, so
  it cannot show how real agents would behave under the change. That remains the job of live tests run by
  a person, and of later jobs.
  - (a) leaves money unrecorded and behavior criteria unpassable.
  - (b) cannot work, because authentication lives in a file under `HOME`, and the agents need `HOME`.
  - (c) makes changes to Athena's own commands unverifiable by exercise.
- **Reversal cost:** Low.
- **Provenance:** User decision after review 2 (N2): no model call may happen outside a job's records and
  budget. Evidenced: review 2's R15, a nested `claude` inside a snapshot authenticates. Stated: every model
  invocation is recorded (bootstrap brief, item 5).

---

## Entries added while building v0

The entries below record what building v0 taught. Each supersedes or amends an accepted entry, which is
left as it was. Their provenance is "build", with the evidence that prompted them.

## D48 · Secret scans read commits through the git directory; staged changes are scanned as a commit

- **Decision:** Amends D17. gitleaks scans only commits, through the repository's git directory, from an
  empty working directory, and with git's location variables removed from its environment. The
  `pre-commit` hook builds a temporary commit object from the index (never on any branch) and scans
  `HEAD..<that commit>`. Redaction runs `gitleaks stdin` from the same empty directory. Admission also scans
  the candidate commit, so a secret fails a job before verification.
- **Alternatives:**
  - (a) gitleaks' own `--pre-commit --staged` mode on the work tree, as first designed.
  - (b) Refuse to run when a `.gitleaksignore` exists in the work tree.
- **Tradeoff:** A temporary commit object per hook run; it is never referenced and is collected with other
  loose objects. (a) lets a `.gitleaksignore` in the work tree exempt a finding whatever
  `--gitleaks-ignore-path` names, so a commit could still carry its own exemption. (b) is fragile: the file
  only needs to exist when the scan runs.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Evidenced: E21, found by `content_cannot_relax_the_scan`.

## D49 · Nested runs copy the working tree, can keep a named scratch, and skip the checks

- **Decision:** Amends D47.
  - Under the marker, Athena copies the working tree it was started in (tracked and untracked files, not
    ignored ones) into a fresh scratch repository, rather than cloning it, so a producer can exercise its own
    uncommitted edits.
  - `ATHENA_SCRATCH_DIR`, when set, names a scratch directory that persists across nested commands, so a
    nested `ask` and a later nested `go` see the same repository. Without it, every command gets a fresh
    copy.
  - A nested `go` records its checks as not run: the outer job runs its own checks on the same tree, and
    running them again inside would recurse.
- **Alternatives:** (a) `git clone`, as first designed, which sees only committed content, so a producer
  exercising Athena would test the base rather than its change. (b) Run the checks in nested runs too.
- **Tradeoff:** A nested `go` shows the command-line flow, not the checks; those stay the outer job's.
  (b) would run the whole test suite inside a check inside a verifier, without bound.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Inferred from the producer's case, which D47 did not consider.

## D50 · Reserved domains may appear in committed content; history uses the tip's allow-lists

- **Decision:** Amends D29. `[personal_data] email_allow` entries now have three forms: a local part
  (`noreply@`), a domain (`@example.com`), and a domain suffix (`.invalid`). The reserved top-level domains
  of RFC 2606 (`.invalid`, `.test`, `.example`) are allowed, since no person can own an address there. The
  full-history check takes both sensitive-content allow-lists, `[secrets]` and `[personal_data]`, from the
  tip, as D17 already did for secrets; commit-time checks still use the parent's.
- **Alternatives:** (a) Rewrite the address in the code and accept a history that fails its own check.
  (b) Exempt the commit.
- **Tradeoff:** Allow-listing at the tip can excuse an old commit retroactively. That is the point for false
  positives, and the change is itself a reviewed governance commit.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Evidenced: the first history check flagged the snapshot identity
  `snapshot@noreply.invalid` committed earlier in the bootstrap.

## D51 · Every reading is committed before Athena waits for the user

- **Decision:** Refines D40. `ask` and `correct` commit the request, the reading, its corrections and
  interventions, and the reader's invocation before showing the prompt. Pressing Enter then makes the
  prediction commit. The proposal commit is therefore the commit that introduced the accepted reading, and
  `Athena-Prediction` names it. Every closing commit of a dispatched job carries `Athena-Prediction`, not
  only landings.
- **Alternatives:** Commit the proposal only when the command exits with the job still proposed, as first
  designed.
- **Tradeoff:** One more commit per interactive job. The alternative loses the reader's invocation record
  if the user interrupts the prompt with Ctrl-C, and the brief requires every model invocation recorded.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Inferred; stated requirement: every model invocation is recorded (brief, item 5).

## D52 · A blocked job closed later records `outcome-abandoned.json`; budget overruns refuse dispatch

- **Decision:** Refines D06 and D28.
  - An outcome that follows `blocked` is named after its state: `outcome-landed.json` when the retried
    landing succeeds, `outcome-abandoned.json` when the job is closed instead. Records stay append-only.
  - When the producer's and verifier's caps together exceed what is left of the job's budget, `go` refuses
    to dispatch and leaves the job proposed, rather than recording a failure; nothing was spent.
  - A landing that the in-process gate refuses closes the job as `failed`, with all its records.
- **Alternatives:** (a) Write the abandonment into `outcome-landed.json`, whose name would then mislead.
  (b) Record a failed outcome for a dispatch that never happened.
- **Tradeoff:** One more file name in the layout.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Inferred.

## D53 · Agents get the machine's network settings, when set

- **Decision:** Amends D04's environment allow-list. `env_pass_if_set` includes the standard proxy and
  certificate variables (`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY` and their lowercase forms,
  `NODE_EXTRA_CA_CERTS`, `SSL_CERT_FILE`, `SSL_CERT_DIR`, `CARGO_HTTP_CAINFO`) and Claude Code's endpoint
  variable, `ANTHROPIC_BASE_URL`. Each passes only when set, and none carries context.
- **Alternatives:** Leave them out, which breaks agents on any network that requires a proxy.
- **Tradeoff:** More names on the list. They configure the network, not the agent.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Inferred.

## D54 · Machine paths are replaced with placeholders before anything is recorded

- **Decision:** Amends D29. Besides secrets and private addresses, redaction replaces the job's state
  directory with `<job>`, the repository's state directory with `<state>`, and the home directory with `~`,
  in agent free text, commands run, denials, and check output.
- **Alternatives:** Record paths as they are, since they are not secrets.
- **Tradeoff:** A recorded path no longer says exactly where a file was on the machine; the placeholder
  says which directory it was under, which is what a reader needs.
- **Reversal cost:** Low.
- **Provenance:** Found while building. Evidenced: the first acceptance jobs recorded clippy's output with
  absolute paths under the home directory, which on a user's machine names the user.

---

## Entries added in the fix round after acceptance verification 1

The entries below answer the first independent acceptance verification,
[`design/reviews/athena-v0-acceptance-1.md`](design/reviews/athena-v0-acceptance-1.md). The user required
its two major findings and four of its minor ones fixed before v0 is accepted, a decision on the escape it
found, and every deferred finding visible in `athena status`. Each entry's provenance names its finding.

## D55 · A landing commit carries its whole job

- **Decision:** Amends D39's case (a). A commit that changes governed content is a landing only when it
  carries `Athena-Job`, `Athena-Prediction`, and an `Athena-Verified-Content` equal to its fingerprint, and
  the job's records show a job that was read, dispatched, and independently verified:
  - the job's `job.json` is on the parent commit, and the prediction commit is an ancestor that added the
    accepted reading;
  - the commit adds the job's `candidate.json`, `verification.json`, and a landed `outcome.json`, or, after
    `blocked`, its `outcome-landed.json`;
  - the candidate and the verification both hold the commit's fingerprint, and the final verdict is pass;
  - the verifier invocation that the verification names is recorded, succeeded, used the job's verifier
    model, and reconciles, and so do the outcome's totals.

  The rule runs in the `commit-msg` hook, in process, and over the history.
- **Alternatives:** (a) Sign records with a key kept outside the repository, so only Athena can write
  them. (b) Keep the one-record rule and rely on review.
- **Tradeoff:** A forger who fabricates every record of a job across two commits, with usage that
  reconciles, still passes. That is deliberate forgery of many records rather than a hand change dressed
  up with one file, and the forged job shows in `athena status` like any other. (a) would close that too,
  but a key on one machine cannot be checked in CI, and v0 has no key management.
- **Reversal cost:** Low.
- **Provenance:** Acceptance verification 1, M1: a hand change with a copied `verification.json` and two
  trailers passed both hooks and the history check (probe 2j). Evidenced by
  `a_hand_written_verification_does_not_make_a_landing`.

## D56 · Scores and outcomes follow from the job's own records

- **Decision:** Amends D06 and D14. The gate judges every record a commit adds to a job against the rest
  of the job:
  - a score must equal the score that the job's records in that commit imply, recomputed with the function
    Athena writes scores with (`athena_score::score_records`), under the rules the score names;
  - a re-score must name an intervention that the same commit adds for the same job;
  - a landed outcome comes only with a valid landing commit of its job (D55), and an outcome after
    `blocked` needs the job's blocked outcome on the parent commit.

  So that Athena and the gate always agree, Athena now computes a closing score from the records on the
  commit the closing commit is built on, once per attempt. An intervention recorded while a job runs
  therefore resolves its `user_intervenes` event, which the closing score used to miss.
- **Alternatives:** (a) Check only that a re-score's cause exists. (b) Make `athena status` ignore records
  it cannot explain.
- **Tradeoff:** Old scores must keep equaling their recomputation, so a change to the scorer needs a new
  rules version that the gate still recomputes, or the history check flags every earlier score. That is
  intended: a score is derived data, and a proper scoring rule is only as good as the record of its inputs.
  (a) leaves forged numbers in place. (b) hides a forgery instead of refusing it.
- **Reversal cost:** Medium.
- **Provenance:** Acceptance verification 1, M2: a hand-written re-score with perfect forecasts, and a
  hand-written `outcome-landed.json` for a rejected job, passed every gate and changed `athena status`
  (probes 3j and 3k).

## D57 · What agents and checks run stays inside the job

- **Decision:** Amends D04 and D47.
  - Agent and check environments set `ATHENA_STATE_DIR` inside the job's own directory, so an Athena that
    an agent or a check runs (always nested, D47) keeps its kill switch, locks, and scratch repositories
    there.
  - `xtask setup` refuses to run under the nested-run marker: it installs tools and adopts the binary that
    the git hooks run, for the whole machine.
  - After every agent run, the driver removes the folder Claude Code names after the snapshot, where it
    spills large tool results despite `--no-session-persistence`, and the run's empty session folder.
  - Setup takes the path of the binary it built from cargo's own messages, so `CARGO_TARGET_DIR` works,
    and a stale `./target` is never adopted.
- **Alternatives:** (a) Point agents' `HOME` or `CLAUDE_CONFIG_DIR` at a scratch directory, which loses
  Claude Code's login. (b) Deny `cargo run -p xtask` in the hook. (c) Redirect `ATHENA_HOME` too.
- **Tradeoff:** The cleanup depends on how Claude Code 2.1.296 names its folders, which the live tests
  re-check at each upgrade. (b) is a second layer that the refusal makes unnecessary. (c) would hide the
  pinned gitleaks from Athena's own tests, which need it, and once setup refuses, nothing nested writes to
  the install directory.
- **Reversal cost:** Low.
- **Provenance:** Acceptance verification 1, m4 and m5. The verification noted that fixing m5 alone would
  let an agent's `cargo run -p xtask -- setup` replace the user's adopted binary, so both are fixed
  together.

## D58 · Deferred work is a queued request

- **Decision:** `athena ask --later "<request>"` records the request and nothing else: no reading and no
  model call. The job is `queued`, which `athena status` lists as open work with the command that picks it
  up. `athena go` reads it first and then runs it like any proposal; `athena close` drops it. Every finding
  of acceptance verification 1 that the fix round defers is queued this way, so a fresh context finds it in
  `athena status`, from the records alone.
- **Alternatives:** (a) A new record type for deferred findings, with its own rule for closing them.
  (b) Read each deferred request now with `athena ask`, leaving it proposed. (c) The review document alone.
- **Tradeoff:** A queued job has no forecasts until it is read. That is right: a forecast made long before
  the work would score the wait, not the reader. (a) needs its own closing rule, while a queued request
  closes the way every job does. (b) spends a reading per finding, and scores readings that wait for weeks.
  (c) is what the user ruled out: capability 10 covers open work.
- **Reversal cost:** Low.
- **Provenance:** User decision for the fix round, point 3: every deferred finding must be visible to a
  fresh context through `athena status`, from Athena's own records, by the smallest mechanism that does it.

## D59 · The verifier checks documented instructions against the repository's own rules

- **Decision:** The change capability's verifier section gains one sentence: for docs, check every
  instruction against the repository's own checks and commit rules, since an instruction a reader would
  follow must work here. The escape that prompted it is recorded as an intervention (category `other`,
  class `docs`) with no job, since that job ran in a throwaway clone.
- **Alternatives:** (a) Change nothing in v0 and queue the improvement. (b) Ask the reader to keep the
  criteria that a correction does not address.
- **Tradeoff:** About 35 tokens of verifier context, within its budget, for the observed class of false
  pass (design §16.1). (b) treats the cause in the reading, but one escape is too little evidence to change
  how corrections are read.
- **Reversal cost:** Low.
- **Provenance:** Acceptance verification 1, m1: the verifier passed a CONTRIBUTING.md telling outside
  contributors to open pull requests of hand-made commits, which the commit gate and CI's history check
  refuse. The correction that made reading 2 had dropped the criterion covering Athena's workflow.
