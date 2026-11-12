//! OLY-B-SET-02: work uses Opus, Sonnet and Haiku, plus Jev for fast structured decisions.
//!
//! The roles come from the repository `athena.toml`, loaded through the real loader, whose validation refuses a
//! `[models]` section that lacks a family. Jev stays shadow-only; its endpoint and model are checked here.

use athena_core::jev;
use athena_data::Config;
use athena_data::config::ConfigError;
use std::path::Path;

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn the_repository_roles_cover_opus_sonnet_and_haiku() {
    let config = Config::parse(&repo_file("athena.toml")).expect("athena.toml loads");
    let m = &config.models;
    let roles: Vec<String> = [&m.reader, &m.producer_default, &m.verifier]
        .into_iter()
        .chain(m.producer_allowed.iter())
        .map(|name| name.to_lowercase())
        .collect();
    for family in ["opus", "sonnet", "haiku"] {
        assert!(
            roles.iter().any(|name| name.contains(family)),
            "no configured role names a {family} model: {roles:?}"
        );
    }
}

#[test]
fn a_copy_without_haiku_is_refused_by_the_loader() {
    let text = repo_file("athena.toml");
    let haiku = "\"claude-haiku-5-5\", ";
    assert!(
        text.contains(haiku),
        "athena.toml no longer lists the haiku id first"
    );
    let without = text.replacen(haiku, "", 1);
    assert_eq!(
        Config::parse(&without).unwrap_err(),
        ConfigError::ModelFamily("haiku")
    );
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let spec = repo_file("docs/spec.toml");
    let start = spec
        .find("id = \"OLY-B-SET-02\"")
        .expect("docs/spec.toml has OLY-B-SET-02");
    let rest = &spec[start..];
    let entry = &rest[..rest.find("[[requirement]]").unwrap_or(rest.len())];
    assert!(!entry.contains("mechanism = \"none\""), "{entry}");
    assert!(entry.contains("Config::validate"), "{entry}");
    assert!(entry.contains("owner = \"Athena\""), "{entry}");
    let measure = entry
        .lines()
        .find_map(|line| line.strip_prefix("measure = "))
        .expect("the entry has a measure line")
        .trim()
        .trim_end_matches('"');
    assert!(measure.ends_with("status: not shown"), "{measure}");
}

#[test]
fn jev_is_the_gateway_model() {
    assert!(
        jev::ENDPOINT.contains("ai-gateway.vercel.sh"),
        "{}",
        jev::ENDPOINT
    );
    assert_eq!(jev::MODEL, "typesafe-ai/jev");
}
