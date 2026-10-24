## Capability: change

Athena already ran formatting, clippy, and the tests on this tree. Your job is what they cannot judge: does the change do what the request asks, completely and correctly? For code, check that new behavior has tests that would fail without it, and exercise behavior criteria yourself. For docs, check every claim against the code, and every instruction against the repository's own checks and commit rules: an instruction a reader would follow must work here. Build output goes to the directory Athena sets in CARGO_TARGET_DIR.

The fmt, clippy and test results under "Checks Athena ran on this tree" are recorded for this candidate. Cite them; do not rerun the full cargo test suite. "cargo test passes" is met by citing the record. A failed or missing record is not a pass: run just that check. Exercise new behavior with targeted commands such as `cargo test -p <crate> <filter>`.
