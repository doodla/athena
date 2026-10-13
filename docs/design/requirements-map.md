# Requirements map

This map points from a sample of `BRIEF.md` requirements to the place in this repository that answers each
one. It is a reading aid, not a test: a row says where to look, not that the requirement is met in full.

**Line numbers.** Every `file:line` here refers to the file as it stands in the same tree as this map.
If either file changes, the numbers move, so re-check them against the quoted text.

**Coverage.** *Direct* means the cited lines state or enforce the requirement. *Partial* means they cover one
part of it. Requirements with no row are not claimed.

## Map

| Requirement | `BRIEF.md` | Evidence | Coverage |
| --- | --- | --- | --- |
| Restarting: "start the next Claude" session "with the prompt `continue` and nothing more" | `BRIEF.md:102` | The `athena restart` command is listed at `CHANGELOG.md:218`; the restart record and its fields are at `docs/records.md:47` and `docs/records.md:71` | Partial: the command and record exist; the prompt text itself is not cited here |
| Restarts: what a restart cost and what was lost is logged | `BRIEF.md:89` | `athena.restart/1` holds `cost`, `minutes`, `tokens` and `lost` (`docs/records.md:71`) | Direct |
| Rust, modular, dependency direction documented, no cycles | `BRIEF.md:187` | Crate table and the test that checks it against `cargo metadata` (`docs/architecture.md:37`, `docs/architecture.md:39`) | Direct |
| Secrets are scanned on every commit | `BRIEF.md:199` | Gate rule M1, run by the `pre-commit` hook (`docs/enforcement.md:15`) | Direct |
| "Nothing reaches the main branch, and nothing deploys, without passing verification." | `BRIEF.md:195` | Gate rule M2: admission scans the candidate before every plumbing commit (`docs/enforcement.md:16`) | Partial: covers the secret gate on commits, not deploys or every verification rule |
| S5: every claim cites its source, and a decision gets a rationale card | `BRIEF.md:320`, `BRIEF.md:321` | The study capability's validation rules: "eight evidence rules that no waiver clears" (`CHANGELOG.md:160-165`) | Partial: validation core only; it is not registered in the CLI |
| Adoption docs: a changelog with releases | `BRIEF.md:337` | `CHANGELOG.md:207` (the `## [0.1.0]` heading) | Partial: one release so far |
| Numbers are measured or labeled as estimates | `BRIEF.md:123` | Unmeasured claims say so, for example "The speedup is not measured." (`CHANGELOG.md:25`) | Partial: one example, not an audit |
