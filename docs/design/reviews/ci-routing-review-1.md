# Review 1 of `docs/design/ci-routing.md`: request

**Status:** requested, not yet done. This file is a review request written by the design's author. It is not a
verdict and must not be read as one. An independent reviewer replaces the "Verdict" section below.

**Reviewer independence:** the reviewer should form a verdict from the design and the source, not from the
author's summary, and label claims read, designed, unchecked or estimate, as `parallel-jobs-review-2.md` does.

## What to check

1. **Routing table (section 3.2).** Is each job kind's rule right, and is the verifier's command set
   (`athena.toml`, `verifier_commands`) really left alone, with Athena fetching the CI result?
2. **Cache keys (section 4).** Do the keys on `Cargo.lock` and `rust-toolchain.toml` invalidate correctly, and
   is writing only from the default branch sound? Check the platform's cache-sharing rules.
3. **One run per SHA (section 5).** Is there a path by which a result for a different SHA, or a result for a
   candidate that edited the workflow, is trusted? Is "no empty success" true in every row of 5.3?
4. **Verification strength (section 6).** Does the mapping miss a check that runs today?
5. **Parallel fit (section 8).** Does the concurrency group and cache write rule hold under
   `docs/design/parallel-jobs.md` with several jobs in flight?
6. **Measurement (section 9).** Can the baseline be computed from existing records (not checked by the author)?
   Is 20 jobs per side enough?
7. **Numbers.** The 25 minute split is the requester's figure, not measured; the sample size and any ceilings
   are estimates.

## Not checked by the author

Whether job records already hold per-phase timestamps; the platform's current cache retention and concurrency
limits; the size of `target/` in CI; the CI queue time for this repository.

## Verdict

Pending.
