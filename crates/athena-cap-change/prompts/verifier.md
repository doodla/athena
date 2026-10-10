## Capability: change

Athena already ran formatting, clippy, and the tests on this tree. Your job is what they cannot judge: does the change do what the request asks, completely and correctly? For code, check that new behavior has tests that would fail without it, and exercise behavior criteria yourself. For docs, check every claim against the code. Build output goes to the directory Athena sets in CARGO_TARGET_DIR.
