//! The design-surface record: what Athena's public surface held at one commit.

use super::*;

/// The schema tag of a design-surface record.
pub const DESIGN_SURFACE_SCHEMA: &str = "athena.design-surface/1";

/// The inventory of Athena's design surface at one commit, computed by code from the files at that commit.
///
/// Each kind maps an item's name to a short detail. Two inventories are compared kind by kind: a name in
/// only one is added or removed, and a name in both with a different detail is changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceInventory {
    /// Every command, such as `ask` or `hestia sweep`, with `visible` or `hidden`.
    pub commands: BTreeMap<String, String>,
    /// Every flag and positional argument, keyed `<command> --flag` or `<command> <arg>`, with its type.
    pub flags: BTreeMap<String, String>,
    /// Every `athena.toml` section and key the config schema accepts, keyed by dotted path, with its type
    /// (`section` for a table).
    pub config_keys: BTreeMap<String, String>,
    /// Every record schema by name without its version (`athena.candidate`), with its versions (`1,2`).
    pub record_schemas: BTreeMap<String, String>,
    /// Every role definition by name, with its version.
    pub roles: BTreeMap<String, String>,
    /// Every workspace crate by package name, with its directory.
    pub crates: BTreeMap<String, String>,
}

impl SurfaceInventory {
    /// The number of items over all kinds.
    pub fn len(&self) -> usize {
        self.commands.len()
            + self.flags.len()
            + self.config_keys.len()
            + self.record_schemas.len()
            + self.roles.len()
            + self.crates.len()
    }

    /// True when no kind holds an item.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// `athena.design-surface/1`: the inventory of the design surface at `commit`, so a later review can compare
/// against it. Records are never edited; the latest by `(at, id)` is the newest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignSurface {
    pub id: RecordId,
    pub at: Timestamp,
    /// The commit the inventory was computed at (40 hexadecimal characters).
    pub commit: String,
    pub inventory: SurfaceInventory,
}

impl DesignSurface {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.commit.len() != 40 || !self.commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("a design surface names its commit in full".into());
        }
        if self.inventory.is_empty() {
            return Err("a design surface lists at least one item".into());
        }
        Ok(())
    }

    /// The most recent of `surfaces`, by time and then id; None when there are none.
    pub fn latest(surfaces: &[DesignSurface]) -> Option<&DesignSurface> {
        surfaces
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }
}
