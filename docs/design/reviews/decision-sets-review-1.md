# Review 1 of `docs/design/decision-sets.md`, with the introspection step of `docs/design/defaults-loop.md`

**Reviewed:** `docs/design/decision-sets.md` (status "proposed (D91)") and `docs/design/defaults-loop.md` as revised
after its review 1 (status "proposed (D90), revised"), taken as one system, against `crates/`, `defaults.toml`,
`docs/decisions.md`, `docs/records.md`, `docs/enforcement.md` and `BRIEF.md`.
**Independence:** I formed the verdict from the two designs and the repository. I used
`docs/design/reviews/defaults-loop-review-1.md` only to know what the revision answered, not as evidence.
**Method:** reading, plus one measurement: I ran `cargo test --locked -p athena-cli --test budgets -- --nocapture`
in a snapshot of the base commit. I made no call to the gateway. Claims are labelled **(read)** when taken from the
file named beside them, **(measured)** when I ran something, **(reported)** when a design says it came from the job
request or the user's probes and I could not check it, **(design)** for proposals and **(estimate)** for numbers
nobody measured. Section and line numbers are those of the base commit. This review edits no design; fixing the
designs is a separate job.

## Verdict: accept with changes

The direction holds: code decides and writes, Jev only answers typed questions and starts in shadow, Athena never
edits `defaults.toml` or a decision, and a failing decision is marked, not retired. The two designs agree on most
of that. Four things stop them from fitting as one system today: (1) the reader's token set has no room for
decision sets, and neither design's budget claims match how budgets are enforced; (2) no place holds the typed
fields (assumptions, signals) for a default; (3) the signal sources `host-error` and `rejection` have no record and
no trigger, so the D87 worked example cannot fire as written; (4) the two designs use different evidence
standards for "this decision is failing". Each is fixable in text. The changes are in section 6. No build should
start before F1 to F4 are settled, and the first build is the small one in section 5.

## Severity scale

**High** makes a central claim false or blocks the build. **Medium** must be settled in a design or the first
build job. **Low** is a correction.

## 1. Fit across the four parts

The four parts are: introspection records (`defaults-loop.md` §3), the defaults loop (§4, §7-§9 there), typed
decisions in sets (`decision-sets.md` §2-§3) and failure signals that put a decision under review
(`decision-sets.md` §4).

| Link | Where the designs say it | Result |
|---|---|---|
| Introspection record names the decisions an event matched | `decision-sets.md` §5; the record fields are in `defaults-loop.md` §3.2 | **Gap (F7).** §3.2 has no field for decision ids, and `athena.introspection/1` is described as complete. |
| A signal firing is itself introspected | `decision-sets.md` §4 "Firing", third bullet; `defaults-loop.md` §3.1 | **Gap (F7).** §4 runs the watcher "after each introspection", so firing, then introspection, then watcher is a cycle with no stop rule. |
| Host errors and check failures as events | `decision-sets.md` §4 sources; `defaults-loop.md` §3.1 triggers | **Gap (F3).** §3.1 triggers are correction, intervention, rejection, failure. A host refusal that does not fail the job is none of them. |
| A proposed default carries assumptions and signals | `decision-sets.md` §5, second bullet; `athena.defaults-proposal/1` in `defaults-loop.md` §8.1 | **Gap (F2).** The proposal record has no such fields, and `defaults.toml` cannot hold them (`deny_unknown_fields`, `crates/athena-data/src/defaults.rs` lines 7 and 16). |
| One matching mechanism serves defaults and signals | `decision-sets.md` §5 ("the same question") against `defaults-loop.md` §4 step 1 | **Partly wrong (F5).** Defaults matching is one `choice` over all default ids plus `none`; signal matching is one `boolean` per signal. Different question shapes, different thresholds. |
| Evidence needed to act | `decision-sets.md` §4 (1 job fires) and §5 ("Repeats ... are the evidence") against `defaults-loop.md` §7.2 (N = 3 distinct jobs, estimate) | **Conflict (F4).** |
| `prevention_kind = default` produces a `proposed` decision | `decision-sets.md` §5; `defaults-loop.md` §7.2, §8 | **Fits**, apart from F2. Review-only on both sides; `defaults-loop.md` §9 step 1 keeps `defaults.toml` unwritten. |
| `prevention_kind = design-change` against a decision | `decision-sets.md` §4; `defaults-loop.md` §3.2 | **Fits.** The registry of failure classes and the decision ids are separate things and stay so. |
| Jev stays in shadow, with a stronger model | `decision-sets.md` §5; `defaults-loop.md` §10 | **Fits in text, gap in method (J6).** The labeled set and scoring of §10.2-§10.3 do not cover the two new questions. |
| Records stay out of the agent snapshot | `decision-sets.md` §6 step 5; `defaults-loop.md` §3.3 | **Fits.** Both read records in Athena code. The new `under-review` input to briefs needs a stated source (F8). |

## 2. Findings

### F1 (High): the reader set has no room, and the budget claims do not match the code

- **Evidence (measured):** the budgets test prints the estimated tokens of each static set: reader 1,798 against a
  budget of 1,800; change producer 883 of 1,100; change verifier 1,088 of 1,300; study producer 1,099 of 1,100;
  study verifier 1,238 of 1,300; contract 326 of 350. The estimator is ⌈bytes/3.0⌉ (`crates/athena-cli/tests/budgets.rs`
  header; `crates/athena-context/src/lib.rs` lines 154-161, 172-183).
- **Consequences:**
  - `decision-sets.md` §3 has the reader "see the one-line rendering of the whole set". The reader set already
    holds the contract, the reader frame, `defaults.render()` and the capability catalog
    (`budgets.rs` lines 24-29) and is 2 tokens under budget. `docs/decisions.md` has 91 entries (counted, `## D`
    headings); even at an assumed 25 tokens per line (estimate) a one-line rendering would be about 2,300 tokens,
    above the whole reader budget. The reader cannot see sets as designed.
  - `decision-sets.md` §3 says an over-budget set "is refused (`check_budget`)" and that lines are added "until
    the live counter (D25) would exceed the role's budget". `check_budget` is called only from tests (grep of
    `crates/`: `lib.rs` tests and `crates/athena-cli/tests/budgets.rs`); it has no caller at job time. `docs/enforcement.md`
    row M18 says the same: "a per-set budget test with the byte estimator as a tripwire". The only live
    measurement is the opt-in `live_the_contract_is_within_its_budget`, for the contract. So per-job truncation by
    budget needs a runtime counter that does not exist. `defaults-loop.md` §2 R2 ("over-budget sets are refused by
    code") has the same flaw: the code refuses a build in tests, not a job (what actually blocked the four jobs is
    reported, and I did not verify it).
  - Both designs leave open whether defaults count against the producer and verifier budgets
    (`decision-sets.md` §3; `defaults-loop.md` §8.3). **Answer (read):** they do not. The budget test composes
    only the contract, the frame and the capability section (`budgets.rs` lines 35-44), while the defaults reach the
    producer and verifier in the per-job brief section "User defaults that apply"
    (`crates/athena-context/src/briefs.rs` line 158). The budgets of 1,100 and 1,300 therefore do not protect the
    per-job text that sets would add.
- **Needed:** state that the reader gets sets only by class and phase narrowing in code with a stated cap, not the
  whole set; say that per-job set size needs a new runtime check (a build item, with its own test), and add a
  budget (or an explicit "unbudgeted") for the per-job brief section.

### F2 (High): the typed fields of a default have no home

- **Evidence:** `decision-sets.md` §2 says a default "stays in `defaults.toml`" with a typed view; §6 step 1 defers
  adding `assumptions` and `signals` to a governance job because `DefaultsModel::parse` rejects unknown fields
  (read: `defaults.rs` lines 7, 16). Defaults have kebab ids; the typed headers of §6 step 2 are for `D01`-`D90`
  entries in `docs/decisions.md`. `defaults-loop.md` §8.1 gives the proposal record no `assumptions` or `signals`
  field.
- **Root cause:** the design treats "a default is a decision" as a view, but nothing stores the part of the view
  the defaults lack. Of 12 defaults (read: `defaults.toml`), none can carry a signal today, and the loop would
  produce new ones with signals that cannot be saved.
- **Needed:** choose one store for the typed fields of expectations: new optional keys in `defaults.toml` (a
  governance job, D26) or a separate typed record keyed by the default's id. Add the fields to
  `athena.defaults-proposal/1` either way.

### F3 (High): the `host-error` and `rejection` signals have no record, no trigger and no timing

- **Evidence:**
  - The host client discards the status code. `GhCli::gh` returns an error built from the command's standard
    error text (`crates/athena-core/src/gh.rs` lines 47-53); no field holds an HTTP status. The merge call is
    `job.host.merge(...)` at `gh.rs` line 485. The D87 signal "HTTP 405 or 422" (`decision-sets.md` §4) would be a
    text match on a message, not a `match` field "decided by code".
  - Rejected outcomes do exist: `OutcomeState::Rejected` (`crates/athena-core/src/run.rs` lines 925 and 1042;
    `crates/athena-data/src/records.rs` line 96). `decision-sets.md` §4 says "rejections ... have no dedicated
    record kind today"; that is partly wrong. A rejected outcome is stored. What is missing is a user rejection of a
    proposal or a plan (`defaults-loop.md` §13, last-but-two bullet).
  - `decision-sets.md` §4 evaluates "after each job closes". The merge refusal happens in `athena land --pr`, which
    runs after the job is closed (`gh.rs` line 485 is in the landing path). The worked example's claim that D87
    is set `under-review` "after that job" has no trigger at landing time.
  - `defaults-loop.md` §3.1 lists "a job fails, a check fails or never finishes, a budget refuses a set" as the
    failures. A refused merge fits none unless it fails a job.
- **Needed:** add a host-error record (status code as a field, operation, job) written by the client; add
  landing as a watcher trigger; add host errors to the introspection triggers of `defaults-loop.md` §3.1; correct
  the rejection sentence.

### F4 (High): two evidence standards, and the worked example's enforcement is wrong

- **Evidence:** `decision-sets.md` §4 fires on "1 job" for a host refusal, and §5 says "Repeats of one
  `failure_class` against one decision are the evidence for review". `defaults-loop.md` §7.2 needs N = 3 distinct
  jobs (estimate) before a candidate exists, and the loop's gate (§2 there) stops the build if fewer than N jobs
  show a repeat. The two designs treat one event as enough to doubt a decision and three as enough to add one,
  without saying why.
  The worked example (`decision-sets.md` §4, D87) gives `enforcement: code-check (tests in pr.rs and gh.rs)`. D87
  (read: `docs/decisions.md` line 1899 onward) states its landing rule as a recommendation in docs
  ("a user who merges by hand should pick a merge commit") and says "The recommendation cannot stop a user from
  squashing on the host"; the tests it cites are for the two rerun fixes. The merge request itself asking for
  `merge` is the unchanged code. So D87 is `brief`/design, not a `code-check` decision, and under `decision-sets.md`
  §3 item 3 a `code-check` decision is not put in a brief at all.
- **Needed:** state the rule (a single event fires review because review costs a short mark in a brief; N = 3
  governs adding defaults) and keep `decision-sets.md` §5's "repeats" sentence consistent with it; correct the
  example's enforcement kind.

### F5 (Medium): "one mechanism" for matching is two

See section 3, J2 and J3. Signal matching and default matching are different questions with different thresholds;
the claim in `decision-sets.md` §5 should say which parts are shared (the client, the stored-field input, the
stored model name) and which are not.

### F6 (Medium): `classes` vocabulary is not closed, and `landing` is not a class

- **Evidence:** `decision-sets.md` §2 says `classes` use "the same vocabulary as `applies_to`". The values in
  `defaults.toml` are `code`, `build`, `tests`, `docs`, `design`, `records`, `config` and `*` (read). The class of
  work in a record is a free `String` (`records.rs` lines 386, 785). The worked example sets `classes = [landing]`,
  which is a phase in the design's own table, not a class in use. `defaults-loop.md` §13 already lists a class
  registry as open.
- **Needed:** close the vocabulary (or state that set selection uses the phase alone for landing decisions) before
  set selection by code is specified.

### F7 (Medium): the introspection record and the watcher need joint rules

- Add `decisions_matched` to `athena.introspection/1` (`defaults-loop.md` §3.2), written by code.
- A signal firing triggers an introspection (`decision-sets.md` §4), and the watcher runs after each introspection.
  Say that an introspection recorded for a firing is not itself watched. `defaults-loop.md` §3.1 already stops
  the recursion for failed introspections only.
- `defaults-loop.md` §7.1 gives events with no intervention one when the introspection is recorded. Host errors
  and check failures would then become interventions with `source = observed` (a value `docs/records.md` line 70
  lists). Say that these do not count toward N in the defaults loop; only user-originated units do, otherwise a
  system fault counts as the user repeating an expectation.

### F8 (Medium): where `under-review` lives and how a brief reads it

`decision-sets.md` §4 sets `under-review` "beside `status`" and `defaults-loop.md` §9 keeps the briefs to the model
at the base commit (D26). Athena never edits a decision, so the state cannot sit in `docs/decisions.md`; it sits in
records. A brief then has an input that is not in the base commit. Name the store, and record it as a component
with a version and hash like the others (`crates/athena-core/src/reading.rs` line 54 shows the defaults as a
component), so a brief remains reproducible from records.

### F9 (Low): small corrections

- `decision-sets.md` §1 says D89's rule and "the `changelog` default say the same thing in two places and neither
  points to the other". D89 is the decision that adds the `changelog` entry in code and names it
  (`docs/decisions.md` line 1957), and `defaults.toml` has no `changelog` id (read). The two places are D89 and
  `ensure_changelog` (`reading.rs` lines 93-115). §6 step 1 then migrates `changelog` among "12 entries" it is not
  one of.
- `docs/decisions.md` D90, "Alternatives": "Each is weighed in the design, section 9"; the alternatives are now
  `defaults-loop.md` §11. Stale after the revision.
- The log header says an accepted entry is never rewritten (line 21), yet D46 (line 1020) and D87 (line 1901)
  carry a note added after acceptance. This supports the header-block plan of `decision-sets.md` §6, but §7 item 8
  should not cite the header as if it had always held.
- `defaults-loop.md` §5 gives "under 1,500 tokens for 12 entries" (estimate). The measured whole reader set is
  1,798 (F1), so the defaults share is not shown by that figure; measure `render()` alone.

## 3. Jev usage against `POST /v1/evaluate`

The reference is `defaults-loop.md` §6, which is **(reported)** from the user's probes and not reproduced anywhere in
this repository. I found no gateway client, no `evaluate` call and no Jev configuration under `crates/`
(grep for `jev` and `/v1/evaluate` found none), so nothing in code can contradict either document, and
`decision-sets.md` §5 cannot be checked against code, only against §6.

What the contract says: route `POST https://ai-gateway.vercel.sh/v1/evaluate`; fields `model`, `state`, `questions`
(a map from a key to `type` of `boolean` or `choice`, `instructions`, `criteria` from option to description); the
answer is the chosen option, a probability per option and a confidence; the chat route refuses Jev.

| # | `decision-sets.md` §5 says | Against §6 | Result |
|---|---|---|---|
| J1 | Uses `POST /v1/evaluate`; `model`, `state`, `questions`; `boolean` or `choice`; `criteria` | Same fields | **Match.** The route's base address and the refusal on the chat route are left to §6, which is acceptable if §6 is cited. |
| J2 | Set membership: "`choice` per decision (or `boolean`)", options apply / not apply | §3.4 and §7.2 there use one `choice` question with the ids as options | **Mismatch in shape.** A choice among two fixed options is a `boolean`; one question per decision means a map of up to N questions in one call. §6 reports the answer for one question only, so the response shape for several is **unverified**. Decide between one `choice` question per role (pick one) and many `boolean` questions, and probe the second before the client is written. |
| J3 | "Code applies thresholds to `p_top`" | §6.1: `p_top` is the probability of the chosen option | **Ambiguous for `boolean`.** If the chosen option is "no match" at 0.9, `p_top` is high and the record did not match. The threshold must read the probability of the "match" or "apply" option. Not stated in either design. |
| J4 | Signal match state: "the record's stored fields" | `defaults-loop.md` §5 lists the stored fields allowed: `what`, `why`, `class_of_work`, `category`, correction text, `failure_class`, `cause` | **Mismatch.** Host-error and check-result records are not on the list, and F3 shows the first has no stored status field. Extend the list and say that no stderr text from other jobs is sent. |
| J5 | Set-membership state: "the reading's class, paths, request summary" | Same list | **Mismatch.** A "request summary" is not a stored field unless it is the reading's own text. Name the field. |
| J6 | Two new typed questions in shadow, "widens" D90's narrowing | `defaults-loop.md` §10 lists four typed decisions; §10.2-§10.3 build the labeled set and scoring only for those | **Gap.** No labels, scoring or promotion criteria exist for set membership or signal match. D91's own text ("in shadow only") is consistent with D90 (read: `docs/decisions.md` lines 1993 and 2019-2021), but there is nothing to promote against. |
| J7 | Confidence only triggers a second call to a stronger model; every answer names its model | §6.1, §6.2 | **Match.** |
| J8 | Jev "must not drop a `requirement`"; those are selected by code | n/a | **Fits** (a design rule, no contract issue). |
| J9 | Cost and context | §5 and §6: 32K context; probe of 409 input and 59 output tokens for five short options (reported) | **Unchecked.** Offering about 91 decisions as options with criteria is, by scaling the probe, roughly 7,000 input tokens (estimate, unmeasured, and scaling is crude). Fits 32K; the cost and the accuracy at that many options are unmeasured. The signal question is smaller if signals are the options of one `choice`; `decision-sets.md` §5 does not consider that form. |

Net: the request shapes match; the answer handling (J3), the input fields (J4, J5), multi-question responses (J2)
and the promotion method (J6) do not yet.

## 4. Claims checked against code and records

`records/` is not in the snapshot, as for the designs' authors. I could check no record id, no count and no
"within the hour" claim. Every item marked **(reported)** in either design stays unverified.

| Claim | Source checked | Result |
|---|---|---|
| `defaults.toml` has 12 entries with `id`, `statement`, `applies_to`, `source` | `defaults.toml`; `defaults.rs` struct `Default` | **Confirmed (read).** Ids: `rust`, `strangers`, `evidence`, `buy-first`, `tested`, `public`, `small`, `verified`, `designed`, `lazy-requests`, `mechanism`, `root-cause`. |
| D26 said "eleven" and is corrected | `docs/decisions.md` line 634 | **Confirmed.** It now reads "eleven when this was written (twelve now)". `defaults-loop.md` §1 is right; review 1's Low item is closed. |
| Budgets contract 350, reader 1,800, producer 1,100, verifier 1,300 | `athena-context/src/lib.rs` lines 154-161 | **Confirmed (read).** |
| `check_budget` refuses an over-budget set | `lib.rs` lines 173-183 | **True only in tests (F1).** |
| `DefaultsModel::parse` rejects unknown fields, duplicates, empty statement or source | `defaults.rs` lines 7, 16, 36-49 | **Confirmed (read).** |
| `render()` carries id, scope and first sentence; `get()` gives the full statement | `defaults.rs` lines 51-83 | **Confirmed (read).** |
| Briefs carry full statements through `get()` | `briefs.rs` lines 82-90, 158 | **Confirmed**; the section is outside the budgeted sets (F1). |
| Intervention has `what`, `why`, `class_of_work`, `category`, `job`, `source`; eight categories | `docs/records.md` line 70; `records.rs` lines 121-142 | **Confirmed (read).** |
| A correction record carries its intervention | `docs/records.md` line 62 | **Confirmed (read).** |
| D87 superseded by D88 for landing shape; rerun fixes stand | `docs/decisions.md` lines 1899-1935 | **Confirmed (read).** `amended_by` rather than `superseded` for D87 is the right reading of line 1901. |
| D88 records `allow_merge_commit` false and the hand rebuild of PR #40 | `docs/decisions.md` lines 1924-1925, 1938, 1947 | **Confirmed (read);** the flags are the host's report "not re-read here" per D88 itself. |
| HTTP 405 refused the merge within the hour | not in D87 or D88 text | **Unverified (reported).** Neither D87 nor D88 mentions a 405; only D91 does. |
| Merge method chosen from the repository's flags | `gh.rs` lines 126-160; tests at 710-714, 814 | **Confirmed (read).** |
| Supersession phrases and lines: D46 line 1020, D79 line 1752, D87 line 1901, D88 line 1922 | `docs/decisions.md` | **Confirmed (read).** D79 reads "This supersedes D46's exclusion", so the parse must be case-insensitive. The "Amends" entries start at line 1060, in the section that begins at line 1055. |
| 91 decisions D01-D91 | `docs/decisions.md`, 91 `## D` headings (measured by count) | Confirmed. `decision-sets.md` says "D01-D90" for the log it migrates; D91 is the design's own. |
| `ensure_changelog` adds `CHANGELOG.md` and a `changelog` entry for change readings | `reading.rs` lines 93-115 | **Confirmed (read).** |
| Rejected outcomes and failure states are recorded | `run.rs` lines 925, 1042; `records.rs` line 96 | **Confirmed;** corrects `decision-sets.md` §4 (F3). |
| `docs/enforcement.md` lists rows with tests, so §6's verification has a model | `docs/enforcement.md` row M18 and neighbours | **Confirmed (read).** |
| BRIEF.md addenda quoted | `BRIEF.md` lines 502-509 | **Confirmed (read).** |
| Jev facts, gateway shapes, probe numbers, the four repeats, the §3.6 cases | not reproducible here | **Unverified (reported).** |

## 5. Smallest first build that would prove the system on real records

**Build: a read-only backtest of one decision, with no Jev, no brief change and no new record kind other than the
typed header.** It does four things, in one job, as `decision-sets.md` §8 steps 2 and 5 reduced to D87 and D88:

1. Write the typed header for D87 and D88 only (`kind`, `scope`, `status`, `enforcement`, `assumptions`, `signals`),
   with D87's enforcement corrected (F4), and parse the supersession phrases of `docs/decisions.md` into
   `superseded` / `amended_by`.
2. Read `records/` and list the real record ids for the D87-era landings (the jobs behind PR #39 and PR #40, and any
   intervention of category `did-work-themselves` or `fixed-environment` with class of work `docs`/landing text),
   as the gate in `defaults-loop.md` §2 requires.
3. Run one exact-match signal over those records (intervention, hand change on a landed branch, threshold 2 jobs
   from the D87 example), by code, using stored fields only.
4. Write one introspection record for the earliest matching event (by a language model, reviewed by a person), with
   the new `decisions_matched` field naming D87.

This exercises each of the four parts once: an introspection record (step 4), the loop's reading of past records
in Athena code (step 2), a typed decision (step 1) and a signal that puts it under review (step 3). It needs nothing
from Jev, and the signal sources it uses (interventions) already exist, so it avoids F3.

**Pass:** (a) the supersession parse returns exactly the four known hits D46→D79 (line 1020 and 1752) and
D87→D88 (lines 1901 and 1922) plus the "Amends" entries, with zero entries lacking a target id; (b) the signal marks
D87 `under-review` citing at least 2 distinct job ids and their intervention ids, all of which exist in `records/`;
(c) run over the same records, the same code marks no other typed decision (D88 included) under review; (d) the
introspection record's `earlier_cases` ids all exist and are in the same class (the verification of
`defaults-loop.md` §3.3).
**Fail:** the signal finds fewer than 2 jobs (the interventions were not recorded with the fields the signal needs,
so the design needs the host-error record first, F3); it fires on an unrelated decision; or any cited id does not
exist. A fail is a finding, not a retry: it closes the first build with the reason, as the gate does.
**Cost and size:** the number of records is unknown until step 2 runs (**unknown**, not estimated).

## 6. Changes required for "accept"

1. **F1:** replace "the reader sees the whole set" with code-side narrowing and a cap; add a runtime check for
   per-job set size; budget (or declare unbudgeted) the per-job "User defaults that apply" section; state that
   defaults do not count against the producer and verifier budgets today; correct "refused by code" in both
   designs to say tests.
2. **F2:** choose the store for assumptions and signals of expectations; add the fields to
   `athena.defaults-proposal/1`.
3. **F3:** specify a host-error record with a status field; add landing and host errors as watcher and
   introspection triggers; correct the statement that rejections have no record.
4. **F4:** one stated evidence standard for review versus for adding a default; correct D87's `enforcement` in the
   worked example.
5. **F5, J2-J5:** state which matching parts are shared; decide the question shape for set membership; define the
   threshold for `boolean` answers; extend the stored-field list of `defaults-loop.md` §5 for host and check
   records; name the "request summary" field; probe a multi-question call before the client is written.
6. **J6:** extend the labeled set, scoring and promotion criteria to the two new questions.
7. **F6, F7, F8:** close the class vocabulary; add `decisions_matched` and the no-recursion rule; name the
   `under-review` store and record it as a brief component.
8. **F9:** the four Low corrections, including D90's stale "section 9".

Unchanged and fine: the review-only stance, the computed (not stored) sets, the rejection of auto-supersession, the
migration order in `decision-sets.md` §6 (no rewriting of the log), and the build order of both designs once
steps 1 to 7 above are applied.

## Not checked

- `records/` (absent): no record id, count, or date was compared with either design.
- The gateway and every number in `defaults-loop.md` §6; no call was made.
- Whether a job at run time ever blocks on the reader set size (only the test enforcement was read).
- `docs/design/athena-v0.md` E12 and the cited `BRIEF.md` lines 25, 144 and 217 were not re-read for this review.
- The token cost of rendering sets; only the static sets were measured (F1).

## Verdict line

**accept with changes** — the eight changes in section 6; F1 to F4 must be settled before any build, and the build
to start with is the one in section 5.

The designs are not edited by this review.

## Addendum: first backtest result (2026-10-14)

The section 5 build ran on the real records and failed (reported by the user; recorded in `decision-sets.md` §9).
Corrections to the pass conditions above:

- **(a)** The parse found 24 hits, 0 lacking a target, plus one the list above did not expect: D11, line 322,
  "Revised ... superseded by D39". It is a partial supersession, so it is `amended_by`, and the expected list is
  corrected to include D11/D39.
- **(b)** D87's signal named class `docs` and found 0 jobs. The real interventions are class `code`; job
  `20261013-192156-ar1x` (9h2j, the hand squash of PR #41 after HTTP 405) is the only one after D87 was recorded
  (kn3q and qk2d predate it). The F3 host-error record is therefore needed before (b) can pass on the 405.
