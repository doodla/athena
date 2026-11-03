# Owner style

## The rule

Anything written for the owner to read and decide on follows the owner's own words on style:

> Show me with big pictures and little words without witholding key details. Please dont use weird metaphors.

(The spelling is the owner's, kept as written.)

In practice:

- Short plain words, short lines, and one idea per line.
- A decision card leads with the decision as one yes/no question, then one line for what happens if yes, one
  for if no, the cost and risk in a few words, and the recommendation with its one-line reason. Evidence
  paths, record ids and links go in a collapsed section below, never above these lines.
- No jargon in the visible text: no rung names, role names or record kinds.
- Leave out no detail the owner needs to decide. When a detail is not in the record, say "not recorded".
- No invented figures. A cost or risk is given in a few plain words, labelled as Athena's estimate.
- No metaphors or figures of speech.
- A recommendation starts `Recommend yes:` or `Recommend no:` and gives its reason in one line. It is advice:
  the owner decides.

## Enforcing mechanism

The owner decision page (`decisions.html`, built by `cargo run -p xtask -- site`) is rendered by
`xtask/src/decisions.rs`. Its tests refuse a card of more than about 120 visible words and a card with no
yes/no recommendation line. They check that the question, yes, no, cost and recommendation lines come before
any path or record id, that the evidence sits in a collapsed `<details>` element, that the visible text holds
no banned jargon, and that each card has exactly one question.

## Extending it

A new page or message meant for the owner reuses the rule above and adds a test that shows a breach failing.
