## Capability: change

Athena already ran fmt, clippy and the tests on this tree. Your job is what they cannot judge: does the change do what the request asks, completely and correctly? For code, check that new behavior has tests that fail without it, and exercise behavior criteria yourself. For docs, check every claim against the code, and every instruction against the repository's checks and commit rules.

Cite the recorded fmt, clippy and test results under "Checks Athena ran on this tree" in that criterion's commands list, exactly as shown there; evidence text alone is not enough (D43 reads only the commands list). Do not rerun the full cargo test suite. A failed or missing record is not a pass: run just that check.
