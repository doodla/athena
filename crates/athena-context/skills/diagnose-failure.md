# Skill: diagnose-failure (version 1)

Method for diagnosing a rejected or failed job. Follow the steps in order and keep the order in your answer.

1. Read the stored failures first. List each unmet criterion, each blocker or major finding, and each failed check. For a failed check, read the end of its output (the tail): the cause is usually in the last lines, not the first. Keep criteria and checks apart; a failed check is not an unmet criterion.
2. Say what happened in one plain sentence. This is the event. Do not give it as the cause.
3. State the cause as something other than a restatement of the event. Ask why the system allowed it: which step, rule or piece of knowledge was missing or wrong. If your cause could be swapped for the event sentence without losing anything, rewrite it.
4. Compare with earlier cases. Check whether a listed earlier case has the same cause. If so, reuse its class and cite its id; if not, say what differs.
5. Name the evidence. For each claim in the cause, cite the stored field, finding, check name or case id it rests on. Use only ids you were given.
6. When evidence is missing (an empty tail, no verdict, no matching case), say so in the answer. State what is missing and what is still supported, mark the cause as a best reading, and make the prevention a way to record the missing evidence. Never invent a detail to fill the gap.
7. Propose a prevention that would have stopped the whole class, not only this instance.

Example (labeled, for illustration only): event "the test run failed"; weak cause "the tests failed"; better cause "nothing ran the new tests against the changed source before delivery, so a broken import was found only at the final check", citing the failed check's tail.
