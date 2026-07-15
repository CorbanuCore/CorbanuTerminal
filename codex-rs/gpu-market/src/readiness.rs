use crate::SecretValue;
use reqwest::StatusCode;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
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

pub trait GpuReadinessProbe: Send + Sync {
    fn probe<'a>(
        &'a self,
        base_url: &'a str,
        model_id: &'a str,
        token: &'a SecretValue,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<EndpointReadinessReport>> + Send + 'a>>;

    fn probe_health<'a>(
        &'a self,
        base_url: &'a str,
        model_id: &'a str,
        token: &'a SecretValue,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + Send + 'a>> {
        Box::pin(async move { Ok(self.probe(base_url, model_id, token).await?.ready()) })
    }
}

#[derive(Clone)]
pub struct GpuEndpointProber {
    client: reqwest::Client,
    request_timeout: Duration,
    cold_chat_timeout: Duration,
    cancellation_deadline: Duration,
}

impl std::fmt::Debug for GpuEndpointProber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuEndpointProber")
            .field("request_timeout", &self.request_timeout)
            .field("cold_chat_timeout", &self.cold_chat_timeout)
            .field("cancellation_deadline", &self.cancellation_deadline)
            .finish()
    }
}

impl Default for GpuEndpointProber {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            request_timeout: Duration::from_secs(30),
            // Large MoE runtimes can accept HTTP before their first-inference kernels finish
            // compiling. Keep one cold request alive instead of injecting a new timed-out probe
            // every controller retry interval.
            cold_chat_timeout: Duration::from_secs(10 * 60),
            cancellation_deadline: Duration::from_millis(500),
        }
    }
}

impl GpuEndpointProber {
    pub fn new(request_timeout: Duration, cancellation_deadline: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            request_timeout,
            cold_chat_timeout: request_timeout,
            cancellation_deadline,
        }
    }

    pub fn with_cold_chat_timeout(mut self, cold_chat_timeout: Duration) -> Self {
        self.cold_chat_timeout = cold_chat_timeout;
        self
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
            .timeout(self.cold_chat_timeout)
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

        let cancellation_client = self.client.clone();
        let cancellation_url = chat_url.clone();
        let cancellation_model = model_id.to_string();
        let cancellation_token = token.expose().to_string();
        let cancellation_timeout = self.request_timeout;
        let cancellation = tokio::spawn(async move {
            let response = cancellation_client
                .post(cancellation_url)
                .bearer_auth(cancellation_token)
                .timeout(cancellation_timeout)
                .json(&serde_json::json!({
                    "model": cancellation_model,
                    "messages": [{
                        "role": "user",
                        "content": "Generate a deliberately long response for cancellation testing."
                    }],
                    "stream": true,
                    "max_tokens": 4_096
                }))
                .send()
                .await?;
            anyhow::ensure!(response.status().is_success());
            let _ = response.bytes().await?;
            anyhow::Ok(())
        });
        tokio::time::sleep(self.cancellation_deadline).await;
        let cancellation_was_dropped = if cancellation.is_finished() {
            false
        } else {
            cancellation.abort();
            cancellation.await.is_err_and(|error| error.is_cancelled())
        };
        // Dropping an in-flight request is only useful if it does not poison the endpoint/client.
        // A fresh authenticated request must still work immediately afterward.
        let cancellation_ok = if cancellation_was_dropped {
            self.client
                .get(format!("{base_url}/models"))
                .bearer_auth(token.expose())
                .timeout(self.request_timeout)
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
        } else {
            false
        };

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

    pub async fn probe_health(
        &self,
        base_url: &str,
        model_id: &str,
        token: &SecretValue,
    ) -> anyhow::Result<bool> {
        let base_url = validate_base_url(base_url)?;
        let models_url = format!("{base_url}/models");
        let no_token = self
            .client
            .get(models_url.as_str())
            .timeout(self.request_timeout)
            .send()
            .await?;
        let wrong_token = self
            .client
            .get(models_url.as_str())
            .bearer_auth("pft-intentionally-wrong-token")
            .timeout(self.request_timeout)
            .send()
            .await?;
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
        Ok(unauthorized(no_token.status())
            && unauthorized(wrong_token.status())
            && model_identity_ok)
    }
}

impl GpuReadinessProbe for GpuEndpointProber {
    fn probe<'a>(
        &'a self,
        base_url: &'a str,
        model_id: &'a str,
        token: &'a SecretValue,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<EndpointReadinessReport>> + Send + 'a>> {
        Box::pin(GpuEndpointProber::probe(self, base_url, model_id, token))
    }

    fn probe_health<'a>(
        &'a self,
        base_url: &'a str,
        model_id: &'a str,
        token: &'a SecretValue,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + Send + 'a>> {
        Box::pin(GpuEndpointProber::probe_health(
            self, base_url, model_id, token,
        ))
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
