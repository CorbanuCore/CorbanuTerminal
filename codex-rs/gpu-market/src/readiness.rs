use crate::SecretValue;
use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointReadinessReport {
    pub rejects_missing_token: bool,
    pub rejects_wrong_token: bool,
    pub model_identity_ok: bool,
    pub chat_ok: bool,
    pub streaming_ok: bool,
    pub cancellation_ok: bool,
    pub tool_call_ok: bool,
}

impl EndpointReadinessReport {
    pub fn ready(&self) -> bool {
        self.rejects_missing_token
            && self.rejects_wrong_token
            && self.model_identity_ok
            && self.chat_ok
            && self.streaming_ok
            && self.cancellation_ok
            && self.tool_call_ok
    }
}

#[derive(Clone)]
pub struct GpuEndpointProber {
    client: reqwest::Client,
    request_timeout: Duration,
    cancellation_deadline: Duration,
}

impl std::fmt::Debug for GpuEndpointProber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuEndpointProber")
            .field("request_timeout", &self.request_timeout)
            .field("cancellation_deadline", &self.cancellation_deadline)
            .finish()
    }
}

impl Default for GpuEndpointProber {
    fn default() -> Self {
        Self::new(Duration::from_secs(30), Duration::from_millis(500))
    }
}

impl GpuEndpointProber {
    pub fn new(request_timeout: Duration, cancellation_deadline: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            request_timeout,
            cancellation_deadline,
        }
    }

    pub async fn probe(
        &self,
        base_url: &str,
        model_id: &str,
        token: &SecretValue,
    ) -> anyhow::Result<EndpointReadinessReport> {
        let base_url = validate_base_url(base_url)?;
        let models_url = format!("{base_url}/models");
        let chat_url = format!("{base_url}/chat/completions");

        let no_token = self
            .client
            .get(models_url.as_str())
            .timeout(self.request_timeout)
            .send()
            .await?;
        let rejects_missing_token = unauthorized(no_token.status());

        let wrong_token = self
            .client
            .get(models_url.as_str())
            .bearer_auth("pft-intentionally-wrong-token")
            .timeout(self.request_timeout)
            .send()
            .await?;
        let rejects_wrong_token = unauthorized(wrong_token.status());

        let models = self
            .client
            .get(models_url)
            .bearer_auth(token.expose())
            .timeout(self.request_timeout)
            .send()
            .await?;
        let model_identity_ok = models.status().is_success()
            && models
                .json::<Value>()
                .await?
                .get("data")
                .and_then(Value::as_array)
                .is_some_and(|models| {
                    models
                        .iter()
                        .any(|model| model.get("id").and_then(Value::as_str) == Some(model_id))
                });

        let chat = self
            .client
            .post(chat_url.as_str())
            .bearer_auth(token.expose())
            .timeout(self.request_timeout)
            .json(&chat_body(model_id, "Reply with exactly READY.", false))
            .send()
            .await?;
        let chat_ok = chat.status().is_success()
            && chat
                .json::<Value>()
                .await?
                .pointer("/choices/0/message/content")
                .and_then(Value::as_str)
                .is_some_and(|content| !content.is_empty());

        let streaming = self
            .client
            .post(chat_url.as_str())
            .bearer_auth(token.expose())
            .timeout(self.request_timeout)
            .json(&chat_body(model_id, "Stream two short words.", true))
            .send()
            .await?;
        let streaming_ok = if streaming.status().is_success() {
            let text = streaming.text().await?;
            text.lines().any(|line| line.starts_with("data:")) && text.contains("[DONE]")
        } else {
            false
        };

        let cancellation = self
            .client
            .post(chat_url.as_str())
            .bearer_auth(token.expose())
            .timeout(self.request_timeout)
            .json(&chat_body(
                model_id,
                "Generate a deliberately long response for cancellation testing.",
                true,
            ))
            .send();
        let cancellation_ok = tokio::time::timeout(self.cancellation_deadline, cancellation)
            .await
            .is_err();

        let tools = self
            .client
            .post(chat_url)
            .bearer_auth(token.expose())
            .timeout(self.request_timeout)
            .json(&serde_json::json!({
                "model": model_id,
                "messages": [{"role": "user", "content": "Call readiness_probe with value ok."}],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "readiness_probe",
                        "description": "Readiness contract probe",
                        "parameters": {
                            "type": "object",
                            "properties": {"value": {"type": "string"}},
                            "required": ["value"],
                            "additionalProperties": false
                        }
                    }
                }],
                "tool_choice": {"type": "function", "function": {"name": "readiness_probe"}}
            }))
            .send()
            .await?;
        let tool_call_ok = tools.status().is_success()
            && tools
                .json::<Value>()
                .await?
                .pointer("/choices/0/message/tool_calls/0/function/name")
                .and_then(Value::as_str)
                == Some("readiness_probe");

        Ok(EndpointReadinessReport {
            rejects_missing_token,
            rejects_wrong_token,
            model_identity_ok,
            chat_ok,
            streaming_ok,
            cancellation_ok,
            tool_call_ok,
        })
    }
}

fn chat_body(model_id: &str, prompt: &str, stream: bool) -> Value {
    serde_json::json!({
        "model": model_id,
        "messages": [{"role": "user", "content": prompt}],
        "stream": stream,
        "max_tokens": 32
    })
}

fn validate_base_url(base_url: &str) -> anyhow::Result<String> {
    let parsed = reqwest::Url::parse(base_url)?;
    anyhow::ensure!(
        parsed.scheme() == "https"
            || parsed
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "::1" | "localhost")),
        "GPU endpoint must use HTTPS or loopback"
    );
    Ok(base_url.trim_end_matches('/').to_string())
}

fn unauthorized(status: StatusCode) -> bool {
    matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
}

#[cfg(test)]
#[path = "readiness_tests.rs"]
mod tests;
