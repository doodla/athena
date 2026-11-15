# Job 20261114-045711-1e6d: verify "Tell the reader to require end-to-end, can-fail behavior criteria"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request described a command ("works the queue within a budget") without stating the end-to-end e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criteria-check-presence-not-behavior (20261014-184842-whyd)
> Cause: The request described a command ("works the queue within a budget") without stating the end-to-end effect: queued items must be landed, and the rating must be of the page after those changes. Nothing required the build to prove that the rat...
> Seen 8 time(s) in 8 jobs. The introspections that record it:
> Introspection 20261015-070659-2kqw (job 20261015-064153-wekz); prevention it proposed: Require that a loop-style command's acceptance criteria state the observable outcome, not the presence of parts. For drain, a test must run one pass with a stubbed queue item. It must assert that `athena land --pr` is invoked for the item. ...
> Introspection 20261015-081905-xw6c (job 20261015-063409-hekk); prevention it proposed: Require any test of a protection or denial to prove it can fail. The test must show the write succeeds on an unlocked snapshot under the same user and environment, then show it is refused once locked. It must fail loudly, not pass silently,...
> Introspection 20261015-160328-7077 (job 20261015-152305-6h8k); prevention it proposed: Add a code check, and a matching test, that the source check covers every content file of the page. The check should enumerate the page's text-bearing inputs (claims.json, intro.json and any others) and fail if any of them is not compared w...
> Introspection 20261015-163343-435x (job 20261015-160432-g1tq); prevention it proposed: A mutation-style check on negative tests that a criterion names. The check would run the named test against the code with the guard disabled or the invariant inverted, and require the test to fail there. It would also require the test to as...
> Introspection 20261101-222454-36yp (job 20261031-031939-wz33); prevention it proposed: Require every behaviour criterion to be demonstrated by a test that exercises the real production path end to end, not a default or no-op hook. Here, a landing through the drain must produce a record commit on the records branch. Split mult...
> Introspection 20261103-074940-hcey (job 20261103-020319-ef9q); prevention it proposed: For any job that renders or reports from repository records, require two things at plan and criteria stage. First, each criterion must be demonstrated by running the real command on the real (or a copied snapshot of the) repository and asse...
> Introspection 20261103-235051-6bj9 (job 20261103-201548-qt18); prevention it proposed: Add a test, required by the reading for any rendered owner-facing text, that builds cards from at least two real or realistic Steward proposals with different content. It must assert three things. The question line is not the title with a f...
> Introspection 20261112-023747-fs2m (job 20261030-001211-6mzd); prevention it proposed: Require every criterion for a new mechanism to name its consumer. The criterion would say which non-test code reads the new data and what observable behavior changes. Its demonstration would run that code path, for example resolving the mod...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause of criteria-check-presence-not-behavior (jobs 20261015-064153-wekz, 20261015-063409-hekk, 20261015-152305-6h8k and others): the reader's brief never told it that a criterion for a loop, protection or rendered output must name the real end-to-end run, its observable effect, and proof it can fail, so criteria checked parts. Add that as a prose-only reading rule in the reader brief, not a refusal rule, with a test that the brief carries it.

Assumptions:
- Goal and audience: the reader agent that writes acceptance criteria, and the verifier and owners who judge them; the goal is that criteria check the requested behavior, not the presence of parts.
- Code survey: read crates/athena-context/src/briefs/reader.rs (READING_RULES array of 13 lines, rendered under '# Reading rules'), crates/athena-core/src/reading/checks.rs and reading/surveys.rs (check_criteria_evidence, the refusal checks that stay unchanged) and crates/athena-context/src/briefs.rs (brief tests). The entry point is the reader brief builder; no record or data file is added, so no AnyRecord or check_record_file round trip is touched.
- Existing code: extends crates/athena-context/src/briefs/reader.rs READING_RULES, following the existing prose-only 'Reused field' rule and its Root cause comment; docs/decisions.md.
- Information is preferred over refusal per the request; a refusal rule cannot judge whether a criterion exercises the end-to-end path, so the rule is marked prose only.
- The reader prompt file has 2 tokens left, so the change goes in the code-held READING_RULES, not the prompt files; if a test counts the brief's tokens, the rule is kept short enough to fit, and a separate budget raise is out of scope.
- Kept/dropped files of the earlier candidates: crates/athena-cli/src/drain.rs, tests/drain.rs, display.rs, main.rs, lib.rs, crates/athena-core/src/{steward,pin,run,verify,run_pin_tests,lib}.rs, crates/athena-git/src/snapshot.rs, README.md and athena.toml are all dropped (not changed here); CHANGELOG.md and docs/decisions.md are kept as deliverables.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/reader.rs READING_RULES gains one new line, marked `(prose only)`, saying: for a loop-style command (drain, queue), protection or denial, or rendered or reported output, a behavior criterion names the real production path run end to end, the observable effect (for example an item landed, a record written, a page changed) and a demonstration that the check fails when the effect or guard is absent; the array length is updated to match. Preceded by a Root cause comment citing jobs 20261015-064153-wekz, 20261015-063409-hekk and 20261015-152305-6h8k.
2. [behavior] A new unit test in crates/athena-context/src/briefs.rs (or reader.rs tests) builds the reader brief and asserts it contains the new rule text; it fails on the code before the change. Verifier runs `cargo test -p athena-context`.
3. [behavior] No refusal check is added: `cargo test -p athena-core` still passes, including tests/refusal_corpus.rs unchanged, so expected 0% of newly refused readings; the rule is information only because giving the reader the information can prevent the failure.
4. [content] A new appended entry in docs/decisions.md names the root cause (the reader was not told that criteria must check the end-to-end effect and that a check can fail), the alternatives (a refusal rule, rejected), and has a Provenance bullet; CHANGELOG.md records the change. No existing entry is edited.
5. [behavior] Athena's recorded fmt, clippy and test checks pass on the final tree (`cargo fmt --check`, `cargo clippy`, `cargo test`).
6. [content] Job 20261015-064153-wekz findings (drain not landing via `athena land --pr`, unbudgeted retry read) are not re-implemented: that candidate's files are dropped, and the class is addressed by the reader rule. Same for 20261015-063409-hekk (root-run write tests that check nothing) and 20261015-152305-6h8k (source check missing intro.json); the new rule names those patterns.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)

## Out of scope

- Adding any refusal check to athena-core reading checks
- Changing prompt files or token budgets
- Changing drain, verifier snapshot locking or source-check code from the earlier jobs
- Changes to athena.toml, defaults.toml or CI workflows
- Editing existing entries of docs/decisions.md or records/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the brief carries the rule and tests fail before the change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the reader brief where criteria originate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule line, one test, docs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): No refusal check is added: `cargo test -p athena-core` still passes, including tests/refusal_corpus.rs unchanged, so expected 0% of newly refused readings; the rule is information only because giving the reader the information can prevent the failure.

## The change

Base b059a35e9117 → candidate 79b0888e57a9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  2 ++
 crates/athena-context/src/briefs.rs        | 50 +++++++++++++++++++++++++-----
 crates/athena-context/src/briefs/reader.rs |  8 ++++-
 docs/decisions.md                          | 23 ++++++++++++++
 4 files changed, 75 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045711-1e6d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045711-1e6d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43662217926 for 79b0888e57a906bce9b0d48d6edee2fb4520b002)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43662217926 for 79b0888e57a906bce9b0d48d6edee2fb4520b002)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43662217926 for 79b0888e57a906bce9b0d48d6edee2fb4520b002)
- ci-result: `not a check` passed (Check results for 79b0888e57a906bce9b0d48d6edee2fb4520b002 come from the CI run https://github.com/doodla/athena/actions/runs/43662217926 (974s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Athena's recorded fmt, clippy and test checks pass on the final tree (`cargo fmt --check`, `cargo clippy`, `cargo test`).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
