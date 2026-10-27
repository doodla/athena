# Job 20261027-082521-0sfz: Fix t3 trigger wording and make the decider test structural

## Request (verbatim)

> try again on 20261027-074457-bx8e

## Reading (accepted by the user)

Retry of the rejected job. Make the t3 trigger in model.json say only what its cited claims support, and replace the prefix-match decider test in xtask/src/site.rs with a structural check backed by a model field. The audience is the newcomer reading the published study page, and the verifier who checks it.

Assumptions:
- The request means the introspection's proposed next request: fix the two remaining findings (t3 wording, structural decider test).
- A structural check needs a model field. I assume an optional 'trigger_actor' string is added to each transition and the schema or validator accepts it; if one already exists, the producer reuses it.
- t3 will be reworded so the actor is the worker or a script, as the claims (c3, c31, c37) say, with trigger_evaluated_by changed to match. If no claim says the agent accepts the report, the producer must not keep the agent as the subject.
- The earlier JSON spacing fix on t3, t8, t10 and t11 and the in-scope glossary entry already landed or carry over unchanged.

## Acceptance criteria

1. [content] In model.json the t3 trigger names only an event that a claim it cites supports (c3, c31, c37). It does not say the agent accepts or records the worker's report unless a cited claim says so. Its trigger_evaluated_by value matches the actor the claims name.
2. [content] The model records each transition's trigger actor in a field, not in the trigger text. Every cited claim id exists in claims.json.
3. [behavior] Run cargo test. The site test checks structurally, with no text prefix or keyword ban: for each transition with trigger_evaluated_by 'agent', the actor field says agent, and every cited claim id resolves. All tests pass.
4. [behavior] Run cargo test. A fixture with an agent-evaluated transition whose trigger describes another actor's event is rejected by the check, and a fixture with a valid agent trigger passes. The tests assert both outcomes.
5. [behavior] Run cargo run -p xtask -- site. The study builds and validates, and the generated page keeps the in-scope glossary entry and a next-step link before the first transition card. It has no f2/f3 model-limit note.
6. [behavior] cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
7. [content] The CHANGELOG.md entry describes only what was done and does not overclaim.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)

## Out of scope

- Changing how much of the page is about Athena rather than firstmate
- Other audience-rating findings that already landed
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy -D warnings and fmt --check must pass, with positive and negative fixtures.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decider rule is enforced by a structural check on a model field, not by a text prefix.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require fixtures that the check rejects or accepts, and a run of the site build.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The wording is plain and the changelog is accurate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the t3 wording, the test, and the changelog change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-074457-bx8e raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (studies/firstmate-work-handling/model.json:37): The new t3 trigger says something no cited claim supports, and the study's own claims contradict it. The trigger is "the agent records a ready PR or a ready branch for the task" (citing c10, c11, c32 and c37). None of these says the agent records anything. c31's source says "the pr= line bin/fm-pr-check.sh records", c3 says "a worker records its ready PR", and c37 has the worker stop with a ready branch, which nothing records. The goal was that the newcomer page makes only cited statements, and this one is not.
> major (xtask/src/site.rs:876-891): The decider test is still a text check (a prefix match), not a structural one, and does not detect a trigger that describes another actor's event. It asserts trigger.starts_with("the agent "). "the agent sees the merge land" would pass, and the new t3, an event the claims give to a script or the worker, does pass. It also passes on the base model.json, so no failing case shows it catches a fault. The root-cause default asks for a structural check, such as an actor or event field in the model or a list of allowed agent actions.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In model.json the t3 trigger names only an event that a claim it cites supports (c3, c31, c37). It does not say the agent accepts or records the worker's report unless a cited claim says so. Its trigger_evaluated_by value matches the actor the claims name.
The model records each transition's trigger actor in a field, not in the trigger text. Every cited claim id exists in claims.json.
Run cargo test. The site test checks structurally, with no text prefix or keyword ban: for each transition with trigger_evaluated_by 'agent', the actor field says agent, and every cited claim id resolves. All tests pass.
Run cargo test. A fixture with an agent-evaluated transition whose trigger describes another actor's event is rejected by the check, and a fixture with a valid agent trigger passes. The tests assert both outcomes.
Run cargo run -p xtask -- site. The study builds and validates, and the generated page keeps the in-scope glossary entry and a next-step link before the first transition card. It has no f2/f3 model-limit note.
cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
The CHANGELOG.md entry describes only what was done and does not overclaim.

## Report

End with the producer report as your structured output.
