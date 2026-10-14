# Job 20261014-050658-areb: verify "Add Jev client and shape-faithful fake; shadow-ask introspection questio"

## Request (verbatim)

> Build step 5 of the build order in docs/design/defaults-loop.md section 12: the Jev client and a fake that follows the real response shape, used only in shadow (it never decides). Facts from live probes through the gateway today (2026-10-13 and 2026-10-14; the operator ran them, agents may not reach the network): POST https://ai-gateway.vercel.sh/v1/evaluate with body {model: 'typesafe-ai/jev', state: <text>, questions: {<key>: {type: 'choice', instructions, criteria: {<option>: <description>}} or {type: 'boolean', instructions}}}. Several questions in one call work. A choice answer is {type: 'choice', choice, probabilities: {<option>: p}, confidence}; a boolean answer is {type: 'boolean', probability} (the probability of true, with no value field). The response has usage {inputTokens, outputTokens} and providerMetadata.gateway.cost as a decimal string (about /bin/bash.00002 a call). /v1/chat/completions refuses Jev as 'a decision model, not a language model'; treat that as a build error, not a retry. In this environment a proxy authenticates calls to *.vercel.sh; elsewhere the key comes from the AI_GATEWAY_API_KEY environment variable and is never written to files, records or output. Use the existing retrying outbound wrapper. Ask Jev the two introspection questions (failure class, and whether it is a repeat) in shadow next to the existing classification, and record both answers so they can be scored later. Live tests are opt-in, as for the other models.

## Reading (accepted by the user)

Build defaults-loop step 5: a Jev client for POST /v1/evaluate and a fake that returns the probed response shape. Both are used only in shadow. Next to the existing introspection classification, Athena asks Jev the failure-class and is-repeat questions and records both answers for later scoring. Jev's answer never changes the recorded class or any outcome.

Assumptions:
- The 'existing retrying outbound wrapper' is the Retry/Retry::STANDARD machinery in crates/athena-core/src/gh.rs. If it only wraps the gh CLI, the Jev client reuses its Retry policy and transient classification. Without a new HTTP crate in the workspace, the client may need one, chosen under buy-first.
- The two introspection questions are the failure class (a choice over the class registry plus new) and whether this is a repeat (boolean). They are asked in one evaluate call.
- Shadow means Jev answers are stored as extra optional fields on the athena.introspection/1 record, or a sibling field, and never feed the class, id or any decision.
- Live facts could not be checked here because agents have no network. Live tests are written, opt-in and not run by the verifier.
- Jev takes the same fields the reading code already passes to the model (section 3.3) as its state text, so no new data is read.

## Acceptance criteria

1. [content] A Jev client module builds POST https://ai-gateway.vercel.sh/v1/evaluate with body {model:'typesafe-ai/jev', state, questions:{key:{type:'choice',instructions,criteria:{option:description}} | {type:'boolean',instructions}}}, and supports several questions per call.
2. [content] Response types parse a choice answer {type:'choice',choice,probabilities,confidence} and a boolean answer {type:'boolean',probability} with no value field, plus usage {inputTokens,outputTokens} and providerMetadata.gateway.cost as a decimal string parsed to a number.
3. [content] A refusal from /v1/chat/completions ('a decision model, not a language model') is a non-retried build error. The client calls /v1/evaluate only.
4. [content] Calls go through the existing retrying outbound wrapper. Only transient failures retry. Auth and 4xx errors do not.
5. [content] The key is read only from AI_GATEWAY_API_KEY when present (the proxy authenticates *.vercel.sh calls otherwise). It is never written to files, records, logs or error text, and a test asserts it is absent from the Debug and error output.
6. [content] A fake Jev returns the same typed response shape. It is selectable in tests and is the only Jev used outside opt-in live tests. Live tests are ignored or opt-in in the same way as the other models' live tests and are skipped by default.
7. [content] Introspection asks Jev the failure-class question (choice over registry ids) and the is-repeat question (boolean) in one evaluate call, in shadow. Both answers (choice, probabilities, confidence, probability, and usage and cost) are stored in the introspection record so they can be scored later. A Jev failure is recorded and never blocks the job or changes the recorded class.
8. [content] Records written with Jev answers validate against the schema. Older introspection records without the new fields still parse.
9. [content] docs/design/defaults-loop.md section 12 marks step 5 as built with what was and was not checked live. CHANGELOG.md has an entry.
10. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` from the repository root. All pass with no live network, and the new fake-based tests cover choice, boolean, multi-question, error and shadow-recording paths.

## Expected deliverables

- crates/athena-core/src/jev.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.toml (modify)
- Cargo.lock (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Letting Jev decide anything, or replacing the language-model classification
- Running live probes or live tests
- Scoring Jev's recorded answers
- Host-error and landing triggers
- Changing CI, lints, athena.toml or defaults.toml
- Using /v1/chat/completions

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Client, fake and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has fake-based tests; cargo test and clippy are clean.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The API key is never written to files, records or output.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing retry wrapper; use a maintained HTTP crate if one is needed and record the call.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by cargo test against the fake; live checks stay opt-in.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the section 12 update are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change covering client, fake and shadow wiring only.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Section 12 notes cite the probe dates and mark unmeasured numbers as estimates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base c3670ca6d62b → candidate bbadada7e9cd. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                             |  15 +
 crates/athena-cli/src/display.rs         |  14 +
 crates/athena-cli/src/wiring.rs          |   2 +
 crates/athena-cli/tests/common/mod.rs    |   2 +
 crates/athena-cli/tests/introspection.rs | 155 +++++-
 crates/athena-cli/tests/proposals.rs     |   1 +
 crates/athena-core/src/gh.rs             | 148 +++++-
 crates/athena-core/src/introspect.rs     | 107 ++++-
 crates/athena-core/src/jev.rs            | 787 +++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs            |   4 +
 crates/athena-core/src/proposals.rs      |   1 +
 crates/athena-core/src/run_pin_tests.rs  |   1 +
 crates/athena-core/src/status.rs         |   5 +-
 crates/athena-data/src/records.rs        |  58 ++-
 docs/decisions.md                        |  26 +
 docs/design/defaults-loop.md             |  19 +
 16 files changed, 1325 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (499 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
