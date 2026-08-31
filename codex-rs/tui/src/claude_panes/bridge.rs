//! Local HTTP bridge that translates between Claude Code's Anthropic API client
//! and upstream provider endpoints (Ambient Chat, Anthropic passthrough).

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;

use super::bridge_translate::ambient_chat_messages_from_claude_request;
use super::bridge_translate::ambient_chat_tools_from_claude_request;
use super::bridge_translate::anthropic_message_response;
use super::bridge_translate::anthropic_tool_use_response;
use super::bridge_translate::bridge_tool_calls_from_ambient_response;
use super::bridge_translate::write_anthropic_stream_error;
use super::bridge_translate::write_anthropic_stream_headers;
use super::bridge_translate::write_anthropic_stream_ping;
use super::bridge_translate::write_anthropic_stream_text_completion;
use super::bridge_translate::write_anthropic_stream_tool_use_completion;
use super::bridge_translate::write_json_response;
use super::bridge_translate::write_json_status_response;
use super::bridge_translate::write_raw_http_response;
use super::turn_types::ClaudeBridgeKind;
use super::turn_types::ClaudeBridgePlan;

pub(crate) const AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS: usize = 3;
const ANTHROPIC_OAUTH_BETA: &str = "oauth-2025-04-20";
pub(crate) async fn run_claude_bridge(plan: ClaudeBridgePlan) -> Result<()> {
    let listener = TcpListener::from_std(plan.listener)
        .context("failed to create async Claude bridge listener")?;
    let api_key = Arc::new(
        plan.upstream_api_key
            .context("Claude bridge provider credential was not resolved")?,
    );
    let client_auth_token = Arc::new(plan.client_auth_token);
    let upstream_base_url = Arc::new(plan.upstream_base_url);
    let upstream_model = Arc::new(plan.upstream_model);
    let kind = plan.kind;
    let http = reqwest::Client::new();
    loop {
        let (stream, _) = listener.accept().await?;
        let api_key = api_key.clone();
        let client_auth_token = client_auth_token.clone();
        let upstream_base_url = upstream_base_url.clone();
        let upstream_model = upstream_model.clone();
        let http = http.clone();
        tokio::spawn(async move {
            let result = match kind {
                ClaudeBridgeKind::AmbientChat => {
                    handle_ambient_bridge_connection(
                        stream,
                        client_auth_token,
                        api_key,
                        upstream_model,
                        http,
                    )
                    .await
                }
                ClaudeBridgeKind::AnthropicPassthrough => {
                    handle_anthropic_passthrough_bridge_connection(
                        stream,
                        client_auth_token,
                        api_key,
                        upstream_base_url,
                        http,
                        /*proxy_count_tokens*/ false,
                    )
                    .await
                }
                ClaudeBridgeKind::AnthropicOauthPassthrough => {
                    handle_anthropic_passthrough_bridge_connection(
                        stream,
                        client_auth_token,
                        api_key,
                        upstream_base_url,
                        http,
                        /*proxy_count_tokens*/ true,
                    )
                    .await
                }
            };
            if let Err(err) = result {
                tracing::debug!(error = %err, "Claude bridge connection failed");
            }
        });
    }
}

pub(crate) async fn handle_ambient_bridge_connection(
    mut stream: tokio::net::TcpStream,
    client_auth_token: Arc<String>,
    api_key: Arc<String>,
    upstream_model: Arc<String>,
    http: reqwest::Client,
) -> Result<()> {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut temp).await?;
        if read == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(pos) = find_header_end(&buffer) {
            break pos;
        }
        if buffer.len() > 1024 * 1024 {
            return Err(anyhow!("Ambient Claude bridge request headers too large"));
        }
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    if !bridge_client_is_authorized(&headers, client_auth_token.as_str()) {
        write_json_status_response(
            &mut stream,
            /*status*/ 401,
            serde_json::json!({
                "error": { "type": "authentication_error", "message": "invalid bridge credential" }
            }),
        )
        .await?;
        return Ok(());
    }
    let request_line = headers.lines().next().unwrap_or_default().to_string();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = stream.read(&mut temp).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
    }
    let body = &buffer[body_start..buffer.len().min(body_start + content_length)];

    if request_line.contains("/v1/messages/count_tokens") {
        write_json_response(&mut stream, serde_json::json!({ "input_tokens": 1 })).await?;
        return Ok(());
    }

    if !request_line.contains("/v1/messages") {
        write_json_status_response(
            &mut stream,
            /*status*/ 404,
            serde_json::json!({ "error": { "type": "not_found", "message": "not found" } }),
        )
        .await?;
        return Ok(());
    }

    let request: Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => {
            write_json_status_response(
                &mut stream,
                /*status*/ 400,
                serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "invalid_request_error",
                        "message": format!("invalid Claude Messages request: {err}")
                    }
                }),
            )
            .await?;
            return Ok(());
        }
    };
    let wants_stream = request
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let max_tokens = request
        .get("max_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(1024)
        .max(1);
    let chat_messages = match ambient_chat_messages_from_claude_request(&request) {
        Ok(messages) => messages,
        Err(err) => {
            write_json_status_response(
                &mut stream,
                /*status*/ 400,
                serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "request_translation_error",
                        "message": err.to_string()
                    }
                }),
            )
            .await?;
            return Ok(());
        }
    };
    let chat_tools = ambient_chat_tools_from_claude_request(&request);
    let mut upstream_body = serde_json::json!({
        "model": upstream_model.as_str(),
        "messages": chat_messages,
        "max_tokens": max_tokens,
    });
    if !chat_tools.is_empty() {
        upstream_body["tools"] = Value::Array(chat_tools);
        upstream_body["tool_choice"] = Value::String("auto".to_string());
    }
    let response = if wants_stream {
        send_ambient_chat_request_with_stream_heartbeat(
            &mut stream,
            upstream_model.as_str(),
            &http,
            api_key.as_str(),
            &upstream_body,
        )
        .await
    } else {
        send_ambient_chat_request_with_retry(&http, api_key.as_str(), &upstream_body).await
    };
    let (status, response_text) = match response {
        Ok(response) => response,
        Err(err) => {
            if wants_stream {
                write_anthropic_stream_error(
                    &mut stream,
                    "upstream_transport_error",
                    &format!("Ambient Claude bridge upstream transport error: {err}"),
                )
                .await?;
            } else {
                write_json_status_response(
                    &mut stream,
                    /*status*/ 502,
                    serde_json::json!({
                        "type": "error",
                        "error": {
                            "type": "upstream_transport_error",
                            "message": err.to_string()
                        }
                    }),
                )
                .await?;
            }
            return Ok(());
        }
    };
    if !status.is_success() {
        if wants_stream {
            write_anthropic_stream_error(
                &mut stream,
                "upstream_error",
                &format!(
                    "Ambient Claude bridge upstream returned HTTP {}: {response_text}",
                    status.as_u16()
                ),
            )
            .await?;
        } else {
            write_json_status_response(
                &mut stream,
                status.as_u16(),
                serde_json::json!({
                    "type": "error",
                    "error": {
                        "type": "upstream_error",
                        "message": response_text
                    }
                }),
            )
            .await?;
        }
        return Ok(());
    }

    let upstream: Value = match serde_json::from_str(&response_text) {
        Ok(upstream) => upstream,
        Err(err) => {
            if wants_stream {
                write_anthropic_stream_error(
                    &mut stream,
                    "upstream_invalid_json",
                    &format!("Ambient Chat response was not JSON: {err}"),
                )
                .await?;
            } else {
                write_json_status_response(
                    &mut stream,
                    /*status*/ 502,
                    serde_json::json!({
                        "type": "error",
                        "error": {
                            "type": "upstream_invalid_json",
                            "message": format!("Ambient Chat response was not JSON: {err}")
                        }
                    }),
                )
                .await?;
            }
            return Ok(());
        }
    };
    let usage = upstream.get("usage").cloned().unwrap_or_else(|| {
        serde_json::json!({
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        })
    });
    let tool_calls = bridge_tool_calls_from_ambient_response(&upstream);
    if !tool_calls.is_empty() {
        if wants_stream {
            write_anthropic_stream_tool_use_completion(
                &mut stream,
                upstream_model.as_str(),
                &tool_calls,
                &usage,
            )
            .await?;
        } else {
            write_json_response(
                &mut stream,
                anthropic_tool_use_response(upstream_model.as_str(), &tool_calls, &usage),
            )
            .await?;
        }
        return Ok(());
    }

    let text = upstream
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or("OK")
        .to_string();
    if wants_stream {
        write_anthropic_stream_text_completion(&mut stream, upstream_model.as_str(), &text, &usage)
            .await?;
    } else {
        write_json_response(
            &mut stream,
            anthropic_message_response(upstream_model.as_str(), &text, &usage),
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn handle_anthropic_passthrough_bridge_connection(
    mut stream: tokio::net::TcpStream,
    client_auth_token: Arc<String>,
    api_key: Arc<String>,
    upstream_base_url: Arc<String>,
    http: reqwest::Client,
    proxy_count_tokens: bool,
) -> Result<()> {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut temp).await?;
        if read == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(pos) = find_header_end(&buffer) {
            break pos;
        }
        if buffer.len() > 1024 * 1024 {
            return Err(anyhow!(
                "Anthropic passthrough bridge request headers too large"
            ));
        }
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
    if !bridge_client_is_authorized(&headers, client_auth_token.as_str()) {
        write_json_status_response(
            &mut stream,
            /*status*/ 401,
            serde_json::json!({
                "error": { "type": "authentication_error", "message": "invalid bridge credential" }
            }),
        )
        .await?;
        return Ok(());
    }
    let request_line = headers.lines().next().unwrap_or_default().to_string();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = stream.read(&mut temp).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
    }
    let body = &buffer[body_start..buffer.len().min(body_start + content_length)];

    if !proxy_count_tokens && request_line.contains("/v1/messages/count_tokens") {
        write_json_response(&mut stream, serde_json::json!({ "input_tokens": 1 })).await?;
        return Ok(());
    }

    if !request_line.contains("/v1/messages") {
        write_json_status_response(
            &mut stream,
            /*status*/ 404,
            serde_json::json!({ "error": { "type": "not_found", "message": "not found" } }),
        )
        .await?;
        return Ok(());
    }

    let upstream_path = request_target_from_request_line(&request_line).unwrap_or("/v1/messages");
    let upstream_url = format!(
        "{}{}",
        upstream_base_url.trim_end_matches('/'),
        upstream_path
    );
    let mut upstream_request = http
        .post(upstream_url)
        .bearer_auth(api_key.as_str())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            "anthropic-version",
            request_header_value(&headers, "anthropic-version").unwrap_or("2023-06-01"),
        );
    if proxy_count_tokens {
        upstream_request = upstream_request.header(
            "anthropic-beta",
            anthropic_oauth_beta_header(request_header_value(&headers, "anthropic-beta")),
        );
    } else if let Some(beta) = request_header_value(&headers, "anthropic-beta") {
        upstream_request = upstream_request.header("anthropic-beta", beta);
    }
    let response = upstream_request
        .body(body.to_vec())
        .send()
        .await
        .context("Anthropic passthrough bridge upstream request failed")?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    let response_body = response
        .bytes()
        .await
        .context("failed to read Anthropic passthrough bridge response")?;
    write_raw_http_response(
        &mut stream,
        status.as_u16(),
        status.canonical_reason().unwrap_or("OK"),
        &content_type,
        response_body.as_ref(),
    )
    .await?;
    Ok(())
}

fn anthropic_oauth_beta_header(incoming: Option<&str>) -> String {
    let incoming = incoming.unwrap_or_default().trim();
    if incoming
        .split(',')
        .map(str::trim)
        .any(|beta| beta.eq_ignore_ascii_case(ANTHROPIC_OAUTH_BETA))
    {
        incoming.to_string()
    } else if incoming.is_empty() {
        ANTHROPIC_OAUTH_BETA.to_string()
    } else {
        format!("{incoming},{ANTHROPIC_OAUTH_BETA}")
    }
}

pub(crate) async fn send_ambient_chat_request_with_retry(
    http: &reqwest::Client,
    api_key: &str,
    upstream_body: &Value,
) -> Result<(reqwest::StatusCode, String)> {
    let mut last_error = None;
    for attempt in 1..=AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS {
        let response = http
            .post("https://api.ambient.xyz/v1/chat/completions")
            .bearer_auth(api_key)
            .json(upstream_body)
            .send()
            .await;

        match response {
            Ok(response) => {
                let status = response.status();
                let should_retry =
                    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
                let retry_delay = ambient_retry_after_delay(response.headers())
                    .unwrap_or_else(|| ambient_bridge_retry_delay(attempt));
                match response.text().await {
                    Ok(response_text) => {
                        if should_retry && attempt < AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS {
                            tracing::warn!(
                                status = status.as_u16(),
                                attempt,
                                max_attempts = AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS,
                                "Ambient Claude bridge upstream returned retriable status"
                            );
                            sleep_ambient_bridge_retry(retry_delay).await;
                            continue;
                        }
                        return Ok((status, response_text));
                    }
                    Err(err) => {
                        let error = anyhow!("Ambient Chat bridge failed to read response: {err}");
                        if should_retry && attempt < AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS {
                            tracing::warn!(
                                status = status.as_u16(),
                                attempt,
                                max_attempts = AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS,
                                error = %error,
                                "Ambient Claude bridge failed to read retriable upstream response"
                            );
                            sleep_ambient_bridge_retry(retry_delay).await;
                            continue;
                        }
                        return Err(error);
                    }
                }
            }
            Err(err) => {
                last_error = Some(anyhow!(
                    "Ambient Chat bridge upstream request failed: {err}"
                ));
            }
        }

        if attempt < AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS {
            let error = last_error
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "unknown upstream transport failure".to_string());
            tracing::warn!(
                attempt,
                max_attempts = AMBIENT_BRIDGE_UPSTREAM_MAX_ATTEMPTS,
                error = %error,
                "Ambient Claude bridge upstream transport failed"
            );
            sleep_ambient_bridge_retry(ambient_bridge_retry_delay(attempt)).await;
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("Ambient Chat bridge upstream request failed")))
}

pub(crate) async fn send_ambient_chat_request_with_stream_heartbeat(
    stream: &mut tokio::net::TcpStream,
    _model: &str,
    http: &reqwest::Client,
    api_key: &str,
    upstream_body: &Value,
) -> Result<(reqwest::StatusCode, String)> {
    write_anthropic_stream_headers(stream).await?;
    let request = send_ambient_chat_request_with_retry(http, api_key, upstream_body);
    tokio::pin!(request);
    let mut heartbeat = interval(Duration::from_secs(10));
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    heartbeat.tick().await;
    loop {
        tokio::select! {
            result = &mut request => return result,
            _ = heartbeat.tick() => {
                write_anthropic_stream_ping(stream).await?;
            }
        }
    }
}

pub(crate) fn ambient_retry_after_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let retry_after = headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    let seconds = retry_after.parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds.min(300)))
}

pub(crate) fn ambient_bridge_retry_delay(attempt: usize) -> Duration {
    Duration::from_millis((attempt as u64).saturating_mul(250))
}

pub(crate) async fn sleep_ambient_bridge_retry(delay: Duration) {
    tokio::time::sleep(delay).await;
}

pub(crate) fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

pub(crate) fn request_target_from_request_line(request_line: &str) -> Option<&str> {
    let mut parts = request_line.split_whitespace();
    let _method = parts.next()?;
    parts.next()
}

fn request_header_value<'a>(headers: &'a str, expected_name: &str) -> Option<&'a str> {
    headers.lines().skip(1).find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case(expected_name)
            .then_some(value.trim())
            .filter(|value| !value.is_empty())
    })
}

fn bridge_client_is_authorized(headers: &str, expected_token: &str) -> bool {
    let Some(value) = request_header_value(headers, "authorization") else {
        return false;
    };
    let mut parts = value.split_whitespace();
    let Some(scheme) = parts.next() else {
        return false;
    };
    let Some(token) = parts.next() else {
        return false;
    };
    scheme.eq_ignore_ascii_case("bearer")
        && parts.next().is_none()
        && constant_time_equal(token.as_bytes(), expected_token.as_bytes())
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len());
    let mut difference = left.len() ^ right.len();
    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}
