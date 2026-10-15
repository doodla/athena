//! Running one agent and recording it (design §10.2, M14).
//!
//! What an invocation used has one definition, shared with the gate's reconciliation rule: the requested
//! model's `modelUsage` tokens (`athena_agent::stream::ResultEvent::requested_usage`). The result's
//! top-level `usage` also counts subagents and is not what is recorded.
//!
//! Every invocation becomes an `athena.invocation/1` record, whatever happens: completed invocations
//! record measured tokens and the reported cost; killed ones record the de-duplicated running sums as a
//! lower bound and a cost bounded by the invocation's cap. Agent free text (commands, denials, details) is
//! redacted before it is recorded. The record is mirrored to the state directory at once, so `athena
//! close` can commit it if this process dies before the closing commit.

use crate::Athena;
use crate::state::StateDir;
use anyhow::Result;
use athena_agent::{AgentInvocation, AgentOutput, DriverError, Usage, UsageSink};
use athena_context::Composed;
use athena_data::records::{
    AnyRecord, Denial, Invocation, InvocationCaps, InvocationContext, InvocationOutcome, Tokens,
};
use athena_data::{Config, JobId, Quantity, Unit};
use athena_gate::redact::Redactor;
use serde::{Deserialize, Serialize};

/// The running usage of an invocation, persisted while it runs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PartialUsage {
    pub seq: u32,
    pub role: String,
    pub model: String,
    pub cap_usd: f64,
    pub timeout_s: u64,
    pub started_at: Option<jiff::Timestamp>,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub commands: Vec<String>,
}

/// A sink that writes the running sums to the state directory after every event.
#[derive(Debug)]
pub struct FileSink {
    path: std::path::PathBuf,
    partial: PartialUsage,
}

impl FileSink {
    pub fn new(state: &StateDir, job: &JobId, partial: PartialUsage) -> FileSink {
        let path = state.partial_usage_file(job, partial.seq);
        let sink = FileSink { path, partial };
        sink.persist();
        sink
    }

    fn persist(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string(&self.partial) {
            let _ = std::fs::write(&self.path, text);
        }
    }

    /// Marks the invocation as recorded, so `close` does not count it twice.
    pub fn finish(self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl UsageSink for FileSink {
    fn update(&mut self, usage: &Usage, commands_run: &[String]) {
        self.partial.input = usage.input;
        self.partial.output = usage.output;
        self.partial.cache_read = usage.cache_read;
        self.partial.cache_write = usage.cache_write;
        self.partial.commands = commands_run.to_vec();
        self.persist();
    }
}

fn tokens_measured(usage: &Usage, basis: &str) -> Tokens {
    let q = |v: u64| Quantity::measured(v as f64, Unit::Tokens, basis);
    Tokens {
        input: q(usage.input),
        output: q(usage.output),
        cache_read: q(usage.cache_read),
        cache_write: q(usage.cache_write),
    }
}

/// Token counts known only as a lower bound.
pub fn tokens_lower_bound(usage: &Usage, basis: &str) -> Tokens {
    let q = |v: u64| {
        Quantity::estimated(Some(v as f64), Unit::Tokens, basis, Some(v as f64), None)
            .expect("non-negative")
    };
    Tokens {
        input: q(usage.input),
        output: q(usage.output),
        cache_read: q(usage.cache_read),
        cache_write: q(usage.cache_write),
    }
}

/// A cost known only to lie between nothing and the invocation's cap.
pub fn cost_bounded_by_cap(cap: f64, why: &str) -> Quantity {
    Quantity::estimated(
        None,
        Unit::Usd,
        format!("{why}; no billing evidence"),
        Some(0.0),
        Some(cap),
    )
    .expect("ordered bounds")
}

/// The host's injected context, as measured at design time.
pub fn host_baseline(config: &Config) -> Quantity {
    Quantity::estimated(
        Some(config.agents.host_baseline_tokens as f64),
        Unit::Tokens,
        "measured once with the pinned host and a role's structured-output tool (athena.toml [agents]); not measured per invocation",
        None,
        None,
    )
    .expect("non-negative")
}

/// What the caller knows about an invocation besides its output.
#[derive(Debug)]
pub struct InvocationFacts<'a> {
    pub job: &'a JobId,
    pub spec: &'a AgentInvocation,
    pub composed: &'a Composed,
    pub config: &'a Config,
    pub fake: bool,
}

impl Athena {
    /// Turns an agent's output into its record, redacting free text.
    pub fn invocation_record(
        &self,
        facts: &InvocationFacts<'_>,
        output: &AgentOutput,
        redactor: &Redactor,
    ) -> Result<Invocation> {
        let spec = facts.spec;
        let mut free_text: Vec<String> = output.commands_run.clone();
        let commands = free_text.len();
        free_text.extend(output.denials.iter().map(|d| d.detail.clone()));
        free_text.push(output.detail.clone().unwrap_or_default());
        redactor.redact_all(&mut free_text)?;
        let detail = free_text.pop().filter(|d| !d.is_empty());
        let denial_details = free_text.split_off(commands);
        let commands_run = free_text;
        let policy_denials: Vec<Denial> = output
            .denials
            .iter()
            .zip(denial_details)
            .map(|(d, detail)| Denial {
                tool: d.tool.clone(),
                detail,
            })
            .collect();

        let measured_basis = if facts.fake {
            "fake driver: no model call"
        } else {
            "Claude Code result.modelUsage for the requested model"
        };
        let tokens = if output.usage_measured {
            tokens_measured(&output.usage, measured_basis)
        } else {
            tokens_lower_bound(&output.usage, "lower bound: usage before termination")
        };
        let cost = match output.cost_usd {
            Some(cost) => Quantity::reported(
                cost,
                Unit::Usd,
                if facts.fake {
                    "fake driver: no model call"
                } else {
                    "Claude Code total_cost_usd, at list price"
                },
            ),
            None => cost_bounded_by_cap(
                spec.max_budget_usd,
                output.detail.as_deref().unwrap_or("ended without a result"),
            ),
        };
        Ok(Invocation {
            job: facts.job.clone(),
            seq: spec.seq,
            role: spec.role,
            driver: self.driver.info(),
            athena: self.build.clone(),
            model: spec.model.clone(),
            context: InvocationContext {
                components: facts.composed.components.clone(),
                prompt_sha256: facts.composed.sha256(),
                prompt_tokens: facts.composed.tokens(),
                host_baseline: host_baseline(facts.config),
                tools: spec.tools.clone(),
            },
            caps: InvocationCaps {
                max_budget_usd: spec.max_budget_usd,
                timeout_s: spec.timeout.as_secs(),
            },
            started_at: output.started_at,
            ended_at: output.ended_at,
            wall_time: Quantity::measured(output.wall_s, Unit::Seconds, "athena monotonic clock"),
            tokens,
            models_used: output.models_used.clone(),
            cost,
            outcome: output.outcome,
            detail,
            turns: output.turns,
            commands_run,
            policy_denials,
        })
    }

    /// Runs one agent and returns its output and record. The record is mirrored to the state directory
    /// before this returns. A driver error (the kill switch, a nested run) means no model call happened.
    pub fn invoke_and_record(
        &self,
        facts: &InvocationFacts<'_>,
        redactor: &Redactor,
    ) -> Result<std::result::Result<(AgentOutput, Invocation), DriverError>> {
        let spec = facts.spec;
        let mut sink = FileSink::new(
            &self.state,
            facts.job,
            PartialUsage {
                seq: spec.seq,
                role: spec.role.as_str().into(),
                model: spec.model.clone(),
                cap_usd: spec.max_budget_usd,
                timeout_s: spec.timeout.as_secs(),
                started_at: Some(crate::now()),
                ..PartialUsage::default()
            },
        );
        let output = match self.driver.invoke(spec, &mut sink) {
            Ok(output) => output,
            Err(error) => {
                sink.finish();
                return Ok(Err(error));
            }
        };
        let record = self.invocation_record(facts, &output, redactor)?;
        let any = AnyRecord::Invocation(Box::new(record.clone()));
        self.state
            .write_live(facts.job, &any.canonical_path(), any.to_json().as_bytes())?;
        sink.finish();
        Ok(Ok((output, record)))
    }
}

/// Records an invocation whose process died with this command: lower-bound tokens from its persisted
/// running sums, and a cost bounded by its cap.
pub fn interrupted_record(
    athena: &Athena,
    job: &JobId,
    partial: &PartialUsage,
    config: &Config,
) -> Invocation {
    let usage = Usage {
        input: partial.input,
        output: partial.output,
        cache_read: partial.cache_read,
        cache_write: partial.cache_write,
    };
    let at = partial.started_at.unwrap_or_else(crate::now);
    Invocation {
        job: job.clone(),
        seq: partial.seq,
        role: athena_data::records::Role::parse(&partial.role)
            .unwrap_or(athena_data::records::Role::Producer),
        driver: athena.driver.info(),
        athena: athena.build.clone(),
        model: partial.model.clone(),
        context: InvocationContext {
            components: Vec::new(),
            prompt_sha256: String::new(),
            prompt_tokens: Quantity::estimated(
                None,
                Unit::Tokens,
                "not recorded: the process died",
                Some(0.0),
                None,
            )
            .expect("bounds"),
            host_baseline: host_baseline(config),
            tools: Vec::new(),
        },
        caps: InvocationCaps {
            max_budget_usd: partial.cap_usd,
            timeout_s: partial.timeout_s,
        },
        started_at: at,
        ended_at: at,
        wall_time: Quantity::estimated(
            None,
            Unit::Seconds,
            "the process died; no end time",
            Some(0.0),
            None,
        )
        .expect("bounds"),
        tokens: tokens_lower_bound(
            &usage,
            "lower bound: usage persisted before the process died",
        ),
        models_used: Default::default(),
        cost: cost_bounded_by_cap(
            partial.cap_usd,
            "the process died before the invocation completed",
        ),
        outcome: InvocationOutcome::Killed,
        detail: Some("recorded by `athena close`: the process running this invocation died".into()),
        turns: None,
        commands_run: Vec::new(),
        policy_denials: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::CountingPinner;
    use athena_agent::claude::output_from_result;
    use athena_agent::fake::FakeAgentDriver;
    use athena_agent::stream::StreamState;
    use athena_data::records::{Build, Role};
    use athena_gate::reconcile::check_invocation;
    use athena_gate::{Rule, Violation};

    const MODEL: &str = "claude-sonnet-5-5";

    /// A result event: the main thread plus a subagent used the requested model, so the top-level totals
    /// (90/52656/3634696/134508) exceed the requested model's own `modelUsage` (6/3136/275219/3821).
    const AGREEING: &str = r#"{"type":"result","subtype":"success","is_error":false,"num_turns":4,"duration_api_ms":900,"total_cost_usd":0.30,"usage":{"input_tokens":90,"output_tokens":134508,"cache_read_input_tokens":3634696,"cache_creation_input_tokens":52656},"modelUsage":{"claude-sonnet-5-5":{"inputTokens":6,"outputTokens":3821,"cacheReadInputTokens":275219,"cacheCreationInputTokens":3136,"costUSD":0.10},"claude-haiku-5-5":{"inputTokens":84,"outputTokens":130687,"cacheReadInputTokens":3359477,"cacheCreationInputTokens":49520,"costUSD":0.20}},"permission_denials":[]}"#;

    /// The same event, except that `total_cost_usd` (0.75) disagrees with the per-model `costUSD` sum
    /// (0.10 + 0.20 = 0.30). The tokens are left alone on purpose: under the `modelUsage` definition the
    /// record's tokens are the requested model's `modelUsage` entry, so a token disagreement cannot arise
    /// through the parser. The cost is the only figure that can disagree end to end.
    const DISAGREEING: &str = r#"{"type":"result","subtype":"success","is_error":false,"num_turns":4,"duration_api_ms":900,"total_cost_usd":0.75,"usage":{"input_tokens":90,"output_tokens":134508,"cache_read_input_tokens":3634696,"cache_creation_input_tokens":52656},"modelUsage":{"claude-sonnet-5-5":{"inputTokens":6,"outputTokens":3821,"cacheReadInputTokens":275219,"cacheCreationInputTokens":3136,"costUSD":0.10},"claude-haiku-5-5":{"inputTokens":84,"outputTokens":130687,"cacheReadInputTokens":3359477,"cacheCreationInputTokens":49520,"costUSD":0.20}},"permission_denials":[]}"#;

    /// Parses a result event and builds the invocation record, as a real invocation does.
    fn record_from(event: &str) -> Invocation {
        let state_dir = tempfile::tempdir().unwrap();
        let athena = Athena {
            git: athena_git::Git::at(state_dir.path()),
            state: StateDir::new(state_dir.path().join("repo")),
            driver: std::sync::Arc::new(FakeAgentDriver::deterministic()),
            capabilities: vec![],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(CountingPinner::new(&"0".repeat(40))),
            jev: None,
        };
        let config = Config::parse(include_str!("../../../athena.toml")).unwrap();
        let job = JobId::parse("20261010-153012-k7q2").unwrap();
        let spec = AgentInvocation {
            job: job.as_str().into(),
            seq: 1,
            role: Role::Producer,
            model: MODEL.into(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            tools: vec![],
            allowed_tools: vec![],
            permission_mode: "dontAsk".into(),
            workdir: state_dir.path().into(),
            env: Default::default(),
            hook: None,
            max_budget_usd: 3.0,
            timeout: std::time::Duration::from_secs(60),
            json_schema: serde_json::json!({}),
            scratch: state_dir.path().into(),
        };
        let composed = athena_context::compose(&[]);
        let facts = InvocationFacts {
            job: &job,
            spec: &spec,
            composed: &composed,
            config: &config,
            fake: false,
        };
        let mut stream = StreamState::default();
        stream.on_line(event);
        let result = stream.result.expect("the event parses as a result");
        let now = crate::now();
        let output = output_from_result(&result, MODEL, now, now, 1.0, vec![]);
        let redactor = athena.redactor(&config, Some(&job)).unwrap();
        athena
            .invocation_record(&facts, &output, &redactor)
            .unwrap()
    }

    fn reconciliation(violations: &[Violation]) -> Vec<&Violation> {
        violations
            .iter()
            .filter(|v| v.rule == Rule::Reconciliation)
            .collect()
    }

    fn measured_tokens(record: &Invocation) -> [f64; 4] {
        let t = &record.tokens;
        [&t.input, &t.output, &t.cache_read, &t.cache_write].map(|q| q.value().unwrap())
    }

    #[test]
    fn a_result_whose_top_level_usage_includes_a_subagent_reconciles_end_to_end() {
        let record = record_from(AGREEING);
        // The top-level usage differs from modelUsage, and the record follows modelUsage.
        assert_eq!(measured_tokens(&record), [6.0, 3821.0, 275219.0, 3136.0]);
        assert_ne!(measured_tokens(&record)[0], 90.0);
        assert_eq!(check_invocation("x", &record), vec![]);
    }

    /// Builds the record through the same parse and record path as the positive test, from an event whose
    /// `total_cost_usd` really differs from the `costUSD` sum. Token disagreement cannot arise through the
    /// parser under the `modelUsage` definition (the tokens come from the requested model's entry), so the
    /// cost is the disagreement covered here. No record is edited by hand.
    #[test]
    fn a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end() {
        let record = record_from(DISAGREEING);
        // The input is known to be bad: modelUsage is present for the requested model, and the reported
        // total differs from the per-model sum.
        assert!(record.models_used.contains_key(MODEL));
        let per_model: f64 = record.models_used.values().map(|u| u.cost_usd).sum();
        let reported = record.cost.value().unwrap();
        assert!(
            (reported - per_model).abs() > 0.01,
            "reported {reported} must differ from the modelUsage sum {per_model}"
        );
        let violations = check_invocation("x", &record);
        let found = reconciliation(&violations);
        assert_eq!(found.len(), 1, "{violations:?}");
        assert!(found[0].detail.contains("per-model costs sum"), "{found:?}");
    }
}
