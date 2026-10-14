//! The Jev client (`docs/design/defaults-loop.md` section 12, build step 5), and a fake that returns the same
//! response shape.
//!
//! Jev is a decision model served through the AI gateway at `POST /v1/evaluate`. A call carries a text
//! `state` and one or more named `questions`; each question is a `choice` over described options or a
//! `boolean`. The reply holds one answer per question, the token `usage`, and the gateway's `cost`. The shapes
//! below follow what was seen from live calls on 2026-10-13 and 2026-10-14. The reply's key for the answers
//! (`answers`) was not part of those notes, so it is an assumption until the opt-in live test has run.
//!
//! **Jev only runs in shadow.** Its answers are recorded so they can be scored later; nothing in Athena reads
//! them to decide anything. The gateway's chat endpoint refuses Jev as "a decision model, not a language
//! model", so this client calls `/v1/evaluate` and nothing else. Such a refusal is a build error and is never
//! retried.
//!
//! Calls go out through [`crate::gh`], the one outbound wrapper: [`HttpPost`] is its transport and
//! [`retry_transient`] its retry loop. Only a transient failure (a server error of 500, 502, 503 or 504, a
//! timeout, or a connection reset) is repeated; an authorisation failure or any other 4xx answer is not.
//!
//! The key is read only from `AI_GATEWAY_API_KEY`, when that is set. Where a proxy authenticates calls to
//! `*.vercel.sh`, it is not set and no `Authorization` header is sent. The key is never written to a file, a
//! record, a log or an error: [`ApiKey`] prints as `<key>`, and error text has the key replaced.
//!
//! [`FakeJev`] is the only Jev used outside the opt-in live test.

use crate::gh::{CurlHttp, HttpPost, Retry, retry_transient, transient_text};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// The one endpoint this client calls.
pub const ENDPOINT: &str = "https://ai-gateway.vercel.sh/v1/evaluate";
/// The model named in every request.
pub const MODEL: &str = "typesafe-ai/jev";
/// The environment variable that holds the gateway key, where a proxy does not authenticate the call.
pub const KEY_VAR: &str = "AI_GATEWAY_API_KEY";
/// Set to `1` to ask the real Jev in shadow. Without it no Jev is asked outside tests.
pub const SHADOW_VAR: &str = "ATHENA_JEV_SHADOW";
/// What the gateway's chat endpoint says when asked for Jev.
const CHAT_REFUSAL: &str = "a decision model, not a language model";

/// One question in a call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    /// Pick one of the options; `criteria` maps each option to its description.
    Choice {
        instructions: String,
        criteria: BTreeMap<String, String>,
    },
    /// Yes or no.
    Boolean { instructions: String },
}

impl Question {
    fn kind(&self) -> &'static str {
        match self {
            Question::Choice { .. } => "choice",
            Question::Boolean { .. } => "boolean",
        }
    }
}

/// The body of `POST /v1/evaluate`.
#[derive(Debug, Serialize)]
struct Request<'a> {
    model: &'a str,
    state: &'a str,
    questions: &'a BTreeMap<String, Question>,
}

/// The JSON body for a call: `{model, state, questions: {key: question}}`.
pub fn request_body(state: &str, questions: &BTreeMap<String, Question>) -> String {
    serde_json::to_string(&Request {
        model: MODEL,
        state,
        questions,
    })
    .expect("a request serializes")
}

/// One answer. A boolean answer is only the probability of true: it has no `value` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Choice {
        choice: String,
        /// The probability of each option.
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
    Boolean {
        /// The probability of true.
        probability: f64,
    },
}

impl Answer {
    fn kind(&self) -> &'static str {
        match self {
            Answer::Choice { .. } => "choice",
            Answer::Boolean { .. } => "boolean",
        }
    }
}

/// Tokens used by one call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,
}

/// A parsed reply.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    /// One answer for each question asked, by key.
    pub answers: BTreeMap<String, Answer>,
    pub usage: Usage,
    /// The gateway's cost in dollars (`providerMetadata.gateway.cost`, a decimal string on the wire), when it
    /// reported one.
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct WireResponse {
    #[serde(alias = "results")]
    answers: BTreeMap<String, Answer>,
    usage: Usage,
    #[serde(rename = "providerMetadata", default)]
    provider_metadata: Option<WireMetadata>,
}

#[derive(Debug, Deserialize)]
struct WireMetadata {
    gateway: Option<WireGateway>,
}

#[derive(Debug, Deserialize)]
struct WireGateway {
    cost: Option<String>,
}

/// Reads a reply body and checks it against the questions that were asked: each key has an answer of the
/// matching type, and the cost, if present, is a decimal number.
pub fn parse_response(
    body: &str,
    questions: &BTreeMap<String, Question>,
) -> Result<Evaluation, JevError> {
    let wire: WireResponse = serde_json::from_str(body).map_err(|e| {
        JevError::Malformed(format!("the reply does not have the expected shape: {e}"))
    })?;
    for (key, question) in questions {
        match wire.answers.get(key) {
            None => {
                return Err(JevError::Malformed(format!(
                    "no answer to question {key:?}"
                )));
            }
            Some(answer) if answer.kind() != question.kind() => {
                return Err(JevError::Malformed(format!(
                    "question {key:?} is a {} but its answer is a {}",
                    question.kind(),
                    answer.kind()
                )));
            }
            Some(_) => {}
        }
    }
    let cost_usd = match wire
        .provider_metadata
        .and_then(|m| m.gateway)
        .and_then(|g| g.cost)
    {
        None => None,
        Some(text) => match text.trim().parse::<f64>() {
            Ok(cost) if cost.is_finite() && cost >= 0.0 => Some(cost),
            _ => {
                return Err(JevError::Malformed(format!(
                    "the cost {text:?} is not a decimal number"
                )));
            }
        },
    };
    Ok(Evaluation {
        answers: wire.answers,
        usage: wire.usage,
        cost_usd,
    })
}

/// Why a call produced no answers. The text never holds the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JevError {
    /// The request is wrong for this endpoint, such as Jev asked through the chat endpoint. Never retried.
    Build(String),
    /// The gateway refused the call with a status other than a server error: bad key, bad request, and so on.
    /// Never retried.
    Refused { status: u16, excerpt: String },
    /// The gateway answered with a server error (500, 502, 503 or 504). Retried.
    Unavailable { status: u16, excerpt: String },
    /// No reply came: the connection failed or the program could not run. Retried only for a timeout or a
    /// connection reset.
    Transport(String),
    /// A reply came and it is not usable.
    Malformed(String),
}

impl JevError {
    /// Why this error is worth asking again, or `None` when it is not.
    pub fn transient_reason(&self) -> Option<String> {
        match self {
            JevError::Unavailable { status, .. } => Some(format!("HTTP {status}")),
            JevError::Transport(text) => transient_text(text),
            _ => None,
        }
    }
}

impl std::fmt::Display for JevError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JevError::Build(text) => write!(f, "Jev request error: {text}"),
            JevError::Refused { status, excerpt } => {
                write!(
                    f,
                    "the gateway refused the Jev call (HTTP {status}): {excerpt}"
                )
            }
            JevError::Unavailable { status, excerpt } => {
                write!(
                    f,
                    "the gateway could not answer the Jev call (HTTP {status}): {excerpt}"
                )
            }
            JevError::Transport(text) => write!(f, "the Jev call got no reply: {text}"),
            JevError::Malformed(text) => write!(f, "the Jev reply is not usable: {text}"),
        }
    }
}

impl std::error::Error for JevError {}

/// Asks Jev. [`JevClient`] is the real one and [`FakeJev`] the fake.
pub trait Evaluate {
    /// Puts every question to Jev in one call over the text `state`.
    fn evaluate(
        &self,
        state: &str,
        questions: &BTreeMap<String, Question>,
    ) -> Result<Evaluation, JevError>;
}

/// The gateway key. It prints as `<key>`.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// The key a lookup finds under [`KEY_VAR`]; an empty value counts as none.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Option<ApiKey> {
        lookup(KEY_VAR)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .map(ApiKey)
    }

    /// The key in the process environment, if there is one.
    pub fn from_env() -> Option<ApiKey> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<key>")
    }
}

/// The real client for `/v1/evaluate`.
pub struct JevClient {
    http: Box<dyn HttpPost>,
    key: Option<ApiKey>,
    /// How a transient failure is repeated.
    pub retry: Retry,
}

impl std::fmt::Debug for JevClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JevClient")
            .field("key", &self.key)
            .field("retry", &self.retry)
            .finish_non_exhaustive()
    }
}

impl JevClient {
    /// A client over `http` that sends `key` as a bearer token when there is one.
    pub fn new(http: Box<dyn HttpPost>, key: Option<ApiKey>) -> Self {
        JevClient {
            http,
            key,
            retry: Retry::STANDARD,
        }
    }

    /// The client for real calls: the transport of [`crate::gh`] and the key from the environment.
    pub fn from_env() -> Self {
        JevClient::new(Box::new(CurlHttp::default()), ApiKey::from_env())
    }

    /// `text` with the key replaced, so no error carries it.
    fn scrub(&self, text: &str) -> String {
        match &self.key {
            Some(ApiKey(key)) => text.replace(key.as_str(), "<key>"),
            None => text.to_string(),
        }
    }

    fn excerpt(&self, body: &str) -> String {
        let flat: String = self
            .scrub(body)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        flat.chars().take(300).collect()
    }

    fn once(
        &self,
        body: &str,
        questions: &BTreeMap<String, Question>,
    ) -> Result<Evaluation, JevError> {
        let bearer = self.key.as_ref().map(|k| k.0.as_str());
        let reply = self
            .http
            .post_json(ENDPOINT, bearer, body)
            .map_err(|e| JevError::Transport(self.scrub(&format!("{e:#}"))))?;
        if (200..300).contains(&reply.status) {
            return parse_response(&reply.body, questions);
        }
        if reply.body.contains(CHAT_REFUSAL) {
            return Err(JevError::Build(format!(
                "the gateway says Jev is {CHAT_REFUSAL}; only {ENDPOINT} can be called"
            )));
        }
        let excerpt = self.excerpt(&reply.body);
        if crate::gh::is_transient(Some(reply.status)) {
            Err(JevError::Unavailable {
                status: reply.status,
                excerpt,
            })
        } else {
            Err(JevError::Refused {
                status: reply.status,
                excerpt,
            })
        }
    }
}

impl Evaluate for JevClient {
    fn evaluate(
        &self,
        state: &str,
        questions: &BTreeMap<String, Question>,
    ) -> Result<Evaluation, JevError> {
        let body = request_body(state, questions);
        retry_transient(
            self.retry,
            None,
            "the Jev call",
            || self.once(&body, questions),
            JevError::transient_reason,
        )
    }
}

/// The Jev to ask in shadow: the real client when [`SHADOW_VAR`] is `1`, otherwise none.
pub fn shadow_from_env() -> Option<Box<dyn Evaluate>> {
    (std::env::var(SHADOW_VAR).as_deref() == Ok("1"))
        .then(|| Box::new(JevClient::from_env()) as Box<dyn Evaluate>)
}

/// A Jev that never reaches the network. It answers from a script of results, one per call, and when the
/// script is empty it answers every question in the real shape: a choice goes to the first option with
/// probability 1 and confidence 0.9, a boolean to probability 0.5. Clones share the script and the call log.
#[derive(Debug, Clone, Default)]
pub struct FakeJev {
    script: Arc<Mutex<VecDeque<Result<Evaluation, JevError>>>>,
    calls: Arc<Mutex<Vec<FakeCall>>>,
}

/// A call the fake received: the state text and the questions.
pub type FakeCall = (String, BTreeMap<String, Question>);

impl FakeJev {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues the result of the next call.
    pub fn then(self, result: Result<Evaluation, JevError>) -> Self {
        self.script.lock().unwrap().push_back(result);
        self
    }

    /// The `(state, questions)` of every call so far.
    pub fn calls(&self) -> Vec<FakeCall> {
        self.calls.lock().unwrap().clone()
    }

    /// The default reply to `questions`.
    pub fn default_evaluation(questions: &BTreeMap<String, Question>) -> Evaluation {
        let answers = questions
            .iter()
            .map(|(key, question)| {
                let answer = match question {
                    Question::Choice { criteria, .. } => {
                        let first = criteria.keys().next().cloned().unwrap_or_default();
                        Answer::Choice {
                            probabilities: criteria
                                .keys()
                                .map(|k| (k.clone(), if *k == first { 1.0 } else { 0.0 }))
                                .collect(),
                            choice: first,
                            confidence: 0.9,
                        }
                    }
                    Question::Boolean { .. } => Answer::Boolean { probability: 0.5 },
                };
                (key.clone(), answer)
            })
            .collect();
        Evaluation {
            answers,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 20,
            },
            cost_usd: Some(0.00002),
        }
    }
}

impl Evaluate for FakeJev {
    fn evaluate(
        &self,
        state: &str,
        questions: &BTreeMap<String, Question>,
    ) -> Result<Evaluation, JevError> {
        self.calls
            .lock()
            .unwrap()
            .push((state.to_string(), questions.clone()));
        match self.script.lock().unwrap().pop_front() {
            Some(result) => result,
            None => Ok(Self::default_evaluation(questions)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gh::HttpReply;
    use anyhow::Result;
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::time::Duration;

    const SECRET: &str = "sk-test-very-secret-1234";

    fn retry() -> Retry {
        Retry {
            attempts: 3,
            delay: Duration::ZERO,
        }
    }

    fn questions() -> BTreeMap<String, Question> {
        BTreeMap::from([
            (
                "class".to_string(),
                Question::Choice {
                    instructions: "Which class fits?".into(),
                    criteria: BTreeMap::from([
                        ("a".to_string(), "first".to_string()),
                        ("b".to_string(), "second".to_string()),
                    ]),
                },
            ),
            (
                "repeat".to_string(),
                Question::Boolean {
                    instructions: "Is it a repeat?".into(),
                },
            ),
        ])
    }

    fn good_body() -> String {
        json!({
            "answers": {
                "class": { "type": "choice", "choice": "b",
                           "probabilities": { "a": 0.2, "b": 0.8 }, "confidence": 0.7 },
                "repeat": { "type": "boolean", "probability": 0.25 }
            },
            "usage": { "inputTokens": 321, "outputTokens": 12 },
            "providerMetadata": { "gateway": { "cost": "0.00002" } }
        })
        .to_string()
    }

    fn reply(status: u16, body: &str) -> Result<HttpReply> {
        Ok(HttpReply {
            status,
            body: body.to_string(),
        })
    }

    /// An HTTP transport that answers from a script and records what it was sent.
    struct ScriptedHttp {
        replies: RefCell<VecDeque<Result<HttpReply>>>,
        sent: Sent,
    }

    /// `(url, bearer, body)` of each post.
    type Sent = std::rc::Rc<RefCell<Vec<(String, Option<String>, String)>>>;

    impl HttpPost for ScriptedHttp {
        fn post_json(&self, url: &str, bearer: Option<&str>, body: &str) -> Result<HttpReply> {
            self.sent.borrow_mut().push((
                url.to_string(),
                bearer.map(str::to_string),
                body.to_string(),
            ));
            self.replies
                .borrow_mut()
                .pop_front()
                .expect("the script has a reply left")
        }
    }

    fn client(replies: Vec<Result<HttpReply>>, key: Option<&str>) -> (JevClient, Sent) {
        let sent = Sent::default();
        let http = ScriptedHttp {
            replies: RefCell::new(replies.into()),
            sent: sent.clone(),
        };
        let mut client = JevClient::new(Box::new(http), key.map(|k| ApiKey(k.to_string())));
        client.retry = retry();
        (client, sent)
    }

    #[test]
    fn the_request_has_the_probed_shape_and_carries_several_questions() {
        let body: Value = serde_json::from_str(&request_body("the state", &questions())).unwrap();
        assert_eq!(
            body,
            json!({
                "model": "typesafe-ai/jev",
                "state": "the state",
                "questions": {
                    "class": { "type": "choice", "instructions": "Which class fits?",
                               "criteria": { "a": "first", "b": "second" } },
                    "repeat": { "type": "boolean", "instructions": "Is it a repeat?" }
                }
            })
        );
    }

    #[test]
    fn a_reply_parses_choice_boolean_usage_and_a_decimal_string_cost() {
        let got = parse_response(&good_body(), &questions()).unwrap();
        assert_eq!(
            got.answers["class"],
            Answer::Choice {
                choice: "b".into(),
                probabilities: BTreeMap::from([("a".into(), 0.2), ("b".into(), 0.8)]),
                confidence: 0.7,
            }
        );
        assert_eq!(got.answers["repeat"], Answer::Boolean { probability: 0.25 });
        assert_eq!(
            got.usage,
            Usage {
                input_tokens: 321,
                output_tokens: 12
            }
        );
        assert_eq!(got.cost_usd, Some(0.00002));
        // A boolean answer is the probability alone: it has no `value` field.
        let shown = serde_json::to_value(&got.answers["repeat"]).unwrap();
        assert_eq!(shown, json!({ "type": "boolean", "probability": 0.25 }));
    }

    #[test]
    fn a_reply_that_does_not_match_the_questions_is_malformed() {
        let mut body: Value = serde_json::from_str(&good_body()).unwrap();
        body["answers"].as_object_mut().unwrap().remove("repeat");
        let err = parse_response(&body.to_string(), &questions()).unwrap_err();
        assert!(
            matches!(err, JevError::Malformed(ref m) if m.contains("repeat")),
            "{err}"
        );

        let mut body: Value = serde_json::from_str(&good_body()).unwrap();
        body["answers"]["repeat"] = json!({ "type": "choice", "choice": "x",
            "probabilities": {}, "confidence": 1.0 });
        assert!(matches!(
            parse_response(&body.to_string(), &questions()),
            Err(JevError::Malformed(_))
        ));

        let mut body: Value = serde_json::from_str(&good_body()).unwrap();
        body["providerMetadata"]["gateway"]["cost"] = json!("cheap");
        assert!(matches!(
            parse_response(&body.to_string(), &questions()),
            Err(JevError::Malformed(_))
        ));
        assert!(parse_response("not json", &questions()).is_err());
    }

    #[test]
    fn the_client_calls_the_evaluate_endpoint_once_with_the_key_as_a_bearer_token() {
        let (client, sent) = client(vec![reply(200, &good_body())], Some(SECRET));
        let got = client.evaluate("state", &questions()).unwrap();
        assert_eq!(got.answers.len(), 2);
        let sent = sent.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "https://ai-gateway.vercel.sh/v1/evaluate");
        assert_eq!(sent[0].1.as_deref(), Some(SECRET));
        assert!(!sent[0].0.contains("chat/completions"));
        assert!(sent[0].2.contains("typesafe-ai/jev"));
    }

    #[test]
    fn without_a_key_no_bearer_token_is_sent() {
        let (client, sent) = client(vec![reply(200, &good_body())], None);
        client.evaluate("state", &questions()).unwrap();
        assert_eq!(sent.borrow()[0].1, None);
    }

    #[test]
    fn the_key_comes_only_from_its_variable() {
        let key = ApiKey::from_lookup(|n| (n == "AI_GATEWAY_API_KEY").then(|| SECRET.to_string()));
        assert_eq!(key, Some(ApiKey(SECRET.into())));
        assert_eq!(ApiKey::from_lookup(|_| None), None);
        assert_eq!(ApiKey::from_lookup(|_| Some("  ".into())), None);
        assert_eq!(
            ApiKey::from_lookup(|n| (n == "OTHER").then(|| "x".into())),
            None
        );
    }

    #[test]
    fn a_transient_failure_is_retried_and_then_answers() {
        let (client, sent) = client(
            vec![
                reply(503, "busy"),
                Err(anyhow::anyhow!("the request failed: operation timed out")),
                reply(200, &good_body()),
            ],
            None,
        );
        assert!(client.evaluate("state", &questions()).is_ok());
        assert_eq!(sent.borrow().len(), 3);
    }

    #[test]
    fn retries_stop_at_the_bound() {
        let (client, sent) = client(
            vec![reply(502, "x"), reply(503, "x"), reply(504, "x")],
            None,
        );
        let err = client.evaluate("state", &questions()).unwrap_err();
        assert!(
            matches!(err, JevError::Unavailable { status: 504, .. }),
            "{err}"
        );
        assert_eq!(sent.borrow().len(), 3);
    }

    #[test]
    fn auth_and_client_errors_are_not_retried() {
        for status in [400, 401, 403, 404, 422, 429] {
            let (client, sent) = client(vec![reply(status, "no")], Some(SECRET));
            let err = client.evaluate("state", &questions()).unwrap_err();
            assert!(matches!(err, JevError::Refused { .. }), "{status}: {err}");
            assert_eq!(sent.borrow().len(), 1, "{status}");
        }
        // A transport failure that is not a timeout or a reset is not retried either.
        let (client, sent) = client(vec![Err(anyhow::anyhow!("could not resolve host"))], None);
        assert!(matches!(
            client.evaluate("state", &questions()),
            Err(JevError::Transport(_))
        ));
        assert_eq!(sent.borrow().len(), 1);
    }

    #[test]
    fn the_chat_endpoint_refusal_is_a_build_error_and_is_not_retried() {
        let body =
            r#"{"error":{"message":"typesafe-ai/jev is a decision model, not a language model"}}"#;
        // Even with a status that would otherwise be retried.
        for status in [400, 503] {
            let (client, sent) = client(vec![reply(status, body)], None);
            let err = client.evaluate("state", &questions()).unwrap_err();
            assert!(matches!(err, JevError::Build(_)), "{status}: {err}");
            assert_eq!(err.transient_reason(), None);
            assert_eq!(sent.borrow().len(), 1);
        }
    }

    #[test]
    fn the_key_is_in_neither_the_debug_output_nor_any_error() {
        let (shown, _) = client(vec![], Some(SECRET));
        assert!(!format!("{shown:?}").contains(SECRET));
        assert!(!format!("{:?}", ApiKey(SECRET.into())).contains(SECRET));

        let echoing = format!("bad key {SECRET} in header");
        let cases: Vec<Result<HttpReply>> = vec![
            reply(401, &echoing),
            reply(500, &echoing),
            reply(200, &echoing),
            Err(anyhow::anyhow!("failed with {SECRET}")),
        ];
        for case in cases {
            let (mut client, _) = client(vec![case], Some(SECRET));
            client.retry.attempts = 1;
            let err = client.evaluate("state", &questions()).unwrap_err();
            assert!(!err.to_string().contains(SECRET), "{err}");
            assert!(!format!("{err:?}").contains(SECRET), "{err:?}");
        }
    }

    #[test]
    fn the_fake_answers_in_the_real_shape_and_logs_its_calls() {
        let fake = FakeJev::new();
        let got = fake.evaluate("state", &questions()).unwrap();
        assert_eq!(got.answers.len(), 2);
        assert!(matches!(&got.answers["class"], Answer::Choice { choice, .. } if choice == "a"));
        assert_eq!(got.answers["repeat"], Answer::Boolean { probability: 0.5 });
        assert_eq!(fake.calls().len(), 1);
        assert_eq!(fake.calls()[0].0, "state");
        // The default reply passes the same checks as a real one.
        let wire = json!({
            "answers": got.answers,
            "usage": got.usage,
            "providerMetadata": { "gateway": { "cost": got.cost_usd.unwrap().to_string() } }
        });
        assert_eq!(
            parse_response(&wire.to_string(), &questions()).unwrap(),
            got
        );
    }

    #[test]
    fn the_fake_replays_a_script_including_errors() {
        let fake = FakeJev::new()
            .then(Err(JevError::Transport("down".into())))
            .then(Ok(FakeJev::default_evaluation(&questions())));
        let clone = fake.clone();
        assert!(clone.evaluate("s", &questions()).is_err());
        assert!(fake.evaluate("s", &questions()).is_ok());
        assert_eq!(fake.calls().len(), 2);
    }

    /// Opt-in: calls Jev through the gateway. Run with `cargo test -p athena-core -- --ignored jev` where a
    /// proxy authenticates the call or `AI_GATEWAY_API_KEY` is set. It costs about $0.00002 (an estimate from
    /// the probes of 2026-10-13 and 2026-10-14).
    #[test]
    #[ignore = "calls Jev through the gateway"]
    fn live_jev_answers_both_question_kinds() {
        let client = JevClient::from_env();
        let got = client
            .evaluate(
                "A job failed because a check deleted a needed file.",
                &questions(),
            )
            .unwrap();
        assert!(matches!(got.answers["class"], Answer::Choice { .. }));
        assert!(matches!(got.answers["repeat"], Answer::Boolean { .. }));
        assert!(got.usage.input_tokens > 0);
    }
}
