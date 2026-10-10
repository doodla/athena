# Non-Rust files

All of Athena's code is Rust. Every other kind of file exists because a platform or a format requires it.
This list is enforced: `every_tracked_file_is_rust_or_a_listed_kind` (xtask) fails on any tracked file
that matches no row. Paths the repository owner keeps private (`athena-exclude`) are the owner's, not
Athena's, and are not checked.

| Files | Why they are not Rust |
|---|---|
| `*.rs` | Rust itself |
| `Cargo.toml`, `Cargo.lock` | Cargo's manifest and lockfile formats |
| `rust-toolchain.toml` | rustup reads the toolchain pin from this file |
| `setup.sh` | runs before any Rust toolchain exists: it installs a pinned rustup and hands off to Rust |
| `.github/workflows/ci.yml` | GitHub Actions reads only YAML workflows; it is a thin call into Rust |
| `.gitignore`, `.gitattributes` | git's own formats |
| `athena.toml`, `defaults.toml` | configuration and the defaults model: data, not code |
| `records/**` | machine-written JSON records, plus verbatim inputs such as the bootstrap brief |
| `*.md` | documentation, and prompt text compiled into the binary (prose, not code) |
| `LICENSE` | license text |

There is no `.gitleaks.toml`: secret-scan allow-list entries live in `athena.toml` and reach gitleaks
through a configuration Athena generates. There are no committed JSON Schemas: they are generated from the
Rust types at run time. The git hooks are symlinks to the `athena` binary, not scripts.
