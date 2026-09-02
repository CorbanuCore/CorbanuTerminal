use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use image::ColorType;
use image::ImageFormat;
use tempfile::tempdir;
use wiremock::Mock;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxServer;

#[derive(Default)]
struct LargeImageFlowResponder {
    calls: AtomicUsize,
}

impl Respond for LargeImageFlowResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => anthropic_success_response("LARGE_IMAGE_ACCEPTED"),
            1 => anthropic_success_response("SMALL_IMAGE_ACCEPTED"),
            2 => ResponseTemplate::new(413).set_body_json(serde_json::json!({
                "type": "error",
                "error": {
                    "type": "request_too_large",
                    "message": "request body exceeds 30 MiB",
                }
            })),
            3 => anthropic_success_response("RECOVERED_AFTER_413"),
            call => ResponseTemplate::new(500).set_body_json(serde_json::json!({
                "type": "error",
                "error": {
                    "type": "unexpected_request",
                    "message": format!("unexpected Fable request {call}"),
                }
            })),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_plan_fable_large_image_succeeds_and_recovers() -> Result<()> {
    if !TmuxServer::should_run("Plan Fable large-image flow")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let test_repo = std::env::var_os("CORBANU_ISOMETRIC_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.clone());
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let fixtures = tempdir()?;
    let large_image = fixtures.path().join("large-noise.png");
    let small_image = fixtures.path().join("small-noise.png");
    write_noise_png(&large_image, /*width*/ 800, /*height*/ 800)?;
    write_noise_png(&small_image, /*width*/ 32, /*height*/ 32)?;
    anyhow::ensure!(
        std::fs::metadata(&large_image)?.len() >= 1_572_864,
        "large image fixture must create a base64 request above the former 2 MiB limit"
    );

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/messages$"))
        .respond_with(LargeImageFlowResponder::default())
        .mount(&server)
        .await;
    write_test_config(codex_home.path(), &test_repo)?;

    let tmux = TmuxServer::start("tmux_plan_fable_large_image")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", codex_home.path().join("log/codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-plan-fable-large-image",
            TerminalSize::new(/*columns*/ 140, /*rows*/ 45),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("PFTERMINAL_PLAN_API_KEY", "tmux-large-image-test")
                .env("PFTERMINAL_PLAN_BASE_URL", format!("{}/v1", server.uri()))
                .env("RUST_LOG", "trace")
                .arg("-c")
                .arg("analytics.enabled=false")
                .arg("--no-alt-screen")
                .arg("-C")
                .arg(&test_repo),
        )
        .current_dir(&test_repo),
    )?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(/*secs*/ 30))?;
    attach_and_submit(
        pane,
        &large_image,
        "Describe this large image.",
        "LARGE_IMAGE_ACCEPTED",
    )?;

    start_new_conversation(pane)?;
    attach_and_submit(
        pane,
        &small_image,
        "Confirm the smaller image works.",
        "SMALL_IMAGE_ACCEPTED",
    )?;

    start_new_conversation(pane)?;
    attach_and_submit(
        pane,
        &small_image,
        "Recover after a one-time payload rejection.",
        "RECOVERED_AFTER_413",
    )?;

    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    session.wait_for_exit(Duration::from_secs(/*secs*/ 15))?;

    let requests = server
        .received_requests()
        .await
        .context("recorded Fable requests")?;
    let message_requests = requests
        .iter()
        .filter(|request| request.url.path().ends_with("/messages"))
        .collect::<Vec<_>>();
    assert_eq!(message_requests.len(), 4);
    assert!(message_requests[0].body.len() > 2 * 1024 * 1024);
    assert!(largest_image_data(&message_requests[0].body)? > 2 * 1024 * 1024);
    assert!(message_requests[1].body.len() < 2 * 1024 * 1024);
    assert!(largest_image_data(&message_requests[1].body)? > 0);
    assert_eq!(message_requests[2].body, message_requests[3].body);
    assert!(largest_image_data(&message_requests[2].body)? > 0);
    Ok(())
}

fn attach_and_submit(
    pane: &crate::support::tmux::TmuxPane<'_>,
    image: &Path,
    prompt: &str,
    response: &str,
) -> Result<()> {
    pane.send_paste(&image.to_string_lossy())?;
    pane.wait_stable_contains("[Image #1]", Duration::from_secs(/*secs*/ 10))?;
    pane.send_literal(prompt)?;
    pane.wait_stable_contains(prompt, Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(response, Duration::from_secs(/*secs*/ 45))?;
    Ok(())
}

fn start_new_conversation(pane: &crate::support::tmux::TmuxPane<'_>) -> Result<()> {
    let prior_viewport = pane.capture_viewport()?;
    pane.send_literal("/new")?;
    pane.wait_stable_contains("/new", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_until(
        "fresh conversation composer",
        Duration::from_secs(/*secs*/ 15),
        |capture| {
            capture.lines().any(|line| {
                line.contains("corbanu resume ") && !prior_viewport.contains(line.trim())
            })
        },
    )?;
    Ok(())
}

fn write_noise_png(path: &Path, width: u32, height: u32) -> Result<()> {
    let mut state = 0x9e37_79b9_u32;
    let mut pixels = vec![0_u8; width as usize * height as usize * 3];
    for pixel in &mut pixels {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *pixel = (state >> 24) as u8;
    }
    image::save_buffer_with_format(
        path,
        &pixels,
        width,
        height,
        ColorType::Rgb8,
        ImageFormat::Png,
    )
    .with_context(|| format!("write image fixture {}", path.display()))
}

fn largest_image_data(body: &[u8]) -> Result<usize> {
    let value: serde_json::Value = serde_json::from_slice(body)?;
    Ok(largest_image_data_in_value(&value))
}

fn largest_image_data_in_value(value: &serde_json::Value) -> usize {
    let current = value
        .pointer("/source/data")
        .and_then(serde_json::Value::as_str)
        .map_or(0, str::len);
    match value {
        serde_json::Value::Array(values) => values
            .iter()
            .map(largest_image_data_in_value)
            .max()
            .unwrap_or(current)
            .max(current),
        serde_json::Value::Object(object) => object
            .values()
            .map(largest_image_data_in_value)
            .max()
            .unwrap_or(current)
            .max(current),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => current,
    }
}

fn anthropic_success_response(text: &str) -> ResponseTemplate {
    let body = format!(
        concat!(
            "event: message_start\n",
            "data: {{\"type\":\"message_start\",\"message\":{{\"id\":\"msg_tmux_image\",",
            "\"model\":\"claude-fable-5\",\"usage\":{{\"input_tokens\":10,\"output_tokens\":0}}}}}}\n\n",
            "event: content_block_start\n",
            "data: {{\"type\":\"content_block_start\",\"index\":0,",
            "\"content_block\":{{\"type\":\"text\",\"text\":\"\"}}}}\n\n",
            "event: content_block_delta\n",
            "data: {{\"type\":\"content_block_delta\",\"index\":0,",
            "\"delta\":{{\"type\":\"text_delta\",\"text\":\"{text}\"}}}}\n\n",
            "event: content_block_stop\n",
            "data: {{\"type\":\"content_block_stop\",\"index\":0}}\n\n",
            "event: message_delta\n",
            "data: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"end_turn\"}},",
            "\"usage\":{{\"input_tokens\":10,\"output_tokens\":4}}}}\n\n",
            "event: message_stop\n",
            "data: {{\"type\":\"message_stop\"}}\n\n"
        ),
        text = text
    );
    ResponseTemplate::new(200).set_body_raw(body, "text/event-stream")
}

fn codex_binary(repo_root: &Path) -> Result<PathBuf> {
    if let Ok(path) = codex_utils_cargo_bin::cargo_bin("codex") {
        return Ok(path);
    }
    if let Ok(path) = codex_utils_cargo_bin::cargo_bin("codex-tui") {
        return Ok(path);
    }
    for binary in ["codex", "codex-tui"] {
        let fallback = repo_root.join("codex-rs/target/debug").join(binary);
        if fallback.is_file() {
            return Ok(fallback);
        }
    }
    anyhow::bail!("Corbanu TUI binary is unavailable; build `codex` or `codex-tui` first")
}

fn write_test_config(codex_home: &Path, test_repo: &Path) -> Result<()> {
    let test_repo = serde_json::to_string(&test_repo.to_string_lossy())?;
    let log_dir = codex_home.join("log");
    let log_dir = serde_json::to_string(&log_dir.to_string_lossy())?;
    let config = format!(
        "model = \"claude-fable-5\"\nmodel_provider = \"pfterminal-plan-anthropic\"\n\
         model_reasoning_effort = \"low\"\nlog_dir = {log_dir}\n\
         suppress_unstable_features_warning = true\n\n\
         [projects.{test_repo}]\ntrust_level = \"trusted\"\n"
    );
    std::fs::write(codex_home.join("config.toml"), config)
        .context("write large-image TUI test configuration")
}
