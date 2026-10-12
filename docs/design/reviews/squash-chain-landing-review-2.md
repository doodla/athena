# Squash-chain landing design: re-check (review 2)

**Verdict: approve with changes.** 0 blockers, 1 major, 3 minor, 2 nit. All ten findings and all three decisions
of [review 1](squash-chain-landing-review-1.md) are answered in the revised design, and the answers match the
sections they cite. Every line number the design gives for the code still matches the code. The revision adds
new rules, and checking them against the code found one gap that decides whether the rule can be built (R2-M1:
nothing says where `previous_job` comes from), one refusal message that the gate would never show (R2-m1), and
two spots where the design contradicts itself or the code layout (R2-m2, R2-m3).

| | |
|---|---|
| Under review | [`docs/design/squash-chain-landing.md`](../squash-chain-landing.md), the revised design (with its Appendix A) at the base commit of this job |
| Checked against | [review 1](squash-chain-landing-review-1.md) (findings M1 to M3, m1 to m5, n1 to n2, and its three decisions); `crates/athena-gate/src/invariant.rs`, `jobs.rs`, `history.rs`, `records.rs`; `crates/athena-data/src/records.rs`; `crates/athena-git/src/attrs.rs`; `crates/athena-core/src/run.rs` |
| Also read | `docs/decisions.md` (D69); `docs/enforcement.md`, `docs/architecture.md`, and `docs/design/athena-v0.md` (the pointers); `CHANGELOG.md` |
| Method | Read the design in full, compared each Appendix A row with the section it cites, re-read every claim the revision added against the code, and compared D69 and the three pointers with the design. No code was run to try the rule. |

Finding ids (R2-M1, R2-m1, R2-n1, and so on) are stable so a response can cite them. Review 1 ids keep their own
names. "§" is a section of the design. Line numbers are those of the base commit and were read, not measured by a
tool.

## Severity scale

The scale is the one in review 1: blocker (must change before building), major (returns the design to its
author), minor (fix before the build job or log with a reason), nit (wording).

---

## Part 1 · Review 1's findings against the revised design

"Confirmed" means the cited section says what Appendix A says it does. "Confirmed, with a gap" names the finding
in this review that qualifies it.

| Review 1 item | Result | Evidence |
|---|---|---|
| **R1-M1** ordering ambiguous | Confirmed, with a gap | §4.1 adds `previous_job`; §4.2 step 4 orders by it, requires one head, no fork, no cycle, none left over, and says repeated and no-op fingerprints order correctly. §5 item 4 lists both cases as tests. The gap is where the field's value comes from (R2-M1). |
| **R1-M2** over-claimed security argument | Confirmed | §3 B ("What it does not give") and §4.4 now say the final verifier saw its own job's diff and stat; the claim is "each step names a passing verification". §3 E is rejected on the accident ground only. This agrees with v0 §6.3 as review 1 read it. |
| **R1-M3** rules under which fingerprints are computed | Confirmed, with a nit | §4.1 says which rules each fingerprint uses; §4.2 step 3 and §4.3 refuse a chain that changes a rules file, with a message and a test (§5 item 4). `Judged` does judge by the parent's rules (`lib.rs`, cited by review 1). See R2-n1 for a case the refusal does not cover. |
| **R1-m1** `jobs::check` missing | Confirmed | §5 item 3 now lists `jobs::check` (the design says lines 330-344; the call and comparison are at `jobs.rs` 330-345), `landing_job` returning a set, and the history walk (`history.rs` 77-78, which is the call at 77 and the insert at 78). §5 item 4 adds a `jobs::check` test and a repeated-job history test. |
| **R1-m2** merge commits | Confirmed, with a gap | Step 2 and §4.6 refuse a chain on more than one parent; a test is listed. The refusal message would not reach the user (R2-m1). |
| **R1-m3** schema choice | Confirmed, with gaps | §4.1 chooses `athena.candidate/2` with a reason; D69's reversal text now says both versions load. The design names `RecordSet::load` under `athena-data`, which is the wrong crate (R2-m3). The place the core writes the fields is now locatable (R2-M1, R2-m3). |
| **R1-m4** partly landed chains | Confirmed, with a gap | §4.2 step 4 and §4.6 specify the head rule for both orders. The two orders are one case in the code, and one of them skips the head rule (R2-m2). |
| **R1-m5** bad middle job | Confirmed | Step 6 says any one job failing refuses the commit; §5 item 4 lists one refusal test per check on a middle job and a missing-middle-job test. |
| **R1-n1** two omitted checks | Confirmed | Step 6 now names `job.json` with an accepted reading and the three-way outcome choice. I compared the whole step 6 list with `invariant.rs` (table in Part 2). |
| **R1-n2** untested argument | Confirmed | §4.4 says the argument is untested; §5 item 4 puts the hand-built two-job chain test first. |

### Review 1's three decisions

All three appear in Appendix A as "stated assumptions" with a reason, and the text carries each one out:
recorded order (§4.1, step 4), refusal of a chain that changes a rules file (step 3, §4.3), refusal on a commit
with more than one parent (step 2, §4.6). D69 states the same two refusals as "assumed defaults after review 1,
which the user may change". The user has not answered; this review treats them as open, as the design does.

---

## Part 2 · The revised design's claims about code, re-checked

| Claim in the design | Where | Result | Evidence |
|---|---|---|---|
| `closing_jobs` and the more-than-one-job refusal | §1 item 1 | Holds | `invariant.rs` 217-243 and 262-268 (the text is at 267). |
| Whole-tree fingerprint equality | §1 item 2, step 1 | Holds | `invariant.rs` 349-359. |
| `check` calls `landing` for a merge commit whose content equals no parent | §4.6 | Holds | `invariant.rs` 126-134. |
| `Candidate` is `deny_unknown_fields`, `athena.candidate/1`, has `base_commit`, `fingerprint`, `changes`, no base fingerprint | §2, §4.1, step 3 | Holds | `records.rs` 579-591; `changes: Vec<ChangeEntry>` with a `path` (543-548). The `changes` field name, which the design called inferred, is confirmed. |
| `jobs::check` compares one landed job to each landed outcome | §5 item 3 | Holds | `jobs.rs` 330-345; the single-job value is computed once and cached (331-333). |
| History walk records one job per commit | §5 item 3 | Holds | `history.rs` 77-78. |
| Step 6 list matches the checks `landing` runs after the trailer checks | §4.2 | Holds | `job.json` and accepted reading (296-303); `prediction_problem` (304, 413-439); fresh, retried, or refused outcome (316-341); records present (342-348); verdict (360-364); candidate commits (365-369); verifier invocation, role, success, model (372-388); reconcile (389-395); no earlier landing (398-405). |
| Step 7 trailer rules | §4.2 | Holds | `invariant.rs` 269-287: verified-content must equal `fp`; a named request job must equal the one job. |
| `fingerprint` is under the rules of a named commit | §4.1 | Holds | `attrs.rs` 139: `fingerprint(git, treeish, rules_commit)`. Rules are `.gitattributes` files (`attrs.rs` module doc, line 3). |
| The core holds the base commit when it builds the candidate | §4.1 "Where the core writes them" | Holds, now located | `run.rs` 456-465 builds `Candidate` with `base_commit: p.clone()` and `fingerprint: None`. The design says the build job must locate the place; this is it. |
| `RecordSet::load` must accept `/1` and `/2` | §4.1, §5 item 1 | Wrong crate | `RecordSet::load` is in `crates/athena-gate/src/records.rs` 27, not `athena-data`. See R2-m3. |

Pointers and the decision:

| Document | Agrees with the design? | Evidence |
|---|---|---|
| D69, `docs/decisions.md` 1502-1538 | Yes | Names option B, `athena.candidate/2` loaded beside `/1`, the two new fields, recorded order, the two refusals, the narrowed security claim (1524-1527), and a reversal text that keeps both schema versions (1530-1533). Alternatives A, C, D, E match §3. Provenance says the argument is inferred and no command was run. |
| `docs/enforcement.md` 9-12 | Yes | Says the gate refuses a chain today, a revised rule is proposed, and it refuses rules-changing chains and merge-commit landings; M23 and the landing rule describe the gate as built. |
| `docs/architecture.md` 5-7 | Yes | Points to the design and says the map describes the code as built. It does not mention the revision, which is fine for a pointer. |
| `docs/design/athena-v0.md` 951-953 | Yes | Says proposed, not in force, order recorded in `candidate.json`, revised after review 1; the section still describes the gate as built. |
| `CHANGELOG.md` 12-19 | Yes | One entry for the revision and one for the first version. |
| Design §6 | Yes | Lists the files the revision changed; those match the four documents above. It says the review file is unchanged, and review 1 still reads as at its base. |

The only formatting mismatch is wrapping (R2-n2).

---

## Findings

### R2-M1 · major · Nothing says where `previous_job` comes from

**Concerns:** §4.1 (`previous_job`), §4.2 step 4, §5 item 2; `crates/athena-core/src/run.rs` 456-465.

The whole order rests on `previous_job`, which review 1 asked for and the design adopted. The design says the
core "already holds the base commit" and can compute `base_fingerprint` with `fingerprint(base, base)`; that is
true (`run.rs` 456-465 holds `p`). But the core holds a commit, not a predecessor job. The design gives no rule
for finding the job whose result that commit is. I searched the crates for any notion of a job's predecessor
and found none: the only chain concept in the code is the retry chain in `reading.rs` (a comment at line 807),
which is a different thing. Questions the design leaves open:

- Is the predecessor the job whose landing commit is the base commit, or the latest landed job in the base's
  history, or something the user names?
- What is `previous_job` when the base commit was reached by a hand change (an intervention) or a
  records-only commit after the previous job? Absent means "first of a chain", which would make the next job a
  second head.
- In a pull request where job 2 is based on job 1's candidate commit and job 1 has not landed, the core knows
  the commit but may have no landing to find.

Without this, the tests in §5 item 4 can hand-build `previous_job`, but the build job cannot write it. **Fix:**
add a paragraph to §4.1 that defines how the core derives `previous_job` from the base commit (and what it
writes after an intervention or a records-only commit), or say the user supplies it and where.

### R2-m1 · minor · The merge-commit refusal message would never be shown

**Concerns:** §4.2 step 2, §4.3 first message, §4.6 "Merge commit"; `invariant.rs` 126-139.

For a commit with more than one parent, `check` calls `landing` and uses only whether the result is `Ok`
(`invariant.rs` 130-134); the refusal reason is dropped. When it is not `Ok` and no intervention applies, the
violation is always "a merge introduces governed content that neither parent has" (135-138). So the message
"a chain landing is refused on a commit with more than one parent" can be produced by `landing` but not reported
for a merge commit, which is the only place it can arise. The behaviour (refusal) is right; the design's
§4.3 promise of a named message is not met. **Fix:** either note that for merge commits the existing merge
message is the one the user sees, or list a change to `check` that passes `landing`'s reason through on the
merge path, and test that.

### R2-m2 · minor · The two partly landed cases are one, and one path skips the head rule

**Concerns:** §4.2 steps 1 and 4; §4.6 "Partly landed chain".

§4.6 describes two cases: J1 landed alone and J2, J3 arrive; and a squash that "adds J1 and J2 where J1's
records were already on the parent". The second is not a different input: `closing_jobs` counts status `A`
only (`invariant.rs` 225), so a job whose records are on the parent is not in `jobs` at all, and the squash
simply adds J2 (and J3, if any). Two things follow.

1. If only J2 is left, `jobs` has one member and step 1 runs today's rule. Step 1 does not check
   `base_fingerprint` or `previous_job`, so §4.6's "J2 is refused otherwise" (when J1's result is not P's
   content) is not what step 1 does: today's rule accepts a single job whose fingerprint equals F.
2. Step 4 says a head whose `previous_job` names a job "already landed on P" is allowed, and §4.6 adds that its
   fingerprint must equal `fingerprint(P; R)`. Step 4 does not say how the gate finds out the job landed on P
   (it needs the `JobFiles` of that id at P, as `landing` loads `before` at 294), or what happens when that
   predecessor's `candidate.json` is `/1` and has no `base_fingerprint`. §3 B says jobs landed before the fields
   exist cannot form a chain, which could be read either way here.

**Fix:** say that one remaining job is judged by today's rule and the case is not refused for a stale base, or
extend step 1 to check the head rule when `previous_job` is present. State how a landed predecessor is found at
P and whether a `/1` predecessor is accepted. Keep the two tests, and merge them if the cases are one.

### R2-m3 · minor · The schema work is placed in the wrong crate and understated

**Concerns:** §4.1 "Schema choice"; §5 items 1 and 2.

§4.1 and §5 item 1 say `RecordSet::load` accepts `/1` and `/2` and place the work in `crates/athena-data`. In the
code `RecordSet::load` is in `crates/athena-gate/src/records.rs` 27-56 and parses by calling
`check_record_file`; the schema name is chosen by the tagged enum `AnyRecord` (`athena-data` `records.rs`
796-824, `#[serde(tag = "schema")]`) with `athena.candidate/1` as the only candidate variant (810-811), and the
canonical schema name is a second table at `records.rs` 914. A `/2` therefore needs a new enum variant or a
shared type, and every consumer that matches `AnyRecord::Candidate` must handle it: I found
`athena-core/src/run.rs` 950-956 (written), `athena-core/src/store.rs` 145, and `athena-gate/src/jobs.rs` 64.
The design's summary of the work is still right in direction; the build job needs the real list. **Fix:** name
the enum, the schema-name table, and the three consumers in §5 items 1 and 2, and correct the crate for
`RecordSet::load`.

### R2-n1 · nit · A rules change between the head's base and the parent is refused for a reason the messages do not name

**Concerns:** §4.1 "Which rules"; §4.2 steps 3 and 4; §4.6 "Parent moved".

Each fingerprint uses its own base commit's rules, and the head's `base_fingerprint` is compared with
`fingerprint(P; R)`. If a rules file changed on the base branch between the head's base and P, the two can
differ for the same content, and the chain is refused with "no job in the commit starts from the parent's
content". Refusing is the safe direction, and §4.6 treats "parent moved" the same way, but §4.6 mentions only
governed content. One sentence naming a rules change would help an adopter read the message. This is stated
from reading `fingerprint` (`attrs.rs` 139-142); I did not construct the case.

### R2-n2 · nit · Uneven line wrapping in new text

**Concerns:** `docs/decisions.md` 1507-1510 and 1532; `docs/design/athena-v0.md` 953.

The revision left some lines far longer than their neighbours (for example the D69 decision lines at 1510 and
1532, and line 953 of the v0 design). The wording is correct; rewrapping to the surrounding width would make
later diffs smaller.

---

## Decisions for the user

None new. The three decisions of review 1 stay open and are held at the author's assumed answers. R2-M1 may need
one: if the predecessor is not derivable from the base commit, the user must choose whether a job names it.

## What this review read and ran

Read: the revised design in full, including Appendix A; review 1 in full; `invariant.rs` 100-439
(`check`, `landing_job`, `closing_jobs`, `landing`, `prediction_problem`); `jobs.rs` 315-351 and the
`JobFiles::load` entry; `history.rs` 55-91; `crates/athena-gate/src/records.rs` 20-70;
`crates/athena-data/src/records.rs` 540-600 and 795-834; `crates/athena-git/src/attrs.rs` 1-180;
`crates/athena-core/src/run.rs` 440-500 and 925-965; D69 in `docs/decisions.md`; the pointers in
`docs/enforcement.md`, `docs/architecture.md`, and `docs/design/athena-v0.md`; the top of `CHANGELOG.md`; the
earlier review 2 file as the format model. I searched `crates/` for "previous", "chain", and `Candidate {` to
look for an existing predecessor notion and for the places a candidate is built or matched.

Ran: no command was needed for the findings, which come from reading. The repository's `cargo test --locked` is
the check that this file passes the wording scan, and `cargo fmt --check` the format check. Counts in this review
(findings, tests, consumers) were counted by reading, not measured by a tool. The consumer list in R2-m3 came
from a search for `AnyRecord::Candidate` and may be incomplete.
