//! Viewport predicates for TUI startup readiness.

/// Reports whether the session header shows a configured model.
///
/// Before the session is configured, the header is a placeholder whose model reads `loading`, and
/// the footer and composer are already visible. Commands such as `/model` are refused in that
/// window, so wait for this before driving them.
pub(crate) fn session_configured(capture: &str) -> bool {
    capture.lines().any(|line| {
        line.contains("model:") && line.contains("/model to change") && !line.contains("loading")
    })
}

#[cfg(test)]
mod tests {
    use super::session_configured;

    #[test]
    fn placeholder_header_is_not_configured() {
        assert!(!session_configured(
            "│ model:     loading   /model to change │"
        ));
        assert!(session_configured(
            "│ model:     gpt-5.6-terra   /model to change │"
        ));
    }
}
