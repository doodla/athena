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
                Role::Verifier
                | Role::Steward
                | Role::Audience
                | Role::Auditor
                | Role::Planner
                | Role::Critic => {}
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
fn a_retry_brief_carries_the_result_and_scope_and_a_reading_without_them_is_refused() {
    // Mode 0: the scenario's reader, which names the earlier job. Mode 1: the default reading, which
    // names no earlier job. Mode 2: a reading that names the job but lists nothing out of scope.
    let model = Model::default();
    let mode = Arc::new(Mutex::new(0u8));
    let earlier = Arc::new(Mutex::new(String::new()));
    let inner = Scenario {
        edits: vec![("README.md".into(), None)],
        ..Scenario::default()
    }
    .script();
    let (picked, job) = (mode.clone(), earlier.clone());
    let script: Script = Box::new(move |spec| {
        let reading = match *picked.lock().unwrap() {
            1 if spec.role == Role::Reader => Some(default_reading()),
            2 if spec.role == Role::Reader => {
                let mut r = default_reading();
                r["summary"] = json!(format!("Retry {}.", job.lock().unwrap()));
                r["out_of_scope"] = json!([]);
                Some(r)
            }
            _ => None,
        };
        match reading {
            Some(r) => FakeTurn::output(r),
            None => inner(spec),
        }
    });
    let fx = Fixture::new();
    let athena = fx.athena(model.script(script));
    let first = reject(&model, &athena, "remove the readme", None, true);
    *earlier.lock().unwrap() = first.clone();

    // The brief states where the job ended, its verdict, and the scope it was read with.
    *mode.lock().unwrap() = 0;
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_some(), "{:?}", proposal.reader_error);
    let brief = model.last_brief();
    let section = brief
        .split("# Previous result and change scope")
        .nth(1)
        .unwrap_or_else(|| panic!("no section:\n{brief}"));
    for needle in [
        format!("Job {first}: end state rejected; final verification verdict fail"),
        "> modify README.md".to_string(),
        "> Other files".to_string(),
    ] {
        assert!(section.contains(&needle), "{needle:?}:\n{section}");
    }

    // A reading that leaves out the previous result is refused, and names what is missing.
    *mode.lock().unwrap() = 1;
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_none());
    let error = proposal.reader_error.unwrap();
    assert!(error.contains("the previous result"), "{error}");
    assert!(error.contains(&first), "{error}");
    assert!(!error.contains("the change scope"), "{error}");

    // A reading that leaves out the change scope is refused too.
    *mode.lock().unwrap() = 2;
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_none());
    let error = proposal.reader_error.unwrap();
    assert!(error.contains("the change scope"), "{error}");
    assert!(!error.contains("the previous result"), "{error}");

    // A first attempt is never refused for it: the default reading names no earlier job and is accepted.
    *mode.lock().unwrap() = 1;
    let proposal = athena
        .ask("add a greeting to the readme", &Lines::default())
        .unwrap();
    assert!(proposal.reading.is_some(), "{:?}", proposal.reader_error);
    assert!(
        !model
            .last_brief()
            .contains("# Previous result and change scope")
    );
}

#[test]
fn a_retry_brief_carries_the_earlier_jobs_files_and_a_reading_that_ignores_them_is_refused() {
    // Mode 0: the scenario's reader, which lists README.md as a deliverable and names the earlier job.
    // Mode 1: names the job and gives a scope, but lists only a new file and excludes nothing the earlier
    // job changed. Mode 2: the same, but its out_of_scope item names README.md.
    let model = Model::default();
    let mode = Arc::new(Mutex::new(0u8));
    let earlier = Arc::new(Mutex::new(String::new()));
    let inner = Scenario {
        edits: vec![("README.md".into(), None)],
        report: producer_report("Deleted README.md as asked.", &["cargo test --locked"]),
        ..Scenario::default()
    }
    .script();
    let (picked, job) = (mode.clone(), earlier.clone());
    let script: Script = Box::new(move |spec| {
        let mode = *picked.lock().unwrap();
        if spec.role == Role::Reader && mode > 0 {
            let mut r = reading(
                &[("NOTES.md", "create")],
                &[("content", "NOTES.md holds a note")],
            );
            r["summary"] = json!(format!("Retry {}: write the note.", job.lock().unwrap()));
            if mode == 2 {
                r["out_of_scope"] = json!(["README.md stays as the earlier job left it."]);
            }
            return FakeTurn::output(r);
        }
        inner(spec)
    });
    let fx = Fixture::new();
    let athena = fx.athena(model.script(script));
    let first = reject(&model, &athena, "remove the readme", None, true);
    *earlier.lock().unwrap() = first.clone();

    // The brief lists the file the earlier job changed, its own account and its measured checks.
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_some(), "{:?}", proposal.reader_error);
    let brief = model.last_brief();
    let section = brief
        .split("# Previous result and change scope")
        .nth(1)
        .unwrap_or_else(|| panic!("no section:\n{brief}"));
    for needle in [
        "Files its candidate changed (1)",
        "> deleted README.md",
        "> Deleted README.md as asked.",
        "Measured checks:\n> ",
    ] {
        assert!(section.contains(needle), "{needle:?}:\n{section}");
    }

    // A reading that names the job and gives a scope but ignores the changed file is refused, and the
    // refusal names the file and no other part.
    *mode.lock().unwrap() = 1;
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_none());
    let error = proposal.reader_error.unwrap();
    assert!(error.contains("the earlier job's files"), "{error}");
    assert!(
        error.contains(&format!("job {first} changed README.md")),
        "{error}"
    );
    assert!(!error.contains("the previous result"), "{error}");
    assert!(!error.contains("the change scope"), "{error}");

    // Excluding the file in out_of_scope is enough to be accepted.
    *mode.lock().unwrap() = 2;
    let (proposal, _) = athena.retry(&first, &Lines::default()).unwrap();
    assert!(proposal.reading.is_some(), "{:?}", proposal.reader_error);
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

// ---------------------------------------------------------------------------------------------------------
// Tree-seeded retries: a rejection with every check passed and only content criteria unmet restarts the
// producer from the rejected candidate; any other rejection restarts from the base.

/// What one producer run was given: its working directory's files (relative paths, `.git` left out), the
/// content of `README.md` there, and its brief.
#[derive(Clone, Debug)]
struct Seen {
    files: Vec<String>,
    readme: String,
    brief: String,
}

fn files_under(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path.strip_prefix(dir).unwrap().display().to_string());
            }
        }
    }
    out.sort();
    out
}

/// The script, what each producer run was given, and every role that was called.
type Phased = (
    athena_agent::fake::Script,
    Arc<Mutex<Vec<Seen>>>,
    Arc<Mutex<Vec<Role>>>,
);

/// A script that plays `first` until the second reader call (the retry's reading) and `second` after it, and
/// records what every producer run was given, plus every role it was called as.
fn two_phases(first: Scenario, second: Scenario) -> Phased {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let roles = Arc::new(Mutex::new(Vec::new()));
    let readers = Arc::new(Mutex::new(0usize));
    let (first, second) = (first.script(), second.script());
    let (kept, called) = (seen.clone(), roles.clone());
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        called.lock().unwrap().push(spec.role);
        if spec.role == Role::Reader {
            *readers.lock().unwrap() += 1;
        }
        if spec.role == Role::Producer {
            kept.lock().unwrap().push(Seen {
                files: files_under(&spec.workdir),
                readme: std::fs::read_to_string(spec.workdir.join("README.md")).unwrap_or_default(),
                brief: spec.user_prompt.clone(),
            });
        }
        if *readers.lock().unwrap() > 1 {
            second(spec)
        } else {
            first(spec)
        }
    });
    (script, seen, roles)
}

const HELLO: &str = "# Demo\n\nHello, reader.\n";
const CONTENT_FINDING: &str = "CONTENT-FINDING-CANARY the description lists no callers";

/// A verdict that leaves the given criteria (1-based) unmet and the other one met.
fn verdict_unmet(unmet: &[usize]) -> Value {
    let criteria: Vec<Value> = (1..=2)
        .map(|n| {
            let met = !unmet.contains(&n);
            json!({
                "criterion": format!("criterion {n}"),
                "met": met,
                "evidence": if met { "ran" } else { "UNMET-EVIDENCE-CANARY missing" },
                "commands": if n == 2 { vec!["cargo test"] } else { Vec::<&str>::new() }
            })
        })
        .collect();
    json!({
        "verdict": "fail",
        "criteria": criteria,
        "findings": [{
            "severity": "major",
            "summary": CONTENT_FINDING,
            "evidence": "the change description is missing",
            "location": "README.md"
        }],
        "request_reading_mismatch": null,
        "summary": "Rejected."
    })
}

/// The first attempt adds `greeting.txt` and edits `README.md`; its verdict is `verdict`.
fn first_attempt(verdict: Value) -> Scenario {
    Scenario {
        edits: vec![
            ("README.md".into(), Some(HELLO.into())),
            ("greeting.txt".into(), Some("hi\n".into())),
        ],
        verdict,
        ..Scenario::default()
    }
}

/// The retry's reading, which says what becomes of the file the first attempt added: a reading of a retry
/// lists or excludes each file the earlier job changed (`greeting.txt` here), or it is refused.
fn retry_reading() -> Value {
    let mut reading = default_reading();
    reading["out_of_scope"] = json!([
        "Other files",
        "greeting.txt stays as the first attempt wrote it."
    ]);
    reading
}

/// The retry's producer edits `README.md` again; its verdict passes.
fn second_attempt() -> Scenario {
    Scenario {
        reading: retry_reading(),
        edits: vec![(
            "README.md".into(),
            Some("# Demo\n\nHello, reader.\n\nChanged by the retry.\n".into()),
        )],
        ..Scenario::default()
    }
}

/// Runs the first attempt to a rejection, then asks to try again on it. Returns the retry's job.
fn reject_then_ask_again(athena: &Athena, text: &str) -> (String, String) {
    let first = athena.ask(text, &Lines::default()).unwrap().job.to_string();
    match athena
        .go(&first, AcceptedBy::User, &Lines::default())
        .unwrap()
    {
        GoResult::Finished(f) => assert_eq!(f.state, OutcomeState::Rejected, "{}", f.reason),
        other => panic!("expected a finished job, got {other:?}"),
    }
    let again = athena
        .ask(&format!("try again on {first}"), &Lines::default())
        .unwrap()
        .job
        .to_string();
    (first, again)
}

fn run_retry(athena: &Athena, job: &str) -> athena_core::run::Finished {
    match athena.go(job, AcceptedBy::User, &Lines::default()).unwrap() {
        GoResult::Finished(f) => *f,
        other => panic!("expected a finished job, got {other:?}"),
    }
}

#[test]
fn a_content_only_rejection_retries_from_the_rejected_tree() {
    let fx = Fixture::new();
    let (script, seen, roles) = two_phases(first_attempt(verdict_unmet(&[1])), second_attempt());
    let athena = fx.athena(script);
    let (first, again) = reject_then_ask_again(&athena, "add a greeting");
    let verifiers_before = roles
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Verifier)
        .count();
    let finished = run_retry(&athena, &again);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);

    // The first attempt started from the base; the retry from the rejected candidate.
    assert!(!seen[0].files.contains(&"greeting.txt".to_string()));
    assert_eq!(seen[0].readme, "# Demo\n");
    assert!(
        seen[1].files.contains(&"greeting.txt".to_string()),
        "{:?}",
        seen[1].files
    );
    assert_eq!(seen[1].readme, HELLO);
    // The brief carries the findings and says where the work starts, and never that it starts from the base.
    assert!(seen[1].brief.contains(CONTENT_FINDING), "{}", seen[1].brief);
    assert!(seen[1].brief.contains("UNMET-EVIDENCE-CANARY"));
    assert!(seen[1].brief.contains(&format!(
        "starts from the rejected candidate of job {first}"
    )));
    assert!(!seen[1].brief.contains("starts from the base"));
    assert!(!seen[0].brief.contains("rejected candidate"));

    // The new candidate is verified in full and lands against the original base: the rejected work and the
    // retry's own edit are both in the landed tree.
    let verifiers_after = roles
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Verifier)
        .count();
    assert!(verifiers_after > verifiers_before, "the verifier ran");
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    assert_eq!(fx.file("greeting.txt").as_deref(), Some("hi\n"));
    assert!(
        fx.file("README.md")
            .unwrap()
            .contains("Changed by the retry.")
    );
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let verification = records.jobs[&again].verification.as_ref().unwrap();
    assert!(!verification.checks.is_empty());
    assert!(verification.checks.iter().all(|c| c.passed));
    let candidate = records.jobs[&again].candidate.as_ref().unwrap();
    let paths: Vec<&str> = candidate.changes.iter().map(|c| c.path.as_str()).collect();
    assert!(paths.contains(&"greeting.txt"), "{paths:?}");
    assert!(paths.contains(&"README.md"), "{paths:?}");
}

#[test]
fn a_seeded_retry_that_changes_nothing_goes_on_with_the_seeded_candidate() {
    let fx = Fixture::new();
    let idle = Scenario {
        edits: vec![],
        ..second_attempt()
    };
    let (script, seen, roles) = two_phases(first_attempt(verdict_unmet(&[1])), idle);
    let athena = fx.athena(script);
    let (_, again) = reject_then_ask_again(&athena, "add a greeting");
    let verifiers_before = roles
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Verifier)
        .count();
    let finished = run_retry(&athena, &again);
    assert!(
        seen.lock().unwrap()[1]
            .files
            .contains(&"greeting.txt".to_string()),
        "the retry was seeded"
    );
    // The producer added nothing, but the seeded candidate already differs from the base: it is the
    // candidate, and it reaches the checks and the verifier and lands.
    let verifiers_after = roles
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Verifier)
        .count();
    assert!(verifiers_after > verifiers_before, "the verifier ran");
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    assert_eq!(fx.file("greeting.txt").as_deref(), Some("hi\n"));
    assert_eq!(fx.file("README.md").as_deref(), Some(HELLO));
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let verification = records.jobs[&again].verification.as_ref().unwrap();
    assert!(!verification.checks.is_empty(), "the checks ran");
    assert!(verification.checks.iter().all(|c| c.passed));
    let candidate = records.jobs[&again].candidate.as_ref().unwrap();
    let paths: Vec<&str> = candidate.changes.iter().map(|c| c.path.as_str()).collect();
    assert!(paths.contains(&"greeting.txt"), "{paths:?}");
    assert!(paths.contains(&"README.md"), "{paths:?}");
}

#[test]
fn an_unseeded_run_whose_producer_changes_nothing_still_fails() {
    let fx = Fixture::new();
    let idle = Scenario {
        edits: vec![],
        ..Scenario::default()
    };
    let (script, seen, roles) = two_phases(idle, second_attempt());
    let athena = fx.athena(script);
    let job = athena
        .ask("add a greeting", &Lines::default())
        .unwrap()
        .job
        .to_string();
    let finished = run_retry(&athena, &job);
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert_eq!(finished.state, OutcomeState::Failed, "{}", finished.reason);
    assert!(
        finished.reason.contains("the producer made no change"),
        "{}",
        finished.reason
    );
    assert!(
        !roles.lock().unwrap().contains(&Role::Verifier),
        "no candidate was verified"
    );
    assert_eq!(fx.file("greeting.txt"), None, "nothing landed");
}

#[test]
fn an_unmet_behavior_criterion_retries_from_the_base() {
    let fx = Fixture::new();
    let (script, seen, _) = two_phases(first_attempt(verdict_unmet(&[2])), second_attempt());
    let athena = fx.athena(script);
    let (_, again) = reject_then_ask_again(&athena, "add a greeting");
    run_retry(&athena, &again);
    let seen = seen.lock().unwrap().clone();
    assert!(!seen[1].files.contains(&"greeting.txt".to_string()));
    assert_eq!(seen[1].readme, "# Demo\n");
    assert!(!seen[1].brief.contains("rejected candidate"));
    assert!(
        seen[1]
            .brief
            .contains("A snapshot of the base commit's files")
    );
}

#[test]
fn a_failed_check_retries_from_the_base() {
    // The test check fails when README.md is gone: the first attempt deletes it and adds a file.
    let fx = Fixture::new();
    let first = Scenario {
        edits: vec![
            ("README.md".into(), None),
            ("greeting.txt".into(), Some("hi\n".into())),
        ],
        verdict: verdict_unmet(&[1]),
        ..Scenario::default()
    };
    let (script, seen, _) = two_phases(first, second_attempt());
    let athena = fx.athena(script);
    let (_, again) = reject_then_ask_again(&athena, "add a greeting");
    run_retry(&athena, &again);
    let seen = seen.lock().unwrap().clone();
    assert!(!seen[1].files.contains(&"greeting.txt".to_string()));
    assert_eq!(seen[1].readme, "# Demo\n", "the base's README.md");
    assert!(!seen[1].brief.contains("rejected candidate"));
}

#[test]
fn a_changed_base_retries_from_the_base() {
    let fx = Fixture::new();
    let (script, seen, _) = two_phases(first_attempt(verdict_unmet(&[1])), second_attempt());
    let athena = fx.athena(script);
    let first = athena
        .ask("add a greeting", &Lines::default())
        .unwrap()
        .job
        .to_string();
    run_retry(&athena, &first);
    // Other work lands in the repository between the rejection and the retry.
    write(fx.root(), &[("OTHER.md", "later work\n")]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git.run(["commit", "-q", "-m", "Other work"]).unwrap();
    let again = athena
        .ask(&format!("try again on {first}"), &Lines::default())
        .unwrap()
        .job
        .to_string();
    run_retry(&athena, &again);
    let seen = seen.lock().unwrap().clone();
    assert!(seen[1].files.contains(&"OTHER.md".to_string()));
    assert!(
        !seen[1].files.contains(&"greeting.txt".to_string()),
        "the old candidate would have undone later work"
    );
    assert!(!seen[1].brief.contains("rejected candidate"));
}

#[test]
fn a_failed_job_retries_from_the_base() {
    // The producer changes nothing, so the first job fails as an empty candidate.
    let fx = Fixture::new();
    let idle = Scenario {
        edits: vec![],
        ..Scenario::default()
    };
    let (script, seen, _) = two_phases(idle, second_attempt());
    let athena = fx.athena(script);
    let first = athena
        .ask("add a greeting", &Lines::default())
        .unwrap()
        .job
        .to_string();
    assert_eq!(run_retry(&athena, &first).state, OutcomeState::Failed);
    let again = athena
        .ask(&format!("try again on {first}"), &Lines::default())
        .unwrap()
        .job
        .to_string();
    run_retry(&athena, &again);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen[1].readme, "# Demo\n");
    assert!(!seen[1].brief.contains("rejected candidate"));
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
