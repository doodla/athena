//! `defaults.toml`: Athena's explicit, versioned model of its user's default expectations (D26).

use serde::{Deserialize, Serialize};

/// The parsed defaults model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultsModel {
    pub version: u32,
    #[serde(rename = "default")]
    pub defaults: Vec<Default>,
}

/// One default expectation, with the source that justifies it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Default {
    pub id: String,
    pub statement: String,
    /// Classes of work it applies to; `*` means every class.
    pub applies_to: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DefaultsError {
    #[error("defaults.toml does not parse: {0}")]
    Parse(String),
    #[error("default id {0:?} appears more than once")]
    Duplicate(String),
    #[error("default {0:?} has an empty statement or source")]
    Empty(String),
}

impl DefaultsModel {
    pub fn parse(text: &str) -> Result<Self, DefaultsError> {
        let model: DefaultsModel =
            toml::from_str(text).map_err(|e| DefaultsError::Parse(e.to_string()))?;
        let mut seen = std::collections::BTreeSet::new();
        for d in &model.defaults {
            if !seen.insert(d.id.clone()) {
                return Err(DefaultsError::Duplicate(d.id.clone()));
            }
            if d.statement.trim().is_empty() || d.source.trim().is_empty() {
                return Err(DefaultsError::Empty(d.id.clone()));
            }
        }
        Ok(model)
    }

    pub fn get(&self, id: &str) -> Option<&Default> {
        self.defaults.iter().find(|d| d.id == id)
    }

    /// The compact text the reader loads: one line per default.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# The user's default expectations (version {})\n\n",
            self.version
        );
        for d in &self.defaults {
            out.push_str(&format!(
                "- {} [{}]: {}\n",
                d.id,
                d.applies_to.join(", "),
                d.statement
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO_DEFAULTS: &str = include_str!("../../../defaults.toml");

    #[test]
    fn repository_defaults_parse() {
        let model = DefaultsModel::parse(REPO_DEFAULTS).unwrap();
        assert!(model.version >= 1);
        assert!(model.get("strangers").is_some());
        assert!(model.render().contains("- rust ["));
    }

    #[test]
    fn duplicates_are_refused() {
        let text = r#"
version = 1
[[default]]
id = "a"
statement = "x"
applies_to = ["*"]
source = "s"
[[default]]
id = "a"
statement = "y"
applies_to = ["*"]
source = "s"
"#;
        assert_eq!(
            DefaultsModel::parse(text),
            Err(DefaultsError::Duplicate("a".into()))
        );
    }
}
