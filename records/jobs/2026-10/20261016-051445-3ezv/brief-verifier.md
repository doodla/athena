# Job 20261016-051445-3ezv: verify "Design CI compute routing with cache reuse and wall-time measurement"

## Request (verbatim)

> Design moving compute to CI with intelligent routing, as the 2026-10-16 BRIEF.md addendum asks, planned through the new planning stage first. A job takes a median 25 min dispatch to outcome (producer ~12, verifier ~6, checks ~6 min), mostly cold cargo builds repeated per job. Decide per job what must run locally versus in CI, keep caches warm, avoid duplicate test runs, keep verification as strong, say how it fits with running jobs in parallel, and say how before/after wall time per verified result will be measured. Design only; get it reviewed.

## Reading (accepted by the user)

Write a design doc, docs/design/ci-routing.md, for moving compute to CI with per-job local/CI routing, warm caches, no duplicate tests, equal verification strength, fit with parallel jobs, and a before/after wall-time-per-verified-result measure. It is for the owner and design reviewers, follows the planning-first addendum in BRIEF.md (2026-10-16), and is design only.

Assumptions:
- The 'new planning stage' is the planning-first ordering in the 2026-10-16 BRIEF.md addendum; the doc records planning notes before the design body.
- Design only, so the review is requested/recorded, not implemented as code.
- CI means GitHub Actions, given .github/ exists.
- The 25 min breakdown is the user's figure, not measured by me.

## Acceptance criteria

1. [content] docs/design/ci-routing.md states the goal (cut the ~25 min median dispatch-to-outcome, mostly repeated cold cargo builds) and its audience, and cites the producer ~12, verifier ~6, checks ~6 min figures as given by the request (labeled as such, not re-measured).
2. [content] A routing table decides per job kind (code, tests, docs, design, config, build, study) what runs locally and what in CI, with the rule and reason for each; the verifier's allowed command set is addressed.
3. [content] Cache strategy is specified: what is cached (cargo registry, target dir, toolchain from rust-toolchain.toml), keys, invalidation on Cargo.lock change, and how cold-start cost is bounded.
4. [content] Duplicate-test avoidance is specified: one run of cargo test/clippy/fmt per commit SHA, with the verifier reusing the CI result keyed by SHA instead of rerunning, and how a stale or missing result is handled.
5. [content] A section shows verification is not weakened: every check now run locally still runs somewhere, with a mapping and the failure behavior when CI is unavailable or red (no empty success).
6. [content] A section explains the fit with parallel job execution (docs/design/parallel-jobs.md): shared caches, concurrency and cost limits, and interaction with the planning-first ordering.
7. [content] A measurement section defines wall time per verified result, with before and after baselines, data source, sample size, and how a regression is detected.
8. [content] Alternatives considered are listed with rejection reasons, per the designed default, and the design follows the existing docs/design format (compare parallel-jobs.md).
9. [content] The doc says it was planned through the planning stage first, and records a review (a reviews/ file or an explicit review request) so the design is reviewed before any build.
10. [behavior] Only docs change: no athena.toml, defaults.toml, workflow, or Rust files are modified; cargo fmt --check and cargo test still pass unchanged.

## Expected deliverables

- docs/design/ci-routing.md (create)
- docs/design/roadmap.md (modify)
- docs/design/parallel-jobs.md (modify)
- docs/design/reviews/ci-routing-review-1.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing CI workflows or changing .github/workflows
- Changing athena.toml, defaults.toml, or any Rust code
- Implementing the planning stage itself or the parallel initiative driver
- Changing existing landing flows

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc with alternatives, reviewed before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers are cited to the request or labeled as estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design doc, no implementation.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Measurement of wall time per verified result is the outcome check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check routing, caching, and dedupe decisions, not just file existence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo fmt --check and test to confirm nothing else changed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 68eef8d55422 → candidate 320cc38df390. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                               |   5 +
 docs/design/ci-routing.md                  | 368 +++++++++++++++++++++++++++++
 docs/design/parallel-jobs.md               |   7 +
 docs/design/reviews/ci-routing-review-1.md |  32 +++
 docs/design/roadmap.md                     |   4 +
 5 files changed, 416 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (734 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
