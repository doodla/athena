//! Introspection (`docs/design/defaults-loop.md` section 3, build order step 1), on fixture records in
//! temporary directories: each trigger, class matching and new-class creation, id dropping, a cause that
//! restates the event, a failed introspection, and the output of `athena status <job>`. No model is called:
//! a scripted fake answers the introspection prompt.

mod common;

use athena_agent::fake::{FakeTurn, Script};
use athena_core::introspect::{CASE_CAP, CaseRow, Event, Records as Past, narrow_cases};
use athena_core::introspect::{
    JEV_CLASS_KEY, JEV_CONTEXT_KEY, JEV_CONTEXT_QUESTION, JEV_NEW_OPTION, JEV_REPEAT_KEY, ask_jev,
};
use athena_core::jev::{Answer, Evaluation, FakeJev, JevError, Question, Usage};
use athena_core::manage::InterveneArgs;
use athena_core::run::GoResult;
use athena_core::store::Records;
use athena_core::{Athena, Ui};
use athena_data::records::{
    AcceptedBy, AnyRecord, ClassStatus, IntrospectionRecord, IntrospectionStatus,
    IntrospectionTrigger, InvocationOutcome, OutcomeState,
};
use athena_data::{JobId, RecordId};
use common::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// The first words of the introspection's system prompt: how the script tells it from the other roles.
const MARKER: &str = "You write one introspection";

type Reply = Box<dyn Fn(&Value) -> Value + Send>;

/// The scripted model: replies in order, and every prompt it was given.
#[derive(Clone, Default)]
struct Model {
    replies: Arc<Mutex<VecDeque<Reply>>>,
    prompts: Arc<Mutex<Vec<Value>>>,
}

impl Model {
    fn say(&self, reply: Value) {
        self.replies
            .lock()
            .unwrap()
            .push_back(Box::new(move |_| reply.clone()));
    }
    fn say_with(&self, reply: impl Fn(&Value) -> Value + Send + 'static) {
        self.replies.lock().unwrap().push_back(Box::new(reply));
    }
    fn script(&self, inner: Script) -> Script {
        let model = self.clone();
        Box::new(move |spec| {
            if !spec.system_prompt.starts_with(MARKER) {
                return inner(spec);
            }
            let prompt: Value = serde_json::from_str(&spec.user_prompt).unwrap();
            model.prompts.lock().unwrap().push(prompt.clone());
            match model.replies.lock().unwrap().pop_front() {
                Some(reply) => {
                    let mut turn = FakeTurn::output(reply(&prompt));
                    turn.cost_usd = 0.002;
                    turn
                }
                None => FakeTurn {
                    outcome: InvocationOutcome::Error,
                    detail: Some("model unavailable".into()),
                    ..FakeTurn::output(Value::Null)
                },
            }
        })
    }
}

fn reply(class: &str, what: &str, cause: &str) -> Value {
    json!({
        "what_happened": what,
        "failure_class": class,
        "new_class": if class == "new" {
            json!({ "name": "check removes a file the repository needs", "description": "A change deleted a file that a check requires." })
        } else {
            Value::Null
        },
        "earlier_cases": [],
        "decisions_matched": [],
        "cause": cause,
        "prevention_kind": "code-check",
        "prevention": "Refuse a candidate that deletes a file a check reads, before the checks run.",
        "proposed_request": "Add a greeting to README.md without deleting any file."
    })
}

fn rejecting() -> Scenario {
    Scenario {
        edits: vec![("README.md".into(), None)],
        ..Scenario::default()
    }
}

fn setup(scenario: Scenario, model: &Model) -> (Fixture, Athena) {
    let fx = Fixture::new();
    let athena = fx.athena(model.script(scenario.script()));
    (fx, athena)
}

fn run_job(athena: &Athena, text: &str) -> athena_core::run::Finished {
    let proposal = athena.ask(text, &Lines::default()).unwrap();
    match athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
        .unwrap()
    {
        GoResult::Finished(f) => *f,
        other => panic!("expected a finished job, got {other:?}"),
    }
}

fn introspections(fx: &Fixture) -> Vec<IntrospectionRecord> {
    Records::load(&fx.git, &fx.head()).unwrap().introspections
}

fn intervene(athena: &Athena, what: &str) -> RecordId {
    athena
        .intervene(
            &InterveneArgs {
                what: what.into(),
                why: "The greeting was wrong.".into(),
                class: "docs".into(),
                category: "other".into(),
                job: None,
                with_staged: false,
            },
            &Lines::default(),
        )
        .unwrap()
        .intervention
        .id
}

fn log(athena: &Athena, what: &str, class: &str, category: &str) -> RecordId {
    athena
        .intervene(
            &InterveneArgs {
                what: what.into(),
                why: "It went wrong.".into(),
                class: class.into(),
                category: category.into(),
                job: None,
                with_staged: false,
            },
            &Lines::default(),
        )
        .unwrap()
        .intervention
        .id
}

/// The model answers with a registry class (the first it is shown, or `new`) and the given cause.
fn say_cause(model: &Model, cause: &'static str, prevention_kind: &'static str) {
    model.say_with(move |prompt| {
        let class = prompt["class_registry"][0]["id"]
            .as_str()
            .unwrap_or("new")
            .to_string();
        let mut answer = reply(&class, "A step was skipped by the builder.", cause);
        answer["prevention_kind"] = json!(prevention_kind);
        answer["prevention"] = json!(format!("Prevention for: {cause}"));
        answer
    });
}

#[test]
fn a_mistake_intervention_is_introspected_with_guidance_for_the_preventing_check_or_knowledge() {
    let model = Model::default();
    say_cause(&model, "Nothing runs the docs test.", "code-check");
    say_cause(&model, "The brief never says which files.", "default");
    say_cause(&model, "Nothing runs the docs test.", "code-check");
    let (fx, athena) = setup(Scenario::default(), &model);
    // `intervene` introspects what it records.
    log(&athena, "Shipped without the docs test", "docs", "mistake");
    log(&athena, "Edited the README", "docs", "other");
    log(&athena, "Mistake: forgot the changelog", "docs", "other");
    let records = introspections(&fx);
    assert_eq!(records.len(), 3);
    for record in &records {
        assert_eq!(
            record.status,
            IntrospectionStatus::Recorded,
            "{:?}",
            record.failure
        );
    }

    let prompts = model.prompts.lock().unwrap();
    assert_eq!(prompts.len(), 3);
    let guidance = prompts[0]["mistake_guidance"].as_str().expect("guidance");
    assert!(guidance.contains("check"), "{guidance}");
    assert!(guidance.contains("knowledge"), "{guidance}");
    assert!(
        prompts[1].get("mistake_guidance").is_none(),
        "{}",
        prompts[1]
    );
    assert!(
        prompts[2]["mistake_guidance"].is_string(),
        "the legacy Mistake: record gets it too"
    );

    // The proposal names the preventing check and the cause.
    let report = athena.mistakes().unwrap();
    assert_eq!(report.total, 2, "the ordinary record is not a mistake");
    let [shared, ..] = report.proposals.as_slice() else {
        panic!("a proposal, got {:?}", report.proposals)
    };
    assert_eq!(shared.kind, "check");
    assert!(
        shared.prevention.contains("Nothing runs the docs test."),
        "{shared:?}"
    );
}

#[test]
fn mistakes_with_the_same_cause_aggregate_into_one_proposal() {
    let model = Model::default();
    say_cause(&model, "Nothing runs the docs test.", "code-check");
    say_cause(&model, "nothing runs the docs test", "code-check");
    say_cause(&model, "The brief never says which files.", "default");
    let (fx, athena) = setup(Scenario::default(), &model);
    for (what, class) in [
        ("Shipped without the docs test", "docs"),
        ("Shipped again without the docs test", "code"),
        ("Edited the wrong file", "code"),
    ] {
        log(&athena, what, class, "mistake");
    }
    let report = athena.mistakes().unwrap();
    assert_eq!(report.total, 3);
    assert_eq!(report.proposals.len(), 2, "{:?}", report.proposals);
    let shared = report.proposals.iter().find(|p| p.mistakes == 2).unwrap();
    assert_eq!(shared.introspections.len(), 2);
    assert_eq!(report.by_cause["Nothing runs the docs test."], 2);
    assert_eq!(report.by_cause["The brief never says which files."], 1);

    // `athena status` shows the recurring cause and the prevention proposed for it, in text and in JSON.
    let out = athena_bin(&fx).arg("status").output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    for needle in [
        "mistakes 3: by class: code 2, docs 1",
        "Nothing runs the docs test. 2",
        "prevention (check, 2 mistake(s)): Prevention for: Nothing runs the docs test.",
        "prevention (knowledge, 1 mistake(s))",
    ] {
        assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
    }
    let out = athena_bin(&fx).args(["status", "--json"]).output().unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["mistakes"]["by_class"]["code"], 2);
    assert_eq!(
        json["mistakes"]["by_cause"]["Nothing runs the docs test."],
        2
    );
    assert_eq!(json["interventions_by_category"], json!({ "mistake": 3 }));
}

#[test]
fn athena_audit_prints_mistakes_before_any_audit() {
    // No job exists, so no audit can be produced; the command still reports the mistakes per class and cause.
    let model = Model::default();
    say_cause(&model, "Nothing runs the docs test.", "code-check");
    let (fx, athena) = setup(Scenario::default(), &model);
    log(&athena, "Shipped without the docs test", "docs", "mistake");
    // The model has no second reply, so this legacy mistake's introspection fails: no cause is recorded.
    log(&athena, "Mistake: a legacy one", "code", "other");

    let out = athena_bin(&fx).arg("audit").output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    for needle in [
        "mistakes 2: by class: code 1, docs 1",
        "mistakes by cause: Nothing runs the docs test. 1, no cause recorded 1",
        "prevention (check, 1 mistake(s)): Prevention for: Nothing runs the docs test.",
    ] {
        assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
    }

    let out = athena_bin(&fx).args(["audit", "--json"]).output().unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["mistakes"]["total"], 2);
    assert_eq!(json["mistakes"]["by_class"]["docs"], 1);
    assert_eq!(json["mistakes"]["by_cause"]["no cause recorded"], 1);
}

fn athena_bin(fx: &Fixture) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_athena"));
    cmd.env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .env("ATHENA_STATE_DIR", fx.state.path())
        .current_dir(fx.root());
    cmd
}

#[test]
fn the_status_command_lists_an_uncited_intervention_and_drops_it_once_cited() {
    let (fx, athena) = setup(Scenario::default(), &Model::default());
    let status_text = |fx: &Fixture| {
        let out = athena_bin(fx).arg("status").output().unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    assert!(!status_text(&fx).contains("follow-up"));
    let id = intervene(&athena, "Edited the README by hand").to_string();

    let text = status_text(&fx);
    for needle in [
        "follow-up",
        id.as_str(),
        "[other]",
        "athena ask \"<request that addresses intervention",
    ] {
        assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
    }
    let out = athena_bin(&fx).args(["status", "--json"]).output().unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["open_interventions"][0]["id"], id.as_str());
    assert_eq!(json["interventions_by_category"]["other"], 1);

    athena
        .ask(&format!("address intervention {id}"), &Lines::default())
        .unwrap();
    let text = status_text(&fx);
    assert!(!text.contains("follow-up"), "{text}");
    let out = athena_bin(&fx).args(["status", "--json"]).output().unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["open_interventions"], json!([]));
}

#[test]
fn a_rejected_job_is_introspected_and_the_class_is_proposed_by_code() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md, so the test check failed and the job was rejected.",
        "Nothing stops a candidate from deleting a file the checks need before the checks run.",
    ));
    let (fx, athena) = setup(rejecting(), &model);
    let base = fx.head();
    let finished = run_job(&athena, "remove the readme");
    assert_eq!(finished.state, OutcomeState::Rejected);

    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let [record] = records.introspections.as_slice() else {
        panic!("one introspection, got {}", records.introspections.len())
    };
    assert_eq!(record.trigger, IntrospectionTrigger::Rejection);
    assert_eq!(record.job.as_ref(), Some(&finished.job));
    assert_eq!(record.status, IntrospectionStatus::Recorded);
    assert!(!record.from_jev);
    assert!(record.evidence.as_ref().unwrap().cost_usd > 0.0);
    // The class is new: added as proposed, with an id assigned by code, and the one record of the registry.
    let [class] = records.classes.as_slice() else {
        panic!("one class")
    };
    assert_eq!(class.status, ClassStatus::Proposed);
    assert_eq!(record.failure_class.as_ref(), Some(&class.id));
    assert!(record.new_class);
    assert!(RecordId::parse(class.id.as_str()).is_ok());
    // Only records were added: no file of the repository was edited.
    let changed = fx
        .git
        .run(["diff", "--name-only", &base, &fx.head()])
        .unwrap();
    assert!(
        changed
            .lines()
            .all(|path| path.starts_with("records/") && !path.ends_with(".toml")),
        "{changed}"
    );
    assert_eq!(fx.file("athena.toml").unwrap(), test_config());
    assert!(athena.check_history().unwrap().violations.is_empty());

    // `athena status <job>` shows it, in text and in JSON.
    let job = finished.job.to_string();
    let out = athena_bin(&fx).args(["status", &job]).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    for needle in [
        "introspection ",
        "rejection",
        "what happened The job removed README.md",
        "check removes a file the repository needs",
        "new, proposed",
        "earlier cases none",
        "cause         Nothing stops a candidate",
        "prevention    code-check: Refuse a candidate",
        "next request  Add a greeting to README.md",
        "a language model, not Jev",
    ] {
        assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
    }
    let out = athena_bin(&fx)
        .args(["status", &job, "--json"])
        .output()
        .unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let shown = &json["introspections"][0];
    assert_eq!(shown["status"], "recorded");
    assert_eq!(shown["from_jev"], false);
    assert!(
        shown["cause"]
            .as_str()
            .unwrap()
            .starts_with("Nothing stops")
    );
    assert_eq!(shown["earlier_cases"], json!([]));
}

#[test]
fn the_prompt_for_a_rejected_job_names_the_unmet_criterion_and_the_major_finding() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The verifier found the greeting criterion unmet and a major finding in the README.",
        "No step checks the criteria before the verifier does, so an unmet one is found late.",
    ));
    let verdict = json!({
        "verdict": "fail",
        "criteria": [
            { "criterion": "criterion 1", "met": false, "evidence": "no greeting found", "commands": [] },
            { "criterion": "criterion 2", "met": true, "evidence": "checked", "commands": ["cargo test"] }
        ],
        "findings": [
            { "severity": "major", "summary": "The greeting is missing", "evidence": "README.md has none",
              "location": "README.md:3" },
            { "severity": "minor", "summary": "A trailing space", "evidence": "line 1", "location": null }
        ],
        "request_reading_mismatch": null,
        "summary": "The greeting is missing."
    });
    let scenario = Scenario {
        verdict,
        ..Scenario::default()
    };
    let (_fx, athena) = setup(scenario, &model);
    let finished = run_job(&athena, "add a greeting");
    assert_eq!(finished.state, OutcomeState::Rejected);

    let prompts = model.prompts.lock().unwrap();
    let [prompt] = prompts.as_slice() else {
        panic!("one prompt, got {}", prompts.len())
    };
    let what = prompt["event"]["what"].as_str().unwrap();
    assert!(
        what.contains("Unmet acceptance criteria (1):"),
        "missing the unmet criteria in:\n{what}"
    );
    assert!(
        what.contains("README.md ends with a greeting line"),
        "missing the criterion text in:\n{what}"
    );
    assert!(
        what.contains("major: The greeting is missing at README.md:3"),
        "missing the major finding in:\n{what}"
    );
    assert!(!what.contains("trailing space"), "{what}");
    assert!(!what.contains("the tests still pass"), "{what}");
    assert!(!what.contains("Failed checks"), "{what}");
}

#[test]
fn a_failed_job_is_introspected() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job changed a governance file that was not a listed deliverable.",
        "The reading does not list governance files when the request does not name them.",
    ));
    let mut scenario = Scenario::default();
    scenario
        .edits
        .push(("rustfmt.toml".into(), Some("style = \"relaxed\"\n".into())));
    let (fx, athena) = setup(scenario, &model);
    let finished = run_job(&athena, "add a greeting");
    assert_eq!(finished.state, OutcomeState::Failed);
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert_eq!(record.trigger, IntrospectionTrigger::Failure);
    assert_eq!(record.category, "failed");
    assert_eq!(record.status, IntrospectionStatus::Recorded);
}

fn chosen(choice: &str, probability: f64) -> Evaluation {
    Evaluation {
        answers: BTreeMap::from([
            (
                JEV_CLASS_KEY.to_string(),
                Answer::Choice {
                    choice: choice.into(),
                    probabilities: BTreeMap::from([(choice.to_string(), 0.75)]),
                    confidence: 0.6,
                },
            ),
            (JEV_REPEAT_KEY.to_string(), Answer::Boolean { probability }),
        ]),
        usage: Usage {
            input_tokens: 410,
            output_tokens: 9,
        },
        cost_usd: Some(0.00002),
    }
}

#[test]
fn jev_is_asked_in_shadow_and_both_answers_are_recorded_without_changing_the_class() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md, so the test check failed and the job was rejected.",
        "Nothing stops a candidate from deleting a file the checks need before the checks run.",
    ));
    // Jev disagrees with the model on purpose: it names an id that is not in the registry.
    let jev = FakeJev::new().then(Ok(chosen("not-a-registry-class", 0.25)));
    let (fx, mut athena) = setup(rejecting(), &model);
    athena.jev = Some(Box::new(jev.clone()));
    let finished = run_job(&athena, "remove the readme");
    assert_eq!(finished.state, OutcomeState::Rejected);

    // One call, both questions, over the text the model was given.
    let calls = jev.calls();
    let [(state, questions)] = calls.as_slice() else {
        panic!("one Jev call, got {}", calls.len())
    };
    assert_eq!(
        questions.keys().map(String::as_str).collect::<Vec<_>>(),
        [JEV_CONTEXT_KEY, JEV_CLASS_KEY, JEV_REPEAT_KEY]
    );
    let sent: Value = serde_json::from_str(state).unwrap();
    assert_eq!(sent, model.prompts.lock().unwrap()[0]);
    assert!(
        matches!(&questions[JEV_CLASS_KEY], Question::Choice { criteria, .. }
        if criteria.keys().map(String::as_str).collect::<Vec<_>>() == [JEV_NEW_OPTION])
    );

    // Both answers, the usage and the cost are in the record, which validates and loads from git.
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    let stored = record.jev.as_ref().expect("Jev's answers are recorded");
    assert_eq!(stored.model, "typesafe-ai/jev");
    let class = stored.class.as_ref().unwrap();
    assert_eq!(class.choice, "not-a-registry-class");
    assert_eq!(class.probabilities["not-a-registry-class"], 0.75);
    assert_eq!(class.confidence, 0.6);
    assert_eq!(stored.repeat_probability, Some(0.25));
    assert_eq!((stored.input_tokens, stored.output_tokens), (410, 9));
    assert_eq!(stored.cost_usd, Some(0.00002));
    assert_eq!(stored.error, None);
    assert!(record.validate().is_ok());

    // It decided nothing: the class is the model's, the record is not from Jev, the outcome is unchanged.
    assert_eq!(record.status, IntrospectionStatus::Recorded);
    assert!(!record.from_jev);
    assert!(record.new_class);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let [class_record] = records.classes.as_slice() else {
        panic!("one class")
    };
    assert_eq!(record.failure_class.as_ref(), Some(&class_record.id));
    assert_ne!(class_record.id.as_str(), class.choice);
}

#[test]
fn a_failed_jev_call_is_recorded_and_blocks_nothing() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md, so the test check failed and the job was rejected.",
        "Nothing stops a candidate from deleting a file the checks need before the checks run.",
    ));
    let jev = FakeJev::new().then(Err(JevError::Refused {
        status: 401,
        excerpt: "bad key".into(),
    }));
    let (fx, mut athena) = setup(rejecting(), &model);
    athena.jev = Some(Box::new(jev));
    let finished = run_job(&athena, "remove the readme");
    assert_eq!(finished.state, OutcomeState::Rejected);
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert_eq!(record.status, IntrospectionStatus::Recorded);
    assert!(record.new_class);
    let stored = record.jev.as_ref().unwrap();
    assert!(stored.error.as_deref().unwrap().contains("HTTP 401"));
    assert!(stored.class.is_none() && stored.repeat_probability.is_none());
    assert!(stored.context_prevention.is_none());
    assert_eq!((stored.input_tokens, stored.output_tokens), (0, 0));
    assert!(record.validate().is_ok());
}

/// A reply to the three questions with this post-mortem answer.
fn with_context(answer: Option<Answer>) -> Evaluation {
    let mut evaluation = chosen("not-a-registry-class", 0.25);
    if let Some(answer) = answer {
        evaluation
            .answers
            .insert(JEV_CONTEXT_KEY.to_string(), answer);
    }
    evaluation
}

fn context_choice(choice: &str) -> Answer {
    Answer::Choice {
        choice: choice.into(),
        probabilities: BTreeMap::from([(choice.to_string(), 0.8)]),
        confidence: 0.7,
    }
}

const OUTCOMES: [&str; 7] = [
    "brief-assembly",
    "failure-class",
    "none",
    "spec-fix",
    "structure",
    "system-facts-line",
    "vocabulary-term",
];

#[test]
fn jevs_call_asks_what_context_would_have_prevented_it_and_the_answer_reads_back() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md.",
        "Nothing stops a deleted file.",
    ));
    let jev = FakeJev::new().then(Ok(with_context(Some(context_choice("spec-fix")))));
    let (fx, mut athena) = setup(rejecting(), &model);
    athena.jev = Some(Box::new(jev.clone()));
    let finished = run_job(&athena, "remove the readme");
    assert_eq!(finished.state, OutcomeState::Rejected);

    let calls = jev.calls();
    let Question::Choice {
        instructions,
        criteria,
    } = &calls[0].1[JEV_CONTEXT_KEY]
    else {
        panic!("the context question is a choice")
    };
    assert!(instructions.contains(
        "What one piece of context, if the agent had read it, would have prevented this?"
    ));
    assert_eq!(instructions, JEV_CONTEXT_QUESTION);
    assert_eq!(
        criteria.keys().map(String::as_str).collect::<Vec<_>>(),
        OUTCOMES
    );

    // A rejected job yields the field, and it reads back from git equal to what was written.
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert!(!record.planning_stopped);
    let answer = record
        .jev
        .as_ref()
        .unwrap()
        .context_prevention
        .as_ref()
        .unwrap();
    assert_eq!(answer.outcome.as_deref(), Some("spec-fix"));
    assert_eq!(answer.error, None);
    assert!(record.validate().is_ok());
    let any = AnyRecord::Introspection(Box::new(record));
    let parsed = AnyRecord::parse("x.json", any.to_json().as_bytes()).unwrap();
    assert_eq!(parsed, any);
}

#[test]
fn a_failed_job_gets_the_context_field_and_no_planning_marker() {
    let model = Model::default();
    model.say(reply(
        "new",
        "A governance file changed.",
        "The reading did not list it.",
    ));
    let mut scenario = Scenario::default();
    scenario
        .edits
        .push(("rustfmt.toml".into(), Some("style = \"relaxed\"\n".into())));
    let (fx, mut athena) = setup(scenario, &model);
    athena.jev = Some(Box::new(
        FakeJev::new().then(Ok(with_context(Some(context_choice("none"))))),
    ));
    let finished = run_job(&athena, "add a greeting");
    assert_eq!(finished.state, OutcomeState::Failed);
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert!(!record.planning_stopped);
    let answer = record.jev.unwrap().context_prevention.unwrap();
    assert_eq!(answer.outcome.as_deref(), Some("none"));
}

#[test]
fn a_landed_job_yields_no_record_and_asks_jev_nothing() {
    let model = Model::default();
    let jev = FakeJev::new();
    let (fx, mut athena) = setup(Scenario::default(), &model);
    athena.jev = Some(Box::new(jev.clone()));
    let finished = run_job(&athena, "add a greeting");
    assert_eq!(finished.state, OutcomeState::Landed);
    assert!(introspections(&fx).is_empty());
    assert!(jev.calls().is_empty());
}

#[test]
fn a_job_that_failed_in_planning_is_marked_and_carries_the_context_field() {
    let model = Model::default();
    model.say(reply(
        "new",
        "Planning stopped.",
        "The plan was rejected by the critic.",
    ));
    let fx = Fixture::with_config(&format!("{}\n[job_plan]\nenabled = true\n", test_config()));
    let scenario = Scenario {
        plans: vec![json!({
        "approach": "Add a greeting line to README.md.",
        "decisions": [],
        "files": [{ "path": "README.md", "action": "modify", "purpose": "Hold the greeting." }],
        "steps": ["Edit README.md."],
        "risks": [],
        "demonstrations": [
            { "criterion": 1, "method": "run the tests", "command": "cargo test --locked",
              "observes": "criterion 1 holds", "fails_without": "the greeting test fails",
              "seam": "the fake agents of this test; exists" },
            { "criterion": 2, "method": "run the tests", "command": "cargo test --locked",
              "observes": "criterion 2 holds", "fails_without": "the suite fails",
              "seam": "the fake agents of this test; exists" }
        ],
        "open_questions": []
        })],
        critiques: vec![json!({
            "verdict": "reject",
            "findings": [{ "severity": "major", "summary": "The plan misses the request",
                "location": "approach", "evidence": "it edits the wrong file",
                "fix": "edit the right file" }],
            "request_reading_mismatch": null,
            "summary": "The critic says reject."
        })],
        ..Scenario::default()
    };
    let jev = FakeJev::new().then(Ok(with_context(Some(context_choice("brief-assembly")))));
    let mut athena = fx.athena(model.script(scenario.script()));
    athena.jev = Some(Box::new(jev));
    let finished = run_job(&athena, "tidy the readme");
    assert_eq!(finished.state, OutcomeState::Failed);
    assert!(
        finished
            .reason
            .starts_with("planning stopped before any code was written:"),
        "{}",
        finished.reason
    );
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert!(record.planning_stopped);
    let outcome = record
        .jev
        .as_ref()
        .unwrap()
        .context_prevention
        .as_ref()
        .and_then(|a| a.outcome.clone())
        .expect("an outcome");
    assert!(OUTCOMES.contains(&outcome.as_str()), "{outcome}");
    assert_eq!(outcome, "brief-assembly");
    assert!(record.validate().is_ok());
    // The marker is stored and reads back.
    let any = AnyRecord::Introspection(Box::new(record));
    assert!(
        any.to_json().contains("\"planning_stopped\": true")
            || any.to_json().contains("\"planning_stopped\":true")
    );
    assert_eq!(
        AnyRecord::parse("x.json", any.to_json().as_bytes()).unwrap(),
        any
    );
}

#[test]
fn a_missing_or_bad_context_answer_is_kept_and_blocks_nothing() {
    let classes = Vec::new();
    // Missing: the class and repeat answers are stored and the field is empty.
    let stored = ask_jev(
        &FakeJev::new().then(Ok(with_context(None))),
        "state",
        &classes,
    );
    assert!(stored.error.is_none());
    assert!(stored.class.is_some() && stored.repeat_probability.is_some());
    assert!(stored.context_prevention.is_none());
    // Out of the list.
    let stored = ask_jev(
        &FakeJev::new().then(Ok(with_context(Some(context_choice("more-docs"))))),
        "state",
        &classes,
    );
    assert!(stored.class.is_some() && stored.repeat_probability.is_some());
    let answer = stored.context_prevention.unwrap();
    assert_eq!(answer.outcome, None);
    assert!(answer.error.unwrap().contains("more-docs"));
    // Wrong type.
    let stored = ask_jev(
        &FakeJev::new().then(Ok(with_context(Some(Answer::Boolean { probability: 0.5 })))),
        "state",
        &classes,
    );
    assert!(stored.class.is_some() && stored.repeat_probability.is_some());
    let answer = stored.context_prevention.unwrap();
    assert_eq!(answer.outcome, None);
    assert!(answer.error.is_some());
}

#[test]
fn jev_is_asked_even_when_the_model_fails_and_that_stays_a_failed_record() {
    // No model reply is scripted, so the model is unavailable; Jev answers anyway.
    let model = Model::default();
    let jev = FakeJev::new();
    let (fx, mut athena) = setup(rejecting(), &model);
    athena.jev = Some(Box::new(jev.clone()));
    run_job(&athena, "remove the readme");
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert_eq!(record.status, IntrospectionStatus::Failed);
    assert!(record.failure_class.is_none());
    assert!(record.jev.as_ref().unwrap().class.is_some());
    assert_eq!(jev.calls().len(), 1);
}

#[test]
fn without_a_jev_nothing_is_asked_and_older_records_without_the_field_still_parse() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md, so the test check failed and the job was rejected.",
        "Nothing stops a candidate from deleting a file the checks need before the checks run.",
    ));
    let (fx, athena) = setup(rejecting(), &model);
    run_job(&athena, "remove the readme");
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert!(record.jev.is_none());
    // The field is left out of the stored JSON, which is also how a record from before step 5 looks.
    let any = AnyRecord::Introspection(Box::new(record));
    let json: Value = serde_json::from_str(&any.to_json()).unwrap();
    assert!(json.get("jev").is_none(), "{json}");
    let parsed = AnyRecord::parse("old.json", any.to_json().as_bytes()).unwrap();
    assert_eq!(parsed, any);
}

#[test]
fn a_landed_job_is_not_an_event() {
    let model = Model::default();
    let (fx, athena) = setup(Scenario::default(), &model);
    let finished = run_job(&athena, "add a greeting");
    assert_eq!(finished.state, OutcomeState::Landed);
    assert!(introspections(&fx).is_empty());
    assert!(model.prompts.lock().unwrap().is_empty());
}

#[test]
fn a_correction_is_introspected_with_its_intervention() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The user corrected the reading: the greeting should be friendlier.",
        "The reader chose a tone without asking, because no default says how to greet.",
    ));
    let (fx, athena) = setup(Scenario::default(), &model);
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    assert!(introspections(&fx).is_empty(), "asking is not an event");
    athena
        .correct(
            proposal.job.as_str(),
            "make it friendlier",
            &Lines::default(),
        )
        .unwrap();
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert_eq!(record.trigger, IntrospectionTrigger::Correction);
    assert_eq!(record.category, "corrected-output");
    assert!(record.intervention.is_some());
    assert_eq!(record.job.as_ref(), Some(&proposal.job));
}

#[test]
fn any_other_intervention_is_introspected() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The user edited the README by hand.",
        "The reading left out a file the user cared about.",
    ));
    let (fx, athena) = setup(Scenario::default(), &model);
    let id = intervene(&athena, "Edited the README by hand");
    let [record] = introspections(&fx)
        .try_into()
        .unwrap_or_else(|_| panic!("one"));
    assert_eq!(record.trigger, IntrospectionTrigger::Intervention);
    assert_eq!(record.intervention.as_ref(), Some(&id));
    assert_eq!(record.class_of_work, "docs");
}

#[test]
fn an_earlier_case_is_matched_ids_not_passed_in_are_dropped_and_only_listed_fields_are_sent() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The user edited the README by hand.",
        "The reading left out a file the user cared about.",
    ));
    model.say_with(|prompt| {
        let class = prompt["class_registry"][0]["id"].as_str().unwrap();
        let case = prompt["earlier_cases"][0]["id"].as_str().unwrap();
        let mut answer = reply(
            class,
            "The user edited the README by hand again.",
            "Hand edits are not learned from: the reading ignores the last edit.",
        );
        answer["earlier_cases"] = json!([case, "20260101-000000-zzzz"]);
        answer["decisions_matched"] = json!(["D87"]);
        answer
    });
    let (fx, athena) = setup(Scenario::default(), &model);
    intervene(&athena, "Edited the README by hand");
    intervene(&athena, "Edited the README by hand once more");

    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(
        records.classes.len(),
        1,
        "the second matched the first's class"
    );
    let [first, second] = records.introspections.as_slice() else {
        panic!("two introspections")
    };
    assert!(first.new_class && !second.new_class);
    assert_eq!(second.failure_class, first.failure_class);
    assert_eq!(second.earlier_cases, std::slice::from_ref(&first.id));
    assert_eq!(
        second.dropped_ids,
        ["20260101-000000-zzzz".to_string(), "D87".to_string()]
    );
    assert!(second.decisions_matched.is_empty());
    assert_eq!(second.status, IntrospectionStatus::Recorded);

    // What crossed to the model is only the listed stored fields, and the first call had no earlier case.
    let prompts = model.prompts.lock().unwrap();
    assert_eq!(prompts[0]["earlier_cases"], json!([]));
    let keys = |value: &Value| {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(
        keys(&prompts[1]["event"]),
        [
            "category",
            "class_of_work",
            "date",
            "job",
            "trigger",
            "what",
            "why"
        ]
    );
    assert_eq!(
        keys(&prompts[1]["earlier_cases"][0]),
        [
            "at",
            "category",
            "cause",
            "class_of_work",
            "failure_class",
            "id",
            "job",
            "what"
        ]
    );
    assert_eq!(
        keys(&prompts[1]["class_registry"][0]),
        ["description", "id", "name"]
    );
}

#[test]
fn a_cause_that_restates_the_event_is_flagged_and_the_record_rated_failed() {
    let model = Model::default();
    model.say(reply(
        "new",
        "The job removed README.md.",
        "the job REMOVED  README.md!",
    ));
    let (fx, athena) = setup(Scenario::default(), &model);
    intervene(&athena, "Edited the README by hand");
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let [record] = records.introspections.as_slice() else {
        panic!("one")
    };
    assert_eq!(record.status, IntrospectionStatus::Failed);
    assert_eq!(record.flags, ["the cause restates what_happened"]);
    assert!(record.what_happened.is_some());
    assert!(records.classes.is_empty(), "a failed record adds no class");
}

#[test]
fn a_failed_introspection_is_recorded_never_blocks_and_is_not_introspected() {
    let model = Model::default(); // no reply queued: the model call fails
    let (fx, athena) = setup(rejecting(), &model);
    let finished = run_job(&athena, "remove the readme");
    // The job closed as it would have without the introspection.
    assert_eq!(finished.state, OutcomeState::Rejected);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert!(records.jobs[finished.job.as_str()].outcome.is_some());
    let [record] = records.introspections.as_slice() else {
        panic!("one")
    };
    assert_eq!(record.status, IntrospectionStatus::Failed);
    assert!(
        record
            .failure
            .as_ref()
            .unwrap()
            .contains("model unavailable")
    );
    assert!(record.what_happened.is_none() && record.failure_class.is_none());
    // Not itself an event: the model was called once, and nothing followed.
    assert_eq!(model.prompts.lock().unwrap().len(), 1);
    // The status shows the failure.
    let out = athena_bin(&fx)
        .args(["status", finished.job.as_str()])
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("failed        model unavailable") || text.contains("model unavailable"));
    assert!(text.contains("· failed"), "{text}");
}

#[test]
fn earlier_cases_are_narrowed_by_class_of_work_and_category_and_capped() {
    let case = |n: u32, class: &str, category: &str| CaseRow {
        id: format!("20261001-0000{n:02}-aaaa"),
        job: None,
        at: format!("2026-10-01T00:00:{n:02}Z"),
        class_of_work: class.into(),
        category: category.into(),
        what: "x".into(),
        failure_class: "c".into(),
        cause: "y".into(),
    };
    let mut past = Past::default();
    for n in 0..7 {
        past.cases.push(case(n, "docs", "failed"));
    }
    past.cases.push(case(10, "docs", "rejected"));
    past.cases.push(case(11, "code", "failed"));
    let event = Event {
        trigger: IntrospectionTrigger::Failure,
        job: None::<JobId>,
        intervention: None,
        class_of_work: "docs".into(),
        category: "failed".into(),
        what: "w".into(),
        why: "y".into(),
        at: "2026-10-02T00:00:00Z".parse().unwrap(),
        planning_stopped: false,
    };
    let cases = narrow_cases(&past, &event);
    assert_eq!(cases.len(), CASE_CAP);
    assert_eq!(CASE_CAP, 5);
    // Newest first, and only the matching class of work and category.
    let ids: Vec<&str> = cases.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "20261001-000006-aaaa",
            "20261001-000005-aaaa",
            "20261001-000004-aaaa",
            "20261001-000003-aaaa",
            "20261001-000002-aaaa"
        ]
    );
}

#[test]
fn the_fixture_ui_is_quiet_when_nothing_is_an_event() {
    struct Count(Mutex<usize>);
    impl Ui for Count {
        fn line(&self, _: &str) {
            *self.0.lock().unwrap() += 1;
        }
    }
    let ui = Count(Mutex::new(0));
    let (_fx, athena) = setup(Scenario::default(), &Model::default());
    // A job that is not closed is not an event.
    let id = JobId::parse("20261001-000000-aaaa").unwrap();
    assert!(athena.introspect_job(&id, &ui).is_none());
}
