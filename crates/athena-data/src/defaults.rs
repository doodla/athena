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

    /// The compact text the reader loads: one line per default, showing only the first
    /// sentence of each statement. Producer and verifier briefs use the full statement
    /// through [`DefaultsModel::get`].
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
                first_sentence(&d.statement)
            ));
        }
        out
    }
}

/// The text up to and including the first sentence end: a period followed by whitespace,
/// or the final period. Statements without such a break are returned whole.
fn first_sentence(statement: &str) -> &str {
    let text = statement.trim();
    match text.find(". ") {
        Some(i) => &text[..=i],
        None => text,
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
    fn render_shows_only_the_first_sentence() {
        let text = r#"
version = 1
[[default]]
id = "multi"
statement = "First rule. Second detail. Third detail."
applies_to = ["*"]
source = "s"
[[default]]
id = "semi"
statement = "One rule; with a clause, and another clause."
applies_to = ["code"]
source = "s"
"#;
        let model = DefaultsModel::parse(text).unwrap();
        let out = model.render();
        assert!(out.contains("- multi [*]: First rule.\n"));
        assert!(!out.contains("Second detail"));
        assert!(out.contains("- semi [code]: One rule; with a clause, and another clause.\n"));
        assert!(
            model
                .get("multi")
                .unwrap()
                .statement
                .contains("Third detail")
        );
    }

    #[test]
    fn behavior_evidence_default_leads_with_the_core_rule() {
        let model = DefaultsModel::parse(REPO_DEFAULTS).unwrap();
        let entry = model.get("behavior-evidence").unwrap();
        assert_eq!(entry.applies_to, vec!["*".to_string()]);
        assert!(model.render().contains(
            "- behavior-evidence [*]: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist.\n"
        ));
    }

    #[test]
    fn root_cause_default_leads_with_the_core_rule() {
        let model = DefaultsModel::parse(REPO_DEFAULTS).unwrap();
        assert!(model.get("root-cause").unwrap().statement.contains(". "));
        assert!(model.render().contains(
            "- root-cause [*]: Fix the root cause, not the symptom or the one instance that surfaced.\n"
        ));
    }

    #[test]
    fn an_explainer_page_request_gets_the_three_standing_defaults() {
        // An explainer page is class docs: select the defaults for docs the way the reader sees them.
        let model = DefaultsModel::parse(REPO_DEFAULTS).unwrap();
        let for_docs = DefaultsModel {
            version: model.version,
            defaults: model
                .defaults
                .iter()
                .filter(|d| d.applies_to.iter().any(|c| c == "*" || c == "docs"))
                .cloned()
                .collect(),
        };
        let out = for_docs.render();
        for line in [
            "- outcome-check [*]: The outcome is the check behind the work, not the activity done.\n",
            "- no-empty-success [*]: An empty queue or finished list never counts as success on its own.\n",
            "- thrash-stop [*]: If the same item fails twice in a loop, the loop stops and reports thrashing.\n",
        ] {
            assert!(out.contains(line), "missing: {line}");
        }
        // A code-only default is not selected for docs, so the selection is real.
        assert!(!out.contains("- rust ["));
        let outcome = &model.get("outcome-check").unwrap().statement;
        assert!(outcome.contains("fresh model reader that took no part in the work"));
        assert!(outcome.contains("below 4 of 5 blocks completion"));
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
