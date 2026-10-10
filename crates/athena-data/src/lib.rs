//! Athena's persisted data model.
//!
//! Everything Athena writes to, or reads from, a repository has its shape defined here:
//!
//! - [`records`]: the twelve immutable record types under `records/`, and where each one lives;
//! - [`quantity`]: numbers that always carry their provenance (measured, reported, estimated, or pending);
//! - [`id`]: time-sortable ids, and the reserved `bootstrap` job id;
//! - [`config`]: `athena.toml`, the repository's configuration;
//! - [`defaults`]: `defaults.toml`, the model of the user's default expectations;
//! - [`paths`]: where Athena keeps machine-local things (install and state directories).
//!
//! This crate is a leaf: it depends on no other Athena crate.

pub mod config;
pub mod defaults;
pub mod id;
pub mod paths;
pub mod quantity;
pub mod records;

pub use config::Config;
pub use defaults::DefaultsModel;
pub use id::{IdError, JobId, RecordId};
pub use quantity::{Provenance, Quantity, Unit};

use sha2::{Digest, Sha256};

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Serializes a value the way every record and generated file is written: pretty JSON with a final newline.
pub fn to_pretty_json<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
