# Study capability M1 design: re-check (review 2)

**Verdict: approve with named changes.** 0 blockers, 0 major, 3 minor, 1 nit. All 17 findings and all eight
decisions of [review 1](study-capability-m1-review-1.md) are answered in the revised design, and the answers
match the text of the sections they cite. Three small wording gaps remain (R2-m1 to R2-m3). They should be fixed
before a build job is asked for, under the design's own gate ([§9](../study-capability-m1.md#9-review-gate)),
but none returns the design to its author.

| | |
|---|---|
| Under review | [`docs/design/study-capability-m1.md`](../study-capability-m1.md), the revised design at the base commit of this job |
| Checked against | [review 1](study-capability-m1-review-1.md) (findings M1–M7, m1–m8, n1–n2, and its eight decisions); the design's Appendix A; `BRIEF.md` lines 306 and 395 |
| Also read | `docs/decisions.md` (D64); `CHANGELOG.md`; `xtask/tests/policy.rs` (the repository's wording scan) |
| Method | Read the design in full, compared each Appendix A row with the section it cites, and re-read the validation table and the run step for gaps the revision itself introduced. No model call and no code was run for the review itself. |

Finding ids (R2-m1 to R2-m3, R2-n1) are stable so a response can cite them. "§" is a section of the design.
Review 1 ids keep their own names (M1, m2, and so on). This review makes no claim about firstmate; the design
makes none either.

## Severity scale

The scale is the one in review 1: blocker (must change before building), major (returns the design to its
author), minor (fix before the build job or log with a reason), nit (wording).

---

## Part 1 · Review 1's findings against the revised design

"Holds" means the cited section says what Appendix A says it does.

| Review 1 item | Result | Evidence in the revised design |
|---|---|---|
| **M1** waiver can disable any rule | Holds | §5.4 splits rules into evidence (non-waivable) and model (waivable). Eight rules are classed evidence; a waiver needs the rule, the ids, the user's reason, and a user intervention record. §3 and §7 agree. Appendix A says "six … and so are the quote, pin, and card-completeness rules"; the table has eight evidence rules (R2-n1). |
| **M2** citations and runs avoid the pin | Holds | `study.run/1` has `sha` (§4); `study.pin/1` has a tag (§4); "Wrong SHA", "Bad citation" cover commit, pull-request, run id, and timestamp (§5.4); `both` needs a file and a run source (§3.1, §5.4 "Uncited claim"). |
| **M3** four rules need judgment | Holds | §3.2 and §5.4: `contradicts` link, `quote` field, provenance-only `authors-say` field, typed `Quantity` (§3.3). §5.5 lists seven judgments that stay with the verifier, including support for `evidenced` cards. |
| **M4** model gaps | Holds, as declared | §5.1 adds `evaluated-by`, unhandled failures, what changes on a retry, the no-answer rule for a person, and declared counters. Parallel regions and fan-out are named as limits and routed to a `model-limit` finding. Review 1 offered "extend or limit"; the design limits, and says why (Appendix A, decision 5). |
| **M5** containment | Holds | §2.1 items 1–2: separate account or container, scratch home, environment allow-list, boundary named in the run record, no run without one. |
| **M6** redaction and spend | Mostly holds | §2.1 item 3 puts redaction before storage. Item 4 names the two meters. One sentence conflicts with the decision it implements (R2-m1). |
| **M7** observability and replay | Holds | §4 "Capture streams" lists every stream of `BRIEF.md:298` with `native`, `added`, or `absent`; §2 "Replay" adds the step. A replay difference has no finding kind (R2-m3). |
| **m1** Pages is part of done | Holds | §1 and §8 name the Pages design as a required part of M1 that is written and reviewed before the build job. |
| **m2** example disagrees | Holds | Claim c2, §5.3, and §5.1 agree. Hand check of §5.3: `attempts` goes 1, 2, 3 on entry to `running`; guards `attempts < 3` and `attempts == 3` split every timeout; three attempts, two requeues. `running -> queued` is a retry edge by the reachability definition. |
| **m3** card completeness | Holds | "Incomplete card" is an evidence rule in §5.4. |
| **m4** "meant to exercise" | Holds | `study.run/1` lists observed transitions; "Unexercised path" uses them; §2 requires scenarios from a separate invocation; §5.5 item 6 assigns the check. |
| **m5** alternatives | Holds | §6 A2-C now rests on the unmet need under the buy-before-build rule; A2-D is added; A4-B is called the explicit downgrade; A1-C states its benefit. |
| **m6** cap, kill switch, blocker | Holds, with a field gap | `study.run/1` has them and `could-not-run` is a finding kind. The finding record has no field that names what it covers (R2-m2). |
| **m7** prompt injection | Holds | §5.5 item 4 states the limit and makes instruction-like text in a cited source a finding. |
| **m8** fresh context, gap log | Holds | §2 "Fresh context" (code composes the context) and "Gap log"; `study.gap/1` in §4. |
| **n1** choice outside table | Holds | §6 A3 chosen option is the bold row. |
| **n2** name the commit | Declined, accepted | Appendix A declines on the rule that design text never pins a commit; the header instead tells a reader how to re-find each cited passage by its quoted words. I agree. |

### Review 1's eight decisions

All eight appear in Appendix A with a reason, and the text of §2.1, §3, §5.1, §5.4, §5.5, §6, and §8 carries
each one out. D64 exists in `docs/decisions.md`. One conflict with decision 3 is in R2-m1.

---

## Part 2 · The checks of §9 on the revised design

1. **The brief is met.** M1 (`BRIEF.md:306`), the evidence rules, and reusable-versus-specific storage are
   covered (§1, §3, §4 storage paragraph). §1 and §8 say the capability is not M1 done (`BRIEF.md:395`). Passes.
2. **No claim about firstmate.** The header and §8 state it; both worked examples (§4, §5.3) are labeled
   synthetic and use the invented "Example Runner". Passes.
3. **The evidence rules close the loop.** The paths review 1 found are closed. I looked for new ones. The one
   judgment left inside an evidence rule is whether a quote means what a card says, and §3.2 and §5.5 assign it
   to the verifier by name. No new path found.
4. **The model fits.** Failures, retries, and concurrency for a non-trivial system are expressible; the two
   cases that are not are named in §5.1 and §7. Passes.
5. **The validation rules are mechanical.** Each of the eighteen rows in §5.4 can be code with one planted fault
   and one clean case, with "Bad citation" using a fixture repository as the section says. "Unexercised path"
   and "Divergence unrecorded" depend on a field the finding record lacks (R2-m2).
6. **The alternatives are fair.** Passes after the m5 changes.
7. **Risk.** Containment, redaction, and caps are now in the capability. Adequate once R2-m1 is fixed.
8. **Scope of this change.** The revision changed the design, the decision log, and the changelog, as §8
   states. This review's own change is the review file, a status update in the design, and a changelog line;
   no governance file changes.

---

## Findings

### R2-m1 · minor · The spend rule can be met by observing alone

**Concerns:** §2.1 item 4, third bullet; Appendix A decision 3.

The bullet says the run does not start "if the spend cap can be neither enforced nor observed". That reads as:
a cap that is observed but not enforced is enough to start. The next sentence, and decision 3, say the opposite:
cost recorded after the fact is never accepted as the cap. **Fix:** write "if the spend cap cannot be enforced
before the run starts, the run does not start", and keep the sentence about cost recorded afterwards.

### R2-m2 · minor · A finding names no element, so two rules cannot test it

**Concerns:** §4 `study.finding/1`; §5.4 "Unexercised path" and "Divergence unrecorded".

"Unexercised path" passes only when a `could-not-run` or `no-observation` finding "names it", and "Divergence
unrecorded" needs a `divergence` finding that "links the two claims". The finding row lists a kind and a
severity, and gives fields only for `waiver`. Without a field for the element ids or claim ids a finding covers,
neither rule can be written as code, and a planted-fault test cannot pass. **Fix:** add a "subject" field
(element ids or claim ids) to `study.finding/1` for every kind.

### R2-m3 · minor · A replay difference has no finding kind

**Concerns:** §2 "Replay"; §4 `study.finding/1`.

§2 says a replay that differs from the original capture "is a `finding`". The kinds in §4 are `divergence`
(defined as between claims), `waiver`, `not-native`, `no-observation`, `could-not-run`, and `model-limit`. None
fits a difference between two runs. **Fix:** add a kind (for example `replay-difference`, with the two run ids
as its subject) or widen `divergence` to cover two runs.

### R2-n1 · nit · "Six evidence rules" in Appendix A

**Concerns:** Appendix A row M1; §5.4.

The row says six evidence rules plus "the quote, pin, and card-completeness rules added by other findings".
The pin rule (Wrong SHA) is already one of review 1's six, so the table's eight evidence rules are the six plus
two (Quote mismatch, Incomplete card), not plus three. **Fix:** say "the eight evidence rules of §5.4".

---

## Decisions for the user

None. The three minors are wording and field gaps with one obvious fix each.

## What this review read and ran

Read: the revised design in full; review 1 in full; `BRIEF.md` lines 306 and 395; the D64 heading in
`docs/decisions.md`; the top of `CHANGELOG.md`; the wording scan in `xtask/tests/policy.rs`.

Ran: no command was needed for the findings, which come from reading. The repository's `cargo test --locked`
is the check that the review file passes the wording scan. Counts in this review (findings, rules) were counted
by reading, not measured by a tool.
