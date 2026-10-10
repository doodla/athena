# Athena v0 design: independent review 2 (re-check)

**Verdict: approve with changes.** The blocker is resolved. Of the five majors, three are resolved and two
are partly resolved. The revision introduced three new major issues. Each open item reduces to a decision
the user can make in one line (see [Decisions for the user](#decisions-for-the-user)), and none calls for a
redesign.

| | |
|---|---|
| Under review | Revision 2 of [`docs/design/athena-v0.md`](../athena-v0.md) and [`docs/decisions.md`](../../decisions.md), commit `50bb230`, including the design's Appendix D (the response to review 1), decisions D38–D45, and experiments E13–E20 |
| Scope | Review 1's blocker and majors (B1, M1–M5), plus anything new of blocker or major severity. Minors and nits are not reopened, except where a response to one created a new problem. |
| Reviewer | The author of [review 1](athena-v0-review-1.md): independent of the design, with access only to the artifacts |
| Method | Read revision 2 in full. Re-ran review 1's isolation and attribute experiments against the new mechanisms, and ran new experiments wherever the revision's own mechanisms needed testing. The appendix lists them as R11–R16. No experiment made a model call. |

Section numbers (§) and decision ids (D) refer to revision 2.

---

## Status of review 1's blocker and majors

| Finding | Status | Evidence | Open decision |
|---|---|---|---|
| **B1** Agents could reach hidden paths and change the user's repository | **Resolved** | Detached snapshots (§4.1, D08) replace linked worktrees. Rebuilt exactly as §4.1 specifies, a snapshot closed every escape from review 1 (R11). The hook is an allow-list (§7.5). The claims are scoped to tool calls, and the executed-code residual is stated (§7.5, §16.2). | — |
| **M1** A commit could relax the rules that judge it | **Partly resolved** | Fixed: the channels review 1 named. Job configuration comes from the base commit (§6.2, D38); gitleaks always runs with Athena's own configuration (§7.4, E17); classification changes are refused, governance files must be listed deliverables, and attributes come from committed files only (§7.3, §4.2, R12). Still open: every check the capability runs can be switched off by a file the candidate controls that is not on the governance list (R13). | Decision 1 |
| **M2** Traceability accepted any existing job id | **Resolved** | The governed-content invariant (§7.3, D39) admits three kinds of commit: (a) a verified landing, (b) a declared intervention, (c) the bootstrap range, plus clean merges. It is checked in the hooks, in process, and over history. `athena-readonly` (D44) keeps the owner's own files outside the invariant without letting jobs change them. Today's tracked content also passes the new personal-data check (R16). | Decision 2, a procedural step |
| **M3** Invocation records were lost on failure paths | **Partly resolved** | Fixed: all five points, one by a different route than proposed (no model calls outside jobs, rather than a maintenance job). Usage is captured as it accrues (§10.2, E16), records are committed before each process exits (§2.3), credentials are checked by `claude auth status` (E20), agents die with their parent (§10.2, §10.4), and reconciliation is a check (§10.3). Still open: model calls started by code an agent runs escape the records and the caps (N2). | Decision 4 |
| **M4** Score summaries could be gamed and broke at zero cost | **Resolved** | Deliverables are scored as a sum over the union of listed and changed paths, with unlisted paths at p = 0 (§9.3). Listing a path at its true probability q has expected score q(1 − q), never worse than the q of leaving it out, so the rule rewards honesty and padding gains nothing. Floors, quantile-order validation, aggregate-only skill, and an unscored models check are in place (§9.1, §9.3, §9.4). | — |
| **M5** v0 carried more than its capabilities need | **Resolved** | There are 10 commands (§3.1), 2 forecasters (§9.2), and 1 workflow (§11.2). There are no committed schemas and no decision-log test (§5.1, §11.3). Jev is out, with retroactive forecasts as its path back (§9.2, D31). Each cut is listed in §14 with the evidence that would pull it in. The only unrequested addition, `athena-readonly`, earns its place. | — |

### Notes on each finding

**B1.** R11 built a snapshot exactly as §4.1 specifies: a temporary index, exclusion by committed rules, `checkout-index`, and the snapshot's own repository with no global or system configuration. From inside it:

- `git show <user commit>:<excluded path>`, `git show HEAD:CLAUDE.md`, and `git diff <empty tree> HEAD -- <excluded paths>` found nothing;
- the user's commit object was absent, and there were no alternates;
- `git update-ref` moved only the snapshot's own branch;
- no file in the snapshot named the user's repository path.

The user's `.git/config`, and every file under `.git` outside the object store, stayed byte-identical. Code that agents run still has the user's privileges, and §16.2 now says so plainly.

**M1.** Revision 2 states the principle "no commit is judged by rules it changes" (D38), and applies it to Athena's own configuration. The check tools read their configuration from the candidate, though, and none of these files is on §7.3's governance list. R13 showed each one switching a failing check to a pass, on cargo 1.97:

- **`cargo test`.** `.cargo/config.toml` with `[target.'cfg(all())'] runner = "true"` turned a failing test (exit 101) into exit 0 ("test result: ok. 0 passed"). The same file defeats the verifier's own `cargo test`, which D43 counts as evidence because the cited command did run.
- **`cargo clippy -- -D warnings`.** A `[lints.clippy] all = "allow"` table in `Cargo.toml` turned a failing run into exit 0.
- **`cargo fmt --check`.** `rustfmt.toml` with `disable_all_formatting = true` turned exit 1 into exit 0, with no warning.

The verifier would see these files in the diff, but noticing them is model judgment, which is what M1 asked mechanisms to replace. Code-level relaxations, such as `#[ignore]` on a failing test, remain the verifier's to judge, which is reasonable.

**M2.** The invariant is sound as specified. One procedural point remains. Range (c) stays open until the bootstrap writes `records/jobs/bootstrap/outcome.json`. Until then, any commit labeled `Athena-Job: bootstrap` can change governed content unverified. The record is append-only, so it can be written once, and it must name the true last bootstrap commit. Writing it belongs at acceptance, and nothing in §17 or `athena status` prompts for it yet.

**M3.** The five mechanisms review 1 asked for are all specified, with experiments behind them. The remaining gap is a path the revision made routine, described in N2.

**M4.** The revision adopted every proposal, the tests are named, and the propriety argument holds.

**M5.** The text grew by about 5,000 words, and the enforcement map from 22 to 24 rows. That growth is almost entirely mechanisms the other findings asked for. The cuts M5 asked for were all made.

---

## New issues introduced by the revision

Each issue arises from a change made in response to review 1. N1 comes from the m10 response, N2 from the m2 response, which review 1 itself proposed, and N3 from the B1 response.

### N1 · major · The persistent build caches let one job's checks run another job's build

**Concerns:** §10.4 (build caches); §5.5 and §7.2 (the verifier judges exactly what will land); BRIEF A5, and the acceptance condition that a job runs "isolated so parallel work could never corrupt other work".

**Evidence.**

- The revision points `CARGO_TARGET_DIR` at two caches, `target/produce` and `target/verify`, that persist across jobs. This replaced review 1's per-job suggestion (Appendix D, m10).
- Cargo decides whether a workspace crate's build is fresh by comparing file modification times. Two snapshots of the same workspace produce the same artifact names.
- R14 put two jobs' copies of one package, with different sources, on one shared cache. Job B's snapshot files were older than job A's build.
  - B's `cargo test` recompiled nothing ("Finished … in 0.01s").
  - It ran A's binaries: A's 15-second test, then a test printing "built from job A".
  - With a private build directory, B ran its own code.
- In Athena, this happens whenever one job's verifier snapshot is written while another job's checks are compiling. A check phase takes minutes, so the window is wide.
- The consequences:
  - the checks and the verifier's own `cargo` commands attest to the wrong code;
  - a candidate whose tests never ran can pass and land;
  - on the producing side, concurrent producers see each other's test results.

**Proposed change.** Give each job its own build directories, one per side, under the state directory. This keeps the separation between the producer's and verifier's builds that §10.4 wants, and it removes cross-job reuse. Cold workspace builds get slower. If speed matters later, a content-addressed compiler cache (sccache, for example) is the buy option, because it keys its outputs by content, not by modification time.

### N2 · major · Athena cannot safely exercise Athena inside a job

**Concerns:** §7.2 and D43 (behavior criteria must cite commands the verifier ran, and the verifier may `cargo run`); §7.5 ("agents do not spawn further agents"); §8 rows M13 and M14 (spend bounded; every invocation recorded); §10.2 ("no model call outside a job"); §11.2 (live tests run when credentials are present).

**Evidence.**

- v0's main workload is changing Athena itself. Exercising a change to `ask`, `correct`, or `go` means running the candidate's own binary in the verifier's snapshot (`cargo run -- ask …`). D43 now makes that exercise mandatory for behavior criteria.
- Agents keep `HOME` and `PATH` (§4), and those two variables are all that authentication needs (E7). R15: from inside a snapshot, with an environment cleared to `HOME` and `PATH`, `claude auth status` reported logged in.
- So the nested Athena's reader, and its producer and verifier on a `go`, make real model calls:
  - outside the job's records: the nested Athena records them in the snapshot's throwaway repository, which is discarded;
  - outside the job's caps: the nested caps come from the candidate's own `athena.toml`.

  The same holds for the live tests, through `cargo test -- --ignored`.
- A non-interactive `ask` also commits its reading (§3.7). Run in the verifier's snapshot, that commit moves the snapshot past C, and §7.2 then invalidates the verifier's own verdict.
- History-dependent commands (`status`, `check --history`) see a one- or two-commit repository without records. So for changes to Athena's own commands, behavior criteria either cost unrecorded money or cannot pass.
- Nothing in the design guards this path.

**Proposed change.** Add a nested-run mode, with these parts:

- The core sets a marker in every agent and check environment.
- Under the marker, Athena uses its fake agent driver (§10.1 already has one) and a scratch repository it creates for the run, never the snapshot's own repository.
- The hook denies `--ignored` and `--include-ignored` to agents.
- Add rows to the map: a test that, under the marker, the real driver refuses to start; and a test that a nested `ask` leaves the snapshot at C.

If the user declines, state the path in §16.2, scope M14 to Athena's own invocations, and have the reader mark criteria about Athena's model-calling commands `content`.

### N3 · major (one-line fix) · The producer cannot delete, rename, or change the mode of a file

**Concerns:** §7.5 (the hook's program list), Appendix B.1 (`action: create|modify|delete`), §2.2 (producer tools).

**Evidence.**

- The hook's Bash allow-list admits only three programs: `cargo` (listed subcommands), read-only `git`, and `ls` (§7.5, line 973; D22).
- None of the producer's tools (Read, Grep, Glob, Edit, Write) deletes a file.
- So a producer cannot remove or rename a file, or set an executable bit, yet a reading may promise a `delete` deliverable (B.1). E15's deletions were made by the experimenter, not by an agent under this policy.
- The first request that removes or renames something will fail.

**Proposed change.** Add `rm`, `mv`, and `chmod` to the producer's list. The hook already confines their paths to the snapshot. Add an allow test that deletes and renames inside the snapshot, and a block test for a path outside it.

---

## Decisions for the user

Each line is one open item; the recommended answer is given first.

1. **M1, check-tool configuration.** Yes (recommended): treat `.cargo/config.toml`, `rustfmt.toml`, `clippy.toml` (each with its dotted variant), and edits to `Cargo.toml`'s `[lints]` or `[workspace.lints]` tables as governance files, and fail a test check that runs zero tests. No: leave them to the verifier's judgment, as a stated residual.
2. **M2, closing the bootstrap range.** Yes (recommended): make committing `records/jobs/bootstrap/outcome.json`, naming the last bootstrap commit, the final step of acceptance, after you accept v0, and have `athena status` flag an open range. No: leave closing the range to the bootstrap's discretion.
3. **N1, build caches.** Yes (recommended): give each job its own build directories, one per side, in place of caches shared across jobs. No: keep shared caches and accept that concurrent jobs can verify each other's builds.
4. **N2, and the rest of M3: nested runs.** Yes (recommended): add a nested-run mode (a marker in every agent and check environment, the fake driver, a scratch repository, and no live tests), so exercising Athena inside a job never spends, commits, or calls a model. No: record the path as a residual and narrow the claim that every invocation is recorded.
5. **N3, file operations.** Yes (recommended): let the producer run `rm`, `mv`, and `chmod` inside its snapshot. No: drop `delete` from the reading's deliverable actions.

---

## Appendix: experiments run for this re-check

All ran on 2026-10-10, with git 2.43.0, cargo 1.97.0, and the pinned Claude Code 2.1.296. None made a model call.

| # | What was checked | Result |
|---|---|---|
| R11 | A snapshot built exactly as §4.1 specifies, from a repository with `records/`, an owner-private directory, a root `.gitattributes` marking `CLAUDE.md` excluded and `BRIEF.md` read-only, and a symlink | Excluded and record files were absent, and the symlink was kept. From inside, `git show` of an excluded path and `git diff <empty tree> HEAD` over excluded paths found nothing. `git log --all` showed one commit. The user's commit object was absent, and there were no alternates. `git update-ref` moved only the snapshot's branch. No snapshot file named the user's path. The user's `.git/config`, and all non-object files under `.git`, were byte-identical. |
| R12 | `git check-attr --source=<commit>` with a global attributes file marking a governed path | `unspecified` with `core.attributesFile=/dev/null` and `GIT_ATTR_NOSYSTEM=1`, as §4.2 specifies; `set` without them |
| R13 | The capability's three checks, run against candidate-controlled tool configuration | `cargo test` on a failing test exited 101, and 0 after adding `.cargo/config.toml` with `runner = "true"`. `cargo clippy -- -D warnings` exited 101, and 0 after adding `[lints.clippy] all = "allow"` to `Cargo.toml`. `cargo fmt --check` exited 1, and 0 after adding `rustfmt.toml` with `disable_all_formatting = true`. |
| R14 | Two copies of one package, with different sources, sharing one `CARGO_TARGET_DIR`; the second copy's files older than the first copy's build | The second copy's `cargo test` recompiled nothing (0.01 s) and executed the first copy's test binaries ("built from job A", plus its 15-second test). With a private target directory, it ran its own code. |
| R15 | `claude auth status`, run inside a snapshot directory with an environment cleared to `HOME` and `PATH`; only the `loggedIn` field was read | `loggedIn = True`: a nested `claude` started by code an agent runs would authenticate |
| R16 | Email addresses in tracked governed and record content, outside the allow-list | None found, so today's history passes the new personal-data check |
