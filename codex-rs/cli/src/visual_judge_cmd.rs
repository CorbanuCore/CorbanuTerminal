use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use clap::Args;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::path::Path;
use std::path::PathBuf;

const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "google/gemini-3.1-pro-preview";
const MAX_VIDEO_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Args)]
pub(crate) struct VisualJudgeCommand {
    /// MP4 screen recording captured through the real user-facing interface.
    #[arg(long)]
    video: PathBuf,

    /// Plain-text visual acceptance rubric for the recorded flow.
    #[arg(long)]
    rubric: PathBuf,

    /// Destination for the machine-readable verdict and defect list.
    #[arg(long)]
    out: PathBuf,

    /// OpenRouter vision-capable model.
    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Verdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct VisualDefect {
    timestamp_seconds: f64,
    severity: String,
    category: String,
    observation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ModelVerdict {
    verdict: Verdict,
    summary: String,
    #[serde(default)]
    defects: Vec<VisualDefect>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualJudgeReport {
    version: u8,
    model: String,
    video: String,
    rubric: String,
    estimated_cost_usd: Option<f64>,
    verdict: Verdict,
    summary: String,
    defects: Vec<VisualDefect>,
}

#[derive(Debug, Deserialize)]
struct CompletionEnvelope {
    choices: Vec<CompletionChoice>,
    usage: Option<CompletionUsage>,
}

#[derive(Debug, Deserialize)]
struct CompletionUsage {
    cost: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct CompletionChoice {
    message: CompletionMessage,
}

#[derive(Debug, Deserialize)]
struct CompletionMessage {
    content: String,
}

pub(crate) async fn run(command: VisualJudgeCommand) -> anyhow::Result<()> {
    let video = tokio::fs::read(&command.video)
        .await
        .with_context(|| format!("read video {}", command.video.display()))?;
    anyhow::ensure!(!video.is_empty(), "video is empty");
    anyhow::ensure!(
        video.len() <= MAX_VIDEO_BYTES,
        "video exceeds the 100 MiB acceptance limit"
    );
    anyhow::ensure!(
        command.video.extension().and_then(|value| value.to_str()) == Some("mp4"),
        "visual-judge currently accepts MP4 recordings"
    );
    let rubric = tokio::fs::read_to_string(&command.rubric)
        .await
        .with_context(|| format!("read rubric {}", command.rubric.display()))?;
    anyhow::ensure!(!rubric.trim().is_empty(), "rubric is empty");

    let api_key = std::env::var("OPENROUTER_API_KEY").context(
        "OPENROUTER_API_KEY is required; fetch it from the PFTerminal vault at use time",
    )?;
    anyhow::ensure!(!api_key.trim().is_empty(), "OPENROUTER_API_KEY is empty");

    let encoded = BASE64.encode(video);
    let prompt = visual_judge_prompt(&rubric);
    let request = json!({
        "model": command.model,
        "temperature": 0,
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": prompt},
                {"type": "video_url", "video_url": {"url": format!("data:video/mp4;base64,{encoded}")}}
            ]
        }]
    });
    let response = reqwest::Client::new()
        .post(OPENROUTER_ENDPOINT)
        .bearer_auth(api_key.trim())
        .header("HTTP-Referer", "https://github.com/agtico/PfTerminal")
        .header("X-Title", "PFTerminal visual-judge")
        .json(&request)
        .send()
        .await
        .context("send visual judge request")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("read visual judge response")?;
    anyhow::ensure!(
        status.is_success(),
        "visual judge provider returned {status}"
    );
    let envelope: CompletionEnvelope =
        serde_json::from_str(&body).context("parse visual judge provider response")?;
    let content = envelope
        .choices
        .first()
        .context("visual judge provider returned no choices")?
        .message
        .content
        .trim();
    let verdict = parse_model_verdict(content)?;

    let report = VisualJudgeReport {
        version: 1,
        model: command.model,
        video: display_path(&command.video),
        rubric: display_path(&command.rubric),
        estimated_cost_usd: envelope.usage.and_then(|usage| usage.cost),
        verdict: verdict.verdict.clone(),
        summary: verdict.summary,
        defects: verdict.defects,
    };
    if let Some(parent) = command
        .out
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    tokio::fs::write(
        &command.out,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .await
    .with_context(|| format!("write verdict {}", command.out.display()))?;

    if acceptance_passes(&report.verdict, &report.defects) {
        println!("PASS visual acceptance: {}", command.out.display());
        Ok(())
    } else {
        anyhow::bail!(
            "FAIL visual acceptance: {} visible defect(s); report: {}",
            report.defects.len(),
            command.out.display()
        )
    }
}

fn visual_judge_prompt(rubric: &str) -> String {
    format!(
        "You are a strict visual acceptance judge. Inspect only what a real user can see in the attached complete screen recording. Evaluate every requirement in the rubric, including idle periods and interaction transitions. Do not infer correctness from hidden state. Any visible violation, instability, clipping, unintended motion, stalled interaction, incorrect direction, or unprofessional presentation is a defect and makes the verdict fail. Return exactly one JSON object with this schema and no markdown: {{\"verdict\":\"pass|fail\",\"summary\":\"string\",\"defects\":[{{\"timestamp_seconds\":0.0,\"severity\":\"p0|p1|p2\",\"category\":\"string\",\"observation\":\"specific visible fact\"}}]}}. A pass must have an empty defects array.\n\nRUBRIC:\n{rubric}"
    )
}

fn parse_model_verdict(content: &str) -> anyhow::Result<ModelVerdict> {
    let verdict: ModelVerdict =
        serde_json::from_str(content).context("vision model did not return exact verdict JSON")?;
    anyhow::ensure!(
        !verdict.summary.trim().is_empty(),
        "vision verdict summary is empty"
    );
    for defect in &verdict.defects {
        anyhow::ensure!(
            defect.timestamp_seconds.is_finite() && defect.timestamp_seconds >= 0.0,
            "vision defect timestamp must be a non-negative number"
        );
        anyhow::ensure!(
            matches!(defect.severity.as_str(), "p0" | "p1" | "p2"),
            "vision defect severity must be p0, p1, or p2"
        );
        anyhow::ensure!(
            !defect.category.trim().is_empty() && !defect.observation.trim().is_empty(),
            "vision defect category and observation are required"
        );
    }
    anyhow::ensure!(
        !matches!(verdict.verdict, Verdict::Pass) || verdict.defects.is_empty(),
        "contradictory vision verdict: pass contains visible defects"
    );
    Ok(verdict)
}

fn acceptance_passes(verdict: &Verdict, defects: &[VisualDefect]) -> bool {
    matches!(verdict, Verdict::Pass) && defects.is_empty()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_clean_visual_result() {
        let parsed = parse_model_verdict(
            r#"{"verdict":"pass","summary":"The recorded flow remains stable.","defects":[]}"#,
        )
        .expect("valid pass");
        assert!(acceptance_passes(&parsed.verdict, &parsed.defects));
    }

    #[test]
    fn rejects_visible_behavior_defects_without_category_special_cases() {
        let parsed = parse_model_verdict(
            r#"{"verdict":"fail","summary":"Visible interaction defects remain.","defects":[{"timestamp_seconds":2.4,"severity":"p1","category":"avatar drifts after input ends","observation":"The avatar continues moving during the idle hold."},{"timestamp_seconds":7.1,"severity":"p1","category":"diagonal route does not converge","observation":"The actor oscillates away from the selected destination."}]}"#,
        )
        .expect("valid failure");
        assert!(!acceptance_passes(&parsed.verdict, &parsed.defects));
        assert_eq!(parsed.defects.len(), 2);
    }

    #[test]
    fn rejects_pass_that_contains_a_defect() {
        let result = parse_model_verdict(
            r#"{"verdict":"pass","summary":"Looks good.","defects":[{"timestamp_seconds":1.0,"severity":"p2","category":"overlap","observation":"Two sprites intersect."}]}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_markdown_or_malformed_provider_output() {
        assert!(parse_model_verdict("```json\n{}\n```").is_err());
        assert!(parse_model_verdict(r#"{"verdict":"fail"}"#).is_err());
    }
}
