# Rendered-page assertions

## The rule

When a job's acceptance criteria are about what a generated study page contains, or must not contain, the
producer adds assertions on the rendered HTML. The HTML comes from `cargo run -p xtask -- site`, not from the
data files or the render function alone. The producer runs those assertions before handing the change over.
A failing assertion blocks handover: the producer fixes the page or the assertion, and does not report the
criterion as met.

## Why

Follow-up jobs on the firstmate-work-handling page were rejected one after another because the criteria
described the page, and nothing checked the built page against them before the verifier looked. The fault was
the missing check on the output, so the check now exists as a test.

## Enforcing mechanism

`xtask/tests/site_page.rs` builds the page by running `xtask site --out <temporary directory>` and checks
the HTML against `studies/firstmate-work-handling`:

- the word-defining entries ("Words used first") and the `#next-steps` link come before the first transition
  card, and the next-steps and glossary sections exist;
- the findings shown with the "what this page cannot show" label are exactly the `model-limit` findings in
  `findings.json`, and no other finding carries that label;
- every transition card says who decides its trigger, as the model's `trigger_evaluated_by` says. The
  expected value is read from `model.json`, and the study must contain at least one agent-decided transition.

`xtask/tests/fixtures/bad_firstmate_page.html` is a committed wrong page. The test
`the_wrong_fixture_page_fails_every_check` runs the same helpers on it and requires each check to report its
fault, so each check is shown able to fail.

## Extending it

A new page criterion adds a helper and a fixture fault to `xtask/tests/site_page.rs`. The scope today is the
firstmate-work-handling page only. The rule is carried to producers by `crates/athena-cap-change/prompts/producer.md`.
