# Job 20261114-024843-q95q: verify "Add accessible core/expert tracks to site; keep links alive; spec"

## Request (verbatim)

> try again on 20261030-060907-6pvy

## Reading (accepted by the user)

Retry of job 20261030-060907-6pvy: extend xtask/src/site.rs with skip link, CSS-only core/expert switch, learning path, reduced-motion and focus styles, but keep the claims list and glossary visible in every track so no in-page link is dead. Add a test that renders each track and fails on any dead or hidden anchor target, then record the mechanism and a `status: not shown` measure for OLY-B-D2-02 in docs/spec.toml.

Assumptions:
- Code survey: I read xtask/src/site.rs and crates/athena-core/src/status.rs. Entry point: site::render (site.rs:241) builds the header, then start_here, states, counters, transitions, rationales, glossary, findings, next_steps and claims_section, then finish (site.rs:628), which writes the about section, the footer and the html wrapper with the viewport meta and CSS. Citations come from cites() (site.rs:155) as `<a href="#claim-ID">`, and their targets are in claims_section. with_evidence (site.rs:889) later inserts the job-evidence section after `</header>`. decisions.rs imports site::CSS and builds its own wrapper. Validation: STATUS_MEASURE_LABELS (crates/athena-core/src/status.rs:312) is the allow-list. A measure must end with `status: <label>` or `status: not shown` naming a test; hestia::spec_gaps flags others (status.rs:520). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip tests are not touched. The property is measured directly by a unit test in xtask/src/site.rs that calls the real render() on the study fixture and inspects the HTML. It does not measure a stand-in.
- Prior job 20261030-060907-6pvy is addressed finding by finding. Major, dead claim links: the claims section and glossary stay in the core track (not marked expert), so every `#claim-` link resolves; the new test checks it. Minor, learning path matching only `<section><h2>`: build the list from all sections, including those with ids, e.g. by matching `<section` generally or by collecting section titles in one place. Minor, evidence section ahead of the reading nav: insert the nav so it comes before the evidence section, or have with_evidence insert after the nav. Minor, decisions page: either give decisions.rs the skip link and main id, or word the spec, design doc and changelog as the explainer page, with decisions getting only the shared CSS rules. The previous candidate's four changed files (xtask/src/site.rs, docs/spec.toml, CHANGELOG.md, docs/design/explainer-site.md) are all deliverables again. The retry starts from base, so that work is redone, not kept as is.
- The introspection's proposed request applies: no dead anchors in any track, and the header's claim that claims are listed at the end must hold. The new request text, `try again`, takes precedence for the spec/measure work. Both the spec work and the link-integrity test are in scope.
- Audience: the site's readers on mobile and with assistive tech, and the verifier checking the rendered HTML. Expert-only detail (marked class expert) may be hidden in the core track via CSS only, with no JavaScript.
- xtask has no library, so tests are unit tests in xtask/src/site.rs, run with `cargo test -p xtask`.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-D2-02 has a mechanism that names xtask/src/site.rs instead of "none", owner Athena, and a measure that ends `status: not shown` and names the test that shows it (a diff line in docs/spec.toml).
2. [content] xtask/src/site.rs page wrapper and CSS contain a skip link to the main content, a core/expert track control with expert content marked, a learning-path ordered list covering every section, including those that have an id, an `@media (prefers-reduced-motion:reduce)` rule and a `:focus-visible` style. The existing viewport meta and `prefers-color-scheme:dark` rules remain.
3. [content] The claims section and the glossary are not hidden by the track CSS. Only sections marked class expert are hidden in the core track, and the header sentence saying the claims are listed at the end stays true.
4. [behavior] A new unit test in xtask/src/site.rs renders the real page in each track (core default and expert ticked). It fails if any in-page `href="#..."` points to a missing id or to an element inside a hidden expert subtree, or if a section the header names is hidden. Run `cargo test -p xtask site`. It passes with the change and fails if claims_section is marked expert.
5. [behavior] A second assertion in that test checks each feature exists in the rendered HTML: skip link, track control, learning-path list, reduced-motion rule, focus-visible rule. Run `cargo test -p xtask site`. The existing site tests, including the job-evidence ordering test, still pass or are updated consistently.
6. [behavior] `cargo test` passes, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes (cite the commands that ran, or the Athena checks recorded on the final tree).
7. [content] CHANGELOG.md has a new entry for this change, and docs/design/explainer-site.md describes the tracks, the learning path and the rule that no track has dead links, without claiming features that the decisions page lacks.

## Expected deliverables

- xtask/src/site.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)
- xtask/src/decisions.rs (modify)

## Out of scope

- Client-side JavaScript or a build toolchain for the site
- Changing other spec requirements
- Editing docs/decisions.md or records/
- Changes to crates/ code, athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check rendered links and features, not only that files exist
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt to pass
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test covers every track and every anchor, not only the claim links that failed
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b09dada51b80 → candidate aa3f1c374d5c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                  |   1 +
 docs/design/explainer-site.md |  12 +++
 docs/spec.toml                |   4 +-
 xtask/src/decisions.rs        |   6 +-
 xtask/src/site.rs             | 209 ++++++++++++++++++++++++++++++++++++++++--
 5 files changed, 221 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-024843-q95q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-024843-q95q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43551058827 for aa3f1c374d5ce484f3ae0bae224a91dd71e4228d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43551058827 for aa3f1c374d5ce484f3ae0bae224a91dd71e4228d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43551058827 for aa3f1c374d5ce484f3ae0bae224a91dd71e4228d)
- ci-result: `not a check` passed (Check results for aa3f1c374d5ce484f3ae0bae224a91dd71e4228d come from the CI run https://github.com/doodla/athena/actions/runs/43551058827 (669s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes (cite the commands that ran, or the Athena checks recorded on the final tree).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
