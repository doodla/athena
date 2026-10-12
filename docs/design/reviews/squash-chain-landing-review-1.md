# Squash-chain landing design: independent review (review 1)

**Verdict: approve with changes.** 0 blockers, 3 major, 5 minor, 2 nit. Option B (a chain-aware landing rule) is
the right direction and the problem statement matches the code. But the design as written has three gaps that
should be closed before a build job is asked for: it does not say how a chain with repeated or unchanged
fingerprints is ordered (R1-M1), its security argument says more than the verifier's inputs support (R1-M2),
and it does not say which path-classification rules each recorded fingerprint uses (R1-M3). None forces a
different option.

| | |
|---|---|
| Under review | [`docs/design/squash-chain-landing.md`](../squash-chain-landing.md), at the base commit of this job |
| Checked against | `crates/athena-gate/src/invariant.rs`, `crates/athena-gate/src/jobs.rs`, `crates/athena-gate/src/history.rs`, `crates/athena-gate/src/landing_tests.rs`, `crates/athena-data/src/records.rs`, `crates/athena-git/src/attrs.rs`, `docs/decisions.md` (D60, D66 to D69), and `docs/design/athena-v0.md` (the "Merge, squash, and rebase merges" passage of §7.3, and §6.3) |
| Method | Read the design in full, then checked each claim it marks **(read)** against the code it names, and each claim it marks **(inferred)** against the code that would decide it. No command was run to reproduce a failure or to try the proposed rule. |

Finding ids (R1-M1, R1-m1, R1-n1, and so on) are stable so a response can cite them. "§" is a section of the
design. Line numbers are those of the base commit.

## Severity scale

Blocker: must change before building. Major: returns the design to its author. Minor: fix before the build job
or log with a reason. Nit: wording.

---

## Part 1 · Do the design's claims match the code?

| Claim in the design | Where | Result | Evidence |
|---|---|---|---|
| `landing` refuses a commit that adds closing records of more than one job | §1 item 1 | **Holds** | `invariant.rs` 262-268: `closing_jobs(judged)`, then a second `jobs.next()` refuses with exactly the quoted text. |
| `landing` requires `candidate.fingerprint` and `verified_fingerprint` to equal the whole landed tree's fingerprint | §1 item 2 | **Holds** | `invariant.rs` 349-359 compare both with `fp`, which `check` computes as `fingerprint(git, &change.tree, &judged.rules)` (line 115). |
| `closing_jobs` collects the four closing record names | §1 | **Holds** | `invariant.rs` 217-243: `candidate.json`, `verification.json`, `outcome.json`, `outcome-landed.json`, status `A` only. |
| `Candidate` records `base_commit` and `fingerprint` but not the base's fingerprint | §2 | **Holds** | `records.rs` 582-591. The struct is `deny_unknown_fields`, as §3 B says. |
| The rules R are "the rules of P" | §4.2 | **Holds** | `Judged::new` sets `rules` to the change's base, the first parent (`lib.rs` 247-249; module doc: "judges a change by the rules of its parent commit"). |
| Per-job checks in step 5 are the ones `landing` runs | §4.2 step 5 | **Mostly holds** | The list matches `invariant.rs` 294-405, but omits the "job.json is in the landed tree and names an accepted reading" check (296-303) and the three-way fresh/retried outcome choice (316-341); the design mentions the latter only in §4.6. See R1-n1. |
| `landing_job` and `HistoryState::landed` must record every job of the chain | §5 item 3 | **Incomplete** | True, but `jobs::check` also calls `landing_job` and compares the one returned job to each landed outcome (`jobs.rs` 330-344). The design never lists it. See R1-m1. |
| "Rebase merge and merge commit keep today's single-job commits and never reach the chain path" | §4.6 | **Holds for rebase; unchecked for merge commits** | `check` calls `landing` for a merge commit too (`invariant.rs` 126-134). A merge commit whose diff against the first parent adds several jobs' records reaches the chain path. See R1-m2. |
| Where the core writes `base_commit` and `fingerprint` | §4.1 | **Unverified, said so** | The design marks this inferred. Not needed for the verdict; see R1-m3 for what it should decide. |
| Verifier "judged the whole candidate, including everything earlier jobs had added" | §3 B, §4.4 | **Not supported** | `docs/design/athena-v0.md` §6.3 says the verifier receives "a `CandidateView` built by the core from git (the diff and stat)". See R1-M2. |
| §7.3 describes a squash as the code and records of "*the* job" | §1 | **Holds** | `docs/design/athena-v0.md`, "Merge, squash, and rebase merges". |
| D66 records the cause, D67 exempts ee4b0dd, D68 extends the exemption | §1 | **Holds** | `docs/decisions.md` D66, D67, D68. D66 cites `landing` lines 253-259 and 340-350; the code is now at 262-268 and 349-359, which the design cites correctly. |

Small mismatch: the design says it adds "pointers in `docs/design/athena-v0.md`, `docs/enforcement.md`, and
`docs/architecture.md`" (§6). I checked the pointer in `athena-v0.md` (the "Proposed, not in force" paragraph);
I did not check the other two.

## Part 2 · What `landing_tests.rs` covers, and what it does not

All six tests I read in the file (its first 470 lines, which hold every `#[test]` the search found) are
single-job. None builds a commit with two jobs' records. That is consistent with
the design (the follow-up in §5 item 4 adds them), but it means the current file gives no evidence about how
the existing per-job code behaves when run twice on one commit.

Covered today: a squash with no trailers lands; a rebase merge lands though the prediction commit is no
ancestor; a squash that changes governed content without the job's records is refused, and so is one whose
records verify other content; a squash whose reading came after the work is refused (two variants); a second
landing after a squash is refused by the history walk; trailers that contradict the records are refused and
trailers alone cannot admit a change.

Failure modes the design's §5 item 4 list covers: missing `base_fingerprint`, wrong head, a fork, an unlinked
job, a last job not matching the tree, a late reading in one job.

**Failure modes the design does not list, and no test covers** (each is argued in the findings):

1. A chain whose fingerprints repeat (a job that reverts an earlier one, or a job that changes no governed
   content) (R1-M1).
2. A chain where one job changes the path classification rules (`.gitattributes`), so fingerprints are computed
   under different rules (R1-M3).
3. One job in the chain already landed earlier on the base branch, with the rest in the squash (R1-m4).
4. A chain where an earlier job has a failing verdict, a different candidate commit, or a failed verifier
   invocation, while the last job is clean (R1-m5).
5. Per-job-record checks in `jobs::check` on a chain commit (R1-m1).
6. The history walk landing a chain and then seeing one of its jobs landed again (the existing "already landed"
   test is single-job) (R1-m1).
7. A merge commit that carries several jobs' records (R1-m2).
8. A chain larger than two jobs where a middle job is missing but the others still link by fingerprint (R1-m5).
9. Old gate reading a new `candidate.json` (R1-m3).

---

## Findings

### R1-M1 · major · The ordering rule is ambiguous when fingerprints repeat

**Concerns:** §4.2 step 3; `invariant.rs` 349-359; `crates/athena-git/src/attrs.rs` `fingerprint`.

Step 3 says exactly one job has `base_fingerprint == fingerprint(P; R)`, and then repeatedly picks "the unique
job whose `base_fingerprint` equals the previous job's `fingerprint`". That is sound only if every fingerprint
in the chain is distinct. A fingerprint is a hash of governed content only (`attrs.rs`: "Equal fingerprints
mean the governed content is byte-identical"), so two cases break it:

- **A job that reverts another.** J1 goes A to B, J2 goes B to A, J3 goes A to C. Then J1 and J3 both have base
  A, and step 3 reports two heads, so a legitimate chain is refused with the fork message.
- **A job that changes no governed content** (for example, a docs-only job whose records and owner paths do
  not move the fingerprint). Its `base_fingerprint` equals its own `fingerprint`, so it links to itself and to
  its predecessor and successor alike. "Unique job" is not unique, and a cycle check would flag it.

**Evidence:** reading §4.2 step 3 against the definition of the fingerprint. I did not run an example.

**Recommended change:** state the order explicitly. Options: record a sequence number or the previous job's id
in `candidate.json` next to `base_fingerprint` (then ordering does not depend on hash uniqueness), or define
the order as a path search through the link graph and say which cases are refused (no-op jobs, repeated
states). Add tests for both cases to the §5 item 4 list.

### R1-M2 · major · The security argument claims the final verifier saw more than it saw

**Concerns:** §3 B ("the final job's verifier saw the whole tree"); §4.4; `docs/design/athena-v0.md` §6.3.

§4.4 says the final verifier "judged the whole candidate, including everything earlier jobs had added, because
those were part of its base." The v0 design says the verifier is composed from the request, the reading, the
defaults, "a `CandidateView` built by the core from git (the diff and stat)", and the check results. So the
final verifier judged its own job's diff. The checks ran on the whole tree, but the review of intent did not.
The earlier jobs' content was judged by their own verifiers, at their own states, and is reached by the new
rule only through record agreement.

Two consequences the design should state:

1. What the new rule adds over option E is only that the earlier records form a consistent chain. Anyone who
   can write one consistent set of records can write the whole chain (the design admits this in §4.4). So the
   chain check raises the cost of accidental mismatches, not of forgery. The argument against E in §3 ("earlier
   jobs' records could be fabricated") applies to B too. B is still better than E, because it catches real
   accidents (a leftover job from another pull request), but the design should say that is the benefit.
2. "Nothing reaches main unverified" holds for the chain in the sense that each step names a passing
   verification, not in the sense that one verifier saw the final tree.

**Evidence:** `docs/design/athena-v0.md` §6.3; `invariant.rs` 354-359 (the same fingerprint equality that
already lets a one-job landing rely on the verifier's candidate view); design §4.4 itself.

**Recommended change:** reword §3 B and §4.4 to claim exactly this, drop "saw the whole tree", and add the
comparison with E in terms of accidents caught, not forgery prevented.

### R1-M3 · major · Fingerprints under different rules may not link

**Concerns:** §4.1; §4.2 steps 1 and 3; `attrs.rs` `fingerprint`; `lib.rs` `Judged::new`.

`fingerprint(treeish, rules_commit)` depends on which commit's committed rules classify the paths. The gate
judges X by the rules of P (`Judged::rules`). §4.1 defines `base_fingerprint` as the base's fingerprint "under
the base commit's own rules". But a job's own `fingerprint` is for its candidate tree, and the design does not
say under which rules the core computes it (the candidate's base, or the candidate itself). If J1 changes
`.gitattributes` so that a path becomes governed or owner, then J1's `fingerprint` (rules of its base) and J2's
`base_fingerprint` (rules of the commit J1 produced) are computed under different rules and need not be equal,
though the content is the same. The step 3 link would refuse a valid chain, and the step 4 equality against
F (computed under the rules of P, not of the last job's base) could fail or, worse, pass for a classification
the last job did not verify.

**Evidence:** `attrs.rs` signature and doc comment; `lib.rs` module doc "never by rules the change itself
makes" (D38) and `Judged::new`; design §4.1 and §4.2. I did not construct the case.

**Recommended change:** say which rules each recorded fingerprint uses, and what happens when a job in the
chain changes the classification rules. The simplest rule: a chain whose jobs change the rules files is
refused with a named message, and a test says so. Alternatively, define every chain fingerprint under the
rules of P and have the core record that.

### R1-m1 · minor · The follow-up list omits `jobs::check`

**Concerns:** §5 item 3; `jobs.rs` 330-344; `history.rs` 77-78.

`jobs::check` calls `landing_job` once per commit and requires the returned single `JobId` to equal each landed
`outcome.json` it finds in the diff. With a chain, the commit adds several landed outcomes, and the second one
fails with "this commit is not a valid landing of that job". D68 shows this check was the second source of
violations on ee4b0dd. §5 item 3 names `landing_job` and `HistoryState::landed` but not this caller. Likewise
the history walk (`history.rs` 77-78) inserts one job per commit.

**Evidence:** `jobs.rs` 330-344 and `history.rs` 77-78.

**Recommended change:** add `jobs::check` to §5 item 3 and change `landing_job` to return a set. Add a test in
`jobs.rs` or `landing_tests.rs` that a valid chain squash passes both `invariant::check` and `jobs::check`, and
a history test that landing a chain and then repeating one of its jobs is refused.

### R1-m2 · minor · The merge-commit claim in §4.6 is not checked

**Concerns:** §4.6 last bullet; `invariant.rs` 126-134.

For a merge commit, `check` returns early if the governed fingerprint equals a parent's. Otherwise it calls
`landing`, which will take the chain path. The design says merge commits "never reach the chain path". They can,
for a merge whose first-parent diff adds several jobs' records and whose content equals no parent. Whether that
should land is not decided.

**Evidence:** `invariant.rs` 126-134; design §4.6.

**Recommended change:** either state that chain landing applies to merge commits too (and test it), or refuse
a chain on a commit with more than one parent.

### R1-m3 · minor · The schema change is left open

**Concerns:** §3 B "Against"; §4.1; §5 item 1; `records.rs` 579-591.

The design offers "`athena.candidate/2` or an optional field with a version bump". These differ. `Candidate`
is `deny_unknown_fields` and is `athena.candidate/1`. An optional new field under `/1` makes older gates reject
every new `candidate.json`, because they would not parse it; a `/2` does the same for old gates but at least
fails by a named schema. Either way the tip's record validation (`RecordSet::load`) must accept both versions
for old landed jobs. D69's reversal text says "the optional field in `candidate.json` can stay", which assumes
the first option. The decision and the design should agree.

**Evidence:** `records.rs` 579-591; D69 "Reversal"; design §3 B and §5 item 1.

**Recommended change:** choose one and say why. Also say how the core finds the base fingerprint: it already
holds the base commit, so a `fingerprint(base, base)` call is available, but the design marks the place
inferred. Locate it before the build job.

### R1-m4 · minor · Partly landed chains are not discussed

**Concerns:** §4.2 step 3; §4.6; `invariant.rs` 398-405.

If J1 landed on the base branch by itself and J2 and J3 arrive in a squash, the squash adds two jobs, and the
head link must equal the parent's fingerprint. That works. But the opposite order, a squash that adds J1 and J2
where J1's records were already on the parent (a records-only commit or an earlier partial landing), is not
discussed: `closing_jobs` counts only status `A`, so J1 would not be a member, and J2's `base_fingerprint`
would equal J1's fingerprint, not P's. §4.6 handles "parent moved" only.

**Evidence:** `invariant.rs` 217-243 (status `A` only); design §4.6.

**Recommended change:** add this case to §4.6 with its expected result and a test.

### R1-m5 · minor · Step 5 does not say a refusal in any job refuses the commit, and no test covers a bad middle job

**Concerns:** §4.2 step 5; §5 item 4.

Step 5 says "for every job, run every per-job check", which implies all must pass, but the test list in §5 item
4 covers only one late reading. A bad earlier job (failing verdict, mismatched candidate commits, verifier
model differing from the job's, unreconciled usage, a job already landed) is the case the chain exists to
prevent. The existing checks are cheap to run per job and each has a refusal message naming the job's
directory, so this is a test gap, not a design flaw.

**Evidence:** `invariant.rs` 360-405 (the checks); design §4.2 and §5 item 4.

**Recommended change:** list in §5 item 4 one refusal test per per-job check, run against a middle job, and a
test with a missing middle job whose neighbours still link.

### R1-n1 · nit · Step 5 omits two checks

**Concerns:** §4.2 step 5; `invariant.rs` 296-303 and 316-341.

The list of per-job checks leaves out that `job.json` must be in the landed tree and name an accepted
reading, and the three-way choice between a fresh landing and one after a blocked outcome. Add them, or say
"every check `landing` runs after the trailer checks".

### R1-n2 · nit · The "(read)" and "(inferred)" labels

**Concerns:** header; §4.4.

The header says no command was run, which is accurate, but §4.4's inferred argument carries the weight of the
recommendation. State in §4.4 that the argument is untested and that a prototype test in `landing_tests.rs`
(a hand-built two-job chain) is the first thing the build job should write.

---

## How the alternatives hold up

The rejection of A (patch composition) is correct: a squash discards the intermediate commits, as D66 (b)
anticipated. The rejection of C and D matches D66 to D68 (a host setting enforces C; D needs one entry per
squash). E is rejected for the right direction but a weaker reason than given (R1-M2). B is the least bad
option and its cost (a schema change and a larger `landing`) is stated.

## Decisions for the user

1. Should a chain's order be recorded explicitly (an extra field) or derived from fingerprints? (R1-M1)
2. Should a chain that changes the path classification rules be refused? (R1-M3)
3. Should chain landing apply to merge commits? (R1-m2)

## What this review read and ran

Read: the design in full; `invariant.rs` lines 1 to 500 where `landing`, `closing_jobs`, `check`,
`prediction_problem`, and `is_exempt` are; `jobs.rs` `check`; `history.rs` `check_history`;
`landing_tests.rs` (helpers and tests, to the first 470 lines); the `Candidate` struct in `records.rs`;
`fingerprint` in `attrs.rs`; `Judged` in `lib.rs`; `docs/decisions.md` D66 to D69; the "Merge, squash, and
rebase merges" passage of `docs/design/athena-v0.md`; the two earlier review files in this directory as a
format model.

Ran: no command was needed for the findings, which come from reading. Counts of findings and tests were
counted by reading, not measured by a tool. The repository's `cargo test --locked` is the check that this file
passes the wording scan.
