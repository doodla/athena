# Job 20261014-213435-xeax: verify "Make the steward a model-backed, read-only agent that queues work"

## Request (verbatim)

> try again on 20261014-192610-xgke

## Reading (accepted by the user)

Retry of the rejected steward job: replace the failure-counting steward with an AgentDriver run using the stored STEWARD_PROMPT, read-only access to Athena's code, records, git log and GitHub CI/PRs/deploys, and queue its items through decide(). This time the gh policy denies every field/method flag form (including attached `-ftitle=x`), and each criterion is exercised by a test. The audience is the Athena owner.

Assumptions:
- The new request is 'try again': the original goal and accepted reading stand, plus the earlier verifier's findings must be fixed.
- Steward git-history access stays a log file of recent commits plus the one-commit snapshot, because private paths are hidden; the limit is documented.
- The steward may borrow the verifier's model/budget unless a steward-specific setting is cheap to add; this is noted in the docs.
- Blind-spot keys use a stable source name, not error text, so repeated failures dedupe.
- Extra files touched by the last attempt (store.rs steward_runs, test helpers handling Role::Steward) are allowed and are listed as deliverables this time.

## Acceptance criteria

1. [content] steward::run invokes an AgentDriver with STEWARD_PROMPT as the system prompt and records the invocation (role, usage, cost) like other roles; note_finished still runs it in the background without blocking.
2. [content] The gh policy in policy.rs is allow-list based for flags: any word starting with -f/-F (attached forms like -ftitle=x, -Fx=@f), --field, --raw-field, -X/--method other than GET, --input, and mutating git/gh commands are denied; unit tests cover these denied forms and the allowed reads (gh run list/view, gh pr list/view, gh api GET).
3. [content] Agent output is structured work items (kind, key, title, rung, source and evidence refs, request text); Athena validates it and routes every item through decide(): rungs at or below the limit queue a Request, higher rungs queue only a proposal with evidence.
4. [content] Each unreadable source (including GitHub unavailable) becomes a blind-spot item whose key uses only the stable source name, not error text; items whose key already exists in records are not queued twice.
5. [content] The old repeated_failures counting is removed or passed to the agent as input; docs/design/roles-and-ladder.md and CHANGELOG.md say the steward now calls a model and state the git-history and model/budget limits.
6. [behavior] Run `cargo test`: crates/athena-cli/tests/steward.rs with the fake driver shows (a) scripted output with a real gap commits the work item and a queued request, (b) empty output makes no commit, (c) a second run with the same keys queues nothing again, (d) a nested run makes no real model call, and (e) the fake agent's recorded reads cover the named sources (jobs, events, introspections, class registry, decisions, git log).
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-data/src/roles.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Giving the steward write access to GitHub, git, or the repository.
- Exposing private paths or full git object history to the agent.
- Changing the requirements ladder rungs or the steward cadence.
- A separate steward model/budget configuration (only noted as future work).

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Policy and steward tests added; cargo test and clippy pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Each criterion is exercised by a fake-driver test before finishing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Policy denies the whole class of attached field flags, not just -ftitle; blind-spot keys made stable.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Read-only access is enforced by the tool policy, with tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG explain the change for outside readers.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the goal from the earlier attempt and findings.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 611d3e0a70d7 → candidate 77b55a73147f. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  14 +
 crates/athena-agent/src/fake.rs         |   7 +
 crates/athena-agent/src/lib.rs          |   5 +-
 crates/athena-agent/src/policy.rs       | 321 ++++++++++-
 crates/athena-cli/src/wiring.rs         |   4 +-
 crates/athena-cli/tests/common/mod.rs   |  19 +-
 crates/athena-cli/tests/retry.rs        |   2 +-
 crates/athena-cli/tests/steward.rs      | 518 +++++++++++++++--
 crates/athena-context/src/lib.rs        |  25 +
 crates/athena-context/src/outputs.rs    |  96 ++++
 crates/athena-core/src/lib.rs           |  11 +-
 crates/athena-core/src/run_pin_tests.rs |   3 +-
 crates/athena-core/src/steward.rs       | 982 +++++++++++++++++++++++++++++---
 crates/athena-core/src/store.rs         |   9 +-
 crates/athena-data/src/records.rs       |   7 +-
 crates/athena-data/src/roles.rs         |  16 +-
 docs/design/requirements-map.md         |   2 +-
 docs/design/roles-and-ladder.md         |  77 ++-
 18 files changed, 1934 insertions(+), 184 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (584 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
