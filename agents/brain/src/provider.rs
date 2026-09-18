//! Providers, wire requests, and the budgeted call. TypeSafe's native endpoint
//! and OpenRouter's decisions endpoint take the same body; they differ in URL,
//! model id, attribution headers, and whether the response reports a cost.
//! Facts here were checked against both OpenAPI documents on 2026-09-18.

use crate::budget::{estimate_tokens, now_unix, Budget, Charge, Pricing};
use crate::decision::{DecisionResponse, Question};
use crate::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

pub const TYPESAFE_ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
pub const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/alpha/decisions";
pub const OPENROUTER_KEY_ENDPOINT: &str = "https://openrouter.ai/api/v1/key";
/// TypeSafe's alias for the newest Jev.
pub const TYPESAFE_MODEL: &str = "jev-latest";
/// OpenRouter's unambiguous id (the `~typesafe/jev-latest` alias is not in its catalog).
pub const OPENROUTER_MODEL: &str = "typesafe/jev-1.13";
/// App attribution OpenRouter shows on its rankings; this repository, nothing else.
pub const APP_REFERER: &str = "https://github.com/blisspixel/fragr";
pub const APP_TITLE: &str = "fragr";
pub const TYPESAFE_KEY_NAMES: &[&str] = &["TYPESAFE_API_KEY", "TYPESAFE"];
pub const OPENROUTER_KEY_NAMES: &[&str] = &["OPENROUTER_API_KEY", "OPENROUTER"];
/// Fixed per-call token overhead for headers and framing in the estimate.
const ESTIMATE_OVERHEAD_TOKENS: u64 = 16;
/// Output tokens assumed per question when estimating (Jev bills none today).
const ESTIMATE_OUTPUT_PER_QUESTION: u64 = 8;

/// Where decisions come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    /// Local rules only; costs nothing and needs no key.
    Local,
    /// TypeSafe's own endpoint.
    Typesafe,
    /// The same model routed through OpenRouter.
    OpenRouter,
}

impl Provider {
    pub fn parse(text: &str) -> Option<Provider> {
        match text.trim().to_ascii_lowercase().as_str() {
            "local" | "none" | "rules" => Some(Provider::Local),
            "typesafe" | "jev" => Some(Provider::Typesafe),
            "openrouter" => Some(Provider::OpenRouter),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Provider::Local => "local",
            Provider::Typesafe => "typesafe",
            Provider::OpenRouter => "openrouter",
        }
    }

    pub fn is_paid(self) -> bool {
        !matches!(self, Provider::Local)
    }

    pub fn endpoint(self) -> Option<&'static str> {
        match self {
            Provider::Local => None,
            Provider::Typesafe => Some(TYPESAFE_ENDPOINT),
            Provider::OpenRouter => Some(OPENROUTER_ENDPOINT),
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Provider::Local => "rules",
            Provider::Typesafe => TYPESAFE_MODEL,
            Provider::OpenRouter => OPENROUTER_MODEL,
        }
    }

    pub fn key_names(self) -> &'static [&'static str] {
        match self {
            Provider::Local => &[],
            Provider::Typesafe => TYPESAFE_KEY_NAMES,
            Provider::OpenRouter => OPENROUTER_KEY_NAMES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// A request as the transport sends it. Holds the bearer token; never log it raw.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Value>,
}

impl HttpRequest {
    /// A printable copy with the bearer token masked.
    pub fn redacted(&self) -> Value {
        let headers: Vec<Value> = self
            .headers
            .iter()
            .map(|(name, value)| {
                let shown = if name.eq_ignore_ascii_case("authorization") {
                    "Bearer ***".to_string()
                } else {
                    value.clone()
                };
                json!([name, shown])
            })
            .collect();
        json!({
            "method": match self.method { Method::Get => "GET", Method::Post => "POST" },
            "url": self.url,
            "headers": headers,
            "body": self.body,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Build a decision request for a paid provider.
pub fn decision_request(
    provider: Provider,
    model: &str,
    api_key: &str,
    state: &str,
    questions: &BTreeMap<String, Question>,
) -> Result<HttpRequest, Error> {
    let Some(url) = provider.endpoint() else {
        return Err(Error::InvalidArgument(
            "the local provider has no endpoint".to_string(),
        ));
    };
    if api_key.trim().is_empty() {
        return Err(Error::MissingApiKey(provider.key_names().join(", ")));
    }
    if questions.is_empty() {
        return Err(Error::InvalidArgument("at least one question".to_string()));
    }
    let mut headers = vec![
        (
            "Authorization".to_string(),
            format!("Bearer {}", api_key.trim()),
        ),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];
    if provider == Provider::OpenRouter {
        headers.push(("HTTP-Referer".to_string(), APP_REFERER.to_string()));
        headers.push(("X-OpenRouter-Title".to_string(), APP_TITLE.to_string()));
        headers.push(("X-Title".to_string(), APP_TITLE.to_string()));
    }
    let body = json!({
        "model": model,
        "state": state,
        "questions": questions,
    });
    Ok(HttpRequest {
        method: Method::Post,
        url: url.to_string(),
        headers,
        body: Some(body),
    })
}

/// Read the calling key's limit and usage (OpenRouter only).
pub fn key_request(provider: Provider, api_key: &str) -> Result<HttpRequest, Error> {
    if provider != Provider::OpenRouter {
        return Err(Error::InvalidArgument(format!(
            "{} does not expose a key status endpoint",
            provider.name()
        )));
    }
    if api_key.trim().is_empty() {
        return Err(Error::MissingApiKey(provider.key_names().join(", ")));
    }
    Ok(HttpRequest {
        method: Method::Get,
        url: OPENROUTER_KEY_ENDPOINT.to_string(),
        headers: vec![(
            "Authorization".to_string(),
            format!("Bearer {}", api_key.trim()),
        )],
        body: None,
    })
}

/// Something that can send an `HttpRequest`. Tests fake it; the binary uses reqwest.
pub trait Transport: Send + Sync {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, Error>;
}

/// Blocking reqwest transport with one timeout for connect plus read.
pub struct HttpTransport {
    client: reqwest::blocking::Client,
}

impl HttpTransport {
    pub fn new(timeout: Duration) -> Result<Self, Error> {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| Error::Transport(err.to_string()))?;
        Ok(HttpTransport { client })
    }
}

impl Transport for HttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, Error> {
        let mut builder = match request.method {
            Method::Get => self.client.get(&request.url),
            Method::Post => self.client.post(&request.url),
        };
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body {
            builder = builder.json(body);
        }
        let response = builder
            .send()
            .map_err(|err| Error::Transport(err.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|err| Error::Transport(err.to_string()))?
            .to_vec();
        Ok(HttpResponse { status, body })
    }
}

/// The most useful message in an error body, whatever shape the provider used.
pub fn api_error_message(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        for path in [
            &["error", "message"][..],
            &["message"][..],
            &["detail"][..],
            &["error"][..],
        ] {
            let mut node = &value;
            let mut found = true;
            for key in path {
                match node.get(key) {
                    Some(next) => node = next,
                    None => {
                        found = false;
                        break;
                    }
                }
            }
            if found {
                if let Some(s) = node.as_str() {
                    return s.to_string();
                }
                if !node.is_null() && !node.is_object() {
                    return node.to_string();
                }
            }
        }
    }
    let trimmed = text.trim();
    let mut short: String = trimmed.chars().take(200).collect();
    if short.len() < trimmed.len() {
        short.push_str("...");
    }
    if short.is_empty() {
        "empty body".to_string()
    } else {
        short
    }
}

/// Turn a raw response into a parsed decision or an API error.
pub fn parse_decision(response: &HttpResponse) -> Result<DecisionResponse, Error> {
    if !(200..300).contains(&response.status) {
        return Err(Error::Api {
            status: response.status,
            message: api_error_message(&response.body),
        });
    }
    serde_json::from_slice(&response.body).map_err(|err| Error::Malformed(err.to_string()))
}

/// What OpenRouter reports about the calling key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyStatus {
    #[serde(default)]
    pub label: Option<String>,
    /// Hard spend limit set on the key, in dollars; `None` means unlimited.
    #[serde(default)]
    pub limit: Option<f64>,
    #[serde(default)]
    pub limit_remaining: Option<f64>,
    #[serde(default)]
    pub usage: Option<f64>,
    #[serde(default)]
    pub is_free_tier: Option<bool>,
}

pub fn parse_key_status(response: &HttpResponse) -> Result<KeyStatus, Error> {
    if !(200..300).contains(&response.status) {
        return Err(Error::Api {
            status: response.status,
            message: api_error_message(&response.body),
        });
    }
    let value: Value =
        serde_json::from_slice(&response.body).map_err(|err| Error::Malformed(err.to_string()))?;
    let data = value.get("data").cloned().unwrap_or(value);
    serde_json::from_value(data).map_err(|err| Error::Malformed(err.to_string()))
}

/// Estimated dollars for a decision request before it is sent.
pub fn estimate_cost(request: &HttpRequest, pricing: &Pricing) -> f64 {
    let body = request
        .body
        .as_ref()
        .map(|b| b.to_string())
        .unwrap_or_default();
    let questions = request
        .body
        .as_ref()
        .and_then(|b| b.get("questions"))
        .and_then(Value::as_object)
        .map(|q| q.len() as u64)
        .unwrap_or(0);
    let input = estimate_tokens(&body) + ESTIMATE_OVERHEAD_TOKENS;
    let output = questions * ESTIMATE_OUTPUT_PER_QUESTION;
    pricing.cost(input, output)
}

/// A completed budgeted call.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub response: DecisionResponse,
    pub charge: Charge,
}

fn lock(budget: &Mutex<Budget>) -> std::sync::MutexGuard<'_, Budget> {
    budget
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One paid call: check the caps, send, record the charge, parse. The budget
/// lock is held only around the check and the record, never during the call.
/// A refused call returns `Error::Budget` and records nothing.
pub fn decide(
    transport: &dyn Transport,
    budget: &Mutex<Budget>,
    provider: Provider,
    model: &str,
    request: &HttpRequest,
) -> Result<Decision, Error> {
    let (estimate, pricing) = {
        let guard = lock(budget);
        let estimate = estimate_cost(request, &guard.pricing);
        guard.check(estimate)?;
        (estimate, guard.pricing)
    };
    let outcome = transport
        .send(request)
        .and_then(|response| parse_decision(&response));
    let mut charge = Charge {
        unix: now_unix(),
        provider: provider.name().to_string(),
        model: model.to_string(),
        estimated_usd: estimate,
        actual_usd: None,
        input_tokens: None,
        output_tokens: None,
        request_id: None,
        ok: false,
    };
    if let Ok(response) = &outcome {
        charge.ok = true;
        charge.actual_usd = response.cost_usd(&pricing);
        charge.input_tokens = response.usage.as_ref().map(|u| u.input_tokens);
        charge.output_tokens = response.usage.as_ref().map(|u| u.output_tokens);
        charge.request_id = response.id.clone();
    }
    lock(budget).record(charge.clone())?;
    Ok(Decision {
        response: outcome?,
        charge,
    })
}

#[cfg(test)]
pub(crate) mod fakes {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Answers every request from a fixed script, in order, repeating the last.
    pub struct FakeTransport {
        pub responses: Vec<Result<HttpResponse, Error>>,
        pub calls: AtomicUsize,
        pub last_request: Mutex<Option<HttpRequest>>,
    }

    impl FakeTransport {
        pub fn new(responses: Vec<Result<HttpResponse, Error>>) -> Self {
            FakeTransport {
                responses,
                calls: AtomicUsize::new(0),
                last_request: Mutex::new(None),
            }
        }

        pub fn ok(body: Value) -> Self {
            FakeTransport::new(vec![Ok(HttpResponse {
                status: 200,
                body: body.to_string().into_bytes(),
            })])
        }

        pub fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl Transport for FakeTransport {
        fn send(&self, request: &HttpRequest) -> Result<HttpResponse, Error> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_request.lock().unwrap() = Some(request.clone());
            let index = n.min(self.responses.len().saturating_sub(1));
            match self.responses.get(index) {
                Some(Ok(response)) => Ok(response.clone()),
                Some(Err(err)) => Err(Error::Transport(err.to_string())),
                None => Err(Error::Transport("no scripted response".to_string())),
            }
        }
    }

    /// A native-shaped answer set that pushes with high confidence.
    pub fn push_answers() -> Value {
        json!({
            "model": "jev-1.13.0",
            "answers": {
                "stance": {"type": "choice", "choice": "push_enemy", "confidence": 0.91, "probabilities": {"push_enemy": 0.91}},
                "weapon": {"type": "choice", "choice": "rail", "confidence": 0.8},
                "danger": {"type": "score", "score": 2.0, "confidence": 0.6}
            },
            "usage": {"input_tokens": 240, "output_tokens": 0}
        })
    }
}

#[cfg(test)]
mod tests {
    use super::fakes::{push_answers, FakeTransport};
    use super::*;
    use crate::budget::Caps;
    use crate::decision::tactical_questions;

    #[test]
    fn provider_table() {
        assert_eq!(Provider::parse("Local"), Some(Provider::Local));
        assert_eq!(Provider::parse("none"), Some(Provider::Local));
        assert_eq!(Provider::parse(" typesafe "), Some(Provider::Typesafe));
        assert_eq!(Provider::parse("jev"), Some(Provider::Typesafe));
        assert_eq!(Provider::parse("OpenRouter"), Some(Provider::OpenRouter));
        assert_eq!(Provider::parse("anthropic"), None);
        assert!(!Provider::Local.is_paid());
        assert!(Provider::Typesafe.is_paid() && Provider::OpenRouter.is_paid());
        assert_eq!(Provider::Local.endpoint(), None);
        assert_eq!(Provider::Typesafe.endpoint(), Some(TYPESAFE_ENDPOINT));
        assert_eq!(Provider::OpenRouter.endpoint(), Some(OPENROUTER_ENDPOINT));
        assert_eq!(Provider::Typesafe.default_model(), "jev-latest");
        assert_eq!(Provider::OpenRouter.default_model(), "typesafe/jev-1.13");
        assert_eq!(Provider::Local.default_model(), "rules");
        assert!(Provider::Local.key_names().is_empty());
        assert_eq!(Provider::Typesafe.key_names()[0], "TYPESAFE_API_KEY");
        assert_eq!(Provider::OpenRouter.key_names()[0], "OPENROUTER_API_KEY");
        for p in [Provider::Local, Provider::Typesafe, Provider::OpenRouter] {
            assert_eq!(Provider::parse(p.name()), Some(p));
        }
    }

    #[test]
    fn decision_requests_differ_only_where_the_providers_do() {
        let questions = tactical_questions();
        let native = decision_request(
            Provider::Typesafe,
            "jev-latest",
            " sk_t ",
            "STATE",
            &questions,
        )
        .unwrap();
        assert_eq!(native.method, Method::Post);
        assert_eq!(native.url, TYPESAFE_ENDPOINT);
        assert_eq!(native.headers.len(), 2);
        assert_eq!(
            native.headers[0],
            ("Authorization".into(), "Bearer sk_t".into())
        );
        let body = native.body.as_ref().unwrap();
        assert_eq!(body["model"], "jev-latest");
        assert_eq!(body["state"], "STATE");
        assert_eq!(body["questions"]["stance"]["type"], "choice");

        let routed = decision_request(
            Provider::OpenRouter,
            "typesafe/jev-1.13",
            "sk_o",
            "STATE",
            &questions,
        )
        .unwrap();
        assert_eq!(routed.url, OPENROUTER_ENDPOINT);
        assert!(routed
            .headers
            .iter()
            .any(|(n, v)| n == "HTTP-Referer" && v == APP_REFERER));
        assert!(routed
            .headers
            .iter()
            .any(|(n, v)| n == "X-OpenRouter-Title" && v == APP_TITLE));
        assert!(routed.headers.iter().any(|(n, _)| n == "X-Title"));
        assert_eq!(routed.body.as_ref().unwrap()["model"], "typesafe/jev-1.13");
        assert_eq!(
            routed.body.as_ref().unwrap()["questions"],
            body["questions"]
        );

        assert!(matches!(
            decision_request(Provider::Local, "rules", "k", "s", &questions),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            decision_request(Provider::Typesafe, "jev-latest", "  ", "s", &questions),
            Err(Error::MissingApiKey(_))
        ));
        assert!(matches!(
            decision_request(Provider::Typesafe, "jev-latest", "k", "s", &BTreeMap::new()),
            Err(Error::InvalidArgument(_))
        ));

        let shown = routed.redacted();
        assert_eq!(shown["method"], "POST");
        assert_eq!(shown["headers"][0][1], "Bearer ***");
        assert!(!shown.to_string().contains("sk_o"));
        assert_eq!(shown["body"]["state"], "STATE");
    }

    #[test]
    fn key_requests_are_openrouter_only() {
        let request = key_request(Provider::OpenRouter, "sk_o").unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.url, OPENROUTER_KEY_ENDPOINT);
        assert!(request.body.is_none());
        assert_eq!(request.redacted()["method"], "GET");
        assert!(matches!(
            key_request(Provider::Typesafe, "k"),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            key_request(Provider::OpenRouter, ""),
            Err(Error::MissingApiKey(_))
        ));
        let status = parse_key_status(&HttpResponse {
            status: 200,
            body: br#"{"data":{"label":"fragr","limit":5.0,"limit_remaining":4.5,"usage":0.5,"is_free_tier":false}}"#.to_vec(),
        })
        .unwrap();
        assert_eq!(status.label.as_deref(), Some("fragr"));
        assert_eq!(status.limit, Some(5.0));
        assert_eq!(status.limit_remaining, Some(4.5));
        assert_eq!(status.is_free_tier, Some(false));
        let bare = parse_key_status(&HttpResponse {
            status: 200,
            body: br#"{"limit":null}"#.to_vec(),
        })
        .unwrap();
        assert_eq!(bare.limit, None);
        assert!(matches!(
            parse_key_status(&HttpResponse {
                status: 401,
                body: br#"{"error":{"message":"No auth"}}"#.to_vec()
            }),
            Err(Error::Api { status: 401, .. })
        ));
        assert!(matches!(
            parse_key_status(&HttpResponse {
                status: 200,
                body: b"[1]".to_vec()
            }),
            Err(Error::Malformed(_))
        ));
    }

    #[test]
    fn error_messages_are_pulled_from_common_shapes() {
        assert_eq!(
            api_error_message(br#"{"error":{"message":"No cookie auth","code":401}}"#),
            "No cookie auth"
        );
        assert_eq!(
            api_error_message(br#"{"message":"rate limited"}"#),
            "rate limited"
        );
        assert_eq!(
            api_error_message(br#"{"detail":"Unauthorized"}"#),
            "Unauthorized"
        );
        assert_eq!(api_error_message(br#"{"error":"plain"}"#), "plain");
        assert_eq!(api_error_message(br#"{"error":42}"#), "42");
        assert_eq!(api_error_message(br#"{"other":true}"#), r#"{"other":true}"#);
        assert_eq!(
            api_error_message(b"  <html>bad gateway</html> "),
            "<html>bad gateway</html>"
        );
        assert_eq!(api_error_message(b""), "empty body");
        let long = "x".repeat(300);
        let shown = api_error_message(long.as_bytes());
        assert_eq!(shown.len(), 203);
        assert!(shown.ends_with("..."));
    }

    #[test]
    fn parse_decision_handles_status_and_shape() {
        let ok = HttpResponse {
            status: 200,
            body: push_answers().to_string().into_bytes(),
        };
        let parsed = parse_decision(&ok).unwrap();
        assert_eq!(parsed.answers.len(), 3);
        let err = parse_decision(&HttpResponse {
            status: 402,
            body: br#"{"error":{"message":"Insufficient credits"}}"#.to_vec(),
        })
        .unwrap_err();
        assert!(matches!(err, Error::Api { status: 402, .. }));
        assert!(err.to_string().contains("Insufficient credits"));
        assert!(matches!(
            parse_decision(&HttpResponse {
                status: 200,
                body: b"not json".to_vec()
            }),
            Err(Error::Malformed(_))
        ));
    }

    #[test]
    fn estimate_counts_body_and_questions() {
        let questions = tactical_questions();
        let request =
            decision_request(Provider::Typesafe, "jev-latest", "k", "STATE", &questions).unwrap();
        let estimate = estimate_cost(&request, &Pricing::default());
        assert!(estimate > 0.0 && estimate < 0.0001, "{estimate}");
        let pricey = Pricing {
            input_per_million: 1_000_000.0,
            output_per_million: 1_000_000.0,
        };
        let body_len = request.body.as_ref().unwrap().to_string().len() as u64;
        let expected = (body_len.div_ceil(4) + 16 + 3 * 8) as f64;
        assert!((estimate_cost(&request, &pricey) - expected).abs() < 1e-6);
        let empty = HttpRequest {
            method: Method::Get,
            url: String::new(),
            headers: vec![],
            body: None,
        };
        assert!((estimate_cost(&empty, &pricey) - 16.0).abs() < 1e-9);
    }

    #[test]
    fn decide_checks_then_records() {
        let questions = tactical_questions();
        let request =
            decision_request(Provider::Typesafe, "jev-latest", "k", "STATE", &questions).unwrap();
        let transport = FakeTransport::ok(push_answers());
        let caps = Caps {
            run_usd: 0.01,
            total_usd: None,
            run_calls: None,
        };
        let budget = Mutex::new(Budget::new(caps, Pricing::default()));
        let decision = decide(
            &transport,
            &budget,
            Provider::Typesafe,
            "jev-latest",
            &request,
        )
        .unwrap();
        assert!(decision.charge.ok);
        assert_eq!(decision.charge.input_tokens, Some(240));
        assert_eq!(decision.charge.provider, "typesafe");
        let actual = decision.charge.actual_usd.unwrap();
        assert!((actual - 240.0 * 0.042 / 1e6).abs() < 1e-15);
        assert_eq!(transport.calls(), 1);
        {
            let guard = budget.lock().unwrap();
            assert_eq!(guard.run_calls(), 1);
            assert!((guard.run_usd() - actual).abs() < 1e-15);
        }
        let sent = transport.last_request.lock().unwrap().clone().unwrap();
        assert_eq!(sent.url, TYPESAFE_ENDPOINT);

        let no_cap = Mutex::new(Budget::new(Caps::default(), Pricing::default()));
        let refused = decide(
            &transport,
            &no_cap,
            Provider::Typesafe,
            "jev-latest",
            &request,
        )
        .unwrap_err();
        assert!(matches!(
            refused,
            Error::Budget(crate::budget::Refusal::NoCap)
        ));
        assert_eq!(transport.calls(), 1, "a refused call is never sent");
        assert_eq!(no_cap.lock().unwrap().run_calls(), 0);

        let failing = FakeTransport::new(vec![Ok(HttpResponse {
            status: 500,
            body: b"boom".to_vec(),
        })]);
        let err = decide(
            &failing,
            &budget,
            Provider::Typesafe,
            "jev-latest",
            &request,
        )
        .unwrap_err();
        assert!(matches!(err, Error::Api { status: 500, .. }));
        let guard = budget.lock().unwrap();
        assert_eq!(guard.run_calls(), 2, "a failed call still counts");
        let last = guard.ledger().charges.last().unwrap();
        assert!(!last.ok);
        assert_eq!(last.actual_usd, None);

        let offline = FakeTransport::new(vec![Err(Error::Transport("refused".into()))]);
        let budget2 = Mutex::new(Budget::new(caps, Pricing::default()));
        let err = decide(&offline, &budget2, Provider::OpenRouter, "m", &request).unwrap_err();
        assert!(matches!(err, Error::Transport(_)));
        assert_eq!(budget2.lock().unwrap().run_calls(), 1);
    }

    #[test]
    fn http_transport_builds_and_reports_connection_errors() {
        let transport = HttpTransport::new(Duration::from_millis(200)).unwrap();
        let request = HttpRequest {
            method: Method::Get,
            url: "http://127.0.0.1:9/nothing".to_string(),
            headers: vec![("X-Test".into(), "1".into())],
            body: Some(json!({"a": 1})),
        };
        assert!(matches!(transport.send(&request), Err(Error::Transport(_))));
        let post = HttpRequest {
            method: Method::Post,
            ..request
        };
        assert!(matches!(transport.send(&post), Err(Error::Transport(_))));
    }
}
