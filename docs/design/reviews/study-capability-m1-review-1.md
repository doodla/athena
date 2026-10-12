# Study capability M1 design: independent review 1

**Verdict: approve with changes.** 0 blockers, 7 major, 8 minor, 2 nits. Under the design's own gate
([§9](../study-capability-m1.md#9-review-gate)) a major finding returns the design to its author, so no build job
should be scheduled until the majors are answered.

| | |
|---|---|
| Under review | [`docs/design/study-capability-m1.md`](../study-capability-m1.md), as it stands at the base commit of this job |
| Requirements | `BRIEF.md` (S1, S3, S4 M1, S5, the Standards, and the Settings lines the design cites) |
| Also read | [`athena-v0.md`](../athena-v0.md) §0, §5.1, §6.2, §16.2; `crates/athena-core/src/capability.rs`; `athena.toml` `[budget]`; `xtask/tests/policy.rs`; [`athena-v0-review-1.md`](athena-v0-review-1.md) for format |
| Reviewer | Independent. The reviewer had no part in the design and saw only the artifacts, not its author's reasoning. |
| Method | Read the design in full and checked it against the files above. Ran five small checks, S1–S5, listed in the appendix. No model call was made. |

Finding ids (M1–M7, m1–m8, n1–n2) are stable so the author's response can cite them. "§" is a section of the
design under review. "BRIEF:n" is a line of `BRIEF.md`. This review makes no claim about firstmate; the
design makes none either (check 2).

### Severity scale

- **blocker**: must change before building.
- **major**: should change before building; returns the design to its author ([§9](../study-capability-m1.md#9-review-gate)).
- **minor**: fix during the build, or log it with a reason.
- **nit**: wording or polish.

---

## Summary

The shape is right and should survive: a separate capability behind the existing seam (§6 A1-A, `BRIEF.md:186`),
claims that carry a source and a mark (§3.1, `BRIEF.md:320`), provenance on rationale (§3.2, `BRIEF.md:321`),
and a model that is typed data with graph rules (§5, `BRIEF.md:306`). The citations are accurate (S1) and the
design stays inside its scope (checks 2 and 8).

Seven things should change before building:

- **M1.** A `finding` waiver can switch off any rule, including the evidence rules the capability exists to enforce.
- **M2.** Several citation kinds and every run can avoid the pin; `both` has no rule.
- **M3.** Four rules in §5.4 need judgment, not code, as written.
- **M4.** The model cannot express several agents working one unit, fan-out, or model-decided transitions.
- **M5.** The run step runs third-party code with the user's privileges and credentials, which v0 said it would not do without containment.
- **M6.** Captured run output goes into a public repository with no redaction step, and the spend caps do not cover the studied system.
- **M7.** The run record lacks most of what `BRIEF.md:298-299` asks to be observable and replayable.

---

## Findings

### Check 1 · The brief is met

M1 (`BRIEF.md:306`), the evidence rules (`BRIEF.md:320-321`), and reusable-versus-specific storage
(`BRIEF.md:277`) are each covered (§1, §3, §4 storage paragraph). What is missing:

#### M7 · major · The run record does not make a run observable or replayable

**Concerns:** §2 step 3, §4 `study.run/1`; `BRIEF.md:298-299`.

`BRIEF.md:298` asks that every run expose the prompts as sent, tool calls, token usage by agent, role and
model, timing, topology, state and memory changes, git activity, and every point where the human is
addressed. `BRIEF.md:299` asks for one-command reproduction from a recorded configuration, "recorded,
redacted, and replayable". `study.run/1` has a configuration, a command line, times, cost, an exit state, and
"location of the captured output". It names none of the capture streams, has no field for what the studied
system exposes natively versus what had to be added (a finding the brief names), and no replay step. Without
these, an `observed` claim about a failure or retry path (§3.1) cannot be checked against the run.
**Fix:** list the capture streams in `study.run/1`, say which are optional, and add a replay entry to §2.

#### m1 · minor · M1's definition of done includes being live on Pages

**Concerns:** §8 (publishing out of scope); `BRIEF.md:395`.

The brief's first milestone is "designed, built, verified, and live on Pages". The design defers publishing to
"a later design". That is acceptable, but the design should name that later design as a required part of M1,
so that this capability does not read as M1 being done.

#### m6 · minor · The run record omits the cap, kill switch, and blocker

**Concerns:** §4 `study.run/1`; `BRIEF.md:110-111`.

The brief asks that a per-run cap and kill switch be recorded, and that a system that cannot run be
downgraded explicitly with the blocker recorded. `study.run/1` records cost but not the cap or the kill
switch, and no record holds a "could not run" blocker; §5.4 "Unexercised path" accepts a free-text note.

#### m8 · minor · Fresh context and the gap log are not covered

**Concerns:** §4 storage; `BRIEF.md:278-279`.

Nothing keeps one study's notes out of another study's reader context, and nothing records capability gaps
found during a study for re-checking against earlier studies. Both are reusable-capability concerns, not
firstmate concerns, and the first study is the cheapest time to build them.

### Check 2 · No claim about firstmate

**Passes.** The design mentions firstmate five times (S3): lines 10, 32, 38, 270, 290. Each is a disclaimer, a
quotation of the brief's plan, or the scope exclusion. Both worked examples (§4, §5.3) are labeled synthetic and
name an invented "Example Runner". No finding.

### Check 3 · The evidence rules close the loop

Looking for a path around §5.4 found four.

#### M1 · major · A waiver can switch off any rule

**Concerns:** §5.4 introduction ("a failure blocks publishing unless a `finding` waiver exists"), §4
`study.finding/1`, §7 row "Evidence and mark rules"; `BRIEF.md:320-321`.

The waiver clause applies to every rule in the table. A waiver for "Uncited claim" or "Stated without
source" publishes an unsupported claim. The page "shows" the waiver, but §7 rates loosening these rules as
the most expensive reversal, and a waiver is a per-study way to loosen them. The design does not say who may
write a waiver or what it must contain. **Fix:** make the evidence rules (uncited claim, wrong SHA, bad
citation, inferred shown as stated, stated without source, unsupported number) non-waivable, and define who
writes the rest and what text the page shows.

#### M2 · major · Citations and runs can avoid the pin

**Concerns:** §2 steps 1 and 3, §3.1, §4 `study.source/1` and `study.run/1`, §5.4 rules "Wrong SHA" and "Bad
citation".

- "Wrong SHA" and "Bad citation" test only file citations. A commit or pull-request citation is never checked
  for existing, or for being reachable from the pin. A `run` citation is never checked against a `study.run`
  record.
- A run record has no SHA field and nothing says the run is of the pinned tree, so a claim marked `both` can
  join a read of one version with a run of another (§2 "Run is separate from read").
- `both` is defined (§3.1) but has no rule: §5.4 "Uncited claim" lists only the `read`-with-run and
  `observed`-with-file mismatches. A `both` claim with two file sources passes.
- BRIEF:275 asks for a tagged release for the later studies; the pin record keeps "how the reference was
  resolved" but no field for the tag name.

**Fix:** add `sha` to `study.run/1`; add rules for commit, pull-request, run-id and timestamp existence; require
`both` to carry at least one file source and one run source.

#### m3 · minor · Rationale cards have no completeness rule

**Concerns:** §3.2, §5.4; `BRIEF.md:321`.

A card must carry decision, alternatives, tradeoff, and reversal cost, but no rule fails a card with an empty
field. An `inferred` card with no alternatives passes every rule.

#### m2 · minor · The synthetic example disagrees with the model's own definition

**Concerns:** §4 claim c2, §5.1, §5.3.

Claim c2 says a timed-out task "is requeued up to three times". §5.1 defines the bound as "the maximum
attempts", and §5.3 writes it as `attempts < 3` with `[retry, bound 3]`. Three attempts is two requeues. The
guard also uses a counter, `attempts`, that no element declares. I found this by hand-simulating the example
(S3). It is small, but the example is what a later reader will copy. Also, §5.1 defines a retry edge as one
that "returns to an earlier state", but no order of states is defined; say "enters a state from which its
source is reachable".

### Check 4 · The model fits

#### M4 · major · The model cannot express several agents, fan-out, or model-decided transitions

**Concerns:** §5.1 (States, Transitions, Concurrency); `BRIEF.md:306`; the brief's own second instance is
described as one that "spawns sub-agents" (`BRIEF.md:24`).

Cases that do not fit:

1. **Several agents on one unit.** A transition has one actor and a unit has one state. Two agents working
   one unit at once need parallel regions, or the model needs a state per combination. The first is not in
   §5.1; the second grows quickly.
2. **Fan-out and join.** A unit that spawns sub-units, and completes when they do, has no parent/child link,
   no join condition, and no failure-propagation rule.
3. **Transitions decided by a model.** A trigger or guard such as "the reviewer agent judges the work
   acceptable" is not a condition code can evaluate. §5.2 says the guard is what "a reader and the validator
   check against runs". Add an `evaluated-by` field (`code`, `agent`, `person`) so the validator knows which
   guards it can check.
4. **Failures that never surface.** A dropped or silently lost unit has no edge by definition. Add a list of
   "failures with no handling", otherwise the model cannot say what it does not cover and §5.4 "Dead end"
   hides it.
5. **Retry that changes something** (a different model, an added instruction, escalation to a person) is a
   retry edge with a bound only. Record what changes.

Human-in-the-loop is partly covered (a `waiting` state kind; human touchpoints, §5.1). It lacks a timeout or
abandonment rule for the person who never answers, so I list it with this finding rather than separately.

#### m4 · minor · "Meant to exercise" is not "exercised"

**Concerns:** §4 `study.scenario/1`, §5.4 "Unexercised path".

A scenario lists which elements it is "meant to exercise". The rule is only mechanical if each run also
records which transitions it observed, as claims or events. Also, nothing says who writes scenarios or how their
author is kept independent of the reader, so a scenario can be written to confirm what was read.

### Check 5 · The validation rules are mechanical

Judged one by one. Ten of the fourteen can be code with a planted fault and a clean case: Uncited claim,
Wrong SHA, Bad citation (a git lookup), Uncited element, Unreachable state, Dead end, Dangling target,
Missing guard, Unbounded retry, and Unexercised path (given m4). Four cannot, as written:

#### M3 · major · Four rules need judgment

**Concerns:** §5.4 rows "Divergence unrecorded", "Inferred shown as stated", "Stated without source",
"Unsupported number"; §9 item 5.

- **Divergence unrecorded** needs someone to decide that two claims "contradict each other". Code cannot.
  Make the claim author state contradiction as a typed link (`contradicts: <claim id>`), and have the rule
  fail when a link has no finding. A contradiction nobody links stays with the verifier agent (§6 A3-B).
- **Inferred shown as stated** is a wording test on rendered text, and rendering is out of scope (§8), so it
  cannot be tested now. A word list is evaded by paraphrase. Make it structural: the renderer takes a card's
  provenance label from the record and has no free-text path for the authors' voice, and the rule checks the
  template, not the prose.
- **Stated without source** says the source must "contain the statement". Code can check that a verbatim
  quote field is a substring of the cited lines at the pinned SHA. Add `quote` to `study.source/1`. Whether the
  quote means what the card says is judgment, and belongs to the verifier.
- **Unsupported number** needs to know what counts as a number in prose. Limit it to typed `Quantity` fields
  (§3.3), where the type already refuses a bare number (v0 §5.1 item 5).

The design says each rule is "checked by a test that holds one planted fault and one clean case" (§5.4). That
sentence is untrue of these four until they are rewritten. A fifth, "Supported by at least one cited claim"
for `evidenced` cards (§3.2), is also judgment and should be named as the verifier's.

### Check 6 · The alternatives are fair

Mostly fair, with three weak spots. Reversal costs (§7) are plausible, and §7 labels them estimates. Checked:
"new crate" low before build, moderate after, matches the dependency test in `xtask/tests/policy.rs`;
"records do not move because they are append-only" matches v0.

#### m5 · minor · Three alternatives are argued from the wrong ground

**Concerns:** §6 A2-C, A4-B, A1-C; `BRIEF.md:103`, `BRIEF.md:110`, `BRIEF.md:122`.

- A2-C (TLA+ or a statechart) is rejected as "a new toolchain" under `BRIEF.md:103`, but that line asks for
  pinned, idempotent setup, which a pinned tool can meet. The real argument is the learning cost, which is
  unmeasured, and the buy-before-build rule (`BRIEF.md:122`) requires naming the unmet need. A fair option is
  typed records now with an export to a checker later; say why not.
- A4-B says read-only "contradicts the brief's lab requirement", but `BRIEF.md:110` allows an explicit
  downgrade to read-only when a system cannot run. Say it is the default to avoid, not forbidden.
- A1-C ("a script collection") is close to a straw man. Its real cost, no gate coverage, is stated; its
  benefit, speed to a first result, is understated.

#### n1 · nit

A3's closing "Choice: A3-A with a verifier on top" is clear but sits outside the table; every other
choice is in the bold row.

### Check 7 · Risk

The design names the third-party-code risk in one sentence (§2 "Spend and safety") and then adds no
mitigation. That is not adequate.

#### M5 · major · The run step does what v0 said needs containment first

**Concerns:** §2 step 3 and "Spend and safety"; `athena-v0.md` §16.2 (first residual); `BRIEF.md:99`, `BRIEF.md:313`.

v0 §16.2 says its agent-tool mechanisms "do not contain the code a job runs", that this was acceptable
"in v0" because "that code is the repository's own", and that "containment is pulled in by the first job that
runs code not written by Athena's agents". The study run is that job. The design cites the residual and
declines to add containment. The runs also need model credentials (`BRIEF.md:99` lists access for running the
studied systems), so the third-party code would run with credentials in its environment. **Fix:** either name
the containment (a throwaway account or container, no user home, an environment allow-list as v0 builds for
agents) as part of this capability, or record a decision by the user to defer it, with the studied systems
limited to ones the user has chosen to trust.

#### M6 · major · Captured output and spend are not controlled

**Concerns:** §4 `study.run/1` ("location of the captured output"), §2 "Spend and safety"; `BRIEF.md:22`,
`BRIEF.md:111`, `BRIEF.md:299`; `athena.toml` lines 24–33.

- The repository is public (`BRIEF.md:22`). Run output can contain tokens, personal data, and paths on the
  host. Step 3 has no redaction step, though `BRIEF.md:299` asks that runs be "redacted". v0's commit gate scans
  for known secret patterns and private addresses, but a refusal at commit is a failure, not a method.
- "Runs inherit v0's invocation caps" is ambiguous. The caps in `athena.toml` `[budget]` are per Athena role
  (reader, producer, verifier) and per job, and meter Athena's own agent calls. A studied system makes model
  calls through its own provider that Athena does not meter. The design does not say how a per-run cap is
  measured or enforced for those calls, only that cost is recorded afterwards.

**Fix:** add a redaction step before output is stored, state which meter enforces the per-run cap, and say what
happens when the cap cannot be observed.

#### m7 · minor · Prompt injection through the studied repository

**Concerns:** §2 step 2; `BRIEF.md:313`.

Reader agents read a third-party tree and write claims that reach a public page. The tree can contain
instructions aimed at an agent. "Read-only role" limits the harm to wrong claims, not to none. Say so, and say
what in M3 or the verifier covers it.

### Check 8 · Scope of this change

**Passes, with one limit.** The tree at the base commit has no `athena-cap-study` crate (`crates/` lists nine
others) and `docs/architecture.md`, `docs/decisions.md`, `docs/enforcement.md` and `docs/records.md` do not mention
the study (S5). That matches the design's §8. I cannot tell from a snapshot with a single commit that the
design was the only file added by its change, so this check rests on S5 and on the design's own statement.

#### n2 · nit

The design's header says BRIEF line numbers are "as of this change". All fourteen cited lines or ranges match
the brief at this commit (S1), so the note is correct. Name the commit when it next changes.

---

## Decisions for the user

1. M1: waivers — forbid for the six evidence rules, or allow them with an explicit user-written reason?
2. M5: containment for run step — build it into this capability, or defer it by decision and study only systems you trust?
3. M6: run caps — meter the studied system's spend from Athena, or accept recorded-after-the-fact cost?
4. M3: judgment rules — rewrite them as structural checks (`contradicts` link, `quote` field), or relabel them as verifier duties?
5. M4: model scope — extend §5.1 for parallel agents and fan-out now, or limit M1 to systems that fit and name the limit?
6. M7: observability — require all `BRIEF.md:298` streams in `study.run/1`, or require the native ones and log the rest as findings?
7. m1: should the Pages publishing design be written before the build job, or after?
8. m5: record a "typed records now, export to a checker later" alternative, or leave A2 as is?

---

## What this review read and ran

Read: the design in full; `BRIEF.md` lines 22–24, 98–125, 183–188, 271–327, and 388–401; `athena-v0.md` §0, §5.1,
§6.2, §16.2; `athena-core/src/capability.rs` (the `Capability` trait, lines 84–101, whose methods suit "a
verified change" and need extending for a multi-step job, as the design's A1 already says); `athena.toml`
`[budget]`; `docs/non-rust-files.md`; `xtask/tests/policy.rs`; and `athena-v0-review-1.md` for format.

## Appendix: experiments

| Id | What | Result |
|---|---|---|
| S1 | Opened `BRIEF.md` at every line the design cites (186, 103, 123, 271–277, 275, 277, 296–300, 300, 306, 306–314, 320, 321, 395) and compared the text | All match |
| S2 | Resolved each in-document anchor the design uses (`#9-review-gate`, `#54-validation-rules`, `#32-rationale-cards`, `#51-elements`, `#6-considered-alternatives`, `#7-reversal-cost`, `#8-out-of-scope`) against the headings | All resolve |
| S3 | Hand-simulated every §5.4 graph rule on the §5.3 example | Unreachable, dead end, dangling target and missing guard all pass; the bound and `attempts` mismatch (m2) found |
| S4 | Searched the design for "firstmate" | Five mentions, none a claim (check 2) |
| S5 | Listed `crates/` and searched `docs/architecture.md`, `decisions.md`, `enforcement.md`, `records.md` for "study" | No crate, no mention |

The repository's `cargo test --locked` passed before this review was added (all suites, including the xtask
policy tests). No number in this review is an estimate except where labeled; the counts of findings and mentions
were counted by reading.
