# System facts

How Athena runs, as far as reading the code cannot show. The planner and critic briefs carry this file
whole. Each fact names its source. Where the source is a request and not code, the fact says so.

- **Host merge rules.** The host squash-merges pull requests. Main requires linear history and branches up to
  date before merging. Source: `docs/design/native-pr-landing.md` (squash-only support) and
  `docs/design/records-history.md` (branch rules on main and `athena/records`).
- **Producer snapshot.** A producer works in a one-commit snapshot of the base commit, in a git repository of
  its own with no remote and no other branches. `records/` and history are absent. Source: the
  "Working directory" text of the producer brief in `crates/athena-context/src/briefs.rs`.
- **Plain copy for readers.** The reader, planner and critic get a plain copy of the files, without git or
  `records/`. Source: the user's request of 2026-10-25 (BRIEF.md addendum); the code that makes the copy was
  not checked when this file was written.
- **Prompt sets and budgets.** Each role's always-loaded set has a fixed token budget, estimated as
  ⌈bytes / 3⌉. Producer and verifier are the largest across capabilities; the planner is measured against the
  producer budget and the critic against the verifier budget. Their briefs, this file included, sit outside
  these sets. A test (`system_facts_budgets_match_computed`) fails when a line below differs from the computed
  value; update the line, never the budget, to fix it. Line form: `- budget <set>: <size> of <budget>`.
  Source: `BudgetSet::budget` in `crates/athena-context/src/lib.rs`; `crates/athena-cli/tests/budgets.rs`.
  - budget contract: 326 of 350
  - budget reader: 1997 of 2000
  - budget producer: 1257 of 1300
  - budget verifier: 1298 of 1300
  - budget planner: 806 of 1300
  - budget critic: 783 of 1300
- **Banned words.** The boundary scan fails any tracked file naming who runs Athena or the machine it runs
  on. The list is not repeated here, so this file passes the scan. Source: `boundary_words()` in
  `xtask/tests/policy.rs`. Email-like text is checked by the `[personal_data]` allow-list in `athena.toml`;
  write none.
- **Request format.** `athena plan` reads: acceptance as bullets (a line starting `-`, `*` or `1.`),
  dependencies as job ids written in the text (such as `20261012-075824-1x28`), and a line `Priority: high`
  (levels urgent, high, normal, low). Without acceptance bullets or a reading's criteria, an item is refused
  for design or code. Source: `crates/athena-core/src/plan.rs`, `crates/athena-core/src/reading.rs`
  (`is_bullet`, `plan_entry`).
- **Land.** `athena land` lands only a job's own pull request, and only once it is merged and verified. Source:
  `crates/athena-cli/src/main.rs` (help text of the drain command), `crates/athena-core/src/gh.rs`.
- **Records branch.** Records live on the branch `athena/records` (own root, never merged into main) after
  C0, the cutover. Source: `docs/design/records-history.md`; `docs/decisions.md` D119.
- **Remotes.** An SSH remote is refused at admission; use an https URL, for example
  `https://github.com/example/project.git`. Source: the user's request of 2026-10-25; the code that refuses it
  was not found when this file was written. Related: `parse_remote` in `crates/athena-core/src/gh.rs`
  supports only github.com remotes for pull requests.
- **CI caches.** GitHub scopes a cache to the branch that saved it, so a pull request's entries serve only
  that branch; the shared toolchain, registry and target entries are saved only from the default branch.
  Source: `.github/workflows/ci.yml` (header and cache comments); `docs/design/ci-routing.md`.
- **Early pull request and CI waits.** Athena opens the pull request early, at verification start, and `land`
  reuses it. Before replacing the early head, `land` waits (every 20 seconds, up to 10 minutes) for the early
  commit's CI run, so the landing run finds the cached pass. Source: `crates/athena-core/src/gh.rs`
  (`early_run_wait`); `CHANGELOG.md` (Unreleased); `crates/athena-core/src/run_pin_tests.rs`.
