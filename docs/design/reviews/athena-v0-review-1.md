# Athena v0 design: independent review 1

**Verdict: approve with changes.** 1 blocker, 5 major, 13 minor, 5 nits.

| | |
|---|---|
| Under review | [`docs/design/athena-v0.md`](../athena-v0.md) and [`docs/decisions.md`](../../decisions.md) (D01–D37), as committed in `9851b36` |
| Requirements | `BRIEF.md` (O1, plus the parts of A1–A13, the Standards, and the Boundaries that v0 touches), and the designer's brief: `records/jobs/bootstrap/brief.md` and `brief-update-1.md` |
| Reference | Ryan Lopopolo, *Harness Engineering*, CC BY 4.0, <https://github.com/lopopolo/harness-engineering>, at commit `226c8d3` |
| Reviewer | Independent. The reviewer had no part in the design and no access to its author's reasoning, only to the artifacts. |
| Method | Read everything above in full, plus the bundle passages the design cites. Ran small experiments with git 2.43.0, with the pinned Claude Code 2.1.296's own help text and binary, with gitleaks 8.30.1's documentation, and against the pinned download locations. No experiment made a model call. The appendix lists them as R1–R10. |

Finding ids (B1, M1–M5, m1–m13, n1–n5) are stable, so the author's response and the re-check round can cite
them. Section numbers (§) and decision ids (D) refer to the design and the decision log; E1–E12 are the
design's own experiments (its Appendix A).

---

## Summary

The spine of this design is right and should survive intact:

- Deterministic code owns the job lifecycle, so no model context orchestrates (D01).
- Every agent is launched with a composed, measured context (§4, E1–E3).
- Records are immutable per-fact JSON files, and state is derived from them (D06, D35).
- Landing is keyed on a content fingerprint, so the bytes that land are the bytes that were verified (D10).
- Every number carries its provenance (D16).
- Every row of the enforcement map pairs a "blocks" test with an "allows" test (§8).

The writing is clear enough for a stranger, the evidence is labeled, and the documents contain none of the
terms the design's own boundary test targets (R10).

Six things should change before building:

- **B1 (blocker).** Agents can read every path their worktree hides, and can change the user's repository.
  Agents work in linked git worktrees and are allowed git. Sparse checkout keeps records and owner-private
  paths off the disk, but one `git show` or `git diff` prints them, `git sparse-checkout disable` restores
  them, and `git update-ref` moves the user's branch. Proposed fix: detached snapshot workspaces, each with its own
  throwaway repository.
- **M1.** A commit can relax the rules that judge it: the job's checks, the secret scanner's allowlist, and
  the exclusion attribute can all come from the content under judgment.
- **M2.** Traceability accepts any existing job id, so unverified changes can reach main and hand-made work
  escapes the intervention log. Proposed fix: the governed-content fingerprint may change only in a verified
  landing or a declared intervention.
- **M3.** Invocation records are lost on the failure paths (killed, left proposed, blocked, interrupted,
  orphaned), and nothing checks that the cost records reconcile.
- **M4.** The score summaries can be gamed by padding the predicted deliverables, and the log-scale interval
  score is undefined when a job costs nothing.
- **M5.** v0 carries more than its ten capabilities need. Cut Jev, the calibration view, five commands, the
  committed schemas, and the schema-freshness and decision-log tests.

None of this calls for a redesign. B1 swaps one mechanism behind `athena-git`'s workspace function. The
majors are local to the gate, the driver, and the scorer, and M5 is subtraction.

### Severity scale

- **blocker**: must change before building.
- **major**: should change before building.
- **minor**: fix during the build, or log it with a reason.
- **nit**: wording or polish.

---

## Findings

### B1 · blocker · Agents can read every path their worktree hides, and can change the user's repository

**Concerns:** §2.4, §4.1 (sparse worktrees), §5.5, §7.2 (the verifier brief), §7.5 (agent tool policy),
§8 rows M11 and M12; D08, D09, D22; E5 and E9.

**Evidence.**

- Every agent works in a *linked* worktree (§4.1): a second working directory that shares the object
  store, the refs, and the configuration of the user's repository. Sparse checkout keeps `records/` and
  `athena-exclude` paths off the disk, but their content stays one git command away.
- The producer and the verifier have Bash, and git is expected on their allow-lists. When the diff is over
  40 KiB, the verifier brief tells the verifier to "inspect it with `git diff`" (§7.2). The design does not
  list the allow rules. The hook denies twelve state-changing git subcommands by name (§7.5), which implies
  that other git commands are expected to pass.
- R1 tested a sparse linked worktree that excluded `records/` and an attribute-marked directory:
  - `git show HEAD:<excluded path>` printed the excluded files;
  - `git diff <empty tree> HEAD -- <excluded paths>` printed them too, so even an allow rule limited to the
    `git diff` that the verifier brief requires is enough to read them;
  - `git sparse-checkout disable`, which is not on the denylist, put them back on disk;
  - `git update-ref` on the shared branch, also not on the denylist, succeeded.
- R2: running `git sparse-checkout set` in a linked worktree wrote `extensions.worktreeConfig=true` into the
  user's shared repository configuration. That is a lasting change to the user's repository, and the design
  does not mention it.
- R3: the pinned Claude Code's help says `--restricted` "confines the file tools to the working
  directories". It says nothing about confining Bash, and the allow rules alone explain E5's refused
  `cat /etc/hostname`. Any allowed command (`cargo test`, a build) therefore runs with the user's full
  privileges. So do the candidate's own build scripts and tests, which the core also runs as checks (§7.2).
- This matters most for v0's main workload, changes to Athena itself. Athena's own commands read history
  through git (`athena show` runs `git log --grep`, §5.2). Athena code run inside a linked worktree, whether
  as a test or by the verifier, therefore reads the user's real repository. That includes this job's
  prediction commit, whose forecasts the verifier is never supposed to see (§4.1).
- The consequences:
  - M11 ("agents never change history or leave their worktree") and M12 ("agents never see records or
    owner-private paths") would pass the tests listed for them while the properties fail.
  - §5.5's "Corrupting other work is not possible" does not hold.
  - The designer's brief accepts v0 only if a job runs "isolated so parallel work could never corrupt other
    work", and says the repository root `CLAUDE.md` "must never reach Athena's agents". Yet
    `git show HEAD:CLAUDE.md` reaches it, and `git log` shows every commit message in the history, the
    owner's included.

**Proposed change.**

1. **Replace linked worktrees with detached snapshots.** For each agent, the core writes the filtered tree
   (without `records/` and without `athena-exclude` paths) into a fresh directory outside the checkout.
   Each directory is its own throwaway git repository:
   - the reader gets no repository at all, since it has no Bash;
   - the producer gets one commit ("base"), so it can review its own work with `git status` and `git diff`;
   - the verifier gets two commits (base, then candidate), so `git diff HEAD~1` is exactly the change.

   Write the files with `read-tree` and `checkout-index`. Do not use `git archive`, which drops
   `export-ignore` paths and rewrites `export-subst` files. With snapshots, no agent's git can reach the real
   history, refs, records, owner-private content, or configuration.
2. **Build the candidate commit C in the real repository from the snapshot's change list:**
   - diff the producer's snapshot against its base (`--no-renames`), and run `admit` on that list;
   - write the changed blobs with `hash-object -w`, and apply them to the prediction commit's full tree in a
     temporary index;
   - `write-tree`, then `commit-tree -p P`.

   Everything downstream (fingerprint, checks, verifier, landing) stays as designed.
3. **Replace the hook's git denylist with an allowlist:** `status`, `diff`, `log`, and `show`, on the
   snapshot's own repository, and nothing else. Deny `-c`, `--git-dir`, `--work-tree`, and `GIT_*`
   overrides.
4. **Scope the claims to what the mechanisms cover.** M11 and M12 cover agents' tool calls. Code that agents
   cause to run (checks, and allowed builds and tests) runs with the user's privileges. Say so in §16.1 and in
   the map. List containment of executed code in §14, pulled in by the first job that runs code Athena did not
   write. Claude Code's own Bash confinement is the natural candidate (m11).
5. **Add the escapes to the "blocks" column.** From an agent workspace, `git show <rev>:<excluded path>`,
   `git sparse-checkout disable`, `git update-ref`, and `git -c core.hooksPath=…` each fail or find nothing.
   Add one more test: a whole job leaves the user's `.git/config` byte-identical.

### M1 · major · A commit can relax the rules that judge it

**Concerns:** §6.2 (checks come from `athena.toml`), §7.2, §7.4 (the gate), §13, §16.1 (`.gitleaks.toml`);
D09, D17, D21.

**Evidence.**

- The design never says which revision's configuration governs a job. "Its checks come from
  `athena.toml`" (§6.2), and the checks "run … on the candidate tree" (§7.2).
  - If the candidate's copy is read, the producer chooses its own checks: a candidate that empties the list
    passes the check phase.
  - If the working tree's copy is read, uncommitted local edits decide, and nothing records them. `job.json`
    lists models and caps but no configuration hash (§5.3).
- R5: by default, gitleaks 8.30.1 relaxes itself from the content it scans:
  - it takes its rules and allowlist from `.gitleaks.toml` in the scanned tree unless `--config` is given;
  - it reads `.gitleaksignore` from the current directory;
  - it honors inline `gitleaks:allow` markers unless `--ignore-gitleaks-allow` is set.

  §7.4 sets none of these flags. So one commit can add a secret together with an allowlist entry, an ignore
  fingerprint, or an inline marker. The pre-commit hook scans a working tree that already holds the new
  allowlist, CI scans the commit that contains it, and all three gates honor the marker.
- The exclusion attribute is content too. A candidate that marks a path `athena-exclude` removes that path
  from the fingerprint, from every agent's view, and from `admit`'s owner-private rule (D09, §7.3).
- The verifier would probably notice such a diff. But the brief asks for mechanisms where the design would
  otherwise rely on a model probably noticing, and hand commits meet no verifier at all.

**Proposed change.** Adopt one rule: **no commit is judged by rules it changes.**

1. **Read each job's configuration from the prediction commit's tree,** never from the candidate or the
   working tree. That covers the checks, caps, models, verifier, admission policy, and defaults model.
   `job.json` records the hashes of the `athena.toml` and `defaults.toml` it used.
2. **Always pass gitleaks `--config`, `--gitleaks-ignore-path`, and `--ignore-gitleaks-allow`.**
   - The commit-time gates take the configuration and ignore list from the parent commit.
   - The full-history scan takes them from HEAD, where allowlisting an old false positive is legitimate.

   An allowlist change then governs only later commits, so a false positive is handled by committing its
   allowlist entry first. Consider a nonzero `--max-decode-depth`, so that encoded secrets are caught too.
3. **Refuse exclusion changes.** `admit` refuses a candidate that adds or removes an `athena-exclude`
   declaration, and a job's exclusion set is evaluated at its prediction commit (m1).
4. **Name the governance files:** `athena.toml`, `defaults.toml`, the gitleaks configuration and ignore file,
   `.gitattributes`, and the workflows. When a job changes one, the reading must list it as a deliverable, and
   the verifier frame requires an acceptance criterion that covers it.
5. **Add tests.** A candidate that deletes a check still has that check run. A commit that adds a generated
   token together with an allowlist entry, an ignore fingerprint, or an inline marker is refused.
6. **State the residual.** CI runs the workflow file of the commit it tests, so without branch protection,
   which is unavailable here, a commit can weaken CI itself. The hooks and the in-process gate run the adopted
   binary and are the backstop.

### M2 · major · Traceability accepts any existing job id, so unverified changes can reach main and hand-made work escapes the intervention log

**Concerns:** §7.3 (the traceability rule), §7.4 (check 5), §8 (rows M6 and M7, and the prose-only list);
D11.

**Evidence.**

- The rule asks only for a trailer "naming a job or record that exists at `HEAD`" (§7.3). A hand commit that
  changes code and carries `Athena-Job: <any earlier job>` or `Athena-Record: <any record>` passes both the
  commit-msg hook and CI.
- M7 ("nothing lands without passing verification") holds on Athena's own landing path. But CI audits only
  commits that carry `Athena-Verified-Content`, and a commit without that trailer is never audited. Merge
  commits are exempt outright, so a merge can introduce content nobody verified.
- The requirements this falls short of:
  - nothing reaches main without passing verification, and every commit traces to the work that produced it
    (A5);
  - every commit on main traces to accepted work (How I'll judge it);
  - doing work yourself is an intervention (O3), and every intervention is logged.

  The design leaves that kind of intervention to the user's memory (§8, prose-only list).

**Proposed change.** Make the governed-content fingerprint the thing the gate protects. Check it in the
commit-msg hook (the index's fingerprint against `HEAD`'s), in the in-process gate, and in CI over history:

- A commit that leaves its first parent's fingerprint unchanged touches only records and owner-private paths,
  and today's trailer rule applies.
- A commit that changes the fingerprint must be one of the following:
  - (a) A landing commit. Its `Athena-Verified-Content` equals its fingerprint, and the
    `records/jobs/<id>/verification.json` it adds holds that fingerprint with a pass.
  - (b) A declared intervention. It carries `Athena-Intervention: <id>` and adds the intervention record
    (category `did-work-themselves` or `fixed-environment`) in the same commit.
  - (c) A bootstrap commit, inside a range closed by one commit id recorded when v0 is accepted.
- A merge commit passes if its fingerprint equals one parent's, since a clean merge brings no new content.
  Otherwise it needs (a) or (b).

This makes "nothing reaches main unverified" a checked invariant of the history. It also captures the most
consequential kind of intervention by mechanism, which shortens the prose-only list. Say plainly that CI
detects a direct push rather than preventing it, because branch protection is unavailable.

Tests that block:
- a code change carrying an earlier job's trailer;
- a merge that introduces new governed content.

Tests that allow:
- a landing commit;
- an intervention commit with its record;
- a records-only commit;
- a commit that touches only owner-private paths.

### M3 · major · Invocation records are lost on the failure paths, and nothing checks that cost records reconcile

**Concerns:** §2.3 and §2.4 (what is committed, and when), §3.1 (`doctor --live`), §3.7, §5.3 (outcome
totals), §10.2, §10.4; D03, D28.

**Evidence.**

- **Killed invocations record nothing.** The driver reads usage and cost from the single result object that
  `--output-format json` prints when Claude Code exits (§10.2). An invocation killed at its deadline or by the
  kill switch (§10.4) prints no result, so its tokens and cost are unknown. The design does not say what is
  recorded then, and these are the expensive invocations.
- **Some records never reach a commit.** They live only in the state directory, which §2.4 calls disposable
  and which a fresh environment does not have:
  - the readings of a job left proposed, by `q` or by an `ask` without a terminal, which prints the reading
    and exits (§3.7);
  - a verified job `blocked` by local edits, whose closing records wait for `athena land`;
  - the in-flight producer of an interrupted job.

  When Athena's user is itself an agent, the first case is the normal path.
- **Some model calls belong to no job.** `athena doctor --live` and the opt-in live tests make model calls
  (§3.1, §11.2) that are never recorded.
- **Orphaned agents keep spending.** The `claude` child runs in its own process group (§10.2). If
  `athena go` itself dies, that group survives. The orphaned agent keeps spending, up to its
  `--max-budget-usd`, with no record. `athena stop` cannot reach it, because the polling (§10.4) ran in the
  process that died.
- **Nothing reconciles the numbers.** No rule relates `usage` to the per-model `modelUsage`,
  `total_cost_usd` to the per-model costs, or the `outcome.json` totals to the invocation records. The brief's
  test is "The token records are complete and reconcile."

**Proposed change.**

1. **Capture usage as it accrues.** Run agents with `--output-format stream-json`. Keep only each message's
   usage, discarding the content so that there is still no transcript, and persist the running sum in the
   state directory. On a kill, record the partial sum as `estimated`, with the basis "lower bound: usage
   before termination", and record the cap as the upper bound.
2. **Adopt one rule: no invocation record outlives the process that made it without being committed.** That
   means:
   - a records-only commit when `ask` exits with its job still proposed;
   - a closing commit for a `blocked` job, plus a later `landing` record when it lands;
   - `abandon` commits whatever the state directory holds, and records missing invocations as estimates.
3. **Record calls outside jobs.** Check credentials with `claude auth status` (R6), which is not an agent
   invocation. Record any model call made outside a job under a `maintenance` job kind.
4. **Tie agents to their parent.** Set the parent-death signal on the child (`PR_SET_PDEATHSIG` on Linux),
   record child process groups in the state directory, and have `athena stop` terminate every recorded group.
   Test it by killing `athena go` mid-invocation.
5. **Make reconciliation a check in `athena check`.**
   - Per invocation: the per-model costs sum to the reported total, within rounding, and the requested model's
     tokens match `usage`.
   - Per job: the totals in `outcome.json` equal the sum of the invocation records.

   A mismatch fails the check.

### M4 · major · Score summaries can be gamed by padding, and break at zero cost

**Concerns:** §9.1, §9.3, §9.4; D14, D15.

**Evidence.**

- **Padding lowers the deliverable score.** A deliverable event exists only for a path the reader lists
  (§9.1), and the summary reports the *mean* Brier score over those events (§9.4). The Brier score is the
  squared error of a probability.
  - Listing extra paths at p = 0.01 that will not change adds terms of 0.0001 each.
  - One real deliverable forecast at p = 0.95, plus nine padded paths, lowers the mean seven-fold.
  - A complete miss (0.81), padded the same way, reads as 0.08.

  Padding also inflates the reader's skill: the reference forecaster forecasts the same reader-chosen paths at
  a base rate, so it scores badly on the padding. Padding also makes `only_listed_paths` easier to forecast
  high, and a reader that lists nothing escapes the component entirely. Each Brier term is proper, but a mean
  over events the forecaster chooses is not. This is the failure the brief names: vague predictions must not
  score well (A2; designer's brief, item 6).
- **The interval score breaks at zero.** The weighted interval score is computed on ln(actual) (§9.3). Cost
  resolves to the producer's and verifier's cost (§9.1). That cost is $0 when the producer fails before any
  API call, and ln 0 is undefined. Quantiles out of order (p10 > p50) are not handled either.
- **Per-job skill is unstable.** Skill, 1 − S_forecaster / S_reference (§9.4), is unbounded below. It swings
  wildly whenever the reference's score is near zero, which happens whenever a base rate is near 0 or 1.

**Proposed change.**

1. **Score deliverables over a set no forecaster chooses.** Use the union of every forecaster's listed paths
   and the paths that actually changed, count an unlisted path as p = 0 (or a stated floor), and report the
   *sum* over that set, not the mean. Padding then adds about nothing, and a missed path costs about 1. Add a
   test that padding cannot lower the score.
2. **Floor before taking logs.** Floor actuals and quantiles at stated minimums (for example $0.001 and 1 s),
   and reject out-of-order quantiles when the reading is parsed.
3. **Report skill only for aggregates.** Compute it from summed scores, above a stated number of jobs. Per
   job, show raw scores.
4. **Drop the `models[m]` events** from scored summaries (M5).

### M5 · major · v0 carries more than its ten capabilities need

**Concerns:** §3.1, §8 (rows M19 and M21), §9.1, §9.2, §9.6, §11.2, §14; D15, D24, D31, D36. The designer's
brief says: "Keep v0 small. … Unless you argue otherwise, v0 leaves out: … Jev integration …".

**Evidence.** The design builds:

- 15 user-facing commands, plus hidden entry points;
- 12 record schemas, plus verbatim briefs;
- 12 lifecycle states and 3 forecasters;
- 22 enforcement mechanisms;
- 10 crates and 2 workflows;
- 37 logged decisions.

Several of these pieces serve none of the ten capabilities:

- **Jev as a shadow forecaster (D31).** The case for it is that "every job run without Jev is calibration
  data lost", and that does not hold. Jev's input is built only from fields the reading record stores (§4.1),
  and Jev never sees outcomes. A later job can therefore compute Jev's forecasts for every v0 job from the
  records, labeled as retroactive. Against that, Jev adds:
  - an HTTPS client whose proxy and certificate handling D23 itself rejected for setup;
  - a credential, plus a driver and its test fake;
  - refusal handling;
  - a third column in every reading and score.

  The user re-affirmed the default of leaving Jev out (`brief-update-1.md`).
- **`models[m]` events.** Code forecasts them from the routing that the same code enforces, so they are
  nearly tautological.
- **`athena calibration`, with reliability tables in tenths.** With a handful of jobs per class, the bins are
  empty.
- **Commands that fold into others:**
  - `show` and `calibration` into `status [<job>]`;
  - `drop` and `abandon` into `close <job> --why`;
  - `land` into `go <job>`: re-running `go` on a verified, blocked job retries the landing;
  - `doctor` into a setup check that makes no model call (M3).
- **Committed `schemas/`, with their freshness test (M19).** The Rust types are the source of truth, and
  `--json-schema` can be generated at run time.
- **The decision-log completeness test (M21).** It enforces a document format that nothing in v0 reads
  mechanically.
- **Two workflows**, where one workflow with two jobs and a weekly schedule does the same. D24 gives no
  reason for two.

**Proposed change.** Cut those pieces, and list each in §14 with the evidence that would pull it back in:

| Cut | Pulled back in by |
|---|---|
| Jev | The first calibration review, or the first typed decision in the flow |
| The calibration view | Ten scored jobs in one class |
| Committed schemas | The first non-Rust consumer of the records |
| M21 | The first incomplete decision-log entry that review missed |

That leaves 10 commands (`ask`, `correct`, `go`, `close`, `status`, `intervene`, `restart`, `check`,
`stop`, `start`), 2 forecasters, and 20 mechanisms, before the additions this review asks for.

Keep everything that carries a capability:

- the three roles and the prediction commit;
- fingerprinted landing, and the gate in three places;
- hermetic launch and `Quantity`;
- append-only per-job records, and derived status;
- the hook, as an allowlist;
- the caps and the kill switch;
- the defaults model;
- the reference forecaster, which is what makes a vague forecaster show zero skill;
- the crate split, which is what makes "no cycles" a mechanism.

### m1 · minor · Fingerprint and landing: three edge cases

**Concerns:** §7.3; D10, D19.

- **The fingerprint, as specified, is not a pure function of the commit.** R7:
  `git check-attr --source=<tree>` honored both `.git/info/attributes` and a user-global
  `core.attributesFile`. A file outside the repository can therefore mark governed paths as excluded. Such a
  file changes the fingerprint, the agents' exclusions, and what `admit` treats as owner-private, while CI,
  which lacks the file, computes a different fingerprint. The fix:
  - Evaluate the attribute from committed `.gitattributes` files only. Run git with an empty
    `core.attributesFile` and `GIT_ATTR_NOSYSTEM=1`, and refuse when `info/attributes` mentions
    `athena-exclude`.
  - Compute fingerprint(P), fingerprint(H), and the landed tree's fingerprint with P's exclusion set, where P
    is the prediction commit and H the branch tip.
  - Treat any change to the exclusion declarations between P and H as stale.
- **Landing does not check that P is still an ancestor of H.** Suppose the user resets the branch past P while
  the job runs. The replay merge, with P as its merge base, then drops the job's prediction records, and every
  gate still passes. Require `git merge-base --is-ancestor P H`, and close the job as stale otherwise.
- **The git minimum is wrong.** `merge-tree --write-tree --merge-base` and `check-attr --source` need git 2.40
  or later, not the 2.38 that D19 states. Setup should check the version.

### m2 · minor · The verifier's power to exercise the change is unspecified

**Concerns:** §2.2, §7.2; BRIEF A7.

- The verifier has "allow-listed Bash", but the design gives no list.
- The only exercise the design names is the capability's checks, and they run tests the producer wrote.
  Passing them is necessary, but it is not independent evidence.

Specify the verifier's allow-list (build and run the candidate). Require behavioral criteria to be checked by
running them, and record the commands as each criterion's evidence (Appendix B.3).

### m3 · minor · Classes of work are free text, invented per request

**Concerns:** §3.5, §9.6, Appendix B.1.

Calibration, intervention trends, and learning curves are all keyed by class, and records are immutable.
Synonyms such as `docs`, `documentation`, and `readme` would therefore fragment every per-class metric for
good. Seed a closed list in configuration, make the reader and `athena intervene` choose from it, and add a
class through a job.

### m4 · minor · The always-loaded budget can be escaped through role frames

**Concerns:** §4.4, §12; D25.

Only the shared contract has a fixed budget. Every agent of a role also loads its role frame and capability
section, and those get budgets "set at their measured size plus about 15 percent" when they are written. A
rule moved from the contract into a frame therefore escapes A13's "every addition is paid for by a removal".
Fix the budget of each role's whole always-loaded set now (see Q5).

### m5 · minor · Personal data in the change itself is guarded by prose only

**Concerns:** §5.6, §7.4, §12 (rule 4); M20.

Redaction covers agents' free text in records. Nothing gates the candidate's content or hand commits, though
the host injects the account's email address into every agent (E3). Apply the redaction pattern as a gate
check on the added lines of every commit, with the same `noreply` allow-list.

### m6 · minor · Re-score paths can collide

**Concerns:** §5.1, §5.2, §9.5.

An intervention command writes `score-<n>.json` into a closed job's directory. If two interventions are
recorded in different clones, both write `score-2.json` with different content, and the merge conflicts. Name
each re-score after the intervention that caused it (`score-<intervention id>.json`), and order re-scores by
time.

### m7 · minor · Five flows are underspecified

**Concerns:** §3.3, §3.5, §5.3, Appendix B.1, Appendix C.

- (a) **The reader's `question`.** How is it shown, how does the user answer, and what does Enter do?
- (b) **A request that no capability fits.** Is there a "declined" outcome, or is it forced into `change`?
- (c) **`restart --minutes` and `--tokens`.** How is each value labeled measured or estimated?
- (d) **The bootstrap's pending cost.** Which command, or documented procedure, writes the
  `athena.resolution/1` record that the user must add?
- (e) **The job's owner,** which A5's trace names. Which field holds it for an ordinary job?

### m8 · minor · D04 misattributes its key number

**Concerns:** §4.2; D04; E2.

D04 says that appending to the default system prompt "costs about 17k tokens per agent". But E2's 17,586
tokens include the 25 default tools, and option (a) can restrict the tools too. The quality cost of dropping
Claude Code's own system prompt for the producer is unmeasured. Measure the default prompt with each role's
tools, and decide on cost per verified result. Reversing the choice is one flag.

### m9 · minor · Adopting a build from a dirty tree breaks reconstruction

**Concerns:** §4.4; §11.1, step 3.

Setup builds whatever the checkout holds, while §4.4 promises that any agent's context "can be reconstructed
exactly from the Athena version that ran it". Embed the source commit and a dirty flag in the binary, record
both in every invocation, and refuse to adopt a dirty build without an explicit flag.

### m10 · minor · Cold builds in fresh workspaces

**Concerns:** §7.2, §10.4; D12.

Every agent workspace starts with no build output, and Claude Code's Bash tool times out after two minutes by
default. A cold `cargo test` of a ten-crate workspace may take longer. Three changes:

- Set `BASH_DEFAULT_TIMEOUT_MS` and `BASH_MAX_TIMEOUT_MS`, which the pinned version supports (R3), in the agent
  environment.
- Prefetch dependencies with `cargo fetch --locked` during setup.
- Consider one build directory per job, shared by the producer and the checks.

### m11 · minor · Two buy-before-build calls are missing or inconsistent

**Concerns:** D22, D23, D36.

- **A missing option.** D22's alternatives omit Claude Code's own confinement of Bash commands, which the
  pinned version ships (R4). On Linux it needs bubblewrap, which is not installed in this environment.
  Evaluate it and record the call: it is the natural containment for item 4 of B1.
- **An inconsistent call.** D23 rejects a Rust HTTP client for setup because curl already handles proxies and
  certificate bundles, while D31 and D36 adopt one for Jev. This is moot if Jev is cut (M5). Otherwise,
  reconcile the two.

### m12 · minor · The reader/producer split is justified by the wrong reason

**Concerns:** §2.2, §15; D01.

The design invokes the bundle's rule that a separate context earns its cost when it buys independent
evidence. But its reason for a separate reader is ordering: "the reading and its predictions must exist
before execution". A single trajectory, paused for acceptance and then resumed, would give the same ordering
more cheaply, since the producer re-reads what the reader read.

The split is defensible on other grounds: forecasts must not become the executor's targets, and the reader
stays read-only. Say that in D01 and §15, and record the split as a departure from the bundle's
one-trajectory default.

### m13 · minor · `records/jobs/` is one flat directory

**Concerns:** §5.2, §5.4.

Every commit rewrites the tree object for `records/jobs/`, and that object grows with every job. Shard the
directory by month (`records/jobs/2026-10/<id>/`); the time-prefixed ids make that free.

### n1 · nit · Attribution, and one overstated departure

**Attribution.** §15 names the work, its author, its license, and its URL, and says that Athena adapts the
bundle's ideas. Three additions:

- link the license, <https://creativecommons.org/licenses/by/4.0/>;
- say plainly that changes were made, as the bundle's `COPYING.md` asks of adaptations;
- say where the attribution will live in the built repository: the README, or a NOTICE file.

**An overstated departure.** §15's "the bundle has the worker produce its own proof" understates the
bundle. Its whole-job closure includes adversarial review, and its one-trajectory thesis endorses a separate
context for exactly that. Athena's verifier follows the bundle here rather than departing from it.

### n2 · nit · D18 cites evidence an adopter cannot check

D18's provenance reads "the repository already uses this pattern for an existing hook". The one existing hook
in this clone is the repository owner's own tool, which is outside Athena. Replace that evidence with an
experiment in the design's Appendix A.

### n3 · nit · `.gitleaks.toml` is missing from the non-Rust inventory

§16.1 relies on `.gitleaks.toml`, but §11.3 does not list it, so test M17 would fail. Alternatively, keep the
allowlist in `athena.toml` and hand it to gitleaks with `--config` (M1).

### n4 · nit · Hermetic flag hygiene

- `--restricted` already ignores user, project, and local settings files (R3), so `--setting-sources ""` is
  redundant. Keep one, and test it.
- E5 should say that the Bash refusal came from the allow rules.
- The pinned version has auto-memory (R3). Set `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1` in the agent environment,
  and add a planted memory file to M10's canary.

### n5 · nit · The token estimator's domain

⌈bytes / 3.0⌉ errs high for prose like the contract, which measured 3.3 bytes per token. It is not
guaranteed to err high for code or non-ASCII text, which can run below 3 bytes per token. Say that the live
measurement is the authority, and that the estimator is a tripwire for prose.

---

## Fit: the ten capabilities and the acceptance test

| # | Capability (designer's brief) | Delivered? | Open findings |
|---|---|---|---|
| 1 | Take a lazy request | Yes (§3) | m7 (b) |
| 2 | Show the reading, correctable in one line, and record checkable predictions before dispatch | Yes (§3.2, §3.3, §9.1) | M4, m7 (a) |
| 3 | Dispatch one job to a sub-agent | Yes (§2, §7.1) | — |
| 4 | Verify independently; nothing verifies its own work | Partly | B1, M1, m2 |
| 5 | Record every model invocation | Partly | M3 |
| 6 | Score with a proper rule, so vague predictions do not score well | Partly | M4 |
| 7 | Land a commit from which the whole trace can be followed | Yes (§7.3) | M2, m1 |
| 8 | Gate every commit for secrets, and scan the full history in CI | Partly | M1 |
| 9 | Record interventions and restarts | Yes (§3.5) | m3, m7 (c) |
| 10 | Get the state of the work from the repository alone | Yes (§3.6; D35) | M3 |

The acceptance test, step by step:

- **One command takes a clean clone to a working Athena.** Yes (§11.1), and every pin resolves (R8). Setup
  should also check for git 2.40 or later (m1), and for a C linker, which `cargo` needs on Linux and rustup
  does not install.
- **The reading and predictions appear before any work.** Yes.
- **The job runs isolated, so parallel work could never corrupt other work.** Not as designed (B1).
- **An independent verdict, a traceable commit, a scored prediction, and machine-readable records in the
  repository.** Yes, once M1, M3, and M4 are addressed.
- **`cargo test`, clippy, and CI pass.** Plausible. The history check passes on today's history: no commit
  modifies a record, and every commit after the first carries `Athena-Job: bootstrap` (R9).
- **Nothing in Athena refers to who runs it or where, and a test enforces that.** The design's documents
  pass a scan for the terms that test targets (R10). This review is written to pass it too.

---

## Shape, enforcement, buying, and craft

- **Shape.** The core/capability seam is one trait (§6.2). The dependency graph is acyclic by construction
  and checked against its documentation (§6.1). The non-Rust files are justified, and the 30-line
  `setup.sh` is a fair platform exception. One file is missing from the inventory (n3). The design does not
  depend on this environment anywhere except D18's evidence (n2).
- **Enforcement.** In form, the map is the design's strongest section: every row names what it blocks and
  what it allows. Its substance is weaker:
  - two rows claim more than their mechanisms deliver (B1);
  - the gate reads its rules from the content it judges (M1);
  - the rule that matters most after acceptance, "nothing reaches main unverified", is enforced only on
    Athena's own path (M2).

  In the other direction, M21 enforces a document format that nothing needs yet (M5).
- **Buy before build.** The build-it-ourselves calls are small and justified: the scoring formulas, the
  base32 encoder, the token estimator, the hook in Rust, and setup in Rust. The options considered are real.
  One option is missing, and one call is inconsistent (m11).
- **Craft and the bundle.** The design is coherent where it matters. Its spine reads as inevitable. Its
  periphery (three forecasters, fifteen commands, the calibration tables) reads as assembled, and the cuts in
  M5 remove most of that. A few absolute claims outrun their mechanisms (§0, §5.5; B1). The bundle is used
  substantively and mostly faithfully, with every departure recorded, though two statements need correction
  (m12, n1).

---

## Answers to the open questions (§16.2)

1. **Prediction commit on the branch, or notes or a side ref (D07)?** Keep the commit on the branch, for
   three reasons:
   - It puts each forecast in the same history as the outcome it forecasts. Every clone, every CI run, and
     the append-only gate see it without extra configuration. Notes and side refs are not fetched or pushed
     by default, and a side ref can be force-moved without leaving a trace in the history.
   - It survives a crash: an interrupted job still leaves its prediction on record.
   - The cost is one extra commit per job.

   One caveat: commit order is evidence only once the commit is shared, because a local rebase can rewrite
   it. Strengthen it cheaply. The closing commit carries `Athena-Prediction: <commit id>`, and CI checks that
   the named commit is an ancestor and that it introduced the forecast files, which the append-only gate then
   protects.
2. **Opus as the default verifier?** Yes, for v0.
   - Model-level separation is the cleanest way to make "nothing verifies its own work" hold while the
     producer may be Sonnet.
   - Most early jobs will change Athena itself, and A7 asks for the highest tier for high-stakes
     conclusions, especially conclusions about Athena's own quality.
   - Keep the configuration rule that the verifier model lies outside the producer set.

   Let evidence pull in a cheaper verifier for a class: verification cost dominating that class's cost per
   verified result, with no escapes over enough jobs.
3. **Is a one-line reading correction an intervention?** The user has answered this: yes, and Athena records
   it as one automatically. This review does not re-open it.
4. **A git attribute to mark owner-private paths (D09)?** Yes, as the *declaration*. It is generic, it
   travels with the tree, and it keeps those paths out of Athena's own configuration. Four conditions:
   - (a) enforcement must not rest on sparse checkout of a linked worktree (B1);
   - (b) the attribute is evaluated from committed files only (m1, R7);
   - (c) it is evaluated at the job's base commit;
   - (d) no candidate may change it (M1).

   Document it as the one setting an adopting owner needs.
5. **Is 350 tokens the right contract budget?** Keep it. With 294 tokens in use, the headroom is about one
   rule, and that is what makes "every addition is paid for by a removal" bite. Also close the escape route:
   give each role's whole always-loaded set a fixed budget now, held under the same discipline (m4). For the
   producer and verifier, that set is the contract, the role frame, and the capability section; the reader's
   set adds the defaults model and the capability catalog. The live count is the authority, and the byte
   estimator a tripwire (n5).
6. **Jev as a shadow forecaster in v0?** Leave it out, as the user's default says.
   - The argument for including it, that calibration data would be lost, does not survive the design's own
     data model. Everything Jev would read is in the reading record, so a later job can compute Jev's
     forecasts for every v0 job, labeled as retroactive.
   - Leaving it out removes an HTTPS client, a credential, a driver, and a forecaster column (M5).
   - Pull it in at the first calibration review, or with the first typed decision in the flow.

---

## Appendix: experiments run for this review

All ran on 2026-10-10. None made a model call or cost anything.

| # | What was checked | Result |
|---|---|---|
| R1 | A sparse linked worktree (git 2.43.0) that excludes `records/` and an attribute-marked directory | `git show HEAD:<excluded path>` printed the excluded files, and so did `git diff <empty tree> HEAD -- <excluded paths>`; `git sparse-checkout disable` restored them to disk; `git update-ref` on the shared branch succeeded |
| R2 | `git sparse-checkout set` inside a linked worktree | The main repository's configuration gained `extensions.worktreeConfig=true` |
| R3 | The help text and binary of the pinned Claude Code 2.1.296 | `--restricted` confines the file tools and ignores user, project, and local settings files; the help does not say it confines Bash. `--permission-prompts none` and `--disable-slash-commands` exist. `BASH_DEFAULT_TIMEOUT_MS`, `BASH_MAX_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_AUTO_MEMORY` appear in the binary. |
| R4 | The same binary, for built-in confinement of Bash commands | It carries settings that confine Bash commands (an on switch, an option to auto-allow confined commands, and a list of excluded commands) and references to bubblewrap; `bwrap` is not installed in this environment |
| R5 | The README of gitleaks 8.30.1 | Configuration precedence ends with `.gitleaks.toml` in the target path; `--gitleaks-ignore-path` defaults to `.`; inline `gitleaks:allow` markers are honored unless `--ignore-gitleaks-allow` is given |
| R6 | `claude auth --help` | It offers `status` ("Show authentication status"), a credentials check that is not an agent invocation |
| R7 | `git check-attr --source=<tree> athena-exclude` | `unspecified` with committed attributes alone; `set` once `.git/info/attributes`, or a global `core.attributesFile`, marks the path |
| R8 | The pins in §11.1 | The gitleaks tag `v8.30.1` exists; the rustup-init 1.28.2 checksum URL answers 200; npm serves `@anthropic-ai/claude-code` 2.1.296 |
| R9 | This repository's history | No commit modifies, deletes, or renames anything under `records/`; every commit after the first carries `Athena-Job: bootstrap` |
| R10 | The design and the decision log, scanned for the terms the boundary test targets | None found as whole words |
