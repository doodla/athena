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
    /// Null unless the request makes Athena faster or cuts wasted time: failed or rejected attempts, planning
    /// stops, timeouts, retries that redo work, thrash, duplicated checks. Then one sentence naming which, and
    /// the request is urgent. A reason is required; an ordinary request is null.
    pub urgent_reason: Option<String>,
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
    /// Only the checks you actually ran.
    pub checks_run: Vec<CheckRunOutput>,
    /// One entry per accepted acceptance criterion: your own check of your work against it.
    pub criteria_check: Vec<CriterionCheckOutput>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CriterionStatusOutput {
    Met,
    Unmet,
}

impl CriterionStatusOutput {
    pub fn as_str(self) -> &'static str {
        match self {
            CriterionStatusOutput::Met => "met",
            CriterionStatusOutput::Unmet => "unmet",
        }
    }
}

/// The producer's self-check against one acceptance criterion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CriterionCheckOutput {
    /// The criterion's text: copy the line from the brief's criteria_check section.
    pub criterion: String,
    pub status: CriterionStatusOutput,
    /// What you read or ran to decide the status.
    pub evidence: String,
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

/// The audience's report: one rating and the findings behind it. Athena validates both before it records them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudienceOutput {
    /// 1 (the reader could not use it) to 5 (the reader got what they came for).
    pub rating: u8,
    /// Empty only when there is nothing to say.
    pub findings: Vec<AudienceFindingOutput>,
}

/// One finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudienceFindingOutput {
    /// One or two sentences.
    pub text: String,
    /// The evidence: a quoted passage, or the heading where it was found.
    pub source: String,
}

/// The auditor's report: whether the outcome serves the goals, and the findings behind that. Athena validates
/// both before it records them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditorOutput {
    /// One or two sentences: whether the outcome serves BRIEF.md and the owner's goals.
    pub summary: String,
    /// Empty only when there is nothing to say.
    pub findings: Vec<AuditorFindingOutput>,
}

/// One finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditorFindingOutput {
    /// One or two sentences.
    pub text: String,
    /// The evidence: a file and line, a command's output, or a quoted passage.
    pub evidence: String,
}

/// The planner's plan (job planning, `docs/design/job-planning.md` section 3). Athena sets whether a file is
/// governance and whether it exists; the planner does not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanOutput {
    /// How the change will be made, in a few sentences.
    pub approach: String,
    pub decisions: Vec<PlanDecisionOutput>,
    pub files: Vec<PlanFileOutput>,
    /// In the order the producer should do them.
    pub steps: Vec<String>,
    pub risks: Vec<PlanRiskOutput>,
    /// Exactly one per acceptance criterion.
    pub demonstrations: Vec<DemonstrationOutput>,
    pub open_questions: Vec<String>,
}

/// One design choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanDecisionOutput {
    pub choice: String,
    pub reason: String,
    pub rejected: Vec<RejectedOptionOutput>,
}

/// An option weighed and turned down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RejectedOptionOutput {
    pub option: String,
    pub why: String,
}

/// A file the plan touches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanFileOutput {
    /// Repository-relative path.
    pub path: String,
    pub action: DeliverableActionOutput,
    pub purpose: String,
}

/// A risk and what limits it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanRiskOutput {
    pub risk: String,
    pub mitigation: String,
}

/// How one acceptance criterion will be shown to hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DemonstrationOutput {
    /// The criterion's number in the brief, from 1.
    #[schemars(range(min = 1))]
    pub criterion: u32,
    pub method: String,
    /// The command or test that shows it, a command the verifier may run.
    pub command: String,
    /// What running it observes.
    pub observes: String,
    /// What would fail if the behavior were missing; required for a behavior criterion.
    pub fails_without: Option<String>,
}

/// The critic's verdict on a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CritiqueVerdictOutput {
    Accept,
    Revise,
    Reject,
}

/// The critic's review of one plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CriticOutput {
    pub verdict: CritiqueVerdictOutput,
    pub findings: Vec<CriticFindingOutput>,
    /// Set when the accepted reading misread the request, with how.
    pub request_reading_mismatch: Option<String>,
    /// One paragraph.
    pub summary: String,
}

/// One finding on a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CriticFindingOutput {
    pub severity: SeverityOutput,
    pub summary: String,
    /// A part of the plan or a file and line in the snapshot.
    pub location: String,
    pub evidence: String,
    /// What the planner should change.
    pub fix: String,
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

/// The audience's schema.
pub fn audience_schema() -> Value {
    schema_of::<AudienceOutput>()
}

/// The auditor's schema.
pub fn auditor_schema() -> Value {
    schema_of::<AuditorOutput>()
}

/// The planner's schema.
pub fn plan_schema() -> Value {
    schema_of::<PlanOutput>()
}

/// The critic's schema.
pub fn critic_schema() -> Value {
    schema_of::<CriticOutput>()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every object schema that lists properties, with the path to it.
    fn object_schemas<'a>(schema: &'a Value, path: &str, found: &mut Vec<(String, &'a Value)>) {
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            found.push((path.to_string(), schema));
            for (name, child) in props {
                object_schemas(child, &format!("{path}.{name}"), found);
            }
        }
        if let Some(items) = schema.get("items") {
            object_schemas(items, &format!("{path}[]"), found);
        }
    }

    fn sample_plan() -> Value {
        json!({
            "approach": "a", "steps": ["s"], "open_questions": [],
            "decisions": [{ "choice": "c", "reason": "r",
                "rejected": [{ "option": "o", "why": "w" }] }],
            "files": [{ "path": "p", "action": "modify", "purpose": "x" }],
            "risks": [{ "risk": "r", "mitigation": "m" }],
            "demonstrations": [{ "criterion": 1, "method": "m", "command": "c",
                "observes": "o", "fails_without": null }]
        })
    }

    fn sample_critique() -> Value {
        json!({
            "verdict": "revise", "request_reading_mismatch": null, "summary": "s",
            "findings": [{ "severity": "major", "summary": "s", "location": "l",
                "evidence": "e", "fix": "f" }]
        })
    }

    #[test]
    fn the_plan_and_critic_schemas_forbid_extra_properties_at_every_level() {
        for (schema, nested) in [
            (plan_schema(), 5), // top, decisions, rejected, files, risks, demonstrations: 6 objects
            (critic_schema(), 1), // top, findings: 2 objects
        ] {
            let mut found = Vec::new();
            object_schemas(&schema, "$", &mut found);
            assert_eq!(found.len(), nested + 1, "{found:?}");
            for (path, object) in found {
                assert_eq!(object["additionalProperties"], json!(false), "{path}");
                let required = object["required"].as_array().unwrap().len();
                assert_eq!(required, object["properties"].as_object().unwrap().len());
            }
            assert!(schema.to_string().find("$ref").is_none());
        }
    }

    #[test]
    fn the_plan_and_critic_outputs_reject_an_extra_property_at_the_top_and_in_a_nested_item() {
        let plan = sample_plan();
        assert!(serde_json::from_value::<PlanOutput>(plan.clone()).is_ok());
        let mut top = plan.clone();
        top["extra"] = json!(1);
        assert!(serde_json::from_value::<PlanOutput>(top).is_err());
        for (list, item) in [
            ("decisions", 0),
            ("files", 0),
            ("risks", 0),
            ("demonstrations", 0),
        ] {
            let mut nested = plan.clone();
            nested[list][item]["extra"] = json!(1);
            assert!(
                serde_json::from_value::<PlanOutput>(nested).is_err(),
                "{list}"
            );
        }
        let mut deep = plan;
        deep["decisions"][0]["rejected"][0]["extra"] = json!(1);
        assert!(serde_json::from_value::<PlanOutput>(deep).is_err());

        let critique = sample_critique();
        assert!(serde_json::from_value::<CriticOutput>(critique.clone()).is_ok());
        let mut top = critique.clone();
        top["extra"] = json!(1);
        assert!(serde_json::from_value::<CriticOutput>(top).is_err());
        let mut nested = critique;
        nested["findings"][0]["extra"] = json!(1);
        assert!(serde_json::from_value::<CriticOutput>(nested).is_err());
    }

    #[test]
    fn the_critic_verdict_is_exactly_accept_revise_or_reject() {
        let schema = critic_schema();
        assert_eq!(
            schema["properties"]["verdict"]["enum"],
            json!(["accept", "revise", "reject"])
        );
        for word in ["accept", "revise", "reject"] {
            let mut critique = sample_critique();
            critique["verdict"] = json!(word);
            assert!(
                serde_json::from_value::<CriticOutput>(critique).is_ok(),
                "{word}"
            );
        }
        for word in ["pass", "fail", "Accept", "approve", ""] {
            let mut critique = sample_critique();
            critique["verdict"] = json!(word);
            assert!(
                serde_json::from_value::<CriticOutput>(critique).is_err(),
                "{word}"
            );
        }
    }

    #[test]
    fn the_auditor_schema_requires_a_summary_and_findings_with_evidence() {
        let schema = auditor_schema();
        let required = |v: &Value| {
            let mut names: Vec<String> = v["required"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_str().unwrap().to_string())
                .collect();
            names.sort();
            names
        };
        assert_eq!(required(&schema), ["findings", "summary"]);
        assert_eq!(
            required(&schema["properties"]["findings"]["items"]),
            ["evidence", "text"]
        );
    }

    #[test]
    fn the_audience_schema_requires_a_rating_and_findings_with_sources() {
        let schema = audience_schema();
        let required = |v: &Value| {
            let mut names: Vec<String> = v["required"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_str().unwrap().to_string())
                .collect();
            names.sort();
            names
        };
        assert_eq!(required(&schema), ["findings", "rating"]);
        assert_eq!(
            required(&schema["properties"]["findings"]["items"]),
            ["source", "text"]
        );
    }

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
    fn the_reading_schema_requires_urgent_reason_and_holds_the_urgent_rule() {
        let schema = reading_schema(&choices());
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&json!("urgent_reason"))
        );
        let doc = schema["properties"]["urgent_reason"]["description"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for term in [
            "faster",
            "wasted time",
            "failed or rejected attempts",
            "planning stops",
            "timeouts",
            "retries",
            "thrash",
            "duplicated checks",
            "reason",
        ] {
            assert!(doc.contains(term), "{term}: {doc}");
        }
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
        assert!(producer_schema()["properties"]["criteria_check"].is_object());
        assert!(
            producer_schema()["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f == "criteria_check")
        );
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
