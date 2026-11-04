# Owner style

## The rule

Anything written for the owner to read and decide on follows the owner's own words on style:

> Show me with big pictures and little words without witholding key details. Please dont use weird metaphors.

(The spelling is the owner's, kept as written.)

In practice:

- Short plain words, short lines, and one idea per line.
- Lead with the decision: one yes/no question, then one line for what happens if yes and one for what happens
  if no. Both lines are about this decision, never the same text on every card.
- Put the evidence (rung, raising role, source, cited records, link) in a collapsed block under the decision.
- Leave out no detail the owner needs to decide. When a detail is not in the record, say "not recorded".
- No figures that the record does not hold. A cost or risk line says "not recorded" and adds only what the
  record shows.
- No metaphors or figures of speech, and none of Athena's internal words (rung names, role names, job, work
  item, proposal, blind spot, commit, path) where the owner reads the decision.
- A recommendation is given only when a record states one, with its reason, in one line. If none does, none is
  shown. That a change is above a role's limit says who decides, not whether it is good: say "you must decide".
- If a title is a problem and not a change, say so plainly instead of dressing it as a question about a change.

## Enforcing mechanism

The owner decision cards page (`decisions.html`, built by `cargo run -p xtask -- site`) is rendered by
`xtask/src/decisions.rs`. `Brief::check` refuses a card of more than 170 words, a card without exactly one
question, a card without a "Who decides:" line, and a card with an internal word above the details block;
titles are reworded to plain words first. Its tests check that the question, the "If yes" line and the "If no"
line differ between proposals and carry each proposal's own facts, that no card recommends, and that the
decision lines come before the evidence, which sits in a closed `<details>` block.

## Extending it

A new page or message meant for the owner reuses the rule above and adds a test that shows a breach failing.
