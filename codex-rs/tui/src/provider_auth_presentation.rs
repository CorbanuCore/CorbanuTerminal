//! User-facing guidance shared by onboarding and the provider manager.

use codex_provider_auth::ApiKeyStorage;
use codex_provider_auth::OpenAiAccountChallenge;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::render::renderable::Renderable;

pub(crate) const OPENAI_BROWSER_GUIDANCE: &str =
    "Open this link in your browser to finish signing in.";
pub(crate) const OPENAI_DEVICE_GUIDANCE: &str =
    "On a remote or headless machine, open this link on another device and enter the code.";

pub(crate) fn api_key_guidance(storage: &ApiKeyStorage) -> String {
    match storage {
        ApiKeyStorage::EnvironmentVariable { env_key } => format!(
            "Paste your {env_key} API key. Corbanu stores it in the encrypted vault and never adds it to chat. An existing environment variable takes precedence; unset it outside Corbanu to use the saved key."
        ),
        ApiKeyStorage::OpenAiAuth => "Paste your OpenAI API key for usage-based billing. Corbanu stores it in the configured OpenAI credential store and never adds it to chat.".to_string(),
    }
}

pub(crate) struct OpenAiChallengeHeader {
    challenge: OpenAiAccountChallenge,
}

impl OpenAiChallengeHeader {
    pub(crate) fn new(challenge: OpenAiAccountChallenge) -> Self {
        Self { challenge }
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("OpenAI account login".bold())];
        let (url, guidance) = if let Some((url, code)) = self.challenge.device_code_values() {
            lines.push(Line::from(format!("Code: {code}").cyan().bold()));
            (url, OPENAI_DEVICE_GUIDANCE)
        } else {
            (
                self.challenge.browser_auth_url().unwrap_or_default(),
                OPENAI_BROWSER_GUIDANCE,
            )
        };
        lines.push(Line::from("Open:"));
        lines.push(Line::from(url.to_string().cyan().underlined()));
        lines.push(Line::from(guidance));
        crate::wrapping::word_wrap_lines(&lines, width.max(1) as usize)
    }
}

impl Renderable for OpenAiChallengeHeader {
    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines(area.width)).render(area, buf);
        let url = self
            .challenge
            .device_code_values()
            .map(|(url, _)| url)
            .or_else(|| self.challenge.browser_auth_url());
        if let Some(url) = url {
            crate::terminal_hyperlinks::mark_url_hyperlink(buf, area, url);
        }
    }
}
