# Job 20261013-231523-8q6p: verify "Review the defaults-loop design against records, code and Jev gateway"

## Request (verbatim)

> Review docs/design/defaults-loop.md, written to docs/design/reviews/defaults-loop-review-1.md, ending with a verdict (accept, accept with changes listed, or reject). Check its evidence against the records it cites and its claims against the code. Also check it against how Jev is actually reached, which I probed today through the gateway: /v1/chat/completions refuses Jev ('a decision model, not a language model'); Jev is called with POST https://ai-gateway.vercel.sh/v1/evaluate, body {model: 'typesafe-ai/jev', state: <text>, questions: {<key>: {type: 'boolean' or 'choice', instructions, criteria: {<option>: <description>}}}}; the answer gives choice, per-option probabilities and a confidence; an optional fallback model can run when confidence is below a threshold (providerOptions.gateway.models with when.confidenceBelow). One probe: the correction 'A CHANGELOG.md entry in the repository's usual style is allowed alongside the design doc', asked against changelog, small, root-cause, designed and none, returned changelog at 0.90 (none 0.06, designed 0.04), confidence 0.87, 409 input and 59 output tokens, cost /bin/bash.000017. Review only.

## Reading (accepted by the user)

Write a review of docs/design/defaults-loop.md at docs/design/reviews/defaults-loop-review-1.md. It checks the design's evidence against the records it cites, its claims against the code, and its assumptions about Jev against the gateway behavior the user probed. It ends with a verdict of accept, accept with changes, or reject, and changes nothing else.

Assumptions:
- The design doc and the records it cites exist as the user describes; I did not read them in this pass.
- The probe figures come from the user and cannot be reproduced here. The review labels them user-reported.
- The cost in the request is garbled ('/bin/bash.000017'). The review quotes it as the user wrote it, flagged as unreadable, and does not guess a value.
- Existing reviews in docs/design/reviews/ set the format.

## Acceptance criteria

1. [content] docs/design/reviews/defaults-loop-review-1.md exists and follows the layout of existing reviews in docs/design/reviews/.
2. [content] The review checks the design's evidence against the records it cites, and notes any cited record that is absent or does not support the claim.
3. [content] The review checks the design's claims against the code, citing file paths for each claim it confirms or refutes.
4. [content] The review checks the design against the Jev gateway facts: /v1/chat/completions refuses Jev; the call is POST /v1/evaluate with model, state and questions (boolean or choice, with instructions and criteria); the answer has choice, per-option probabilities and confidence; fallback runs through providerOptions.gateway.models with when.confidenceBelow.
5. [content] The review reports the probe result (changelog 0.90, none 0.06, designed 0.04, confidence 0.87, 409 input and 59 output tokens) as the user's measurement, and says what it does and does not show, since it is one probe.
6. [content] The review ends with a verdict line: accept, accept with changes listed (the changes enumerated), or reject.
7. [content] No file other than the review is changed, and the design doc is not edited.

## Expected deliverables

- docs/design/reviews/defaults-loop-review-1.md (create)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing docs/design/defaults-loop.md or applying the review's changes.
- Changing code, tests, configuration or CHANGELOG.md.
- Calling the gateway or Jev; the probe results come from the user.
- Implementing the defaults loop.

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every review finding cites a record, a file path, or the user's probe, and any number is labeled measured or estimated.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written so an outside reader can follow it without the conversation.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file, and nothing else changes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Findings name the underlying design flaw, not just the one instance.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The review includes no secrets or personal data, such as the user's email.

## The change

Base a98f1dd2ab69 → candidate e5a00b597b9c. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/design/reviews/defaults-loop-review-1.md | 272 ++++++++++++++++++++++++++
 1 file changed, 272 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (382 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
