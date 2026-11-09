# Agent reference

Optional depth for the five roles Athena runs: reader, planner, critic, producer and verifier. Each role's
prompt points here with "if docs/agent-reference.md exists in the working copy". This file holds explanation
and examples only. Every rule a role must follow is stated in that role's prompt, so a role that cannot read
this file loses depth, not correctness.

The file exists only in Athena's own repository. It is read at run time from the agent's working copy and is
not compiled into the binary, so on any other repository the pointer finds nothing and the prompt stands alone.

Where a section says "enforced by", the named code refuses or flags the work whether or not the prompt was read.

## Reader

**Goal and audience.** A reading names what the request is for and who will use or judge the result. Example:
for "tidy the docs", the goal is a docs set a newcomer can follow and the audience is a first-time reader, so one
criterion should say what that reader can now do, not only that files changed.

**Governance files.** These files set how Athena and the build behave: `athena.toml`, `defaults.toml`,
`.gitattributes`, `setup.sh`, `rust-toolchain.toml`, cargo, rustfmt and clippy configuration, `[lints]` tables,
and anything under `.github/workflows/`. A producer may edit one only when the reading lists it as a deliverable
and a criterion names it. Example: a request to add a CI step needs the workflow file as a deliverable and a
criterion such as "the workflow runs the new step". Enforced by admission of the reading and the scope check.

**Forecasts.** Each probability (`p_reading_accepted`, `p_verification_passes`, `p_user_intervenes`) is scored
with a proper scoring rule: you score best by reporting what you believe, not by hedging toward 0.5. Cost and wall
time are three percentiles (10th, 50th, 90th) that cover producer and verifier together, from dispatch to
verdict, including a cold build of the checks. The brief gives a reference forecast built from past jobs; move
away from it only for a reason particular to this request, and write that reason in `departures`.

**Judging a reading.** Athena runs the reading rules listed in the reader brief and refuses a reading that breaks a
checked rule, then asks again once with the refusal as a correction. The brief's rule list names the check behind
each rule; the brief, not this file, is the authority on them.

## Planner

**Plan fields.** `decisions` pair each choice with its reason and the options rejected. `files` lists every file
touched with the action and purpose. `steps` run in producer order. `risks` each carry a mitigation.
`open_questions` hold only what the code could not settle, prefixed `User:` or `Host:` where the brief asks.

**Dependents.** A dependent is anything that reads or relies on a thing the plan changes. Example: changing a
budget constant has the budget tests, the facts file that quotes it, and any test that pins the old value as
dependents. A plan with no dependents list is refused by admission.

**Demonstrations.** One per criterion. Example of a usable entry: method "run the existing budget test with
output shown", entry point `check_budget`, seam the real prompt constants, observes "each set is at or under its
budget", and fails without the behavior because an over-budget set makes the check return an error. Example of an
unusable entry: "check that the file exists", which shows presence and not behavior.

**Seams.** A seam is where a test can substitute or observe a dependency. Naming the seam, the fake or fixture and
how it is injected lets the critic see that the test exercises production code and not a copy of it.

**Commit shapes.** Example shapes for a history rule: job commits, a records-only commit, an owner-path commit,
an intervention commit, and a retry-after-block landing. The table gives the rule's outcome for each, and the
fixture holds a realistic mix of them.

## Critic

**Severity examples.** The definitions are in the critic prompt; these show how they fall.
- blocker: the plan edits a file that does not exist, or names a function the code lacks, so no producer can follow it.
- major: a criterion has no demonstration, or a demonstration only checks that a file exists.
- minor: a risk lacks a mitigation, or a step is out of order but recoverable.
- nit: a heading or phrase the critic would word differently.

**Verdict.** `accept` means the producer can start from this plan as written. `revise` means the planner can fix the
plan in one more round. `reject` means the request or reading cannot be planned as it stands.

## Producer

**The report.** The structured output states what changed and why, the checks run and their results, open
issues, mistakes noticed, facts about the repository discovered along the way, and context or tools the producer
wished for. It goes to the user and never to the verifier, who sees only the candidate diff.

**Governance files.** If the work needs one that the reading does not list, stop and put it in `open_issues`. Do
not edit it to make progress.

**Requirement rows.** In `docs/design/requirements-map.md` a row marked Covered earns a major finding from Athena.
Mark rows Partial and let verification decide.

**Criteria check.** Copy each criterion line from the brief's criteria_check section into the entry's criterion
field, then set `met` or `unmet` and cite what you read or ran. A criterion only Athena's own recorded checks can
show is reported with evidence that starts "left to Athena's checks".

## Verifier

**Severity examples.** The definitions are in the verifier prompt; these show how they fall.
- blocker: the change deletes data, leaks a secret, or breaks a feature that worked.
- major: a criterion is not met, or an applicable default such as "tested" is violated.
- minor: a missing edge-case test, or documentation that is correct but unclear.
- nit: naming or formatting taste.

**Commands.** Athena counts a behavior criterion met only when it saw each cited command run. A line such as
`cargo fmt --check && cargo clippy` is judged one simple command at a time, so cite both parts. Enforced by the
verdict step in `crates/athena-core/src/verify.rs`.

**Snapshot.** The verifier works in a copy of the candidate. Output that depends on `records/` or history says so,
because those paths are absent there and a result about the copy is not a result about the real repository.
