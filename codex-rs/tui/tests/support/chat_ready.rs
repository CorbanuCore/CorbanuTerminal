//! Viewport predicates for TUI startup readiness.

use std::time::Duration;

use anyhow::Result;

use super::tmux::TmuxPane;

/// Reports whether the session header shows a configured model.
///
/// Before the session is configured, the header is a placeholder whose model reads `loading`, and
/// the footer and composer are already visible. Commands such as `/model` are refused in that
/// window, so wait for this before driving them. Only the session header card has a row that
/// starts with a lowercase `model:` label; narrow terminals truncate the rest of that row, so the
/// `/model to change` hint is not required.
///
/// Upstream's header has no card or model row: `>_ <product> (v…)` over the working
/// directory, with `loading` in its place until the session is configured.
pub(crate) fn session_configured(capture: &str) -> bool {
    let lines = capture.lines().collect::<Vec<_>>();
    let card = lines.iter().any(|line| {
        line.trim()
            .strip_prefix('│')
            .map(str::trim_start)
            .is_some_and(|row| row.starts_with("model:") && !row.contains("loading"))
    });
    let header = lines.iter().enumerate().any(|(index, line)| {
        let line = line.trim();
        line.starts_with(">_ ")
            && line.contains("(v")
            && lines[index..]
                .iter()
                .take(3)
                .all(|line| !line.contains("loading"))
    });
    card || header
}

/// Waits until the session header shows a configured model.
///
/// `Corbanu Terminal` is not a startup signal: the onboarding provider list and the placeholder
/// header both contain it before the session is configured (#1714, #1748). Call this right after
/// startup, while the header is still in the viewport, before sending a slash command or reading
/// state the startup writes.
pub(crate) fn wait_session_configured(pane: &TmuxPane<'_>, timeout: Duration) -> Result<String> {
    pane.wait_stable_until("configured session header", timeout, session_configured)
}

#[cfg(test)]
mod tests {
    use super::session_configured;

    #[test]
    fn placeholder_header_is_not_configured() {
        assert!(!session_configured(
            "│ model:     loading   /model to change │"
        ));
        assert!(!session_configured("│ model:     loading   /model t… │"));
        assert!(session_configured(
            "│ model:     gpt-5.6-terra   /model to change │"
        ));
        assert!(session_configured(
            "  │ model:     gpt-5.6-terra   /model… │"
        ));
    }

    #[test]
    fn upstream_header_is_configured_once_loading_is_gone() {
        assert!(!session_configured(
            "  >_ Corbanu Terminal (v0.1.48)      loading\n"
        ));
        assert!(!session_configured(
            "  >_ Corbanu Terminal (v0.1.48)\n     loading\n"
        ));
        assert!(session_configured(
            "  >_ Corbanu Terminal (v0.1.48)\n     ~/project\n\n› Ask Corbanu Terminal to do anything\n"
        ));
    }

    #[test]
    fn other_model_text_is_not_the_header() {
        assert!(!session_configured(
            "Choose a provider account for Corbanu Terminal"
        ));
        assert!(!session_configured("│ Model:  gpt-5.6-terra │"));
        assert!(!session_configured("  model: gpt-5.6-terra"));
    }
}
