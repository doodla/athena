//! The structured outputs of the three roles (design Appendix B).
//!
//! Each output is a Rust type. Its JSON Schema is generated from the type at run time, hardened (every
//! property required, no additional properties), and narrowed with the job's closed lists (classes,
//! capabilities, producer models). Claude Code validates the agent's output against it (`--json-schema`),
//! and Athena parses the result into the type again, with unknown fields denied.

use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// B.1: the reader's reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadingOutput {
    /// Imperative, at most 72 characters.
    pub title: String,
    /// One to three sentences: how the request was read.
    pub summary: String,
    pub class_of_work: String,
    /// A capability id from the catalog, or null with a decline reason.
    pub capability: Option<String>,
    pub decline_reason: Option<String>,
    pub producer_model: String,
    pub defaults_applied: Vec<DefaultAppliedOutput>,
    pub assumptions: Vec<String>,
    /// One question, only when the defaults do not settle an ambiguity; the reading still stands.
    pub question: Option<String>,
    pub acceptance_criteria: Vec<CriterionOutput>,
    pub deliverables: Vec<DeliverableOutput>,
    pub out_of_scope: Vec<String>,
    pub forecast: ForecastOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DefaultAppliedOutput {
    pub id: String,
    pub how: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriterionKindOutput {
    Content,
    Behavior,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CriterionOutput {
    pub text: String,
    pub kind: CriterionKindOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableActionOutput {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliverableOutput {
    /// Repository-relative path.
    pub path: String,
    pub action: DeliverableActionOutput,
    /// The probability that this path changes.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub p: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuantilesOutput {
    #[schemars(range(min = 0.0))]
    pub p10: f64,
    #[schemars(range(min = 0.0))]
    pub p50: f64,
    #[schemars(range(min = 0.0))]
    pub p90: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForecastOutput {
    #[schemars(range(min = 0.0, max = 1.0))]
    pub p_reading_accepted: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub p_verification_passes: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub p_user_intervenes: f64,
    /// Producer plus verifier, in US dollars.
    pub cost_usd: QuantilesOutput,
    /// From dispatch to the final verdict, in seconds.
    pub wall_time_s: QuantilesOutput,
    /// The request-specific reason for any departure from the reference forecast.
    pub departures: Option<String>,
}

/// B.2: the producer's report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerOutput {
    /// What changed and why, for the user.
    pub summary: String,
    pub files: Vec<String>,
    pub checks_run: Vec<CheckRunOutput>,
    pub open_issues: Vec<String>,
    /// Errors you noticed in your own work.
    pub mistakes: Vec<String>,
    /// Facts about the repository you had to discover.
    pub learnings: Vec<String>,
    /// Context or tools you wished you had.
    pub desires: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CheckResultOutput {
    Pass,
    Fail,
    NotRun,
}

impl CheckResultOutput {
    pub fn as_str(self) -> &'static str {
        match self {
            CheckResultOutput::Pass => "pass",
            CheckResultOutput::Fail => "fail",
            CheckResultOutput::NotRun => "not-run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckRunOutput {
    pub command: String,
    pub result: CheckResultOutput,
    pub note: Option<String>,
}

/// B.3: the verifier's verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerdictOutput {
    pub verdict: VerdictValue,
    /// One entry per acceptance criterion, in the order given.
    pub criteria: Vec<CriterionVerdictOutput>,
    pub findings: Vec<FindingOutput>,
    pub request_reading_mismatch: Option<String>,
    /// One paragraph for the user.
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerdictValue {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CriterionVerdictOutput {
    pub criterion: String,
    pub met: bool,
    pub evidence: String,
    /// The commands that exercised it, exactly as run; required for behavior criteria.
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeverityOutput {
    Blocker,
    Major,
    Minor,
    Nit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindingOutput {
    pub severity: SeverityOutput,
    pub summary: String,
    pub evidence: String,
    pub location: Option<String>,
}

/// The steward's report: the work items it found. Athena validates every one before it queues anything.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StewardOutput {
    /// Empty when there is nothing to report.
    pub items: Vec<StewardItemOutput>,
}

/// What a steward's work item is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StewardKindOutput {
    /// A source the steward could not read; `source` names it.
    BlindSpot,
    /// A failure that repeats; the evidence cites the jobs.
    RepeatedFailure,
    /// A change on a rung the steward may not change itself.
    Proposal,
}

/// A rung of the requirements ladder, highest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RungOutput {
    OwnerGoals,
    BriefRequirements,
    DesignDecisions,
    DefaultsConventions,
    Implementation,
}

/// What a reference points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RefKindOutput {
    Job,
    Path,
    Commit,
}

/// A pointer to where data lives: a job id, a path in the workspace, or a commit id. Never the data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StewardRefOutput {
    pub kind: RefKindOutput,
    pub value: String,
}

/// One work item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StewardItemOutput {
    pub kind: StewardKindOutput,
    /// Stable for the same finding, so it is not queued twice. Ignored for a blind spot, whose key is made
    /// from its source.
    pub key: String,
    /// One line.
    pub title: String,
    pub rung: RungOutput,
    /// Where the finding came from. For a blind spot, the name of the source that could not be read.
    pub source: StewardRefOutput,
    /// Required unless the item is a blind spot.
    pub evidence: Vec<StewardRefOutput>,
    /// What to ask Athena to do, when the rung is at or below your limit.
    pub request: Option<String>,
}

/// The closed lists that narrow the reader's schema.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReaderChoices {
    pub classes: Vec<String>,
    pub capabilities: Vec<String>,
    pub producer_models: Vec<String>,
}

/// The JSON Schema of `T`, inlined and hardened.
pub fn schema_of<T: JsonSchema>() -> Value {
    let settings = SchemaSettings::draft2020_12().with(|s| {
        s.inline_subschemas = true;
        s.meta_schema = None;
    });
    let mut value = settings
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value();
    harden(&mut value);
    value
}

/// Requires every property and forbids additional ones, at every level of a schema. Drops `format`
/// annotations (such as `double`), which some validators reject as unknown.
fn harden(schema: &mut Value) {
    let Value::Object(map) = schema else { return };
    map.remove("format");
    if let Some(Value::Object(props)) = map.get_mut("properties") {
        props.values_mut().for_each(harden);
        let keys: Vec<Value> = props.keys().cloned().map(Value::String).collect();
        map.insert("required".into(), Value::Array(keys));
        map.insert("additionalProperties".into(), Value::Bool(false));
    }
    for key in ["items", "anyOf", "oneOf", "allOf", "prefixItems"] {
        match map.get_mut(key) {
            Some(Value::Array(list)) => list.iter_mut().for_each(harden),
            Some(child @ Value::Object(_)) => harden(child),
            _ => {}
        }
    }
}

/// The reader's schema, narrowed to the job's closed lists.
pub fn reading_schema(choices: &ReaderChoices) -> Value {
    let mut schema = schema_of::<ReadingOutput>();
    let props = schema["properties"]
        .as_object_mut()
        .expect("the reading schema has properties");
    props.insert(
        "class_of_work".into(),
        json!({ "type": "string", "enum": choices.classes }),
    );
    let mut capabilities: Vec<Value> = choices
        .capabilities
        .iter()
        .cloned()
        .map(Value::String)
        .collect();
    capabilities.push(Value::Null);
    props.insert("capability".into(), json!({ "enum": capabilities }));
    props.insert(
        "producer_model".into(),
        json!({ "type": "string", "enum": choices.producer_models }),
    );
    schema
}

/// The producer's schema.
pub fn producer_schema() -> Value {
    schema_of::<ProducerOutput>()
}

/// The verifier's schema.
pub fn verdict_schema() -> Value {
    schema_of::<VerdictOutput>()
}

/// The steward's schema.
pub fn steward_schema() -> Value {
    schema_of::<StewardOutput>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choices() -> ReaderChoices {
        ReaderChoices {
            classes: vec!["docs".into(), "other".into()],
            capabilities: vec!["change".into()],
            producer_models: vec!["claude-haiku-5-5".into()],
        }
    }

    #[test]
    fn schemas_require_every_field_and_forbid_others() {
        let schema = reading_schema(&choices());
        assert_eq!(schema["additionalProperties"], json!(false));
        let required = schema["required"].as_array().unwrap();
        assert!(
            required.contains(&json!("question")),
            "optional fields are required, and nullable"
        );
        let forecast = &schema["properties"]["forecast"];
        assert_eq!(forecast["additionalProperties"], json!(false));
        assert!(
            schema.to_string().find("$ref").is_none(),
            "subschemas are inlined"
        );
    }

    #[test]
    fn the_reading_schema_is_narrowed_to_the_closed_lists() {
        let schema = reading_schema(&choices());
        assert_eq!(
            schema["properties"]["class_of_work"]["enum"],
            json!(["docs", "other"])
        );
        assert_eq!(
            schema["properties"]["capability"]["enum"],
            json!(["change", null])
        );
    }

    #[test]
    fn outputs_round_trip() {
        let verdict = VerdictOutput {
            verdict: VerdictValue::Pass,
            criteria: vec![CriterionVerdictOutput {
                criterion: "x".into(),
                met: true,
                evidence: "read it".into(),
                commands: vec![],
            }],
            findings: vec![],
            request_reading_mismatch: None,
            summary: "fine".into(),
        };
        let text = serde_json::to_string(&verdict).unwrap();
        assert_eq!(
            serde_json::from_str::<VerdictOutput>(&text).unwrap(),
            verdict
        );
        assert!(
            verdict_schema()["properties"]["verdict"]["enum"]
                .as_array()
                .is_some()
        );
        assert!(producer_schema()["properties"]["checks_run"].is_object());
    }

    #[test]
    fn the_steward_schema_requires_every_field_of_an_item() {
        let schema = steward_schema();
        assert_eq!(schema["required"], json!(["items"]));
        let item = &schema["properties"]["items"]["items"];
        assert_eq!(item["additionalProperties"], json!(false));
        let required = item["required"].as_array().unwrap();
        for field in [
            "kind", "key", "title", "rung", "source", "evidence", "request",
        ] {
            assert!(required.contains(&json!(field)), "{field}");
        }
        assert!(schema.to_string().find("$ref").is_none());
        let output: StewardOutput = serde_json::from_value(json!({ "items": [{
            "kind": "repeated-failure", "key": "k", "title": "t", "rung": "defaults-conventions",
            "source": { "kind": "path", "value": "records/jobs" },
            "evidence": [{ "kind": "job", "value": "20261010-153012-k7q2" }],
            "request": null
        }] }))
        .unwrap();
        assert_eq!(output.items[0].kind, StewardKindOutput::RepeatedFailure);
        assert_eq!(output.items[0].rung, RungOutput::DefaultsConventions);
    }
}
