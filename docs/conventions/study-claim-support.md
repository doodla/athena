# Study claim support

## The rule

When a job's files are under `studies/` (`model.json`, `intro.json` and similar study text), the producer
checks every statement it adds or rewords:

1. Open each claim id the statement cites in `claims.json`.
2. Check that the claim's text states what the sentence says, including who does the action.
3. List each changed statement with its supporting claim text in the change description.

If no claim supports a statement, the producer adds a sourced claim to `claims.json` or drops the statement.
The changelog never says claims support a wording unless they do.

## Why

Study-page edits kept adding or rewording statements that their cited claims did not support, for example a
sentence that gave an action to a different actor than the claim names. The verifier found this on five jobs.
A cited claim id looks like support, so only reading the claim text shows the gap.

## Enforcing mechanism

Prose-only. No code checks claim support. The rule is carried to producers by
`crates/athena-cap-change/prompts/producer.md`, and the verifier checks it by reading the cited claims
against the listed statements in the change description.
