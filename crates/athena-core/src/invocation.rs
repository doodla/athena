//! Running one agent and recording it (design §10.2, M14).
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
            "Claude Code result.usage"
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
