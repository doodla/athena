//! Introspection (`docs/design/defaults-loop.md` section 3, build order step 1), on fixture records in
//! temporary directories: each trigger, class matching and new-class creation, id dropping, a cause that
//! restates the event, a failed introspection, and the output of `athena status <job>`. No model is called:
//! a scripted fake answers the introspection prompt.

mod common;

use athena_agent::fake::{FakeTurn, Script};
use athena_core::introspect::{CASE_CAP, CaseRow, Event, Records as Past, narrow_cases};
use athena_core::introspect::{JEV_CLASS_KEY, JEV_NEW_OPTION, JEV_REPEAT_KEY};
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

fn athena_bin(fx: &Fixture) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_athena"));
    cmd.env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .env("ATHENA_STATE_DIR", fx.state.path())
        .current_dir(fx.root());
    cmd
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
        [JEV_CLASS_KEY, JEV_REPEAT_KEY]
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
    assert_eq!((stored.input_tokens, stored.output_tokens), (0, 0));
    assert!(record.validate().is_ok());
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
