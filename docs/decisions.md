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
  Claude Code is pinned at 2.1.299, and agents never spawn further agents.
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
- **Revised** (review 1, B1, m2): the verifier works in a snapshot, and its exercise is made mechanical (D43).
  Later (PR #59 false claim): the prompt says the snapshot lacks records/, owner files and history, so runs there are snapshot results only, never real-repository results; a test pins the wording.

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
  - Each role's always-loaded set also has a fixed budget: reader 2,000, producer 1,100, and verifier
    1,300. The reader budget was 1,800 until D93, then 1,930 until the `behavior-evidence` default.
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
  - `defaults.toml` held eleven defaults when this was written (twelve now, after `root-cause`), each with an id, a statement, scope, and its source in `BRIEF.md`.
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

- **Decision:** A test fails on any tracked file that is code in another language, found by extension or a
  `#!` first line. Data and documents are allowed anywhere. A short named allow-list (`setup.sh` and the CI
  workflow), each with its reason, is described in `docs/non-rust-files.md`. Owner paths are outside Athena
  and exempt.
- **Alternatives:**
  - (a) List the files in prose only.
  - (b) Refuse any file kind not in a ledger (the first version; it failed a JSON test fixture, which is
    data and not code).
- **Tradeoff:** A new kind of code could use an extension the list lacks, but the shebang check and a short
  list cover the common cases; data needs no upkeep.
- **Reversal cost:** Low.
- **Provenance:** Stated: non-Rust files only where a platform requires them, each with its reason (A4).
- **Revised** (review 1, n3, M5): there is no `.gitleaks.toml` and no `schemas/` directory.
- **Revised** (2026-10-14, common sense over rigid rules): the check aims at the intent of A4 (all code is
  Rust) instead of listing every file kind.

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
    cold builds are shown to dominate job time or cost. (Superseded by D79: the sccache exclusion no longer
    holds; per-job target directories stay.)
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

---

## Entries added after acceptance verification 2

## D60 · A history exemption names one earlier commit, in full; request outcomes carry totals

- **Decision:** Amends D39's exemption list and D06.
  - Every `[history] exempt` entry in `athena.toml` must be a full commit id (40 hexadecimal characters,
    or 64 in a SHA-256 repository); anything else makes the configuration fail to parse. The invariant
    matches entries exactly, never as prefixes.
  - A commit that adds an entry must already have the named commit in its history, so an exemption can
    only name the past, such as a commit that predates Athena.
  - The history check reports a tip whose configuration does not parse as a violation, and judges with
    the built-in configuration, instead of stopping; `athena check` and `athena status` open such a
    repository, while the commands that run agents still refuse it.
  - An outcome of a request job must record its totals; only the bootstrap's outcome has none.
- **Alternatives:** (a) Require only full-length ids, since a future commit's full id cannot be ground to
  match. (b) Drop exemptions, and with them the history before Athena.
- **Tradeoff:** (a) alone would leave an entry naming a commit that does not exist; requiring the past costs
  one ancestry query per added entry and states the rule directly. (b) would make every repository with
  earlier history fail its own check.
- **Reversal cost:** Low.
- **Provenance:** User decision after acceptance verification 2, N1: an entry `"0"`, added by a declared
  intervention, hid a later hand edit whose commit id began with 0 from `athena check --history` (11
  retries). The totals rule follows the same verification's note that an outcome with `"totals": null`
  skipped reconciliation. Evidenced by `a_short_history_exemption_does_not_hide_a_hand_change`.

## D61 · The history check reads the exempt list from the tip

- **Decision:** Amends D60. `athena check --history` takes `[history] exempt` from the tip's
  `athena.toml`, as it does `[secrets]` and `[personal_data]`. If the tip's configuration does not parse,
  the built-in list applies. Per-commit checks (pre-commit and similar) have no tip and keep judging a
  commit by its parent's configuration. D60's validation is unchanged: full ids only, and a new entry must
  name a commit already in history, so an entry still cannot cover a commit made after it.
- **Alternatives:** (a) Keep the parent's list. (b) Read the list from the commit that adds the entry.
- **Tradeoff:** (a) meant no entry ever took effect, since an entry is added after the commit it names, so
  an adopter could not excuse history from before Athena. (b) needs a second lookup per commit for the
  same result.
- **Reversal cost:** Low.
- **Provenance:** Acceptance review 3, probe 4: an entry added after the fact had no effect. Evidenced by
  `an_exemption_added_later_excuses_the_commit_it_names` and
  `an_exemption_does_not_cover_a_later_commit_or_one_not_in_history`.

## D62 · The producer brief says what the verifier can see

- **Decision:** Amends D43. `producer_brief` in `crates/athena-context/src/briefs.rs` gains a "What the
  verifier sees" section: the verifier sees only the candidate diff in a fresh snapshot; evidence must cite
  commands it can run itself, and the producer's own `git status` and `git diff` runs do not count; commit
  hashes made during verification do not exist in the real history, so none may be pinned or cited in code,
  docs, or design-revision text. D43's rule in `crates/athena-core/src/verify.rs` is unchanged.
- **Alternatives:** (a) Loosen D43 for evidence the producer reports. (b) Leave the guidance out and rely on
  the verifier's rejection.
- **Tradeoff:** (a) lets a verifier pass on evidence it never saw. (b) was the cause: the producer's brief
  never said what the verifier sees, so the design revision (job 20261012-073105-mj6d) failed twice, once
  citing `git status` and `git diff` runs the verifier never saw (D43), and once pinning BRIEF.md to a commit
  that exists only in the synthetic verification history. The section costs about 100 tokens of producer
  context.
- **Reversal cost:** Low.
- **Provenance:** User report on job 20261012-073105-mj6d. Enforced by
  `producer_briefs_say_what_the_verifier_can_see`.

## D63 · Cited commands match whole simple commands inside what the verifier ran

- **Decision:** Amends D43. `judge` in `crates/athena-core/src/verify.rs` splits each command the verifier
  ran, and each command it cites, into simple commands on `&&`, `||`, `;`, `|`, and newlines, never inside
  quotes, trims and collapses whitespace outside quotes, and drops a trailing `2>&1`. A cited command is seen when each of its simple
  commands equals a simple command the verifier ran. A different argument, filter, or package is still
  unseen, and a command never run still makes a behavior criterion unmet with the finding "cited but not
  seen running". Trailing filters such as `| tail -n 30` no longer hide the command before them.
- **Alternatives:** (a) Keep exact whole-string matching and tell verifiers to cite the full line (the
  prompt already says "exactly as you ran them"). (b) Match by prefix or substring. (c) Accept citations the
  verifier did not run.
- **Tradeoff:** (a) keeps failing verifiers that ran the command inside a chain or pipe and cited the piece.
  (b) would accept `cargo test` as evidence for `cargo test --all`, or text inside a quoted argument. (c)
  defeats D43. Splitting is a purpose-built splitter, not a shell parser: it can only make more citations
  seen when their pieces are whole commands that ran, so an unusual quoting form errs toward unseen.
- **Reversal cost:** Low: one function and its tests.
- **Provenance:** Jobs 20261012-075010-86ja and 20261012-073105-mj6d were rejected on D43 findings even
  though the producers' cited test commands (ending in `2>&1`, which the policy allows) were run. The job
  records are not available to this change, so the cause is inferred from the code: exact whole-string
  matching fails when the verifier ran the command chained or piped and cited a piece. D62 changed the
  producer brief and so could not address a matcher on the verifier side. Enforced by
  `cited_commands_match_whole_simple_commands_inside_what_ran` and
  `cited_commands_with_other_arguments_or_never_run_stay_unseen`.

## D64 · The study capability M1 design: eight decisions after review 1

- **Decision:** In [`docs/design/study-capability-m1.md`](design/study-capability-m1.md):
  1. Evidence rules (and the quote, pin, and card-completeness rules) are non-waivable; model rules are
     waivable only by a user-written reason that cites a user intervention, shown on the page.
  2. Containment of the run step is part of the capability, and no run starts without a boundary; the
     fallback is an explicit read-only downgrade.
  3. A per-run cap is enforced before the run (Athena's kill switch plus a provider-side spend limit on
     the credential); cost recorded afterwards is never the cap.
  4. The four judgment rules become structural checks (`contradicts` link, `quote` field, provenance-only
     `authors-say` field, typed `Quantity`); the rest are listed as verifier duties.
  5. The model gains model-decided guards, unhandled failures, retry changes, and no-answer rules; parallel
     regions and fan-out are named limits of M1.
  6. Every `BRIEF.md:298` stream is listed per run as native, added, or absent; absences are findings.
  7. The Pages design is a required part of M1 and is written and reviewed before the build job.
  8. "Typed records now, export to a checker later" is recorded as alternative A2-D.
- **Alternatives:** The other option in each of the review's eight questions: allow waivers of evidence
  rules with a reason; defer containment and study only trusted systems; accept cost recorded after the
  fact; relabel all judgment rules as verifier duties; extend the model for parallel regions and fan-out
  now; require every stream; write the Pages design after the build; leave A2 as it was.
- **Tradeoff:** Each choice is the more conservative option where `BRIEF.md` does not settle it, at the cost
  of more build work before a first study (containment, redaction, a spend limit) and a narrower first
  model. The design needs a re-check (review 2) before building.
- **Reversal cost:** Low now, because only the design changes. See the design's §7 for costs after a build.
- **Provenance:** Stated for decisions 2, 3, 6 and the evidence rules: `BRIEF.md:110-111`, `BRIEF.md:298-299`,
  `BRIEF.md:320-321`. Inferred, as the designer's choice of the conservative option, for 4, 5, 7 and 8. The
  user asked for all eight to be decided without being asked again.

## D65 · The study capability's first build is the validation core only

- **Decision:** The first build of the studying capability is the crate `athena-cap-study`: the typed
  `study.*/1` records, the `study.model/1` model, the eighteen validation rules of the design's §5.4 as pure
  functions, a `Capability` impl with prompts, and a git-lookup trait with an in-memory fixture. Containment,
  the run step, replay, the Pages explainer and publish step, and registration in the CLI wiring are deferred
  to later jobs. The review-2 changes (R2-m1 to R2-m3) are applied to the design in the same change. The
  `Capability` trait is unchanged; whether it fits a multi-step study job is left open until the run step is
  built. The records are not yet part of `AnyRecord` or the gate.
- **Alternatives:** (a) Build everything in the design in one job. (b) Build the crate with registration, so
  `athena ask` can start a study. (c) Fold the rules into `athena-cap-change`.
- **Tradeoff:** (a) is a large change that is hard to verify and mixes untrusted-code containment with pure
  rules. (b) would let a study start before containment exists, against D64 decision 2. (c) is rejected in
  the design's A1. The cost of this scope is that nothing runs yet: the rules are tested on fixtures, and
  their fit to real studies is unmeasured.
- **Reversal cost:** Low: a new crate with no dependents. Record shapes are the part that costs more to
  change once studies exist; they gain a new schema version then.
- **Provenance:** Inferred. The scope follows the design's order (§2, §8) and the request to build the
  capability again after an earlier attempt was not admitted; the earlier attempt's records are not
  available to this change, so why it failed is not known.

## D66 · Proposal: remedy for the history check failing on the multi-job squash ee4b0dd

- **Decision:** Proposed, awaiting the user's choice; nothing is implemented by this entry. `athena.toml`,
  workflows, code, and tests are unchanged. Recommendation: **(c) stop squash-merging pull requests that
  carry more than one job, and rewrite nothing**, with **(a) one `[history] exempt` entry for ee4b0dd** as
  the way to turn main's check green once the user accepts that this commit's content is excused.
  Implementation of either is a follow-up job.
  - *Cause (read from code, not from the failing output, which this change did not have).* `landing` in
    `crates/athena-gate/src/invariant.rs` (lines 253-259) collects the jobs whose closing records the commit
    adds (`closing_jobs`) and refuses when there is more than one: "a commit adds closing records for more
    than one job". Design §7.3 ("Merge, squash, and rebase merges", `docs/design/athena-v0.md`) describes a
    squash as one commit that adds the code and every record of *the* job, checked as one landing. A squash
    of PR #8 that carries several jobs' records is therefore refused. Even without that line, lines 340-350
    require each job's `candidate.json` fingerprint and `verification.json` verified fingerprint to equal the
    fingerprint of the whole landed tree; with several jobs, each verified only its own candidate, so at most
    one could match. The history check walks every commit, so the failure stays on main until the commit is
    excused or no longer judged.
  - *Mechanism of (a).* `check` returns no violations for a commit in the tip's `[history] exempt` list
    (`invariant.rs` lines 92-100). D60 requires a full 40-character id naming a commit already in history;
    D61 reads the list from the tip, so an entry added now excuses ee4b0dd. The full id is not in the
    snapshot this change was made from, so it must be read from `git log` on the real repository.
- **Alternatives:**
  - (a) Exempt ee4b0dd via `[history] exempt`. One line of configuration, no code. The commit's content then
    skips the whole invariant, including "nothing reaches main unverified", for every job in it. The jobs'
    own records remain in the tree and still get the per-record checks.
  - (b) Teach `landing` to accept a commit that lands several jobs when each job's records are complete.
    This needs a per-job content rule, because a combined tree cannot equal each job's fingerprint: for
    example, verify each job's candidate as a patch against the parent and require the union to equal the
    landed tree, plus an order rule between jobs. That is a new proof in the gate (design §7.3 and a new
    decision, new tests in `crates/athena-gate/src/landing_tests.rs`), and it weakens "one verified
    candidate is the landed content".
  - (c) Stop squashing multi-job PRs: land them by merge commit or rebase merge (§7.3 says both are checked
    per commit), or keep one job per PR. No history is rewritten, so ee4b0dd stays on main and still needs
    (a) to pass; (c) alone prevents a repeat.
  - (d) Rewrite main to split the squash. Rejected: out of scope, destructive, and breaks every clone.
- **Tradeoff:** (a) is the cheapest and fully reversible but is a hole for one commit whose verification
  cannot be checked by the gate; it applies to this one commit only (D60: a full id cannot cover a later
  commit). (b) fixes the class of failure but is the largest change, in the part of the gate that guards
  the main property, for a case the design chose not to support ("one commit... of the job"). (c) costs a
  merge-strategy habit and a repository setting, and is enforced by nothing unless the host restricts
  allowed merge methods. Choosing (c) with (a) leaves the invariant as strict as it is and fixes main now.
- **Reversal cost:** (a) low: delete the list entry. (b) high: a new gate rule that later history depends
  on. (c) low: a setting.
- **Provenance:** Inferred. The user reported that main's history check has failed since PR #8 was
  squash-merged as ee4b0dd with many job landings. The failing output and the full commit id were not
  available to this change, so the cause is read from `invariant.rs`, design §7.3, D60, and D61, and the
  existing squash tests in `crates/athena-gate/src/landing_tests.rs` (all single-job); no command was run
  to reproduce the failure.

## D67 · The user chose option (a) of D66: ee4b0dd is exempt from the history check

- **Decision:** Resolves D66. The user chose (a): the squash commit ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef
  (PR #8) is added to `[history] exempt` in `athena.toml`, the full 40-character id. Its content is excused
  from the governed-content invariant, and its jobs' own records still get the per-record checks. The entry
  is one line in `athena.toml`, with a comment that gives this reason. No code, tests, or workflows change.
  - Option (c), stop squash-merging multi-job pull requests, is **not yet adopted**. Nothing enforces it,
    and it is a repository setting, so it needs a separate decision and a follow-up.
  - Options (b) and (d) of D66 are not chosen and stay open.
- **Alternatives:** (b) Teach `landing` to accept a multi-job commit, a new gate rule. (c) Stop squashing
  multi-job pull requests. (d) Rewrite main to split the squash.
- **Tradeoff:** The exemption is the cheapest and fully reversible choice. Its cost is a hole in the
  invariant for one commit: "nothing reaches main unverified" is not checked for ee4b0dd. D60 limits the hole
  to that commit, since a full id cannot cover a later one, so any later multi-job squash needs its own entry
  or a different merge method.
- **Reversal:** Delete the `ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef` entry and its comment from
  `[history] exempt` in `athena.toml`. The history check then judges ee4b0dd again and fails on main until
  that is resolved by (b), (c), or (d). Reversal cost: low.
- **Provenance:** User decision on D66, given as "(a)". The full id is the one the user supplied. The cause
  is as D66 records it, read from code and not reproduced. Enforced by the existing history check and the
  `[history] exempt` validation of D60 and D61. D60 also requires the named commit to be in history already;
  the user's report places ee4b0dd on main, but this change did not check that against the real repository.

## D68 · An exemption also excuses the job records its commit adds

- **Decision:** A commit on `[history] exempt` is skipped by the job-records check as well as by the
  governed-content invariant. The records the exempt commit adds, such as a landed outcome and a score, are
  not flagged. One helper, `invariant::is_exempt`, decides this for both checks, using the tip's list in a
  history walk and the parent's list in a per-commit check. The exemption still covers only that one commit:
  records added by a later commit are judged as before.
- **Why:** D67 exempted the squash commit named there, but the history check kept reporting per-record
  violations for the jobs whose records arrived in it: "outcome says landed, but this commit is not a valid
  landing" and "score.json does not follow". Both come from the job-records check, which judges a landed
  outcome by the invariant's landing rule and a score by the job's records in the same tree. A commit the
  invariant excuses cannot satisfy either, so excusing the content but not its records left D67 ineffective.
  This also corrects D67's statement that the jobs' records "still get the per-record checks".
- **Alternatives:** Exempt only landed outcomes and keep the score check (the score would still be flagged,
  since it cannot follow from records the gate never validated). Add a separate exempt list for records
  (a second place to keep in step).
- **Tradeoff:** The hole D67 accepts for one commit grows to the records it adds: for that commit nobody
  checks that a landed outcome is a real landing or that a score follows from its records. The secrets and
  content checks, and the tip's validation and reconciliation of every record, still run.
- **Reversal:** Remove the early return in `jobs::check`; D67's entry then excuses content only. Cost: low.
- **Provenance:** Follows D66 and D67. Tests in `crates/athena-gate/src/history.rs` show hand-written landed
  outcome and score records pass in an exempt commit, are flagged in a non-exempt one, and are flagged in a
  commit after the exempt one.

## D69 · Accepted: a squash of a chain of verified jobs lands by a chain-aware landing rule

- **Status:** Accepted and implemented. The user asked for the gate rule ("now the gate rule for squash-chain
  landing") after the record and core half was built. `landing` in `crates/athena-gate/src/invariant.rs` now
  accepts a chain as described below; `landing_job` returns the set of jobs, `jobs::check` accepts every job
  of the chain, and the history walk records every one. Tests are named in `docs/enforcement.md` (M30). No
  `[history] exempt` entry, `athena.toml`, workflow, or record changed. The decision text below is the
  original proposal, kept as the reasons for the rule.
- **Decision:** (Written as a proposal; accepted as written.) **Option B** of
  `docs/design/squash-chain-landing.md`. Each job's `candidate.json` (new schema `athena.candidate/2`, loaded
  beside `/1`) gains `base_fingerprint` and `previous_job`; `landing` accepts one commit that adds the
  closing records of several jobs when they form one linear chain, in the recorded `previous_job` order,
  from the parent's fingerprint to the landed tree's fingerprint (each job's base fingerprint equals the
  previous job's candidate fingerprint), the last job's candidate and verified fingerprints equal the
  landed tree, and every job passes the other per-job checks it passes today, including that its reading
  precedes its own work. A chain of one is today's rule. Implementation (the fields, the gate change in
  `crates/athena-gate/src/invariant.rs`, tests in `landing_tests.rs`, and updates to design §7.3 and
  `docs/enforcement.md`) is a follow-up job. A chain is refused if a job changes a path classification
  rules file, or if the landing commit has more than one parent (assumed defaults after review 1, which the
  user may change). After review 2: the core derives `previous_job` as the latest landed job in the base
  commit's history (absent if none; also an assumption the user may change); one job left from a partly
  landed chain is judged by the single-job rule; a landed predecessor, including a `/1` one, is found by
  loading its records at the parent; for a merge commit the existing merge message is shown. D60-D68 stay in
  force.
- **Alternatives:**
  - (A) Per-job patch composition proof: needs the intermediate commits' blobs, which a squash discards.
  - (C) Restrict merge methods (D66 (c)): the restriction the request asks to remove, enforced only by a
    host setting.
  - (D) A `[history] exempt` entry per squash (D60, D67, D68): skips the whole invariant for the commit and
    needs a follow-up commit each time.
  - (E) Check only the last job: ignores whether the earlier jobs' records belong to this tree.
- **Tradeoff:** B keeps "nothing reaches main unverified" in the sense that the landed content equals the
  last job's verified fingerprint and each step names a passing verification; the last verifier did not
  review the whole tree. Over option E it catches accidental mismatches, not forgery. It keeps M23 (each
  job's reading precedes its own work) and needs no exemption. It costs a schema change in `candidate.json`,
  a more complex `landing`, and it proves intermediate states only by unsigned records agreeing, the same
  limit as §7.3's timestamp proof. Jobs landed before the fields exist cannot form a chain.
- **Reversal:** Before implementation: reject or edit this entry and the design. After implementation: remove
  the chain branch in `landing`, which restores the one-job-per-commit rule; `athena.candidate/2` and its
  fields can stay, since the gate keeps loading both versions. Cost: medium once chain squashes are on
  main, since history then depends on the rule (as D66 noted for (b)).
- **Provenance:** Requested by the user ("adopters choose their own merge method"). Claims about `landing`
  (the more-than-one-job refusal and the whole-tree fingerprint equality), `Candidate` fields, and §7.3 are
  read from `invariant.rs`, `crates/athena-data/src/records.rs`, and the design. The security argument and
  where the core writes the candidate are inferred; no command was run to reproduce a failure or to try the
  rule.

## D70 · The squash of PR #9, f2a8453, is exempt from the history check

- **Decision:** At the user's request, the squash commit f2a8453380196b95f2e2cc629ffa969af57e6c85 (PR #9) is
  added to `[history] exempt` in `athena.toml`, as a full 40-character id, with a comment that gives the reason.
  Most of PR #9's jobs predate `athena.candidate/2`, so the chain landing rule of D69 cannot link them: it
  needs the `base_fingerprint` and `previous_job` fields that only `/2` records carry. Following D60, D67,
  and D68, the commit's content skips the governed-content invariant and the job-records check. No code,
  test, or workflow changes.
- **Alternatives:**
  - Teach the gate to link jobs recorded before `athena.candidate/2`: a new gate rule, not chosen.
  - Restrict merge methods (D66 option c): enforced only by a host setting, and it would not excuse this
    commit, which is already on main.
  - Rewrite main (D66 option d): destructive; rejected there.
- **Tradeoff:** The cheapest and fully reversible choice. Its cost is a hole in the invariant for this one
  commit: "nothing reaches main unverified" is not checked for f2a8453. D60 limits the hole to that commit,
  since a full id cannot cover a later one.
- **Reversal:** Delete the `f2a8453380196b95f2e2cc629ffa969af57e6c85` entry and its comment from
  `[history] exempt` in `athena.toml`. The history check then judges f2a8453 again. Reversal cost: low.
- **Provenance:** User request. The full id is the one the user supplied; this change did not check that it
  names a commit in history, which D60 requires. The reason is the user's statement, not reproduced here.
  Enforced by the existing history check and the `[history] exempt` validation of D60 and D61.

## D71 · A score is recomputed from the interventions that existed when it was made

- **Decision:** `jobs::expected_score` counts only interventions whose `at` is not after the score's own
  `at`. In a squash, the job's score, its later interventions and their re-scores arrive in one tree. The
  closing score was made before those interventions, with `user_intervenes` unresolved (outcome false,
  provisional). Recomputing it from the whole tree counted them and resolved the event early, so a re-score
  that `athena intervene --job` wrote legitimately differed in that event's `outcome`, `provisional` and
  `brier` and was refused as not following from the records. A re-score is still the recomputed closing
  score passed through `athena_score::rescore`, so a forged one (for example with zeroed Brier values) is
  still refused.
- **Alternatives:** Exempt such commits from the history check (D66/D67): hides every other check too.
  Change how `athena intervene` writes scores: not needed, the written scores are correct.
- **Tradeoff:** A forged intervention record dated later than its score would be ignored for that score;
  the re-score's cause must still be an intervention the commit adds for the job, so this gives no way to
  pass a score that differs from the rules.
- **Reversal:** Remove the `at` filter in `expected_score`. Reversal cost: low.
- **Provenance:** Cause found by reading `expected_score`, `score_records` and `rescore`; reproduced by the
  new tests in `landing_tests.rs`, which fail without the filter on `events[2].brier.reader` (the
  `user_intervenes` event). The two job ids named in the request were not examined.

## D72 · Pull requests are landed natively; in a repository Athena does not own, its records stay outside it

- **Decision:** Proposed, design only: `docs/design/native-pr-landing.md`. Athena would push a job's work
  branch with a regular push on the user's explicit command, open the pull request through the user's
  authenticated host client, and observe the merge; it would not merge on its own. Two modes: in a repository
  the user owns, records travel in the pull request as now; in stealth mode the records stay in Athena's state
  directory and the pull request holds only the change. The prose-only rule "Athena does not push" would be
  replaced by "pushes only on command, only its own work branch, never non-fast-forward"; "published history
  is never rewritten" stays. No rule changes until the build exists.
- **Alternatives:** Keep landing local (fails the brief's native-PR addendum); call the host's web API with a
  token Athena holds (new secret class); push records on a side ref (visible in the third-party repository);
  keep records in a second repository (deferred).
- **Tradeoff:** Stealth gives up the repository-side audit in CI, so the proof of verification is local only.
- **Reversal:** Do not build it; nothing else depends on this document. Reversal cost: low.
- **Provenance:** `BRIEF.md` addenda of 2026-10-12 (native PRs, brownfield, stealth); D10, D69,
  `docs/enforcement.md` rows M6, M7, M23, M30. Read from those files; no command was run. This replaces an
  earlier draft of the same design that failed only the boundary scan (M19) on its wording.

## D73 · The producer brief lists the words the boundary scan rejects

- **Decision:** Amends D62's brief. `producer_brief` gains a "Words you must not write" section before the
  checks. It lists the scan's words and fragments, and says the scan is whole-word and case-insensitive and
  covers singular, plural and hyphenated forms. `athena-context` owns the brief's list
  (`briefs::boundary_words`, `briefs::boundary_fragments`), built from fragments so no tracked file contains
  the terms. The scanner in `xtask/tests/policy.rs` keeps its own list; `athena-context` is a dev-dependency
  of `xtask` only, so the documented graph (M21, which counts normal dependencies) is unchanged, and
  `the_producer_brief_lists_exactly_the_scanners_terms` fails if the two lists differ. The scan is unchanged.
- **Alternatives:** (a) Make the scanner read the brief's list: a larger change to a rule's mechanism.
  (b) Write the list in `prompts/producer.md`: the scan would reject that file. (c) Leave producers to learn
  from the failing check.
- **Tradeoff:** Two copies of the list, tied by a test. The section costs about 150 tokens of producer
  context, and it shows the producer the banned terms, which it must then avoid writing.
- **Reversal cost:** Low.
- **Provenance:** User report of repeated boundary-scan failures by producers, the latest in a design review.
  Enforced by `producer_briefs_list_the_boundary_words_before_the_checks` and the test named above. The
  token figure is an estimate.

## D74 · Only landed jobs form a squash's chain

- **Decision:** Amends D69. `closing_jobs` in the gate's landing check leaves out a job whose recorded
  outcome in the commit's tree is not landed (rejected, blocked and never landed, failed, and so on). A
  squash ships the records of a rejected try, but its work never reached the branch, so it is no link.
  Counting it made the landed retry, whose `previous_job` names the rejected try, look like a fork ("both
  follow"), and the retry's follow-up like an invalid landing. A chain head may now name a previous job
  that is not in the chain if that job's recorded outcome is not landed; the head's `base_fingerprint`
  must still equal the parent's content. A previous job that is absent, or has no outcome record, is still
  "not linked". A job with no outcome record at all stays in the set and is refused by its landing check.
  Landed jobs that really fork are still refused.
- **Alternatives:** Require rejected jobs to chain (they do not: they never landed); ignore every
  `previous_job` that is not in the commit (opens a hole for unlanded or invented work).
- **Tradeoff:** A squash made only of a rejected job no longer reports a per-job reason; it is a commit
  that lands no job, and fails as an ungoverned content change.
- **Reversal:** Drop the `not_landed` filter and the extra head case. Reversal cost: low.
- **Provenance:** Cause found by reading `closing_jobs` and `chain_problem`; reproduced by the new tests in
  `landing_tests.rs` (a rejected job, its landed retry and a follow-up). Detail in
  `docs/design/squash-chain-landing.md` §4.6.

## D75 · The core pins a study's repository ref to a commit, and mounts the tree read-only

- **Decision:** A study reads a system "at one pinned commit", so the core, not an agent, picks it. The
  `Capability` trait gains `pins_repo` (false by default; the study capability returns true). In `athena go`,
  after the budget check and before the prediction commit, a pinning capability's `repo_url` and `repo_ref`
  are read from its `[capabilities.<id>]` section of `athena.toml` at the base commit, and `Athena.pinner`
  (a `Pinner`) resolves them to a full 40-hex commit id and checks the tree out under the job's directory.
  A capability that pins but has no `repo_url` and `repo_ref`, a URL that carries credentials, a ref that
  does not resolve, or a pinner answer that is not 40-hex stops `go` with an error before anything is
  committed. The URL, ref and commit id go into the new optional `pin` field of the job record, which is
  committed with the prediction; other jobs have no `pin`. The real `GitPinner` shells out to `git`
  (`ls-remote`, a depth-1 fetch of the commit, `read-tree` and `checkout-index`), so no git library is
  added. Before each agent runs, the tree is copied to `pinned-source/` in the producer's and the verifier's
  working directories. Read-only is enforced by file permissions: every file and directory loses its write
  bits, so a process that honors them cannot edit, add or remove anything in the mount. The agents' own
  policy forbids writing there. It is not an operating-system mount, and a process with the owner's rights
  could still `chmod` it. The mount is listed in the snapshot repository's `info/exclude`, so it never
  appears in a candidate's change list. Removing a job's directories first restores write access
  (`pin::remove_tree`). The repository is named only in configuration; no Rust source names one.
- **Alternatives:** (a) Let the study producer run `git ls-remote` itself: the pin would be an agent's
  claim, not the core's, and the verifier could not rely on it. (b) A git library: a large dependency for
  two commands. (c) A bind or overlay mount: needs privileges and is not portable; permissions are enough
  for the policy-abiding agents this guards against. (d) Pin in the reader or in the prediction commit's
  text: the record field is typed and checked. (e) Pin for every capability: the change capability works on
  the user's own repository, which `base_commit` already pins.
- **Tradeoff:** Each study copies the tree twice (once per agent). The record format gains a field, which
  older readers of `athena.job/1` with `deny_unknown_fields` reject, though jobs without a pin serialize
  exactly as before. A pin failure (network, a missing ref) stops the job at `go`, leaving it proposed to
  retry.
- **Reversal cost:** Low to moderate. Remove the `pins_repo` call and `pin.rs`, the `Athena.pinner` field and
  the config keys. Job records already written keep their `pin` field, which would then need to stay
  accepted by the parser.
- **Provenance:** Inferred from the study capability's design (`docs/design/study-capability-m1.md`) and
  the code read in `run.rs`. Enforced by the dispatch tests in `run_pin_tests.rs` (one call, the commit in
  the job record, the read-only mount seen by both agents, no call without a configured repository), the
  change capability's test in `lifecycle.rs` (no call, no pin), and the real pinner's tests in `pin.rs`
  (local `file://` repositories, no network).

## D76 · The squash of PR #14, 5a73478, is exempt from the history check

- **Decision:** At the user's request, the squash commit 5a734782e76daac9b12bd8833c97a73927c8cf94 (PR #14) is
  added to `[history] exempt` in `athena.toml`, as a full 40-character id, with a comment that gives the
  reason: the commit changes `records/` but names no job. Following D60, D67, D68, and D70, the commit's
  content skips the governed-content invariant and the job-records check. No code, test, or workflow
  changes.
- **Alternatives:**
  - Add the job records that PR #14 is missing, so the commit names its work: those records were not
    available to this change, and writing them after the fact is outside this job.
  - Teach the gate to link records that name no job: a new gate rule, not chosen.
  - Rewrite main to drop or reshape the squash (D66 option d): destructive; rejected there.
- **Tradeoff:** The cheapest and fully reversible choice. Its cost is a hole in the invariant for this one
  commit: records that the history check cannot tie to a job are accepted for 5a73478 without that check.
  D60 limits the hole to that commit, since a full id cannot cover a later one.
- **Reversal:** Delete the `5a734782e76daac9b12bd8833c97a73927c8cf94` entry and its comment from
  `[history] exempt` in `athena.toml`. The history check then judges 5a73478 again. Reversal cost: low.
- **Provenance:** User request. The full id is the one the user supplied; this change did not check that it
  names a commit in history, which D60 requires. The producer's snapshot has no git history, so the check
  could not run here. The reason is the user's statement, not verified against the commit's content.
  Enforced by the existing history check and the `[history] exempt` validation of D60 and D61.

## D77 · Firstmate on main is the first configured study repository

- **Decision:** `[capabilities.study]` in `athena.toml` sets `repo_url` to
  `https://github.com/kunchenguid/firstmate.git` and `repo_ref` to `main`, so a study job can start (D75).
  The pin tests' `config()` helper in `run_pin_tests.rs` now drops any `repo_url` and `repo_ref` lines from
  the real `athena.toml` before it adds its own (or none), so duplicate TOML keys cannot occur. Running the
  study itself is a later job.
- **Alternatives:** Keep the keys commented out and pass them per job: a study would refuse to start until
  someone edits the file. Give the tests their own config text: they would stop checking the real file.
- **Tradeoff:** Every study job reads firstmate until the keys change. The helper matches lines by key
  prefix, so a future key beginning with `repo_url` or `repo_ref` would also be dropped.
- **Reversal cost:** Low. Comment the two keys out again; the helper still works.
- **Provenance:** User request, retrying a job whose pin tests failed on duplicate keys. Enforced by both
  tests in `run_pin_tests.rs`, run against the real `athena.toml`.

## D78 · The first study is kept as records, and a kept test validates it against the pinned tree

- **Decision:** `studies/firstmate-work-handling/` holds the M1 study of firstmate's work handling as typed
  records: `pin.json`, `claims.json`, `rationales.json`, `model.json`, `findings.json`, and
  `validation.json`. The pin is the full commit id the user gave; `pinned-source/` carries no git metadata,
  so the id was taken from the request and not re-derived. Tests in `rules.rs` read the records, answer the
  rules' git questions from the files of `pinned-source/` (a `MountedTree` that serves only that commit), and
  assert that every rule passes, that `validation.json` equals what the rules return, that a wrong SHA or a
  changed quote or line range is flagged, that no placeholder pin appears, that every claim text starts with
  `[mechanism]` or `[prose-only]`, and that a guard of `none` is never marked as evaluated by code. Where
  `pinned-source/` is absent (a plain checkout), the two tree-bound tests return early. The study read the
  tree only: no run, so its failure and retry edges carry a `could-not-run` finding, and a `model-limit`
  finding records that `guard_evaluated_by` has no value for "nobody", so `agent` stands in for a guard of
  `none`. `findings.json` is a sixth record file because the rules need findings and the study has no other
  place for them.
- **Alternatives:** Mark claims `observed` by running firstmate: no recorded boundary for a run was given.
  Add a "none" value to `EvaluatedBy`: a schema change outside this job. Read the tree through `git`: the
  mount has no history.
- **Tradeoff:** The tests need the mount to prove anything, and a plain checkout skips them. The enforcement
  tag lives in the claim text because `study.claim/1` has no field for it. The model is small: five states
  and eight transitions, chosen to cover dispatch, capacity, delivery, teardown and relaunch, not all of
  firstmate.
- **Reversal cost:** Low. Delete `studies/firstmate-work-handling/`, its tests, and the inventory row.
- **Provenance:** Read from the pinned tree; every citation is checked by the tests above, which fail on a
  wrong path, line range, SHA or quote.

## D79 · sccache is pinned by version and built by `cargo install --locked`; adoption in `athena.toml` follows

- **Decision:** This supersedes D46's exclusion of sccache (D46's per-job target directories are unchanged).
  - `[tools.sccache]` is an optional table of `athena.toml` holding only `version`. Absent, nothing changes.
  - When present, setup runs `cargo install --locked --version <v> --root <athena_home>/tools/sccache/<v>
    sccache`, skips it if `tools/sccache/<v>/bin/sccache` exists, and stops with a message if the install
    fails.
  - A capability's `env` values may use `{athena_home}`. `paths::sccache_binary` derives the binary path
    from the one pinned version, and a test checks that the install root and the binary share it.
  - `athena.toml` is not changed here: the installed gate cannot parse a field it does not know, so the
    configuration adopts the pin, `RUSTC_WRAPPER` and `SCCACHE_DIR` in a later job, after this build is
    adopted. (Done in D80.)
- **Alternatives:**
  - (a) Release tarballs with a SHA-256 per platform, as gitleaks has. Faster to install, but each digest
    must be fetched from upstream and checked in; no digest is guessed or placeholder-filled here.
  - (b) `cargo install --locked` (chosen). No digests; cargo's lockfile pins the dependencies.
  - (c) No cache, as D46 left it. Nothing to maintain, and every job compiles cold.
- **Tradeoff:** The first setup compiles sccache from source, which takes minutes (estimate, not measured
  here) and needs a working toolchain and network. The crate's published checksums are verified by cargo,
  but Athena pins no archive digest of its own. The speedup is not measured in this change, because nothing
  yet uses the wrapper.
- **Reversal cost:** Low. Remove the table, the setup step, and the placeholder.
- **Provenance:** User request after the first attempt at this change failed. Inferred: that the
  no-digest path avoids repeating that failure. Enforced by tests in `config.rs`, `paths.rs`, `env.rs`
  and `xtask/src/main.rs`.

## D80 · `athena.toml` adopts the pinned sccache for the change and study capabilities

- **Decision:** This completes the step D79 deferred.
  - `athena.toml` gains `[tools.sccache]` with `version = "0.10.0"` and no other field.
  - The change and study `env` tables each set `RUSTC_WRAPPER` to
    `{athena_home}/tools/sccache/0.10.0/bin/sccache`, `SCCACHE_DIR` to `{athena_home}/sccache`, and
    `CARGO_INCREMENTAL` to `0` (sccache does not cache incremental compilation).
  - `CARGO_TARGET_DIR` and `CARGO_TERM_COLOR` are unchanged, so D46's per-job target directories stand.
  - A test in `config.rs` checks that the wrapper path uses the pinned version in both capabilities.
- **Alternatives:** (a) Pin without setting the wrapper: nothing would use the cache. (b) Wire only the
  change capability: a study would compile cold. (c) Chosen: both.
- **Tradeoff:** The wrapper path is written out in two places beside the pin, so a version bump edits three
  lines; the test fails if they disagree. If the binary is not installed, builds fail until setup is run. The
  speedup is not measured here.
- **Reversal cost:** Low. Remove the pin and the three env keys from each capability.
- **Provenance:** User request, following D79. Enforced by the tests in `config.rs`.

## D81 · `setup --ci` skips the sccache install

- **Decision:** Setup step 4 prints a skip message and does not call the sccache install when `--ci` is set.
  - The choice is the pure function `skip_sccache(ci)` in `xtask/src/main.rs`, covered by a unit test.
  - Setup without `--ci` still installs the pinned sccache. `athena.toml`, the `[tools.sccache]` pin and the
    CI workflow are unchanged.
- **Alternatives:** (a) Cache the sccache build in CI: adds workflow changes and cache upkeep. (b) Drop the
  pin: the change and study capabilities would lose their cache. (c) Skip under `--ci` (chosen).
- **Tradeoff:** CI never exercises the sccache install path; only the unit tests do. CI sets no
  `RUSTC_WRAPPER`, so nothing in CI needs the binary.
- **Reversal cost:** Low. Remove the flag check in step 4.
- **Provenance:** User request: PR CI rose from about 4 to about 9 minutes (the user's figures) after D79
  and D80. Enforced by a test in `xtask/src/main.rs`.

## D82 · A retry carries the earlier reading, the user's corrections, and a discard notice

- **Decision:** The reader's "Earlier attempt" section also shows the retried job's final reading (summary,
  acceptance criteria, deliverables), the corrections and answers the user gave to its readings, and says that
  the attempt's changes were discarded and are not in the repository the reader sees.
  - `EarlierAttempt` gains the fields; `carry` in `reading.rs` fills them, redacted like the other carried text.
  - The carried request is still the original request when the retried job was itself a retry.
  - A job with no reading or no corrections renders without those headings.
- **Alternatives:** (a) Keep carrying only the request and findings: a retry then loses the scope the user
  agreed on. (b) Carry every reading: longer briefs, and earlier readings were superseded. (c) Final reading and
  all corrections (chosen).
- **Tradeoff:** The reader brief grows by one reading and the correction lines. Referenced jobs fill the same
  fields but are rendered as before.
- **Reversal cost:** Low. Remove the fields and the rendering block.
- **Provenance:** User request to retry a job whose retry lost its reading and corrections. Enforced by tests in
  `briefs.rs` and `reading.rs`.

## D83 · The roadmap puts acceleration first, with the firstmate study alongside

- **Decision:** `docs/design/roadmap.md` gains a first step, RM-0, for acceleration, and RM-6 (the firstmate
  study) is marked as running alongside it.
  - RM-0 holds the sccache hit rate, concurrent jobs (including collisions on `CHANGELOG.md` and
    `docs/decisions.md`), native pull-request landing, and timings fed back, each with a "done when" test.
  - The rows of the old RM-8 and AD-11 moved into RM-0; RM-8 no longer exists and other step numbers are kept.
- **Alternatives:** (a) Renumber all steps: breaks references. (b) Keep the old order: the brief's addendum of
  2026-10-13 (`BRIEF.md:494-496`) makes acceleration Athena's mandate. (c) New RM-0 (chosen).
- **Tradeoff:** The sccache "0 hits" figure is the user's report and is not measured in the repository. The
  roadmap stays a proposal and implements nothing.
- **Reversal cost:** Low. Docs only.
- **Provenance:** User request after the 2026-10-13 addendum. No test enforces it.

## D84 · The sccache wiring is removed from `athena.toml`; the cause of 0 hits was not reproduced

- **Decision:** This reverses D80. D79's optional `[tools.sccache]` support in code and setup stays.
  - `athena.toml` loses `[tools.sccache]` and, in both the change and study `env` tables, `RUSTC_WRAPPER`,
    `SCCACHE_DIR` and `CARGO_INCREMENTAL`. D46's per-job `CARGO_TARGET_DIR` and `CARGO_TERM_COLOR` stay.
  - Tests in `config.rs` pin the absence: no pin in `athena.toml`, none of the three keys in either `env`
    table, and the optional pin table still parses.
  - **Diagnosis status: not reproduced.** The producer for this job may run only cargo build, check, test,
    clippy, fmt, run, metadata, tree and doc, so it could not run `sccache --show-stats` before or after a
    build. No stats were collected, and the user's figures (0 hits, 644+ misses, test check about 165 to 190
    seconds) are reports, not measurements.
  - What the reading of the code supports, as hypotheses only: misses (not build failures) mean the wrapper
    ran, so the binary was installed. A per-job `CARGO_TARGET_DIR` and per-job snapshot paths put different
    absolute paths in the arguments of workspace crates, which can change their keys; the cause for registry
    crates was not identified.
- **Alternatives:** (a) Keep the wiring and measure later: every job pays for a wrapper process that, as
  reported, never hits. (b) Change D46's per-job target directories to share paths: out of scope and a larger
  design change. (c) Remove the wiring (chosen) until someone can run `sccache --show-stats` and see hits.
- **Tradeoff:** Builds are cold again, as before D80. Re-adding the cache needs a measured cause first.
- **Reversal cost:** Low. Restore the pin and the three env keys per capability, and update the tests.
- **Provenance:** User request RM-0 item 1, with the revert fallback allowed. Enforced by tests in
  `config.rs`.

## D85 · The native pull-request landing design answers review 2

- **Decision:** Amends D72; design text only, no rule changes. `docs/design/native-pr-landing.md` now says: a
  landed job is identified from records and a stored pull request number, never from the trailers of a host
  merge (review 2 F1, F6); one pull request per job, with a squash of several relying on M30 and its limits
  (F2); per-job pull requests are what it supplies for the parallel pool, and re-verification is only on the
  user's request (F3); a pre-report check that merged content equals the verified fingerprint, and stealth's
  lack of audit, are recorded (F4); stealth writes no model `Co-Authored-By` unless asked (F5); cited
  references were re-checked (F7); the four user questions of review 2 are in §9 with the assumption taken
  (F8).
- **Alternatives:** Design pull requests that hold several jobs (larger; left for a later design); re-verify
  stale jobs automatically (spends budget unasked); leave the open questions out (the review asked for them).
- **Tradeoff:** The smaller option was taken for each question the user has not answered, so the user may
  reverse any of them in §9. The roadmap order is not edited here.
- **Reversal cost:** Low. Docs only.
- **Provenance:** `docs/design/reviews/native-pr-landing-review-2.md`. The user's observations E1 and E2 are
  reported and were not reproduced. No command was run to reproduce any behavior.

## D86 · Native pull-request landing: review 3 answered, and slice 1 (owned-mode core) built

- **Decision:** Amends D72 and D85. The design now says: the merged-content check compares the governed-content
  fingerprint of the merged commit with the verified one, a refused merge is in the named state
  `merged-unverified`, and the user leaves it by re-verifying on the new tip as a new job (N1); the pull request
  number, host, repository and pushed head live in the state directory in both modes, and the records schema
  does not change (N2); the stealth body drops the job id and Athena vocabulary, the leak check covers the body,
  stealth pushes a neutral branch name, and the push guard compares against the name Athena chose (N3); a
  stealth squash of several jobs runs the per-job checks over the local store (N4). Slice 1 adds
  `crates/athena-core/src/pr.rs`: the link store, the push guard (own branch only, fast-forward only, no force
  option) and the merged-content check (found by number, never by trailers). Nothing calls it yet.
- **Alternatives:** Compare a diff instead of a fingerprint (needs a second definition; rejected); add a
  record for the number in owned mode (a path through the gate for a value known only after the push;
  rejected); build the command in the same job (too large; later slices).
- **Tradeoff:** Most merges from a moved base are refused until re-verified, so parallel jobs land in the
  host before they land in Athena's accounting. The link file sits in the job's state directory, which
  `clean_job` removes; the later command slice must keep it until the merge is observed.
- **Reversal cost:** Low. One new module with no callers; docs.
- **Provenance:** `docs/design/reviews/native-pr-landing-review-3.md`, findings N1-N4. Tests in `pr.rs`.

## D87 · Merge commits are the landing shape for pull requests; `land --pr` reruns after the merge

- **Superseded by D88** for the landing shape (merge commits only). The two rerun fixes below stand.
- **Decision:** Pull requests land as merge commits. The opt-in REST merge already requests `merge`
  (unchanged), and the design (`docs/design/native-pr-landing.md` §5.2a, `docs/design/athena-v0.md` §7.3) now
  says a user who merges by hand should pick a merge commit. The gate still accepts squash and rebase merges
  (D69, M30); nothing is exempted from the history check. Two fixes to the rerun of `athena land --pr` ride
  with it: the merged-content check fetches the merged commit from `origin` (object only, no ref moves,
  `Git::ensure_commit`) before fingerprinting it, and when the job's pull request link exists the host is asked
  about the merge before the landed commit is required on the local branch (`gh::confirm_existing_merge`).
- **Alternatives:** Keep squash and rely on the chain rule (rejected: it breaks ancestry, below); exempt the
  squash commit from the history check (rejected: weakens the check); move the work branch after a squash
  (rewrites history).
- **Tradeoff:** The history shows each job's commits and a merge commit per pull request, so it is longer than
  a squashed history. The recommendation cannot stop a user from squashing on the host.
- **Reversal cost:** Low. Docs, plus two small code paths.
- **Provenance:** Evidence reported by the user from landing PRs #39 and #40, not re-run here. Squash-merging
  PR #39 broke ancestry, and the next pull request conflicted. A simulated history of two merge-commit landings
  passed `athena check --history` with 0 violations. The new tests are in `crates/athena-core/src/pr.rs`,
  `crates/athena-core/src/gh.rs` and `crates/athena-git/src/lib.rs`.

## D88 · The merge method follows the repository; a squash is followed by a resync of the local branch

- **Decision:** Supersedes D87's "pull requests land as merge commits". Sri, the repository owner, chose to
  make Athena work on squash-only repositories rather than enable merge commits on this one, so that it works
  on any user's repository. This repository's host reports `allow_squash_merge` true, `allow_merge_commit`
  false and `allow_rebase_merge` false. Three changes (`docs/design/native-pr-landing.md` §5.2a):
  (1) the REST merge reads those three flags from `GET repos/{owner}/{repo}` and requests the first allowed
  method in the order merge, squash, rebase (so a repository that allows merge commits behaves as before), and
  fails with a clear error when none is allowed; a squash carries `commit_title` and `commit_message` built
  from the job's landed commit, with the trailers in the last paragraph and exactly one `Athena-Job`
  (`Git::squash_text`); (2) once the merge is seen and the merged content is verified, Athena rebuilds the
  local branch on the merged commit by replaying the commits after the job's landed commit, so later landed
  jobs, records and hand work are kept and the next `athena land --pr` opens a clean pull request
  (`Athena::resync_onto_merge`, `Git::replay`); (3) the docs no longer say pull requests always land as merge
  commits. D87's two rerun fixes (fetch the merged commit; ask the host before requiring the landed commit)
  stand.
- **Alternatives:** Enable merge commits on this repository (rejected by Sri: Athena would then work only on
  repositories that allow them); keep squash and rebuild the landed commit by hand after each job, as for PR
  #40 (the failure this removes); exempt the squash from the history check (weakens the check); rewrite the
  pushed work branch (Athena never forces a push beyond its own branch).
- **Tradeoff:** The resync rewrites the local branch's unpushed commits after the landed job: their ids change
  and their authors become the person running the command. A later commit that conflicts with the squash stops
  the resync with an error and leaves the branch as it was (the user rebuilds by hand, as before). The squash
  text is Athena's; whether the host appends its own text to a supplied message was not observed here, and the
  landing rule does not depend on trailers (M30).
- **Reversal cost:** Low. One host read, one message builder and one replay; docs.
- **Provenance:** Sri's choice, given in the request for this job; the flags are the host's report quoted in
  that request, not re-read here. The failure is PR #40, which needed a hand rebuild (reported, not re-run).
  Tests: `crates/athena-core/src/gh.rs` (method choice, squash-only request, none allowed),
  `crates/athena-core/src/land.rs` (resync on a real temporary repository),
  `crates/athena-git/src/lib.rs` (`replay`, `squash_text`, `reset_keep`) and
  `crates/athena-gate/src/landing_tests.rs` (history check passes on a squash built from that message).

## D89 · The deliverables list is the single statement of scope, and change readings always list CHANGELOG.md

- **Decision:** `ensure_changelog` (`crates/athena-core/src/reading.rs`) runs on every reading built from the
  reader's output. For a reading whose capability is `change`, it adds `CHANGELOG.md` (modify) to the
  deliverables and a `changelog` entry to `defaults_applied` citing BRIEF.md's "tagged releases and a
  changelog", unless each is already there. Study and declined readings are left alone. The verifier brief gains
  a scope section: the expected deliverables list is the one statement of scope, and a changed file outside it
  is the scope finding, judged against the list and not against how a criterion is worded. The rule reads no
  acceptance-criterion text.
- **Alternatives:** Tell the reader to list the changelog in `reader.md` (rejected: it put the reader prompt
  over its 1800-token budget, and a prompt line is not enforced); add a `changelog` entry to `defaults.toml`
  (rejected: the defaults model is the user's, not changed here); infer scope from the wording of criteria
  (rejected: heuristics differ between readings and cause both false findings and misses).
- **Tradeoff:** Every change reading lists `CHANGELOG.md`, including a change that has nothing to record; the
  `changelog` default shows "(not in the defaults model)" as its statement in briefs, since the entry is written
  by code.
- **Reversal cost:** Low. One function, one call and one brief section; tests.
- **Provenance:** stated: BRIEF.md line 337, "Standalone, with tagged releases and a changelog". The scope
  rule is the designer's reasoning (inferred). Tests: `crates/athena-core/src/reading.rs`
  (`ensure_changelog`) and `crates/athena-context/src/briefs.rs` (scope section).

## D90 · Repeats become proposed, versioned defaults: Jev matches in shadow, code decides, the user reviews

- **Decision:** Proposed, design only (`docs/design/defaults-loop.md`); nothing is built.
  - Jev only answers typed questions through `POST /v1/evaluate`: match probabilities of an intervention
    against each default, the assignment of a record to a cluster or failure class (or a pairwise yes/no),
    and the kind of prevention. A language model drafts the statement, `applies_to` and title. Code decides
    the thresholds, the number of distinct jobs that makes a candidate, id uniqueness, the version bump (one
    per accepted job) and validation through `DefaultsModel::parse`, and it writes the proposal record.
  - Introspection is a first-class step: every correction, intervention, rejection and failure triggers an
    introspection that Athena runs and records (what happened, failure class and earlier cases, cause beyond
    the instance, prevention, proposed next request). Athena code outside the agent snapshot reads past
    records and passes a filtered summary; a retry starts from the record; the defaults loop consumes it.
  - The case for the loop is unproved until `records/` is read; no build starts unless at least N distinct
    jobs show a repeat a default would cover.
  - A proposal has an id, a statement, `applies_to`, a version bump and `sources` citing records. It is
    review-only: Athena never edits `defaults.toml`. The user accepts by asking for a job that lists
    `defaults.toml` as a deliverable (D26).
  - Jev runs in shadow against a stronger model on a labeled set from past records, is scored against two
    references, and is promoted only on stated criteria.
  - This narrows D31: Jev may be added, in shadow, for this one use. D31 stands for everything else.
- **Alternatives:** keep noticing repeats by hand; rules without a model; a stronger model only; Jev or
  Athena applying changes itself; a separate Athena-owned defaults file; counting records instead of jobs.
  Each is weighed in the design, section 9.
- **Tradeoff:** A shadow period costs spend and a labeled set before any live use. In return, a wrong
  proposal cannot reach a brief, and D89's finding (the defaults model is the user's) is kept.
- **Reversal cost:** Low while only the design exists; docs.
- **Provenance:** stated: the request to close the defaults loop with Jev; `BRIEF.md` line 217 (shadow new
  decision types against a stronger model). The four repeats come from the job request and were not
  checked against `records/`, which was not readable when this was written (reported); D89 and D25 show
  two of them in the repository (read). The numbers in the promotion criteria are estimates. No independent
  review of the design is recorded yet.
- **Revised** (review 1, F1-F5 and the Medium and Low items; BRIEF.md addendum "Introspection on every
  correction"): Jev's role narrowed to typed questions, R2 relabelled a counter-example, the evaluate route and
  signals added, the reader budget restated, introspection added. The introspection cases come from the job
  request and are reported, not checked against records.
- **Note, added after review 1 of the pair (D90 and D91):** the "Alternatives" line above cites "the design,
  section 9". After the revision the alternatives are in `docs/design/defaults-loop.md` section 11. The line above
  is left as written; this note is the correction.

## D91 · Decisions are typed, grouped into sets by scope, and carry assumptions and failure signals

- **Decision:** Proposed, design only (`docs/design/decision-sets.md`); nothing is built. Every decision gets
  typed fields: kind (requirement, expectation, design, process), scope (classes of work, paths, phase:
  reading, producing, verifying, landing), status (active, proposed, superseded by a named decision), source and
  enforcement (code check, brief, prose only). Sets are computed from scope, and a job's active decisions are
  applied to each role's brief within its token budget, as defaults are now; defaults are the `expectation`
  kind. Each decision states its assumptions and observable signals (rejections, interventions, corrections,
  check failures, host errors). Code watches the signals in records; a signal that fires puts the decision under
  review, marked in briefs, and Athena never retires or edits a decision itself. Jev may answer two more typed
  questions (set membership, signal match), in shadow only; code matching works without it. This extends D90's
  narrowing of D31 and leaves D31 standing otherwise.
- **Alternatives:** a flat log with a status column; stored set files; all decisions in `defaults.toml`;
  automatic supersession; Jev for all membership; signals as free prose; assumptions without signals; rewriting
  D01-D90. Each is weighed in the design, section 7.
- **Tradeoff:** Each decision costs a short header and, for the ones that matter, assumptions and signals to
  write. In return a decision like D87 (which assumed merge commits were allowed and was refused with HTTP 405)
  is flagged on its first failure instead of staying applied.
- **Reversal cost:** Low while only the design exists; docs.
- **Provenance:** stated: `BRIEF.md` addendum "2026-10-13: Decision sets, and knowing when a decision is not
  working". The D87 refusal is reported in the job request and not re-run; D87 and D88 are read. Review 1
  (`docs/design/reviews/decision-sets-review-1.md`) gave accept with changes.
- **Revised** (review 1, section 6): the reader gets sets only through code-side narrowing with a cap, and a
  runtime per-job set-size check is a build item (refusal of an over-budget static set is a test, not a job-time
  check); a default's assumptions and signals are stored in a separate typed record keyed by the default's id; a
  host-error record with a status field is written by the client and landing is a watcher trigger; one evidence
  rule (one event puts a decision under review, three distinct jobs add a default); D87's example enforcement is
  `brief`; the class vocabulary is closed and landing is a phase; the under-review state is a hashed brief
  component; set membership is one boolean per decision, thresholded on the probability of the true option.
  Alternatives 9 and 10 were added to the design. No review of the revision is recorded yet.

## D92 · The Jev client rides the existing outbound wrapper, over the `curl` program, and runs only in shadow

- **Decision:** Build step 5 of `docs/design/defaults-loop.md` section 12. `crates/athena-core/src/jev.rs`
  holds a client for `POST https://ai-gateway.vercel.sh/v1/evaluate` (and no other endpoint) and a fake that
  returns the same typed response. The outbound wrapper in `gh.rs` gains a transport trait (`HttpPost`), its
  implementation over the installed `curl` program (`CurlHttp`), and the retry loop it already had in
  `GhClient::api`, now also callable on its own (`retry_transient`), with the same `Retry::STANDARD` bound and
  transient classification (500, 502, 503, 504, timeouts, resets). Auth errors and other 4xx answers are not
  retried. The chat endpoint's refusal ("a decision model, not a language model") is a build error and is not
  retried. The key is read only from `AI_GATEWAY_API_KEY`, passed to `curl` on its standard input rather than
  in its argument list, printed as `<key>`, and replaced in error text. Introspection asks Jev the failure-class
  choice and the is-repeat boolean in one call and stores both answers, the usage and the cost in an optional
  `jev` field of `athena.introspection/1`; the class, the status and every outcome come from the language model
  and code as before (`from_jev` stays false). Athena asks no Jev unless `Athena::jev` is set: tests set the
  fake, and the command line sets the real client only when `ATHENA_JEV_SHADOW=1`.
- **Alternatives:** an HTTP crate (`ureq` or `reqwest`) in the workspace; calling the chat endpoint; reading
  Jev's choice to set the recorded class; asking the two questions in two calls; making the shadow call a
  default of the command line.
- **Tradeoff:** The `curl` program is a prerequisite for the opt-in shadow call, and its stdout is parsed for a
  status line, which is less typed than a crate. In return the workspace gains no dependency (none could be
  fetched where this was built) and the guard that confines HTTP to the one wrapper keeps holding. The
  `answers` key of the reply was not in the probe notes and is an assumption until the live test has run.
- **Reversal cost:** Low: swap `CurlHttp` for a crate-backed `HttpPost`; the `jev` field is optional.
- **Provenance:** stated: the probe facts in the job request of 2026-10-14 (endpoint, body, answer shapes,
  usage, cost string, the chat refusal, the proxy). Not re-run: no network was available when this was built.

## D93 · A request's goal is carried to the verdict, and the reader budget rises from 1,800 to 1,930 tokens

- **Decision:** The reader prompt names the request's audience, requires one acceptance criterion written as an
  outcome that audience can check, forbids putting the request's own goal in out of scope, and says to read the
  output a request is about before planning a change to it. The producer prompt treats the criteria as a floor
  and serves the goal. The verifier prompt judges user-facing output as its audience would and rates a
  request-reading mismatch at least major. `judge()` enforces the rating: a non-empty
  `request_reading_mismatch` adds a major finding and a reason, so the verdict is not pass. `BudgetSet::Reader`
  rises from 1,800 to 1,930 (the reader figure in D25); the producer and verifier budgets are unchanged.
- **Measured:** The budgets test (`cargo test --locked -p athena-cli --test budgets -- --nocapture`) prints the
  reader set at 1,927 tokens with the new text (⌈bytes/3.0⌉), against 1,798 before, so the new rules cost 129
  tokens. The other sets fit: change producer 876 of 1,100, change verifier 1,148 of 1,300, study producer 1,092
  of 1,100, study verifier 1,297 of 1,300, contract 326 of 350.
- **Why removals cannot pay for it:** The reader set was 2 tokens under budget, and its remaining text is the
  schema-bound rules each output field needs, the defaults model and the capability catalog, which are not
  prose to cut. Removing 129 tokens would delete rules the reader needs to produce a valid reading. The producer
  and verifier sets were also full (study producer 1,099 of 1,100 before), so they paid for their additions by
  trimming wording instead.
- **Alternatives:** Keep 1,800 and drop reader rules (the goal is lost again); move the rules into the contract
  (moving rules between components escapes nothing, M18).
- **Tradeoff:** A larger reader prompt on every job, 129 tokens, in return for a request's goal being named,
  checkable and enforced at the verdict. The 3-token headroom is as thin as before.
- **Reversal cost:** Low: restore 1,800 and remove the rules.
- **Provenance:** measured: the budgets test above; read: `crates/athena-context/src/lib.rs`.

## D94 · A job may not mark its own requirement rows Covered, and a cited document is checked by the verifier

- **Case:** Job 20261014-174815-7bf8 landed a steward that never calls a model. Its request cited a `BRIEF.md`
  addendum, but the verifier judged only the reading's criteria and never compared the change with the
  addendum. The same job moved its own rows A9-1 and A10-1 of `docs/design/requirements-map.md` to Covered, and
  nobody checked that independently.
- **Decision:** Two rules. (1) *Code:* `judge()` takes the map text at the base and at the candidate. For each
  row that is Covered at the candidate and was not Covered at the base (moved from Partial or Not covered, or
  added already Covered) it adds an Athena-sourced major finding naming the row ID and a reason, so the final
  verdict is not pass. Rows that stay Covered or move away from Covered add nothing. `run.rs` reads both texts
  from the verifier's snapshot (`HEAD~1` and `HEAD`). The producer prompt states the rule. (2) *Prose only:*
  the verifier prompt says that when the request cites a document, the verifier reads it and compares the
  change with it, and that falling short is a `request_reading_mismatch` (already rated major by D93). Rows
  A9-1 and A10-1 are put back to Partial, each with its reason (the steward never calls a model; cost and
  yield tracking, reshaping and retiring are not built).
- **Alternatives:** (a) Only reset the two rows (fixes the instance, not the gap). (b) Reuse the CLI's
  `strategy::parse_map` (core cannot depend on the CLI) or move it into `athena-data` (a larger change for one
  small parser; the duplicate is about twenty lines and is tested). (c) Detect cited documents mechanically
  (the request is free text; no reliable rule tells a cited document from a mentioned one). (d) Hard-block the
  landing beyond the normal fail verdict (a major finding already stops it).
- **Tradeoff:** A row can no longer reach Covered inside the job that builds it, so a truly finished row stays
  Partial until a later change or the user marks it; in return nobody grades their own work. The map parser is
  duplicated, so a change to the row format must change both. The document comparison depends on the
  verifier reading the request, and nothing checks that it did. The prompts grew, and the producer and
  verifier sets were kept inside their budgets by trimming wording (change producer 881 of 1,100, change
  verifier 1,148 of 1,300, study producer 1,097 of 1,100, study verifier 1,298 of 1,300, measured with the
  budgets test).
- **Reversal cost:** Low: remove the check in `judge()` and the two prompt lines.
- **Provenance:** read: the request of this job and the map rows; measured: the budgets test.

## D95 · The producer must run the full `cargo test --locked`, and a report without it is sent back once

- **Case:** The producer prompt said the producer "may" run the checks. A docs, TOML or data-only candidate could
  therefore reach verification with the full test suite never run, although policy and config tests read tracked
  non-Rust files: boundary words, unlisted file kinds, `defaults.toml` and the study `model.json`.
- **Decision:** (1) *Prose:* the producer frame requires the full `cargo test --locked` before finishing, for
  every change, with the reason, and says `not-run` is not an acceptable finish. (2) *Code:*
  `produce_and_verify` in `run.rs` inspects the producer's `checks_run`. Unless the entry with the command
  `cargo test --locked` has the result `pass` or `fail`, the producer is re-invoked once in the same working
  directory with a "Sent back" section appended to its brief. If the second report still lacks it, the job fails
  with a reason naming the command, before admission, checks and the verifier. A capability that pins a repository
  (the study) is exempt. The producer budget rises from 1,100 to 1,300 tokens, because the frame is shared and the
  study producer set was at 1,097 of 1,100; measured now: change producer 1,033, study producer 1,249.
- **Alternatives:** (a) Prose only (an unenforced request). (b) Make Athena's check runner always run tests (the
  verifier path already does; the gap is the producer finishing without feedback). (c) Fail at once with no
  send-back (wastes a producer run for a fixable omission). (d) More than one send-back (unbounded cost).
- **Tradeoff:** The gate trusts the producer's own report: it proves the producer claimed a result, not that it
  ran, and Athena's checks still run the suite afterwards. A producer that reports a different spelling of the
  command is sent back. The study set has a longer frame than it needs, and a higher budget for all producers.
- **Reversal cost:** Low: remove the gate, the prompt line and restore the budget.
- **Provenance:** read: `run.rs`, the producer frame; measured: the budgets test and the lifecycle tests.

## D96 · Agents get GitHub credentials, when set, so the steward's `gh` can read

- **Case:** The steward reported that `gh run list` exited 4 and asked for `gh auth login` or `GH_TOKEN`. Cause,
  read from `crates/athena-agent/src/env.rs` and `athena.toml`: an agent starts from an empty environment, only
  `HOME` and `PATH` always pass, and `env_pass_if_set` named no GitHub variable. `gh` therefore had no token
  unless a `gh` config happened to sit under `HOME`.
- **Decision:** Follows D04 and D53 in using the allow-list. `env_pass_if_set` includes `GH_TOKEN`, `GITHUB_TOKEN`, `GH_HOST` and
  `GH_ENTERPRISE_TOKEN`. Each passes only when set and non-empty; a test in `env.rs` pins this. The owner
  exports a read-only token before running Athena. Without one, the steward reports the `github` blind spot.
  `GH_CONFIG_DIR` is not passed. The `gh` read-only command policy in `policy.rs` is unchanged.
- **Alternatives:** (a) Rely on a `gh` config under `HOME`: it depends on the owner's machine, holds a login
  that may carry write scope, and was absent here. (b) No access, with the blind spot recorded: safe but leaves
  CI, pull requests and deploys unread.
- **Tradeoff:** The token reaches every agent role's environment, not only the steward's, although policy lets
  only the steward run `gh`. A read-only token limits the damage; a token with write scope should not be used.
- **Reversal cost:** Low: remove the four names.
- **Provenance:** read: `env.rs`, `athena.toml`, `roles-and-ladder.md`. Whether the owner's shell has a token set
  was not checked.

## D97 · After a merge, `land --pr` watches the Pages deploy and re-runs it once

- **Case:** Transient 5xx and timeout retries with a per-retry report line already live in the one `gh`
  wrapper (`GhClient::api`, `Retry::STANDARD`, three attempts). What was missing was the deploy after a merge:
  the `pages` workflow can time out or be cancelled, and nothing looked.
- **Decision:** When `athena land --pr` ends with the link `Landed`, `watch_pages` in `gh.rs` lists the
  `pages.yml` runs for the merged commit (`GET actions/workflows/pages.yml/runs?head_sha=`), through the same
  retrying client, so a 5xx on the listing is retried and reported like any other call. It waits for the
  newest run to complete: at most 60 looks ten seconds apart, about ten minutes, and three looks for a run
  that has not appeared. A `success` is left alone. A `timed_out` or `cancelled` run is re-run once
  (`POST actions/runs/<id>/rerun`) and waited for with the same bound; if that try fails too it is reported
  and not re-run. Any other conclusion is reported and not re-run. No run for the commit, or a run still
  going at the bound, is reported and is not an error. Every step is a line to the user naming the run and its
  conclusion. A failure of the watch itself is a warning line; the merge stays landed. Reads and the re-run are
  logged under the existing `read-checks` host operation, because that list is closed.
- **Alternatives:** (a) Add a workflow that re-runs itself: needs more token scope and is out of scope.
  (b) Re-run until success: a loop with no bound. (c) Fail the landing on a Pages failure: undoes nothing and
  blocks later work for a deploy problem. (d) A new record type for retries: the report line is enough.
- **Tradeoff:** The wait can add ten minutes to a landing in a repository that has Pages. Running `land --pr`
  again on an already landed job watches again and can re-run an old cancelled run once more.
- **Reversal cost:** Low: remove `watch_pages` and its call.
- **Provenance:** read: `gh.rs`, `pages.yml`; measured: the tests in `crates/athena-cli/tests/external_calls.rs`.
  The live behavior against GitHub was not run.

## D98 · `athena drain` works the steward's queue, lands each item through the pull-request path, and rates only after a landing

- **Case:** The steward queues requests and the audience rates the live page, but running each queued request,
  landing it and rating the page was done by the owner one command at a time. A first attempt at a loop ran
  and rated without a land step, so its rating could describe a page the loop had not changed, and could be
  read as success.
- **Decision:** (1) *Command:* `athena drain --budget <usd> [--rating-threshold N] [--job-limit N] [--json]`
  (`crates/athena-cli/src/drain.rs`). The queue is the unread requests on the steward's channel
  (`steward::queued_work`); a request the user deferred with `ask --later` is not taken. For each, the loop
  reads and runs the job (accepted by the flag, as `--yes` accepts), lands it with `Athena::land_pr`, the
  path of `athena land --pr`, and then rates the live page. Only a merged, verified pull request is a landing; an
  open one is not. (2) *Rated after landing, by code:* the rating call sits behind a landing in the loop, and a
  rating records `after_landing` from the run's own landed count. `target met` needs `after_landing` and a value
  at or above the threshold. A run that landed nothing shows the last recorded rating marked as not taken after
  a landing, says the page is unchanged, and cannot stop with `target met`. (3) *Budget:* before every model
  call (a reading, a run, the re-reading of a retry, a rating) the money spent so far, by what the calls cost,
  plus the call's cap from `athena.toml` (reader, job, verifier) must fit `--budget`. A job starts only when
  its reading and its run caps both fit. A job that cost more than its cap leaves less room, never more.
  (4) *Stop reasons:* `target met`; `budget spent`, with the rest still queued; `repeated failure`, when
  one item fails twice, naming it; `queue empty`, stated as not success and spending nothing when the queue is
  empty at the start; and `job limit`, which the flag name implies. An item has two attempts: after a
  rejected or failed run the second reads the job again with `athena retry` and runs it; after a passed run
  that did not land, the second lands again without running. (5) *People:* open proposals (work items above the
  steward's limit) are listed as waiting; the loop's steps have no way to accept, approve, reject or land one.
  (6) *No config key:* the budget, threshold and job limit are flags. They are per run and a committed key
  would change what a run may spend without anyone choosing it for that run. The command exits 0 only when the
  target was met, and 2 for every other stop.
- **Alternatives:** (a) A `[drain]` table in `athena.toml`: governed content that needs a verified job to change,
  for a per-run choice. (b) Rate before and after each landing: a stale rating then looks like a result.
  (c) Count an open, green pull request as landed: the live page has not changed, so the rating would measure
  nothing. (d) Check the budget only when a job starts: the retry's re-reading and the rating would spend past it.
  (e) Take the audience's low-rating proposals as work: they need a person's decision like any proposal.
- **Tradeoff:** With `[landing] merge_via_rest` off, a drain cannot land anything: the pull request stays open, the
  item fails twice and the run ends in `repeated failure` naming it. Costs are read from the records after
  each call, so a call that fails before it records anything is counted as free. The caps are upper bounds
  from the config, so a budget can stop a drain earlier than the true cost would. The rating waits on the
  Pages watch of D97 inside the land step, not on anything of its own.
- **Reversal cost:** Low: remove `drain.rs`, its command and display, and `steward::queued_work`.
- **Provenance:** measured: the drain unit tests and `crates/athena-cli/tests/drain.rs` (stubbed queue, land
  step and ratings, and one end-to-end run with the fake agent). The live behavior against GitHub and the
  live page was not run.

## D99 · Drain's "target met" also needs a passing check of the page against the source

- **Case:** Drain stopped on "target met" because the audience rated the explainer 4 of 5, while an independent
  reader who checked the page against the pinned firstmate source rated it 3: the work-slot counter was wrong
  and `model.json` contradicted itself about it. A reader who sees only the page cannot catch a wrong fact.
- **Decision:** (1) `Steps::check_page` is asked whenever a rating taken after a landing reaches the target;
  `target met` needs the rating and a passing check. (2) The real check, `model_contradictions`, reads each
  `studies/*/model.json` at the head commit and fails when a counter named in a retry's `changes` lists too few
  transitions in `changed_by`: one for taking a place and one for each way it is freed (the alternatives of
  the text, split on " or "). It needs no model call. (3) When it fails, drain keeps running, the rating line
  says the target was held back by the source check, and the report carries the finding (`held_back`, and a
  `held back` line). (4) The other four stop reasons are unchanged.
- **Alternatives:** (a) An independent reader inside drain: it costs a model call per rating and is more than
  this fault needs. (b) Fix only the wrong sentence: the next wrong fact would end drain again.
- **Tradeoff:** The check counts alternatives in a sentence, so it finds this kind of contradiction and not every
  wrong fact; it does not read the source. A study whose model has no retry text naming a counter is never held
  back by it.
- **Reversal cost:** Low: remove `check_page`, `model_contradictions` and `held_back`.
- **Provenance:** measured: the drain unit tests, `crates/athena-cli/tests/drain.rs`, and a unit test that the
  shipped firstmate model passes.

## D100 · Drain's "target met" needs an outcome-changing landing and a model-backed check of all page text

- **Case:** D99's check read only `model.json` for self-consistency, so a wrong statement in `claims.json`, in the
  intro, in its transition or in a glossary entry could not hold drain back, and a run whose landings touched
  only other files (docs, the loop's own code) could still stop on "target met" from a rating of a page it had
  not changed. A rating by a reader who sees only the page, and a check that never reads the source, are both
  blind to a wrong fact.
- **Decision:** (1) *Outcome-changing landing, by code:* a landing returns the paths it changed, read from the
  landed job's candidate record. `target met` needs at least one landing in the run that changed a path under
  `studies/` (the page sources), `xtask/` (the site generator) or `.github/workflows/pages.yml` (what publishes
  it) (`drain::OUTCOME_PATHS`). Otherwise the run goes on, the source check is not asked, and the report says no
  landing changed the outcome (`outcome_changed`, and an `outcome` line). (2) *All page text against the
  source:* `RealSteps::check_page` reads every file of every `studies/<name>/` at the head commit except
  `pin.json` (the reference) and `validation.json` (rule names and pass flags) (`pin::STRUCTURAL_FILES`).
  Everything else is page text and goes to the checker: `claims.json`, `intro.json` (intro, transition,
  glossary), `rationales.json`, `findings.json`, `model.json` and any file added later, so a new text file is
  covered without anyone listing it. The core pins the source at the study's `pin.json` sha through the
  `Pinner` (and fails if the pin resolves to another commit), mounts it read-only beside the text, and asks
  one model call per study (`Athena::check_study_text`). The checker reports each statement the source
  contradicts and each fact it leaves out, with the file and the source lines, and lists the files it read; a
  file it does not list, a report that does not parse, a pin that fails, or text over 400,000 bytes fails the
  check. The failing statements are named in `held_back`. (3) *Floor:* each study's `model_contradictions`
  check from D99 still runs first, costs nothing, and fails the check without a model call. (4) *Budget:* the
  check is a model call per study, so `Steps::check_cap` is the verifier's cap times the number of studies
  and must fit the budget before the check starts, or drain stops as `budget spent`; what the calls cost is
  added to the spend whether the check passes or not. (5) *Role:* the checker runs as the verifier role, on the
  verifier's model, cap and timeout, with `Read`, `Grep` and `Glob` only; it is not the audience, whose
  stored prompt and rating records mean something else.
- **Alternatives:** (a) Check only `claims.json`: the intro, transition and glossary are what a newcomer reads
  first and would stay unchecked. (b) A list of the files to check: a file added later is silently
  unchecked, so the list is of the exempt files instead, and the default is covered. (c) One call over all
  studies: simpler to reserve, but one study's text can crowd out another's in one context, and the
  reserve for one call cannot tell how many studies it covers; one call per study keeps the reserve exact.
  (d) Count a landing outcome-changing by the job's declared deliverables: a producer can change more than it
  declared; the candidate's change list is what landed. (e) Keep only the model-free floor: it finds a
  model that contradicts itself and nothing the source contradicts. (f) A new role for the checker: a record
  schema change for a call that is not a job.
- **Tradeoff:** The check costs a model call per study on every rating at the target, and only a model
  reads the source, so it can miss a wrong fact or report one that is not wrong; a person still decides
  what to do with a held-back finding. The checker's invocation is not committed as a record (it belongs to
  no job, and a verifier invocation without a job would appear as one), so its cost is in the drain report
  and not in the records. A landing whose candidate record cannot be read counts as changing nothing. Paths
  are matched by prefix, so a new site generator outside `xtask/` needs `OUTCOME_PATHS` extended.
- **Reversal cost:** Low: remove `check_study_text`, the `Steps::check_cap` reserve and `OUTCOME_PATHS`; D99's floor
  remains.
- **Provenance:** measured: `crates/athena-cli/tests/drain.rs` (a wrong statement injected into the claims, the
  intro, the transition and a glossary entry, plus an omitted fact, each failing the real `check_page` with a
  fake agent and a stub pinner and holding drain back; a text file added to a study is covered; the reserve
  scales with the study count; the cost is in the spend; a landing outside the page sources cannot meet the
  target) and the unit tests in `drain.rs` and `pin.rs`. The live model was not run.
