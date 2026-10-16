//! The repository's own `athena.toml` and `defaults.toml` must parse with this crate's schema.
//!
//! This test is how a commit that changes the schema and the configuration together is judged by the
//! schema it brings: it runs in the candidate's own `cargo test`. The landing gate and admission skip their
//! own parse when a commit changes this crate (`athena_gate::defers_to_schema_test`) and rely on it.

use athena_data::{Config, DefaultsModel};
use std::path::Path;

fn repo_file(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn athena_toml_parses_with_this_crates_schema() {
    if let Err(e) = Config::parse(&repo_file("athena.toml")) {
        panic!("athena.toml does not parse with this crate's schema: {e}");
    }
}

#[test]
fn defaults_toml_parses_with_this_crates_schema() {
    if let Err(e) = DefaultsModel::parse(&repo_file("defaults.toml")) {
        panic!("defaults.toml does not parse with this crate's schema: {e}");
    }
}

#[test]
fn an_unknown_section_is_what_the_test_would_catch() {
    let text = format!("{}\n[bogus]\nfield = 1\n", repo_file("athena.toml"));
    assert!(Config::parse(&text).is_err());
}
