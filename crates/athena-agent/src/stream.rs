//! Reading Claude Code's `stream-json` output as it arrives (design §10.2, E16).
//!
//! - Usage arrives on `assistant` events. Several events of one API message repeat its usage, so usage is
//!   de-duplicated by message id, keeping the largest counts seen for each.
//! - Per-message output counts undercount the final figure, so running sums are a lower bound; the final
//!   `result` event carries the measured totals, per-model usage, and the reported cost.
//! - Message content is discarded. Only tool names, Bash command lines, denials, and the structured
//!   output are kept, so no transcript exists.

use crate::Usage;
use athena_data::records::{Denial, ModelUsage};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The final `result` event, reduced to what Athena records.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResultEvent {
    pub is_error: bool,
    pub subtype: String,
    pub usage: Usage,
    pub models_used: BTreeMap<String, ModelUsage>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u32>,
    pub duration_api_ms: Option<u64>,
    pub structured: Option<Value>,
    pub denied_tool_use_ids: BTreeSet<String>,
    pub denials: Vec<Denial>,
    pub text: Option<String>,
}

impl ResultEvent {
    /// What the invocation used, by the one definition the gate reconciles against: the requested model's
    /// `modelUsage` tokens. The result's top-level `usage` also counts subagents on other terms, so it is
    /// used only when `modelUsage` has no entry for the model.
    pub fn requested_usage(&self, model: &str) -> Usage {
        match self.models_used.get(model) {
            Some(u) => Usage {
                input: u.input,
                output: u.output,
                cache_read: u.cache_read,
                cache_write: u.cache_write,
            },
            None => self.usage,
        }
    }
}

/// The state built from the events seen so far.
#[derive(Debug, Clone, Default)]
pub struct StreamState {
    messages: BTreeMap<String, Usage>,
    /// Bash tool uses in order: (tool_use id, command).
    bash: Vec<(String, String)>,
    tools_used: BTreeSet<String>,
    pub result: Option<ResultEvent>,
    /// The session id Claude Code gave the run, from the first event that names one.
    pub session_id: Option<String>,
    pub lines: usize,
    pub unparsed: usize,
}

fn count(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn usage_of(value: &Value) -> Usage {
    Usage {
        input: count(value, "input_tokens"),
        output: count(value, "output_tokens"),
        cache_read: count(value, "cache_read_input_tokens"),
        cache_write: count(value, "cache_creation_input_tokens"),
    }
}

impl StreamState {
    /// Takes one line of output.
    pub fn on_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        self.lines += 1;
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            self.unparsed += 1;
            return;
        };
        if self.session_id.is_none() {
            self.session_id = event
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        match event.get("type").and_then(Value::as_str) {
            Some("assistant") => self.on_assistant(&event),
            Some("result") => self.result = Some(parse_result(&event)),
            _ => {}
        }
    }

    fn on_assistant(&mut self, event: &Value) {
        let Some(message) = event.get("message") else {
            return;
        };
        if let (Some(id), Some(usage)) = (
            message.get("id").and_then(Value::as_str),
            message.get("usage"),
        ) {
            let seen = usage_of(usage);
            let entry = self.messages.entry(id.to_string()).or_default();
            entry.input = entry.input.max(seen.input);
            entry.output = entry.output.max(seen.output);
            entry.cache_read = entry.cache_read.max(seen.cache_read);
            entry.cache_write = entry.cache_write.max(seen.cache_write);
        }
        if let Some(content) = message.get("content").and_then(Value::as_array) {
            for item in content {
                if item.get("type").and_then(Value::as_str) != Some("tool_use") {
                    continue;
                }
                let name = item.get("name").and_then(Value::as_str).unwrap_or("?");
                self.tools_used.insert(name.to_string());
                if name == "Bash"
                    && let Some(command) = item.pointer("/input/command").and_then(Value::as_str)
                {
                    let id = item
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    if !self
                        .bash
                        .iter()
                        .any(|(seen, _)| !id.is_empty() && *seen == id)
                    {
                        self.bash.push((id, command.to_string()));
                    }
                }
            }
        }
    }

    /// The de-duplicated running sums: a lower bound until the result arrives.
    pub fn partial_usage(&self) -> Usage {
        self.messages
            .values()
            .fold(Usage::default(), |acc, u| Usage {
                input: acc.input + u.input,
                output: acc.output + u.output,
                cache_read: acc.cache_read + u.cache_read,
                cache_write: acc.cache_write + u.cache_write,
            })
    }

    /// Bash commands that ran: every Bash tool use except those the result lists as denied.
    pub fn commands_run(&self) -> Vec<String> {
        let denied = self
            .result
            .as_ref()
            .map(|r| r.denied_tool_use_ids.clone())
            .unwrap_or_default();
        self.bash
            .iter()
            .filter(|(id, _)| !denied.contains(id))
            .map(|(_, c)| c.clone())
            .collect()
    }

    pub fn tools_used(&self) -> Vec<String> {
        self.tools_used.iter().cloned().collect()
    }
}

fn parse_result(event: &Value) -> ResultEvent {
    let mut result = ResultEvent {
        is_error: event
            .get("is_error")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        subtype: event
            .get("subtype")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        usage: event.get("usage").map(usage_of).unwrap_or_default(),
        cost_usd: event.get("total_cost_usd").and_then(Value::as_f64),
        turns: event
            .get("num_turns")
            .and_then(Value::as_u64)
            .map(|n| n as u32),
        duration_api_ms: event.get("duration_api_ms").and_then(Value::as_u64),
        structured: event
            .get("structured_output")
            .filter(|v| !v.is_null())
            .cloned(),
        text: event
            .get("result")
            .and_then(Value::as_str)
            .map(str::to_string),
        ..ResultEvent::default()
    };
    if let Some(models) = event.get("modelUsage").and_then(Value::as_object) {
        for (model, u) in models {
            result.models_used.insert(
                model.clone(),
                ModelUsage {
                    input: count(u, "inputTokens"),
                    output: count(u, "outputTokens"),
                    cache_read: count(u, "cacheReadInputTokens"),
                    cache_write: count(u, "cacheCreationInputTokens"),
                    cost_usd: u.get("costUSD").and_then(Value::as_f64).unwrap_or(0.0),
                },
            );
        }
    }
    if let Some(denials) = event.get("permission_denials").and_then(Value::as_array) {
        for d in denials {
            let tool = d
                .get("tool_name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            if let Some(id) = d.get("tool_use_id").and_then(Value::as_str) {
                result.denied_tool_use_ids.insert(id.to_string());
            }
            let detail = match d.pointer("/tool_input/command").and_then(Value::as_str) {
                Some(command) => command.to_string(),
                None => d
                    .get("tool_input")
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            };
            result.denials.push(Denial { tool, detail });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSISTANT_1: &str = r#"{"type":"assistant","message":{"id":"msg_1","model":"claude-haiku-5-5","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"cargo test"}}],"usage":{"input_tokens":2,"cache_creation_input_tokens":2380,"cache_read_input_tokens":0,"output_tokens":16}}}"#;
    const ASSISTANT_1_AGAIN: &str = r#"{"type":"assistant","message":{"id":"msg_1","model":"claude-haiku-5-5","content":[{"type":"tool_use","id":"tu_2","name":"Bash","input":{"command":"git -C /elsewhere log"}}],"usage":{"input_tokens":2,"cache_creation_input_tokens":2380,"cache_read_input_tokens":0,"output_tokens":16}}}"#;
    const ASSISTANT_2: &str = r#"{"type":"assistant","message":{"id":"msg_2","model":"claude-haiku-5-5","content":[{"type":"text","text":"done"}],"usage":{"input_tokens":2,"cache_creation_input_tokens":241,"cache_read_input_tokens":2380,"output_tokens":3}}}"#;
    const RESULT: &str = r#"{"type":"result","subtype":"success","is_error":false,"num_turns":3,"duration_api_ms":1300,"total_cost_usd":0.0006219,"usage":{"input_tokens":4,"cache_creation_input_tokens":2621,"cache_read_input_tokens":2380,"output_tokens":147},"modelUsage":{"claude-haiku-5-5":{"inputTokens":4,"outputTokens":147,"cacheReadInputTokens":2380,"cacheCreationInputTokens":2621,"costUSD":0.0006219}},"permission_denials":[{"tool_name":"Bash","tool_use_id":"tu_2","tool_input":{"command":"git -C /elsewhere log"}}],"structured_output":{"verdict":"pass"},"result":"ok"}"#;

    #[test]
    fn usage_is_deduplicated_by_message_and_is_a_lower_bound_until_the_result() {
        let mut state = StreamState::default();
        for line in [ASSISTANT_1, ASSISTANT_1_AGAIN, ASSISTANT_2] {
            state.on_line(line);
        }
        let partial = state.partial_usage();
        assert_eq!(
            partial,
            Usage {
                input: 4,
                output: 19,
                cache_read: 2380,
                cache_write: 2621
            }
        );
        state.on_line(RESULT);
        let result = state.result.clone().unwrap();
        assert_eq!(
            result.usage.output, 147,
            "the final count exceeds the partial sum (E16)"
        );
        assert_eq!(result.models_used["claude-haiku-5-5"].cost_usd, 0.0006219);
        assert_eq!(result.structured.unwrap()["verdict"], "pass");
    }

    #[test]
    fn denied_commands_are_not_counted_as_run() {
        let mut state = StreamState::default();
        for line in [ASSISTANT_1, ASSISTANT_1_AGAIN, RESULT] {
            state.on_line(line);
        }
        assert_eq!(state.commands_run(), vec!["cargo test".to_string()]);
        assert_eq!(
            state.result.unwrap().denials[0].detail,
            "git -C /elsewhere log"
        );
    }

    #[test]
    fn garbage_lines_are_counted_not_fatal() {
        let mut state = StreamState::default();
        state.on_line("not json");
        state.on_line("");
        assert_eq!((state.lines, state.unparsed), (1, 1));
    }
}
