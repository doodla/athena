# Owner style

## The rule

Anything written for the owner to read and decide on follows the owner's own words on style:

> Show me with big pictures and little words without witholding key details. Please dont use weird metaphors.

(The spelling is the owner's, kept as written.)

In practice:

- Short plain words, short lines, and one idea per line.
- Show the thing itself, such as the title, the rung, the source and the evidence, instead of describing it.
- Leave out no detail the owner needs to decide. When a detail is not in the record, say "not recorded".
- No figures that the record does not hold. A cost or risk is shown only when a field of the record gives it.
- No metaphors or figures of speech.
- A recommendation is given, with its reason, in one line. It is advice: the owner decides.

## Enforcing mechanism

The owner decision briefs page (`decisions.html`, built by `cargo run -p xtask -- site`) is rendered by
`xtask/src/decisions.rs`. Its tests refuse a brief of more than about 120 words of displayed prose and a brief
with no recommendation line, and check that a brief's title, rung, source and evidence come from its own
proposal and that nothing is invented.

## Extending it

A new page or message meant for the owner reuses the rule above and adds a test that shows a breach failing.
