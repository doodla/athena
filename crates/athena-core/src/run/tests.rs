#[cfg(test)]
mod critic_soundness_tests {
    use super::super::*;
    use serde_json::json;

    const SOUNDNESS_SUMMARY: &str =
        "Soundness: the refusal rule matches common words, so it has false positives";

    /// A critic output as the model returns it, parsed as the production round parses it, with one
    /// soundness finding at `severity`.
    fn critic_output(verdict: &str, severity: &str) -> CriticOutput {
        serde_json::from_value(json!({
            "verdict": verdict,
            "findings": [{
                "severity": severity,
                "summary": SOUNDNESS_SUMMARY,
                "location": "Approach",
                "evidence": "The rule matches words that common requests contain.",
                "fix": "Read the request's structure instead of matching words."
            }],
            "request_reading_mismatch": null,
            "summary": "A crude heuristic."
        }))
        .unwrap()
    }

    fn verdict_for(verdict: &str, severity: &str) -> CritiqueVerdict {
        let output = critic_output(verdict, severity);
        let findings = critic_findings(Vec::new(), &output);
        plan_verdict(CritiqueVerdict::Accept, &findings)
    }

    #[test]
    fn soundness_finding_blocker_makes_the_plan_revise() {
        let output = critic_output("accept", "blocker");
        let findings = critic_findings(Vec::new(), &output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Blocker);
        assert_eq!(findings[0].source, FindingSource::Critic);
        assert!(findings[0].summary.starts_with("Soundness:"));
        // The critic said accept; the blocker finding overrides it.
        assert_eq!(
            plan_verdict(CritiqueVerdict::Accept, &findings),
            CritiqueVerdict::Revise
        );
        assert_eq!(verdict_for("accept", "major"), CritiqueVerdict::Revise);
    }

    #[test]
    fn soundness_finding_minor_or_nit_leaves_the_plan_accepted() {
        assert_eq!(verdict_for("accept", "minor"), CritiqueVerdict::Accept);
        assert_eq!(verdict_for("accept", "nit"), CritiqueVerdict::Accept);
    }

    #[test]
    fn soundness_finding_does_not_soften_the_critics_own_verdict() {
        let findings = critic_findings(Vec::new(), &critic_output("revise", "minor"));
        assert_eq!(
            plan_verdict(CritiqueVerdict::Revise, &findings),
            CritiqueVerdict::Revise
        );
        assert_eq!(
            plan_verdict(CritiqueVerdict::Reject, &findings),
            CritiqueVerdict::Reject
        );
    }
}

/// Characterization of the order `produce_and_verify` works in (structure pass, R0): a fake job goes through
/// `Athena::go` and the test reads what the UI said and which invocation numbers were recorded. It guards the
/// moves of the produce and verify code against a change of order.
#[cfg(test)]
mod produce_and_verify_order_tests {
    use super::super::*;
    use crate::Quiet;
    use crate::capability::{CapabilityCard, CheckSpec, ToolGrant};
    use athena_agent::fake::{FakeAgentDriver, FakeTurn};
    use athena_agent::{AgentInvocation, Usage};
    use athena_data::config::CheckKind;
    use athena_data::records::PolicyViolation;
    use athena_git::Git;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    const CRITERION: &str = "README.md describes the demo";

    /// A capability that edits files and runs one configured check, so the baseline has something to say.
    /// The check is the command `check` (`true` passes, `false` fails).
    struct Docs {
        check: &'static [&'static str],
    }

    /// A check that passes on any tree.
    const PASSES: &[&str] = &["true"];
    /// A check that passes on the base README and fails once the README is edited.
    const FAILS_ON_EDIT: &[&str] = &["sh", "-c", "test \"$(cat README.md)\" = '# Demo'"];

    impl Capability for Docs {
        fn id(&self) -> &'static str {
            "study"
        }
        fn card(&self) -> CapabilityCard {
            CapabilityCard {
                id: "study",
                purpose: "describe a demo.",
                when: "for a request to describe a demo.",
                produces: "files describing the demo.",
            }
        }
        fn producer_section(&self) -> athena_context::PromptAsset {
            PRODUCER_FRAME
        }
        fn verifier_section(&self) -> athena_context::PromptAsset {
            VERIFIER_FRAME
        }
        fn producer_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                commands: BTreeMap::new(),
                writes: true,
                permission_mode: "acceptEdits".into(),
            }
        }
        fn verifier_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                // One command, so a behavior demonstration has something the verifier may run.
                commands: BTreeMap::from([("cargo".to_string(), vec!["test".to_string()])]),
                writes: false,
                permission_mode: "dontAsk".into(),
            }
        }
        fn checks(&self, _: &JobConfig) -> Vec<CheckSpec> {
            vec![CheckSpec {
                name: "probe".into(),
                run: self.check.iter().map(|part| part.to_string()).collect(),
                kind: CheckKind::Command,
                skip_when_only: None,
            }]
        }
        fn env(&self, _: &JobConfig) -> BTreeMap<String, String> {
            BTreeMap::new()
        }
        fn admit(&self, _: &ChangeList, _: &JobConfig) -> Vec<PolicyViolation> {
            Vec::new()
        }
    }

    /// A UI that keeps every line, in order.
    #[derive(Default)]
    struct Lines(Mutex<Vec<String>>);

    impl Ui for Lines {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    fn athena(repo: &tempfile::TempDir, state: &tempfile::TempDir) -> Athena {
        athena_with(
            repo,
            state,
            PASSES,
            crate::test_support::plenty_of_space,
            base_script(),
        )
    }

    /// An Athena over `repo` whose capability checks with the command `check`, whose free space is read by
    /// `free_space`, and whose agents follow `script`.
    fn athena_with(
        repo: &tempfile::TempDir,
        state: &tempfile::TempDir,
        check: &'static [&'static str],
        free_space: crate::state::FreeSpace,
        script: athena_agent::fake::Script,
    ) -> Athena {
        let state =
            crate::state::StateDir::new(state.path().join("repo")).with_free_space(free_space);
        let driver = FakeAgentDriver::new(script).with_registry(state.registry());
        Athena {
            git: Git::at(repo.path()),
            state,
            driver: std::sync::Arc::new(driver),
            capabilities: vec![Box::new(Docs { check })],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"a".repeat(40))),
            jev: None,
        }
    }

    /// The fake agents of this repository: a reader, a producer that edits the README, and a verifier that
    /// passes it.
    fn base_script() -> athena_agent::fake::Script {
        Box::new(|spec: &AgentInvocation| {
            let mut turn = match spec.role {
                Role::Reader => FakeTurn::output(json!({
                    "title": "Describe the demo",
                    "summary": "Describe the demo in README.md.",
                    "class_of_work": "docs",
                    "capability": "study",
                    "decline_reason": null,
                    "producer_model": "claude-haiku-5-5",
                    "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
                    "assumptions": [],
                    "question": null,
                    "acceptance_criteria": [{ "text": CRITERION, "kind": "content" }],
                    "deliverables": [{ "path": "README.md", "action": "modify", "p": 0.9 }],
                    "out_of_scope": ["Other files"],
                    "forecast": {
                        "p_reading_accepted": 0.8,
                        "p_verification_passes": 0.7,
                        "p_user_intervenes": 0.1,
                        "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
                        "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
                        "departures": null
                    }
                })),
                Role::Producer => {
                    let mut t = FakeTurn::output(json!({
                        "summary": "Described the demo.",
                        "files": ["README.md"],
                        "checks_run": [],
                        "criteria_check": [
                            { "criterion": CRITERION, "status": "met", "evidence": "read it" }
                        ],
                        "open_issues": [], "mistakes": [], "learnings": [], "desires": []
                    }));
                    t.edits = vec![("README.md".into(), Some("# Demo\n\nA demo.\n".into()))];
                    t
                }
                Role::Verifier => FakeTurn::output(json!({
                    "verdict": "pass",
                    "criteria": [
                        { "criterion": CRITERION, "met": true, "evidence": "checked", "commands": [] }
                    ],
                    "findings": [],
                    "request_reading_mismatch": null,
                    "summary": "Checked."
                })),
                Role::Planner | Role::Critic => unreachable!("planning is off in this repository"),
                Role::Steward => FakeTurn::output(json!({ "items": [] })),
                Role::Audience => FakeTurn::output(json!({ "rating": 3, "findings": [] })),
                Role::Auditor => FakeTurn::output(json!({ "summary": "Fine.", "findings": [] })),
            };
            turn.cost_usd = 0.01;
            turn.usage = Usage {
                input: 1,
                output: 1,
                cache_read: 1,
                cache_write: 1,
            };
            turn
        })
    }

    fn repo() -> tempfile::TempDir {
        repo_with_config("")
    }

    /// A repository whose planning is on for every job, with these many critic rounds.
    fn repo_with_planning(max_rounds: u32) -> tempfile::TempDir {
        repo_with_config(&format!(
            "\n[job_plan]\nenabled = true\nmode = \"all\"\nplanner_usd = 0.75\ncritic_usd = 0.75\n\
             planner_timeout_s = 600\ncritic_timeout_s = 600\nmax_rounds = {max_rounds}\n"
        ))
    }

    /// The test repository with `extra` appended to its `athena.toml`.
    fn repo_with_config(extra: &str) -> tempfile::TempDir {
        // Without `[job_plan]` (a job with no planning round) and without a pinned repository, unless
        // `extra` adds them.
        let raw = include_str!("../../../../athena.toml");
        let mut in_job_plan = false;
        let config: String = raw
            .split_inclusive('\n')
            .filter(|line| {
                let key = line.trim_start();
                if key.starts_with('[') {
                    in_job_plan = key.trim_end() == "[job_plan]";
                }
                !(in_job_plan || key.starts_with("repo_url") || key.starts_with("repo_ref"))
            })
            .collect();
        let config = format!("{config}{extra}");
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        for (path, text) in [
            ("athena.toml", config.as_str()),
            ("defaults.toml", include_str!("../../../../defaults.toml")),
            ("README.md", "# Demo\n"),
        ] {
            std::fs::write(dir.path().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        dir
    }

    /// Commits `text` at `path` in the test repository.
    fn commit_file(repo: &tempfile::TempDir, path: &str, text: &str) {
        let git = Git::at(repo.path());
        let full = repo.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Add a file\n\nAthena-Job: bootstrap"])
            .unwrap();
    }

    #[test]
    fn a_request_with_a_banned_word_is_refused_before_admission_and_a_clean_one_is_admitted() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let before = athena.head().unwrap();
        // Built from fragments so this file passes the scan it tests.
        let banned = ["oper", "ator"].concat();
        let request = format!("describe the demo\nthen ask the {banned} to check it");
        for refused in [
            athena.ask(&request, &Quiet).unwrap_err().to_string(),
            athena.queue(&request).unwrap_err().to_string(),
        ] {
            assert!(refused.contains("not admitted"), "{refused}");
            assert!(
                refused.contains(&format!("request:2: {banned}")),
                "{refused}"
            );
        }
        // Nothing was committed and no model was asked: no job exists.
        assert_eq!(athena.head().unwrap(), before);

        // The same request, reworded, passes the scan and is admitted.
        let proposal = athena
            .ask("describe the demo\nthen check it", &Quiet)
            .unwrap();
        assert!(proposal.reading.is_some());
        assert_ne!(athena.head().unwrap(), before);
    }

    #[test]
    fn a_request_naming_a_file_with_a_boundary_word_is_admitted() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        // Built from fragments so this file passes the scan it tests.
        let banned = ["sand", "box"].concat();
        let dirty = format!("# Notes\n\nIt ran in a {banned}.\n");
        commit_file(&repo, "BRIEF.md", &dirty);
        commit_file(&repo, "docs/dirty.md", &dirty);
        let before = athena.head().unwrap();

        // The named files hold the word, but the request's own text does not: the scan finds nothing.
        let text = "map the requirements in BRIEF.md and docs/dirty.md";
        assert!(crate::reading::scan_request_input(text).is_empty());

        // `ask --later` admits it.
        athena.queue(text).unwrap();
        let queued = athena.head().unwrap();
        assert_ne!(queued, before);

        // `ask` admits it and reads it.
        let proposal = athena.ask(text, &Quiet).unwrap();
        assert!(proposal.reading.is_some());
        assert_ne!(athena.head().unwrap(), queued);
    }

    #[test]
    fn a_request_whose_own_text_has_a_boundary_word_is_refused() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let banned = ["sand", "box"].concat();
        commit_file(&repo, "BRIEF.md", "# Brief\n\nNothing to see.\n");
        let before = athena.head().unwrap();

        // The request names a clean file; the word is in its own text, on line 2.
        let text = format!("map the requirements in BRIEF.md\nit ran in a {banned}");
        for refused in [
            athena.ask(&text, &Quiet).unwrap_err().to_string(),
            athena.queue(&text).unwrap_err().to_string(),
        ] {
            assert!(refused.contains("not admitted"), "{refused}");
            assert!(
                refused.contains(&format!("request:2: {banned}")),
                "{refused}"
            );
        }
        assert_eq!(athena.head().unwrap(), before);

        // The scan itself reports one finding, from the request.
        assert_eq!(
            crate::reading::scan_request_input(&text),
            vec![crate::reading::InputFinding {
                source: "request".into(),
                line: 2,
                term: banned
            }]
        );
    }

    #[test]
    fn a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let ui = Lines::default();
        let GoResult::Finished(finished) = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap()
        else {
            panic!("expected the job to finish");
        };
        assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);

        // The lines the three stages say, in the order they said them; the rest (dispatch, landing) is
        // not the order under test.
        let lines = ui.0.lock().unwrap().clone();
        let stages: Vec<&str> = lines
            .iter()
            .map(String::as_str)
            .filter(|l| l.starts_with("dispatched ") || l.starts_with("landed "))
            .map(|l| l.split_once(' ').unwrap().0)
            .collect();
        assert_eq!(stages, ["dispatched", "landed"]);
        let first = lines
            .iter()
            .position(|l| l.starts_with("dispatched "))
            .unwrap();
        let said: Vec<&str> = lines[first..]
            .iter()
            .map(String::as_str)
            .take_while(|l| !l.starts_with("landed "))
            .collect();
        assert_eq!(said.len(), 7, "{said:#?}");
        assert!(said[0].starts_with("dispatched "), "{said:#?}");
        assert_eq!(said[1], "baseline   probe ok");
        // The producer and verifier lines carry a model, a wall time, a cost and a count or verdict.
        assert!(
            said[2].starts_with("producer   claude-haiku-5-5 · ")
                && said[2].ends_with(" · $0.01 · 0 commands run"),
            "{said:#?}"
        );
        assert!(
            said[3].starts_with("candidate  README.md (modified) · "),
            "{said:#?}"
        );
        assert!(said[4].starts_with("checks     probe ok ("), "{said:#?}");
        // The replay has no history to replay here, and a fake driver builds nothing: it says why.
        assert!(said[5].starts_with("replay     no replay: "), "{said:#?}");
        assert!(
            said[6].starts_with("verifier   ") && said[6].ends_with(" · $0.01 · pass"),
            "{said:#?}"
        );

        // The invocation numbers: the reading is 1, the producer 2, the verifier 3, and the baseline was
        // taken before the producer, so it names the producer's number.
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let job = &records.jobs[proposal.job.as_str()];
        let mut invocations: Vec<(u32, Role)> =
            job.invocations.iter().map(|i| (i.seq, i.role)).collect();
        invocations.sort();
        assert_eq!(
            invocations,
            [(1, Role::Reader), (2, Role::Producer), (3, Role::Verifier)]
        );
        let baseline = job
            .baseline
            .as_ref()
            .expect("the job recorded its baseline");
        assert_eq!(baseline.before_invocation, 2);
        assert_eq!(baseline.checks.len(), 1);
    }

    /// A plan for the one criterion of `base_script`'s reading that creates README.md, which exists in the base:
    /// it breaks Athena's plan rule that a file to create must not exist.
    fn bad_plan() -> serde_json::Value {
        plan_with_action("create")
    }

    /// A plan that passes Athena's plan rules: it modifies README.md.
    fn good_plan() -> serde_json::Value {
        plan_with_action("modify")
    }

    fn plan_with_action(action: &str) -> serde_json::Value {
        json!({
            "approach": "Edit README.md to describe the demo.",
            "decisions": [],
            "files": [{ "path": "README.md", "action": action, "purpose": "Describe the demo." }],
            "steps": ["Edit README.md."],
            "risks": [],
            "demonstrations": [{
                "criterion": 1,
                "method": "read the file",
                "command": "cat README.md",
                "observes": "README.md describes the demo",
                "fails_without": null
            }],
            "open_questions": []
        })
    }

    fn accepting_critique() -> serde_json::Value {
        json!({
            "verdict": "accept", "findings": [], "request_reading_mismatch": null,
            "summary": "The plan is sound."
        })
    }

    fn revising_critique() -> serde_json::Value {
        json!({
            "verdict": "revise",
            "findings": [{
                "severity": "major",
                "summary": "The demonstration shows nothing the file does not already say",
                "location": "demonstrations",
                "evidence": "It only reads the file.",
                "fix": "Name a check that fails without the change."
            }],
            "request_reading_mismatch": null,
            "summary": "The critic asks for a revision."
        })
    }

    /// What a scripted job saw: the role and prompt of every invocation, in order.
    type Calls = Arc<Mutex<Vec<(Role, String)>>>;

    /// `base_script` with the planner and critic answering from queues. The last answer of a queue repeats;
    /// an empty critic queue fails the test on the first critic call.
    fn planning_script(
        planner: Vec<serde_json::Value>,
        critic: Vec<serde_json::Value>,
        calls: Calls,
    ) -> athena_agent::fake::Script {
        let inner = base_script();
        let (planner, critic) = (Mutex::new((planner, 0)), Mutex::new((critic, 0)));
        Box::new(move |spec: &AgentInvocation| {
            calls
                .lock()
                .unwrap()
                .push((spec.role, spec.user_prompt.clone()));
            let queue = match spec.role {
                Role::Planner => &planner,
                Role::Critic => &critic,
                _ => return inner(spec),
            };
            let (answers, next) = &mut *queue.lock().unwrap();
            assert!(!answers.is_empty(), "unexpected {:?} call", spec.role);
            let answer = answers[(*next).min(answers.len() - 1)].clone();
            *next += 1;
            let mut turn = FakeTurn::output(answer);
            turn.cost_usd = 0.01;
            turn.usage = Usage {
                input: 1,
                output: 1,
                cache_read: 1,
                cache_write: 1,
            };
            turn
        })
    }

    /// Runs a job over `repo` with these planner and critic answers; returns the outcome and every call.
    fn run_planned(
        repo: &tempfile::TempDir,
        planner: Vec<serde_json::Value>,
        critic: Vec<serde_json::Value>,
    ) -> (Box<Finished>, Vec<(Role, String)>) {
        let state = tempfile::tempdir().unwrap();
        let calls = Calls::default();
        let athena = athena_with(
            repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            planning_script(planner, critic, calls.clone()),
        );
        let (_, finished) = finished_job(&athena);
        let calls = calls.lock().unwrap().clone();
        (finished, calls)
    }

    /// `run_planned` for a reading whose one criterion is a behavior criterion, which Athena's plan rules
    /// hold to a named test seam.
    fn run_planned_behavior(
        repo: &tempfile::TempDir,
        planner: Vec<serde_json::Value>,
        critic: Vec<serde_json::Value>,
    ) -> (Box<Finished>, Vec<(Role, String)>) {
        let state = tempfile::tempdir().unwrap();
        let calls = Calls::default();
        let inner = planning_script(planner, critic, calls.clone());
        let script: athena_agent::fake::Script = Box::new(move |spec: &AgentInvocation| {
            let mut turn = inner(spec);
            if spec.role == Role::Reader
                && let Some(reading) = turn.structured.as_mut()
            {
                reading["acceptance_criteria"][0]["kind"] = "behavior".into();
            }
            turn
        });
        let athena = athena_with(
            repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            script,
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let go = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
            .unwrap();
        let GoResult::Finished(finished) = go else {
            panic!("expected the job to finish: {go:?}");
        };
        let calls = calls.lock().unwrap().clone();
        (finished, calls)
    }

    /// A plan for a behavior criterion: a `cargo test` demonstration with a fails_without and this seam.
    fn behavior_plan(seam: serde_json::Value) -> serde_json::Value {
        let mut plan = good_plan();
        plan["demonstrations"] = json!([{
            "criterion": 1,
            "method": "run the suite",
            "command": "cargo test --locked",
            "observes": "the demo test passes",
            "fails_without": "the demo test fails",
            "seam": seam
        }]);
        plan
    }

    fn prompts_of(calls: &[(Role, String)], role: Role) -> Vec<&str> {
        calls
            .iter()
            .filter(|(r, _)| *r == role)
            .map(|(_, p)| p.as_str())
            .collect()
    }

    #[test]
    fn a_mechanical_finding_is_revised_before_the_critic_and_the_critic_sees_round_1() {
        let repo = repo_with_planning(3);
        let (finished, calls) = run_planned(
            &repo,
            vec![bad_plan(), good_plan()],
            vec![accepting_critique()],
        );
        assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
        let roles: Vec<Role> = calls.iter().map(|(r, _)| *r).collect();
        assert_eq!(
            roles,
            [
                Role::Reader,
                Role::Planner,
                Role::Planner,
                Role::Critic,
                Role::Producer,
                Role::Verifier
            ]
        );
        let planner = prompts_of(&calls, Role::Planner);
        assert!(!planner[0].contains("already exists in the base"));
        assert!(
            planner[1].contains("already exists in the base"),
            "the planner is shown Athena's finding: {}",
            planner[1]
        );
        let critic = prompts_of(&calls, Role::Critic);
        assert!(
            critic[0].contains("## Plan under review (round 1)"),
            "the first plan the critic sees is round 1: {}",
            critic[0]
        );
    }

    #[test]
    fn after_a_mechanical_revision_the_critic_still_reviews_in_three_rounds() {
        let repo = repo_with_planning(3);
        let (finished, calls) = run_planned(
            &repo,
            vec![bad_plan(), good_plan()],
            vec![revising_critique()],
        );
        assert_eq!(finished.state, OutcomeState::Failed);
        assert!(
            finished
                .reason
                .starts_with(athena_data::records::PLANNING_STOPPED_PREFIX),
            "{}",
            finished.reason
        );
        assert!(
            finished
                .reason
                .contains(athena_data::records::PLANNING_CAP_REASON),
            "{}",
            finished.reason
        );
        assert!(
            finished.reason.contains("round 3 of 3"),
            "{}",
            finished.reason
        );
        let roles: Vec<Role> = calls.iter().map(|(r, _)| *r).take(8).collect();
        assert_eq!(
            roles,
            [
                Role::Reader,
                Role::Planner,
                Role::Planner,
                Role::Critic,
                Role::Planner,
                Role::Critic,
                Role::Planner,
                Role::Critic
            ]
        );
        assert!(
            !calls
                .iter()
                .any(|(r, _)| matches!(r, Role::Producer | Role::Verifier)),
            "nothing was built"
        );
        let critic = prompts_of(&calls, Role::Critic);
        assert_eq!(critic.len(), 3);
        for (prompt, round) in critic.iter().zip(1..) {
            assert!(
                prompt.contains(&format!("## Plan under review (round {round})")),
                "round {round}: {prompt}"
            );
        }
    }

    #[test]
    fn a_plan_that_still_breaks_a_rule_after_two_mechanical_passes_stops_without_the_critic() {
        let repo = repo_with_planning(4);
        // The critic queue is empty, so any critic call fails the test.
        let (finished, calls) = run_planned(&repo, vec![bad_plan()], vec![]);
        assert_eq!(finished.state, OutcomeState::Failed);
        let reason = &finished.reason;
        assert!(
            reason.starts_with(athena_data::records::PLANNING_STOPPED_PREFIX),
            "{reason}"
        );
        assert!(reason.contains("already exists in the base"), "{reason}");
        assert!(reason.contains("after 2 mechanical revisions"), "{reason}");
        assert!(
            !reason.contains(athena_data::records::PLANNING_CAP_REASON),
            "{reason}"
        );
        // The first plan and two mechanical passes; no critic, producer or verifier.
        assert_eq!(prompts_of(&calls, Role::Planner).len(), 3);
        for role in [Role::Critic, Role::Producer, Role::Verifier] {
            assert!(prompts_of(&calls, role).is_empty(), "{role:?} was called");
        }
    }

    #[test]
    fn a_seamless_behavior_demonstration_is_sent_back_before_the_critic_and_a_seam_is_accepted() {
        let repo = repo_with_planning(3);
        let seam = json!("run_planned with a scripted planner and critic queue; exists");
        let (_, calls) = run_planned_behavior(
            &repo,
            vec![behavior_plan(json!(null)), behavior_plan(seam)],
            vec![accepting_critique()],
        );
        let roles: Vec<Role> = calls.iter().map(|(r, _)| *r).take(4).collect();
        assert_eq!(
            roles,
            [Role::Reader, Role::Planner, Role::Planner, Role::Critic],
            "the seamless plan is revised with no critic call in between: {}",
            calls[3].1
        );
        let planner = prompts_of(&calls, Role::Planner);
        assert!(!planner[0].contains("does not name its test seam"));
        assert!(
            planner[1].contains("does not name its test seam"),
            "the planner is shown the seam finding: {}",
            planner[1]
        );
        assert!(
            planner[1].contains("behavior criterion 1"),
            "the finding names the criterion: {}",
            planner[1]
        );
        let critic = prompts_of(&calls, Role::Critic);
        assert!(
            critic[0].contains("Test seam: run_planned with a scripted planner"),
            "the critic is shown the seam: {}",
            critic[0]
        );
    }

    #[test]
    fn a_behavior_plan_that_never_names_a_seam_stops_planning_without_the_critic() {
        let repo = repo_with_planning(4);
        // A blank seam counts as none. The critic queue is empty, so any critic call fails the test.
        let (finished, calls) =
            run_planned_behavior(&repo, vec![behavior_plan(json!("  "))], vec![]);
        assert_eq!(finished.state, OutcomeState::Failed);
        let reason = &finished.reason;
        assert!(
            reason.starts_with(athena_data::records::PLANNING_STOPPED_PREFIX),
            "{reason}"
        );
        assert!(reason.contains("does not name its test seam"), "{reason}");
        assert!(reason.contains("behavior criterion 1"), "{reason}");
        assert!(reason.contains("after 2 mechanical revisions"), "{reason}");
        assert_eq!(prompts_of(&calls, Role::Planner).len(), 3);
        for role in [Role::Critic, Role::Producer, Role::Verifier] {
            assert!(prompts_of(&calls, role).is_empty(), "{role:?} was called");
        }
    }

    /// `script` with a chosen cost for the reader (read from a cell, so a test can change it between jobs) and
    /// for every planner call. The fake driver enforces no cap, so a turn may cost more than the role's cap.
    fn costing(
        script: athena_agent::fake::Script,
        reader: Arc<Mutex<f64>>,
        planner: f64,
    ) -> athena_agent::fake::Script {
        Box::new(move |spec: &AgentInvocation| {
            let mut turn = script(spec);
            match spec.role {
                Role::Reader => {
                    turn.cost_usd = *reader.lock().unwrap();
                    // A reading of a retry must name the job it retries.
                    if let Some((_, rest)) = spec.user_prompt.split_once("try again on ")
                        && let Some(id) = rest.split_whitespace().next()
                        && let Some(reading) = turn.structured.as_mut()
                    {
                        reading["summary"] = format!("Retry {id}: describe the demo.").into();
                    }
                }
                Role::Planner => turn.cost_usd = planner,
                _ => {}
            }
            turn
        })
    }

    #[test]
    fn round_one_critic_check_follows_the_shared_reserve() {
        // job_usd is 12.00 and the reader costs 0.01, so left at the first critic is 11.99 less the planner.
        // The critic's reserve is this critic, every later round, the producer and the verifier: 8.75 for 3
        // rounds and 5.75 for one. The old check (critic + producer + verifier = 5.75) let the 3-round job
        // call the critic at 5.75 with its later rounds unaffordable.
        let rows: [(u32, f64, bool, &str); 5] = [
            (3, 6.24, false, "$3.00 short"),
            (3, 3.25, false, "$0.01 short"),
            (3, 3.24, true, ""),
            (1, 6.25, false, "$0.01 short"),
            (1, 6.24, true, ""),
        ];
        for (max_rounds, planner_cost, critic_called, shortfall) in rows {
            let repo = repo_with_planning(max_rounds);
            let state = tempfile::tempdir().unwrap();
            let calls = Calls::default();
            let script = costing(
                planning_script(vec![good_plan()], vec![accepting_critique()], calls.clone()),
                Arc::new(Mutex::new(0.01)),
                planner_cost,
            );
            let athena = athena_with(
                &repo,
                &state,
                PASSES,
                crate::test_support::plenty_of_space,
                script,
            );
            let (job, finished) = finished_job(&athena);
            let calls = calls.lock().unwrap().clone();
            let label = format!("{max_rounds} rounds, planner ${planner_cost}");
            assert_eq!(
                prompts_of(&calls, Role::Critic).len(),
                usize::from(critic_called),
                "{label}: {}",
                finished.reason
            );
            if critic_called {
                assert_eq!(finished.state, OutcomeState::Landed, "{label}");
                continue;
            }
            assert_eq!(finished.state, OutcomeState::Failed, "{label}");
            assert!(
                finished
                    .reason
                    .starts_with(athena_data::records::PLANNING_STOPPED_PREFIX),
                "{label}: {}",
                finished.reason
            );
            assert!(
                finished.reason.contains(shortfall),
                "{label}: {}",
                finished.reason
            );
            assert!(
                finished.reason.contains("[budget] job_usd"),
                "{label}: {}",
                finished.reason
            );
            for role in [Role::Producer, Role::Verifier] {
                assert!(prompts_of(&calls, role).is_empty(), "{label}: {role:?}");
            }
            // The plan stands as a record although no critic reviewed it.
            let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
            let stored = &records.jobs[job.as_str()];
            assert_eq!(stored.plans.len(), 1, "{label}");
            assert!(stored.critiques.is_empty(), "{label}");
        }
    }

    #[test]
    fn dispatch_holds_a_job_that_cannot_afford_every_planning_round() {
        // Three rounds are planned: admission needs 10.00 and passes at 12.00. The reader then spends 4.00,
        // so 8.00 is left: enough for the producer and verifier (5.00) but not for three rounds on top
        // (9.50). The job is held before the planner, with the shortfall.
        let repo = repo_with_planning(3);
        let state = tempfile::tempdir().unwrap();
        let calls = Calls::default();
        let script = costing(
            planning_script(vec![good_plan()], vec![accepting_critique()], calls.clone()),
            Arc::new(Mutex::new(4.0)),
            0.01,
        );
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            script,
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        assert!(proposal.reading.is_some());
        let head = athena.head().unwrap();
        let error = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
            .expect_err("the job cannot afford every planning round");
        let text = format!("{error:#}");
        assert!(
            error
                .downcast_ref::<crate::reading::BudgetShort>()
                .is_some(),
            "{text}"
        );
        assert!(text.contains("$1.50 short"), "{text}");
        assert!(text.contains("[budget] job_usd"), "{text}");
        let calls = calls.lock().unwrap().clone();
        let roles: Vec<Role> = calls.iter().map(|(r, _)| *r).collect();
        assert_eq!(roles, [Role::Reader], "no planner, critic or producer ran");
        assert_eq!(athena.head().unwrap(), head, "no prediction commit");
    }

    #[test]
    fn a_retry_is_judged_on_its_own_spend_and_stopped_before_planning() {
        let repo = repo_with_planning(3);
        let state = tempfile::tempdir().unwrap();
        let calls = Calls::default();
        let reader_cost = Arc::new(Mutex::new(0.01));
        let script = costing(
            planning_script(vec![good_plan()], vec![revising_critique()], calls.clone()),
            reader_cost.clone(),
            0.01,
        );
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            script,
        );
        // The funded job runs its planner and critic through every round and fails in planning.
        let (job, finished) = finished_job(&athena);
        assert_eq!(finished.state, OutcomeState::Failed, "{}", finished.reason);
        let planners =
            |calls: &Calls| prompts_of(&calls.lock().unwrap().clone(), Role::Planner).len();
        let ran = planners(&calls);
        assert_eq!(ran, 3);

        // The retry is a new job judged on its own spend, which starts at 0: the old job's spend is not
        // carried over, so it is admitted. Its reader now spends 4.00, which leaves less than the 9.50
        // that three rounds, the producer and the verifier need.
        *reader_cost.lock().unwrap() = 4.0;
        let (proposal, _) = athena.retry(job.as_str(), &Quiet).unwrap();
        assert!(proposal.reading.is_some(), "{:?}", proposal.reader_error);
        let head = athena.head().unwrap();
        let error = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
            .expect_err("the retry cannot afford every planning round");
        let text = format!("{error:#}");
        assert!(
            error
                .downcast_ref::<crate::reading::BudgetShort>()
                .is_some(),
            "{text}"
        );
        assert!(text.contains("$1.50 short"), "{text}");
        assert_eq!(planners(&calls), ran, "the retry reached no planner");
        assert_eq!(athena.head().unwrap(), head);
    }

    const MARKER: &str = "MARKER";

    /// Commits `total` jobs, each with a request, an accepted reading and a job record, as an older history.
    /// The first `marked` readings carry the marker in their title. The ids are dated 2020, so they are older
    /// than any job the test runs.
    fn fixture_reading(id: &JobId, title: &str) -> Reading {
        use athena_data::records::{
            Criterion, CriterionKind, Deliverable, Forecast, Forecasts, Quantiles,
        };
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: id.clone(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: title.into(),
            summary: "Add a greeting.".into(),
            class_of_work: "feature".into(),
            capability: Some("study".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "cargo test --locked passes".into(),
                kind: CriterionKind::Behavior,
            }],
            deliverables: vec![Deliverable {
                path: "src/greeting.rs".into(),
                action: DeliverableAction::Modify,
                p: 0.9,
            }],
            out_of_scope: vec!["the changelog".into()],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    fn seed_accepted_jobs(athena: &Athena, total: usize, marked: usize) {
        use athena_data::records::Request;
        let mut files = Vec::new();
        for n in 0..total {
            let id = JobId::parse(&format!("20200101-{n:06}-{n:04}")).unwrap();
            let request = Request {
                job: id.clone(),
                text: "add a greeting".into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            };
            let title = if n < marked { MARKER } else { "Add a greeting" };
            let reading = fixture_reading(&id, title);
            let job: Job = serde_json::from_value(json!({
                "id": id.as_str(), "kind": "request", "title": "Add a greeting",
                "class_of_work": null, "capability": "study", "accepted_reading": 1,
                "accepted_by": null,
                "owner": { "role": "producer", "capability": null, "model": null, "note": null },
                "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
                "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
                "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
                "tokens": null, "note": null, "pin": null
            }))
            .unwrap();
            files.push(NewFile::record(&AnyRecord::Request(request)));
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(reading))));
            files.push(NewFile::record(&AnyRecord::Job(Box::new(job))));
        }
        athena
            .commit_records(
                &files,
                &message(
                    "Seed accepted readings",
                    "Fixture history for the replay.",
                    &[],
                ),
            )
            .unwrap();
    }

    /// The base rules: the real reading rules.
    fn real_rules() -> std::rc::Rc<dyn ReadingChecker> {
        std::rc::Rc::new(|request: &str, reading: &Reading| {
            Ok(crate::reading::check_reading_text(reading, request))
        })
    }

    /// A candidate-like rule change: the real rules plus a refusal for the marker in the title.
    fn rules_refusing_the_marker() -> std::rc::Rc<dyn ReadingChecker> {
        std::rc::Rc::new(|request: &str, reading: &Reading| {
            Ok(match crate::reading::check_reading_text(reading, request) {
                crate::reading::ReadingCheck::Accept if reading.title.contains(MARKER) => {
                    crate::reading::ReadingCheck::Refuse("the marker rule refuses this".into())
                }
                other => other,
            })
        })
    }

    fn finished_job(athena: &Athena) -> (JobId, Box<Finished>) {
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let GoResult::Finished(finished) = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
            .unwrap()
        else {
            panic!("expected the job to finish");
        };
        (proposal.job, finished)
    }

    #[test]
    fn replay_holds_over_threshold_rejects_the_job_and_records_the_list() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        seed_accepted_jobs(&athena, 10, 3);
        // The seam is read before tree resolution, the builds and the subcommand probe, so the fake
        // driver's "no replay" skip is not reached.
        let _seam = crate::replay_seam::install(real_rules(), rules_refusing_the_marker());
        let (job, finished) = finished_job(&athena);
        assert_eq!(
            finished.state,
            OutcomeState::Rejected,
            "{}",
            finished.reason
        );

        let head = athena.head().unwrap();
        let records = Records::load(&athena.git, &head).unwrap();
        let jr = &records.jobs[job.as_str()];
        let replay = jr.replay.as_ref().expect("replay.json is recorded");
        assert_eq!(replay.no_replay, None);
        assert_eq!(replay.total, 10);
        assert_eq!(replay.newly_refused.len(), 3, "{:?}", replay.newly_refused);
        assert!(replay.held && !replay.intended);
        let listed: Vec<&str> = replay
            .newly_refused
            .iter()
            .map(|r| r.job.as_str())
            .collect();
        assert_eq!(
            listed,
            [
                "20200101-000002-0002",
                "20200101-000001-0001",
                "20200101-000000-0000"
            ]
        );
        // The Athena blocker finding names the same list.
        let verification = jr.verification.as_ref().expect("verification.json");
        let blocker = verification
            .findings
            .iter()
            .find(|f| f.severity == Severity::Blocker)
            .expect("a blocker finding");
        assert_eq!(blocker.source, FindingSource::Athena);
        let text = format!("{} {}", blocker.summary, blocker.evidence);
        for id in &listed {
            assert!(text.contains(id), "{id}: {text}");
        }
        assert_eq!(verification.final_verdict, Verdict::Fail);
        // The records check accepts every file of the job directory, and no free-form replay.md exists.
        let dir = job.dir();
        let names = athena
            .git
            .run([
                "ls-tree",
                "-r",
                "--name-only",
                head.as_str(),
                "--",
                dir.as_str(),
            ])
            .unwrap();
        let paths: Vec<&str> = names.lines().collect();
        assert!(
            paths.iter().any(|p| p.ends_with("/replay.json")),
            "{paths:?}"
        );
        assert!(!paths.iter().any(|p| p.ends_with("replay.md")), "{paths:?}");
        for path in &paths {
            let bytes = athena.git.show_file(&head, path).unwrap().unwrap();
            athena_data::records::check_record_file(path, &bytes)
                .unwrap_or_else(|e| panic!("{path}: {e}"));
        }
    }

    #[test]
    fn replay_view_carries_the_declared_share_and_reason() {
        let id = JobId::parse("20200101-000000-0000").unwrap();
        let record = ReplayRun {
            total: 50,
            newly_refused: vec![athena_data::records::ReplayRefusal {
                job: id.clone(),
                reason: "the marker rule refuses this".into(),
            }],
            intended: true,
            held: false,
            no_replay: None,
        }
        .record(&id, "a", "b");
        let reason = "readers now ask for a new required section, because past readings lack it";
        let mut reading = fixture_reading(&id, "Add a greeting");
        reading.acceptance_criteria[0].text = "expected 80% of newly refused readings".into();
        reading.out_of_scope = vec!["the changelog".into(), reason.into()];
        // A share above 25% with a reason: the view carries both, as the verifier brief site passes them.
        let view = replay_view(&record, &reading);
        assert_eq!(view.declared, Some(80));
        assert_eq!(view.reason.as_deref(), Some(reason));
        assert_eq!(view.unsupported, None);
        assert_eq!((view.total, view.newly_refused.len()), (50, 1));
        // The same share with no reason is unsupported, and the 10% limit applied.
        reading.out_of_scope = vec!["the changelog".into()];
        let view = replay_view(&record, &reading);
        assert_eq!(
            (view.declared, view.reason, view.unsupported),
            (None, None, Some(80))
        );
        // A reading that declares nothing leaves all three empty.
        let plain = fixture_reading(&id, "Add a greeting");
        let view = replay_view(&record, &plain);
        assert_eq!(
            (view.declared, view.reason, view.unsupported),
            (None, None, None)
        );
    }

    #[test]
    fn replay_within_threshold_records_the_list_and_lands() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        seed_accepted_jobs(&athena, 10, 1);
        let _seam = crate::replay_seam::install(real_rules(), rules_refusing_the_marker());
        let (job, finished) = finished_job(&athena);
        assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let replay = records.jobs[job.as_str()].replay.clone().expect("a replay");
        assert_eq!((replay.total, replay.newly_refused.len()), (10, 1));
        assert!(!replay.held);
    }

    /// Wraps `script` to log the role of every invocation it receives, in order.
    fn role_log(
        script: athena_agent::fake::Script,
        log: Arc<Mutex<Vec<Role>>>,
    ) -> athena_agent::fake::Script {
        Box::new(move |spec: &AgentInvocation| {
            log.lock().unwrap().push(spec.role);
            script(spec)
        })
    }

    /// Runs one job over a seeded history with this candidate rule; returns the roles invoked up to the
    /// verifier, the (seq, role) invocation records up to the verifier, and the replay record.
    fn run_with_candidate(
        candidate: std::rc::Rc<dyn ReadingChecker>,
    ) -> (Vec<Role>, Vec<(u32, Role)>, Replay) {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            role_log(base_script(), log.clone()),
        );
        seed_accepted_jobs(&athena, 10, 3);
        let _seam = crate::replay_seam::install(real_rules(), candidate);
        let (job, _) = finished_job(&athena);
        let roles = log.lock().unwrap().clone();
        let upto = roles
            .iter()
            .position(|r| *r == Role::Verifier)
            .expect("the verifier ran");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let jr = &records.jobs[job.as_str()];
        let mut invocations: Vec<(u32, Role)> =
            jr.invocations.iter().map(|i| (i.seq, i.role)).collect();
        invocations.sort();
        invocations.truncate(upto + 1);
        (
            roles[..=upto].to_vec(),
            invocations,
            jr.replay.clone().expect("a replay"),
        )
    }

    #[test]
    fn replay_makes_no_model_call_with_or_without_a_candidate_rule_change() {
        // Run one: the candidate rules equal the base. Run two: the candidate adds a refusal, so the job is held.
        let (plain_roles, plain_records, plain) = run_with_candidate(real_rules());
        let (changed_roles, changed_records, changed) =
            run_with_candidate(rules_refusing_the_marker());
        assert!(!plain.held && plain.newly_refused.is_empty());
        assert!(changed.held && changed.newly_refused.len() == 3);
        // The replay stage runs before the verifier, so a model call it made would put a role here.
        assert_eq!(
            plain_roles,
            [Role::Reader, Role::Producer, Role::Verifier],
            "{plain_roles:?}"
        );
        assert_eq!(plain_roles, changed_roles);
        assert_eq!(plain_records, changed_records);
        assert_eq!(
            plain_records.iter().map(|(_, r)| *r).collect::<Vec<_>>(),
            [Role::Reader, Role::Producer, Role::Verifier]
        );
    }

    #[test]
    fn replay_makes_no_model_call_because_the_checker_is_built_from_a_path_alone() {
        // `BinaryChecker::new` takes a path: there is no driver or agent handle to call. Run on a path that
        // is no program, it fails to spawn, which is all it can do.
        let checker = BinaryChecker::new(PathBuf::from("/nonexistent/athena"));
        let id = JobId::parse("20200101-000000-0000").unwrap();
        let reading = fixture_reading(&id, "Add a greeting");
        assert!(checker.check("add a greeting", &reading).is_err());
    }

    /// How many times each agent role was invoked.
    #[derive(Default)]
    struct Counts {
        planner: std::sync::atomic::AtomicUsize,
        producer: std::sync::atomic::AtomicUsize,
        reader: std::sync::atomic::AtomicUsize,
        verifier: std::sync::atomic::AtomicUsize,
    }

    impl Counts {
        fn counter(&self, role: Role) -> Option<&std::sync::atomic::AtomicUsize> {
            match role {
                Role::Planner => Some(&self.planner),
                Role::Producer => Some(&self.producer),
                Role::Reader => Some(&self.reader),
                Role::Verifier => Some(&self.verifier),
                _ => None,
            }
        }

        /// The (planner, producer, reader, verifier) counts so far.
        fn all(&self) -> (usize, usize, usize, usize) {
            let get =
                |c: &std::sync::atomic::AtomicUsize| c.load(std::sync::atomic::Ordering::SeqCst);
            (
                get(&self.planner),
                get(&self.producer),
                get(&self.reader),
                get(&self.verifier),
            )
        }
    }

    /// The base script, counting each planner, producer, reader and verifier invocation. When `state_root` is
    /// given, the producer also leaves a file in its job's build directory (`target/produce`), as a build would.
    fn counting(counts: Arc<Counts>, state_root: Option<PathBuf>) -> athena_agent::fake::Script {
        let inner = base_script();
        Box::new(move |spec: &AgentInvocation| {
            if let Some(counter) = counts.counter(spec.role) {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            if spec.role == Role::Producer
                && let Some(root) = &state_root
            {
                let dir = root
                    .join("jobs")
                    .join(&spec.job)
                    .join("target/produce/debug");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("big"), vec![7u8; 20_000]).unwrap();
            }
            let mut turn = inner(spec);
            // A reading of a retry must name the job it retries, as a reader that read the request would.
            if spec.role == Role::Reader
                && let Some((_, after)) = spec.user_prompt.split_once("try again on ")
                && let Some(structured) = turn.structured.as_mut()
            {
                let earlier: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                structured["summary"] = json!(format!(
                    "Describe the demo in README.md, as job {earlier} did."
                ));
            }
            turn
        })
    }

    fn finished(result: GoResult) -> Finished {
        match result {
            GoResult::Finished(finished) => *finished,
            GoResult::Reread(_) => panic!("expected the job to finish"),
        }
    }

    fn readme_at_head(athena: &Athena) -> String {
        let bytes = athena.git.show_file("HEAD", "README.md").unwrap().unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn produce_dir_is_removed_after_checks_pass_and_kept_when_a_check_fails() {
        for (check, passes) in [(PASSES, true), (FAILS_ON_EDIT, false)] {
            let repo = repo();
            let state = tempfile::tempdir().unwrap();
            let root = state.path().join("repo");
            let counts = Arc::new(Counts::default());
            let athena = athena_with(
                &repo,
                &state,
                check,
                crate::test_support::recording_plenty_space,
                counting(counts, Some(root.clone())),
            );
            let proposal = athena.ask("describe the demo", &Quiet).unwrap();
            let ui = Lines::default();
            let done = finished(
                athena
                    .go(proposal.job.as_str(), AcceptedBy::User, &ui)
                    .unwrap(),
            );

            // Every read of the free space saw the job directories as they were at that moment. The first read
            // with the verifier's snapshot present is the gate before the verifier; the one before it is the
            // gate before the checks.
            let probes = crate::test_support::probes_of(&root);
            let gate = probes
                .iter()
                .position(|p| p.verifying)
                .expect("the gate before the verifier read the free space");
            assert!(gate > 0, "{probes:?}");
            assert!(
                probes[gate - 1].produce_present,
                "the producer's build directory existed when the checks started: {probes:?}"
            );
            if passes {
                assert!(
                    !probes[gate].produce_present,
                    "the checks passed, so target/produce is gone before the verifier gate: {probes:?}"
                );
                assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
            } else {
                assert!(
                    probes[gate].produce_present,
                    "a check failed, so target/produce is kept: {probes:?}"
                );
                assert_eq!(done.state, OutcomeState::Rejected, "{}", done.reason);
            }
        }
    }

    /// A check result as a run killed at its deadline leaves it: no exit code, the deadline note, not passed.
    fn killed_at_the_deadline(spec: &CheckSpec, took_s: f64, limit_s: u64) -> CheckRun {
        CheckRun {
            name: spec.name.clone(),
            command: spec.run.clone(),
            exit_code: None,
            passed: false,
            tests_ran: None,
            duration: athena_data::Quantity::measured(
                took_s,
                athena_data::Unit::Seconds,
                "athena monotonic clock",
            ),
            output_tail: String::new(),
            note: Some(crate::checks::deadline_note(limit_s)),
        }
    }

    #[test]
    fn a_check_killed_at_its_deadline_closes_as_an_environment_stop() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let counts = Arc::new(Counts::default());
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            counting(counts.clone(), None),
        );
        // The limit is the one athena.toml sets; the run took a little longer than it.
        let limit = JobConfig::at(&athena.git, "HEAD")
            .unwrap()
            .config
            .budget
            .check_timeout_s;
        assert_eq!(limit, 1800);
        let _seam = check_seam::install(std::rc::Rc::new(move |spec: &CheckSpec| {
            killed_at_the_deadline(spec, limit as f64 + 5.2, limit)
        }));
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let ui = Lines::default();
        let done = finished(
            athena
                .go(proposal.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Failed, "{}", done.reason);
        assert!(
            crate::disk::is_environment_reason(&done.reason),
            "{}",
            done.reason
        );
        assert!(
            crate::disk::resumable_stop(
                crate::store::JobState::Closed(done.state),
                Some(&done.reason)
            ),
            "{}",
            done.reason
        );
        assert!(
            done.reason.contains("1805s of 1800s limit"),
            "{}",
            done.reason
        );
        let said = ui.0.lock().unwrap().clone();
        assert!(
            said.iter()
                .any(|l| l.starts_with("failed     ") && l.contains("1805s of 1800s limit")),
            "{said:#?}"
        );
        // The verifier never ran: the stop came before it.
        let (planner, producer, _, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
    }

    #[test]
    fn a_real_nonzero_check_failure_is_still_rejected() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let counts = Arc::new(Counts::default());
        // No seam: the check is a real process that exits 1 once the producer edits the README.
        let athena = athena_with(
            &repo,
            &state,
            FAILS_ON_EDIT,
            crate::test_support::plenty_of_space,
            counting(counts.clone(), None),
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let ui = Lines::default();
        let done = finished(
            athena
                .go(proposal.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Rejected, "{}", done.reason);
        assert!(
            !crate::disk::is_environment_reason(&done.reason),
            "{}",
            done.reason
        );
        // The verifier ran: a failed check is judged, not held.
        assert_eq!(counts.all().3, 1);
    }

    #[test]
    fn low_disk_before_the_verifier_resumes_without_planner_or_producer() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("repo");
        let counts = Arc::new(Counts::default());
        // Enough free space for the gate before the checks; too little for the gate before the verifier.
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::low_once_verifying,
            counting(counts.clone(), Some(root.clone())),
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        let ui = Lines::default();
        let stopped = finished(athena.go(first.as_str(), AcceptedBy::User, &ui).unwrap());
        assert_eq!(stopped.state, OutcomeState::Failed, "{}", stopped.reason);
        assert!(
            crate::disk::is_environment_reason(&stopped.reason),
            "{}",
            stopped.reason
        );
        assert!(
            stopped.reason.contains("before the verifier"),
            "{}",
            stopped.reason
        );
        let said = ui.0.lock().unwrap().clone();
        assert!(
            said.iter().any(|l| l.starts_with("checks     ")),
            "{said:#?}"
        );
        // The producer ran once; no planner, and no verifier.
        let (planner, producer, read_before, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_before >= 1);
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let candidate_fingerprint = records.jobs[first.as_str()]
            .candidate
            .as_ref()
            .and_then(|c| c.fingerprint.clone())
            .expect("the stopped job recorded its candidate");
        // The job's directory is gone, but its candidate is kept, at the stage that stopped.
        assert!(!athena.state.job_dir(&first).exists());
        assert_eq!(
            athena.state.read_resume(&first).map(|m| m.stage),
            Some(crate::state::ResumeStage::Verifier)
        );

        // Space is back. The retry reads again (the reader runs), then resumes.
        std::fs::write(root.join("freed"), b"").unwrap();
        let ui = Lines::default();
        let (retry, _) = athena.retry(first.as_str(), &ui).unwrap();
        assert_eq!(retry.reader_error, None);
        assert!(
            ui.0.lock()
                .unwrap()
                .iter()
                .any(|l| l.starts_with("resume     ")),
            "{:#?}",
            ui.0.lock().unwrap()
        );
        let (planner, producer, read_after, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_after > read_before, "the reader ran again");
        let ui = Lines::default();
        let done = finished(
            athena
                .go(retry.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
        // No planner and no producer ran in the retry; only the verifier did.
        assert_eq!(counts.all(), (0, 1, read_after, 1));
        let said = ui.0.lock().unwrap().clone();
        assert!(
            !said.iter().any(|l| l.starts_with("checks     ")),
            "the retry resumed at the verifier: {said:#?}"
        );
        assert!(
            said.iter().any(|l| l.starts_with("verifier   ")),
            "{said:#?}"
        );
        assert_eq!(
            done.verification.as_ref().unwrap().verified_fingerprint,
            candidate_fingerprint,
            "the verifier judged the kept candidate"
        );
        assert!(readme_at_head(&athena).contains("A demo."));
        assert_eq!(athena.state.read_resume(&first), None, "the mark is spent");
        assert_eq!(athena.state.read_resume(&retry.job), None);
    }

    #[test]
    fn a_retry_reader_brief_carries_the_earlier_jobs_files_approach_and_checks() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        // The base script, keeping every prompt the reader is given.
        let prompts: Arc<Mutex<Vec<String>>> = Arc::default();
        let inner = counting(Arc::new(Counts::default()), None);
        let kept = prompts.clone();
        let script: athena_agent::fake::Script = Box::new(move |spec: &AgentInvocation| {
            if spec.role == Role::Reader {
                kept.lock().unwrap().push(spec.user_prompt.clone());
            }
            inner(spec)
        });
        let athena = athena_with(
            &repo,
            &state,
            FAILS_ON_EDIT,
            crate::test_support::plenty_of_space,
            script,
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        let rejected = finished(
            athena
                .go(first.as_str(), AcceptedBy::User, &Lines::default())
                .unwrap(),
        );
        assert_eq!(
            rejected.state,
            OutcomeState::Rejected,
            "{}",
            rejected.reason
        );

        // The retry's reader is shown what the rejected job did, from its candidate and verification.
        let (retry, _) = athena.retry(first.as_str(), &Quiet).unwrap();
        assert_eq!(retry.reader_error, None);
        let prompts = prompts.lock().unwrap();
        let brief = prompts.last().unwrap();
        let section = brief
            .split_once(athena_context::briefs::PRIOR_RESULT_HEADING)
            .unwrap_or_else(|| panic!("no section:\n{brief}"))
            .1;
        for needle in [
            "Files its candidate changed (1)",
            "\n> modified README.md\n",
            "\n> Described the demo.\n",
            "Measured checks:\n> probe: failed (exit code 1",
        ] {
            assert!(section.contains(needle), "{needle:?}:\n{section}");
        }
    }

    /// A UI that dies, as a process does, when the checks start.
    struct DiesAtChecks;

    impl Ui for DiesAtChecks {
        fn line(&self, text: &str) {
            if text.starts_with("checks     ") {
                panic!("the process died");
            }
        }
    }

    #[test]
    fn interrupted_job_with_a_candidate_resumes_without_planner_or_producer() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let counts = Arc::new(Counts::default());
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            counting(counts.clone(), None),
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        // The process dies after the candidate is built, as the checks start: the job is left dispatched.
        let died = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            athena.go(first.as_str(), AcceptedBy::User, &DiesAtChecks)
        }));
        assert!(died.is_err(), "the job was cut short");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        assert_eq!(records.jobs[first.as_str()].state(), JobState::Dispatched);
        assert!(!athena.running(first.as_str()));
        let (planner, producer, read_before, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        let candidate_fingerprint = athena
            .state
            .read_resume(&first)
            .and_then(|m| m.candidate.fingerprint)
            .expect("the candidate was kept");

        let (retry, _) = athena.retry(first.as_str(), &Quiet).unwrap();
        assert_eq!(retry.reader_error, None);
        let (planner, producer, read_after, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_after > read_before, "the reader ran again");
        let ui = Lines::default();
        let done = finished(
            athena
                .go(retry.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
        // No planner and no producer ran in the retry. The checks had not finished, so they ran in it.
        assert_eq!(counts.all(), (0, 1, read_after, 1));
        let said = ui.0.lock().unwrap().clone();
        assert!(
            said.iter().any(|l| l.starts_with("resume     ")),
            "{said:#?}"
        );
        assert!(
            said.iter().any(|l| l.starts_with("checks     ")),
            "{said:#?}"
        );
        assert_eq!(
            done.verification.as_ref().unwrap().verified_fingerprint,
            candidate_fingerprint
        );
        assert!(readme_at_head(&athena).contains("A demo."));
        assert_eq!(athena.state.read_resume(&first), None, "the mark is spent");
    }
}
