//! OLY-B-S3-01: the lab run configuration.
//!
//! All three systems are run for real within spend caps on purpose-built tasks, and every run is observable
//! and reproducible from a recorded configuration. A [`LabConfig`] is that recorded configuration: the task, a
//! pinned commit, and a positive spend cap in USD for each of the three systems. [`LabConfig::parse`] refuses
//! anything incomplete, [`LabConfig::check_run`] refuses a run whose recorded spend has reached its cap, and
//! [`LabConfig::fingerprint`] names the configuration so a run can be tied to the exact settings it used.
//!
//! The module runs nothing and spends nothing; a caller supplies the TOML text and the recorded spend.
//!
//! ```toml
//! task = "purpose-built task name"
//! commit = "0123456789abcdef0123456789abcdef01234567"
//! [caps_usd]
//! athena = 5.0
//! firstmate = 5.0
//! hermes = 5.0
//! ```

use std::collections::BTreeMap;
use thiserror::Error;
use toml_edit::DocumentMut;

/// The three systems every lab configuration must cap.
pub const SYSTEMS: [&str; 3] = ["athena", "firstmate", "hermes"];

/// Why a lab configuration or a run was refused.
#[derive(Debug, Error, PartialEq)]
pub enum LabError {
    #[error("the lab configuration is not valid TOML: {0}")]
    Syntax(String),
    #[error("the lab configuration has no task")]
    NoTask,
    #[error("the lab configuration has no pinned commit (40 hex characters)")]
    NoCommit,
    #[error("the lab configuration has no spend cap for system {0:?}")]
    MissingSystem(&'static str),
    #[error("the spend cap for system {0:?} must be a positive number of USD")]
    BadCap(String),
    #[error("the lab configuration names {0:?}, which is not one of the three systems")]
    UnknownSystem(String),
    #[error(
        "system {system:?} has spent ${spent}, which reaches its cap of ${cap}; the run is refused"
    )]
    CapSpent {
        system: String,
        spent: f64,
        cap: f64,
    },
}

/// A recorded lab configuration, valid by construction.
#[derive(Debug, Clone, PartialEq)]
pub struct LabConfig {
    pub task: String,
    pub commit: String,
    /// Spend cap in USD per system.
    pub caps_usd: BTreeMap<String, f64>,
}

impl LabConfig {
    /// Parses and validates a lab configuration.
    pub fn parse(text: &str) -> Result<LabConfig, LabError> {
        let doc: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| LabError::Syntax(e.to_string()))?;
        let task = doc
            .get("task")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or(LabError::NoTask)?
            .to_string();
        let commit = doc
            .get("commit")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|c| c.len() == 40 && c.chars().all(|ch| ch.is_ascii_hexdigit()))
            .ok_or(LabError::NoCommit)?
            .to_ascii_lowercase();
        let table = doc.get("caps_usd").and_then(|v| v.as_table());
        let mut caps_usd = BTreeMap::new();
        if let Some(table) = table {
            for (name, item) in table.iter() {
                if !SYSTEMS.contains(&name) {
                    return Err(LabError::UnknownSystem(name.to_string()));
                }
                let cap = item
                    .as_float()
                    .or_else(|| item.as_integer().map(|i| i as f64))
                    .filter(|c| c.is_finite() && *c > 0.0)
                    .ok_or_else(|| LabError::BadCap(name.to_string()))?;
                caps_usd.insert(name.to_string(), cap);
            }
        }
        for system in SYSTEMS {
            if !caps_usd.contains_key(system) {
                return Err(LabError::MissingSystem(system));
            }
        }
        Ok(LabConfig {
            task,
            commit,
            caps_usd,
        })
    }

    /// Allows a run of `system` only while its recorded spend is below its cap.
    pub fn check_run(&self, system: &str, spent_usd: f64) -> Result<(), LabError> {
        let cap = *self
            .caps_usd
            .get(system)
            .ok_or_else(|| LabError::UnknownSystem(system.to_string()))?;
        if spent_usd >= cap {
            return Err(LabError::CapSpent {
                system: system.to_string(),
                spent: spent_usd,
                cap,
            });
        }
        Ok(())
    }

    /// A stable 64-bit FNV-1a fingerprint of the task, commit and caps, as 16 hex digits. The same
    /// configuration always gives the same value; changing any field changes it.
    pub fn fingerprint(&self) -> String {
        let mut canonical = format!("task={}\ncommit={}\n", self.task, self.commit);
        for (system, cap) in &self.caps_usd {
            canonical.push_str(&format!("cap.{system}={cap:?}\n"));
        }
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in canonical.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        format!("{hash:016x}")
    }
}
