//! `athena retry <job>` (`docs/design/defaults-loop.md` section 3.5, build order step 2), on fixture
//! records in temporary directories: a retry is seeded from the job's introspection record, shows earlier
//! cases of the same class and the preventions already tried, refuses a job that is not rejected or
//! failed, and falls back to "try again on" when the job has no usable introspection. No model is called:
//! scripted fakes answer the reader and the introspection prompts.

mod common;

use athena_agent::fake::{FakeTurn, Script};
use athena_core::Athena;
use athena_core::run::GoResult;
use athena_core::store::{JobState, Records};
use athena_data::records::{AcceptedBy, InvocationOutcome, OutcomeState, Role};
use common::*;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// The first words of the introspection's system prompt: how the script tells it from the other roles.
const MARKER: &str = "You write one introspection";

type Reply = Box<dyn Fn(&Value) -> Value + Send>;

/// Scripted introspection replies, in order, and every reader brief the fake reader was given.
#[derive(Clone, Default)]
struct Model {
    replies: Arc<Mutex<VecDeque<Reply>>>,
    briefs: Arc<Mutex<Vec<String>>>,
    producers: Arc<Mutex<usize>>,
}

impl Model {
    fn say_with(&self, reply: impl Fn(&Value) -> Value + Send + 'static) {
        self.replies.lock().unwrap().push_back(Box::new(reply));
    }

    fn last_brief(&self) -> String {
        self.briefs.lock().unwrap().last().cloned().unwrap()
    }

    fn script(&self, inner: Script) -> Script {
        let model = self.clone();
        Box::new(move |spec| {
            if spec.system_prompt.starts_with(MARKER) {
                let prompt: Value = serde_json::from_str(&spec.user_prompt).unwrap();
                return match model.replies.lock().unwrap().pop_front() {
                    Some(reply) => FakeTurn::output(reply(&prompt)),
                    None => FakeTurn {
                        outcome: InvocationOutcome::Error,
                        detail: Some("model unavailable".into()),
                        ..FakeTurn::output(Value::Null)
                    },
                };
            }
            match spec.role {
                Role::Reader => model.briefs.lock().unwrap().push(spec.user_prompt.clone()),
                Role::Producer => *model.producers.lock().unwrap() += 1,
                Role::Verifier | Role::Steward | Role::Audience | Role::Auditor => {}
            }
            inner(spec)
        })
    }
}

const PREVENTION: &str =
    "Refuse a candidate that deletes a file a check reads, before the checks run.";

/// An introspection answer. `class` is `"new"` or the id of a class on the prompt's registry.
fn answer(prompt: &Value, what: &str, cause: &str) -> Value {
    let (class, new_class) = match prompt["class_registry"].get(0) {
        Some(class) => (class["id"].clone(), Value::Null),
        None => (
            json!("new"),
            json!({ "name": "check removes a file the repository needs", "description": "A change deleted a file that a check requires." }),
        ),
    };
    let cases: Vec<Value> = prompt["earlier_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].clone())
        .collect();
    json!({
        "what_happened": what,
        "failure_class": class,
        "new_class": new_class,
        "earlier_cases": cases,
        "decisions_matched": [],
        "cause": cause,
        "prevention_kind": "code-check",
        "prevention": PREVENTION,
        "proposed_request": "Add a greeting to README.md without deleting any file."
    })
}

fn setup(model: &Model) -> (Fixture, Athena) {
    let fx = Fixture::new();
    let scenario = Scenario {
        edits: vec![("README.md".into(), None)],
        ..Scenario::default()
    };
    let athena = fx.athena(model.script(scenario.script()));
    (fx, athena)
}

/// Asks, then runs the job; it is rejected because the scenario deletes README.md. When `introspect` is
/// set, the model answers the introspection of the rejection; otherwise it is recorded as failed.
fn reject(
    model: &Model,
    athena: &Athena,
    text: &str,
    correction: Option<&str>,
    introspect: bool,
) -> String {
    let proposal = athena.ask(text, &Lines::default()).unwrap();
    let job = proposal.job.to_string();
    if let Some(line) = correction {
        // The correction is introspected at once, with no reply queued: it records a failure.
        athena.correct(&job, line, &Lines::default()).unwrap();
    }
    if introspect {
        model.say_with(|p| {
            answer(
                p,
                "The job removed README.md, so the test check failed.",
                "Nothing stops a candidate from deleting a file the checks need.",
            )
        });
    }
    match athena
        .go(&job, AcceptedBy::User, &Lines::default())
        .unwrap()
    {
        GoResult::Finished(f) => assert_eq!(f.state, OutcomeState::Rejected),
        other => panic!("expected a finished job, got {other:?}"),
    }
    job
}

/// Two rejected jobs of the same class; the second had a correction. Returns both ids.
fn two_cases(model: &Model, athena: &Athena) -> (String, String) {
    let first = reject(model, athena, "remove the readme", None, true);
    let second = reject(
        model,
        athena,
        "remove the readme again",
        Some("name the file README.md"),
        true,
    );
    (first, second)
}

#[test]
fn a_retry_starts_from_the_introspection_and_shows_earlier_cases_and_tried_preventions() {
    let model = Model::default();
    let (fx, athena) = setup(&model);
    let (first, second) = two_cases(&model, &athena);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let first_case = records
        .introspections
        .iter()
        .find(|i| i.job.as_ref().map(|j| j.to_string()) == Some(first.clone()))
        .unwrap()
        .id
        .to_string();
    assert_eq!(
        records.introspections.len(),
        3,
        "{:?}",
        records.introspections
    );

    let producers_before = *model.producers.lock().unwrap();
    let (proposal, seeded) = athena.retry(&second, &Lines::default()).unwrap();
    assert!(seeded);
    // The retry is read and proposed; it is not run.
    assert_eq!(*model.producers.lock().unwrap(), producers_before);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let job = &records.jobs[proposal.job.as_str()];
    assert_eq!(job.state(), JobState::Proposed);
    assert_eq!(
        job.request.as_ref().unwrap().text,
        format!("try again on {second}")
    );

    let brief = model.last_brief();
    for needle in [
        // The proposed request, cause and prevention of the retried job's introspection.
        "> Add a greeting to README.md without deleting any file.",
        "> Nothing stops a candidate from deleting a file the checks need.",
        PREVENTION,
        // The original request and the accepted correction, as retries already carry them.
        "> remove the readme again",
        "> name the file README.md",
        // An earlier case of the same class: its id and what happened.
        &first_case,
        "(job ",
        "The job removed README.md, so the test check failed.",
        "This is case 2 of its class",
        // Its prevention is shown as already tried, and as the same one proposed again.
        "Prevention already tried: > code-check: Refuse a candidate",
        "the same prevention as the one above",
    ] {
        assert!(
            brief.contains(needle),
            "the brief omits {needle:?}:\n{brief}"
        );
    }
}

#[test]
fn a_job_with_no_earlier_case_says_so() {
    let model = Model::default();
    let (_fx, athena) = setup(&model);
    let job = reject(&model, &athena, "remove the readme", None, true);
    let (_, seeded) = athena.retry(&job, &Lines::default()).unwrap();
    assert!(seeded);
    let brief = model.last_brief();
    assert!(brief.contains("# Why it happened"), "{brief}");
    assert!(brief.contains("No earlier case of this class is recorded."));
}

#[test]
fn a_job_that_is_not_rejected_or_failed_is_refused() {
    let model = Model::default();
    let (fx, athena) = setup(&model);
    let proposal = athena.ask("remove the readme", &Lines::default()).unwrap();
    let before = fx.head();
    let error = athena
        .retry(proposal.job.as_str(), &Lines::default())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("only a rejected or failed job can be tried again"),
        "{error}"
    );
    assert_eq!(fx.head(), before, "nothing was recorded");
}

/// A job that was dispatched and has no closing record, as after a machine restart: it is read and run
/// (the scenario rejects it), then the history is cut back to just before the closing commit.
fn interrupted(fx: &Fixture, athena: &Athena, text: &str) -> String {
    let proposal = athena.ask(text, &Lines::default()).unwrap();
    let job = proposal.job.to_string();
    athena
        .go(&job, AcceptedBy::User, &Lines::default())
        .unwrap();
    for _ in 0..6 {
        let records = Records::load(&fx.git, &fx.head()).unwrap();
        if records.jobs[&job].state() == JobState::Dispatched {
            return job;
        }
        fx.git.run(["reset", "-q", "--hard", "HEAD~1"]).unwrap();
    }
    panic!("the job never was dispatched without an outcome");
}

#[test]
fn an_interrupted_job_is_tried_again_with_its_original_request() {
    let model = Model::default();
    let (fx, athena) = setup(&model);
    let job = interrupted(&fx, &athena, "remove the readme");
    let (proposal, seeded) = athena.retry(&job, &Lines::default()).unwrap();
    assert!(!seeded, "an interrupted job has no introspection");
    assert!(proposal.reading.is_some());
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let request = records.jobs[proposal.job.as_str()]
        .request
        .as_ref()
        .unwrap();
    assert_eq!(request.text, format!("try again on {job}"));
    let brief = model.last_brief();
    assert!(brief.contains("# Earlier attempt"), "{brief}");
    assert!(brief.contains("> remove the readme"), "{brief}");
    assert_eq!(
        records.jobs[&job].state(),
        JobState::Dispatched,
        "the interrupted job is left as it was"
    );
}

#[test]
fn a_job_that_is_still_running_is_refused_by_retry() {
    use std::os::unix::process::CommandExt;
    let model = Model::default();
    let (fx, athena) = setup(&model);
    let job = interrupted(&fx, &athena, "remove the readme");
    // A live process group registered for the job, as a running producer has.
    let mut sleeper = Command::new("sleep")
        .arg("30")
        .process_group(0)
        .spawn()
        .unwrap();
    athena
        .state
        .registry()
        .register(sleeper.id() as i32, &job, "producer")
        .unwrap();
    let before = fx.head();
    let result = athena.retry(&job, &Lines::default());
    let _ = sleeper.kill();
    let _ = sleeper.wait();
    let error = result.unwrap_err().to_string();
    assert!(error.contains("is running"), "{error}");
    assert_eq!(fx.head(), before, "nothing was recorded");
}

#[test]
fn a_job_with_no_introspection_falls_back_to_try_again_on() {
    let model = Model::default();
    let (_fx, athena) = setup(&model);
    // No reply is scripted, so the introspection of the rejection is recorded as failed.
    let job = reject(&model, &athena, "remove the readme", None, false);
    let (proposal, seeded) = athena.retry(&job, &Lines::default()).unwrap();
    assert!(!seeded);
    assert!(proposal.reading.is_some());
    let brief = model.last_brief();
    assert!(brief.contains("# Earlier attempt"), "{brief}");
    assert!(brief.contains("> remove the readme"));
    assert!(!brief.contains("# Why it happened"));
    assert!(!brief.contains("Proposed next request"));
}

#[test]
fn the_retry_command_takes_a_job_and_the_same_options_as_ask() {
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_athena"))
            .env_remove("ATHENA_NESTED")
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    let retry = run(&["retry", "--help"]);
    assert!(retry.contains("<JOB>"), "{retry}");
    let ask = run(&["ask", "--help"]);
    for option in ["--yes", "--json"] {
        assert!(retry.contains(option), "retry lacks {option}");
        assert!(ask.contains(option));
    }
}
