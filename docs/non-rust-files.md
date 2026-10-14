# Non-Rust files

All of Athena's code is Rust. The rule is about code, not about file formats: data and documents (JSON,
TOML, YAML, Markdown, plain text, licenses, and the like) are allowed anywhere in the tree with no entry
here, including test fixtures.

This rule is enforced: `every_tracked_file_is_rust_or_data_or_an_allowed_exception` (xtask) fails on any
tracked file that is code in another language, found by either of:

- an extension such as `.py`, `.js`, `.ts`, `.sh`, `.rb`, `.go` (the full list is in `xtask/tests/policy.rs`);
- a `#!` first line, whatever the file is named.

`a_stray_script_is_flagged` shows the check catches a stray script and lets data and documents through.
Paths the repository owner keeps private (`athena-exclude`) are the owner's, not Athena's, and are not
checked.

## Allow-list

Non-Rust code is allowed only where it must exist. The same list is a named constant in
`xtask/tests/policy.rs`; the two are kept in step.

| File | Why it is not Rust |
|---|---|
| `setup.sh` | runs before any Rust toolchain exists: it installs a pinned rustup and passes control to Rust |
| `.github/workflows/ci.yml` | GitHub Actions reads only YAML workflows; it is a thin call into Rust |

## Notes

There is no `.gitleaks.toml`: secret-scan allow-list entries live in `athena.toml` and reach gitleaks
through a configuration Athena generates. There are no committed JSON Schemas: they are generated from the
Rust types at run time. The git hooks are symlinks to the `athena` binary, not scripts.
