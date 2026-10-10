use std::collections::HashSet;

const FALLBACK_MODEL_METADATA_WARNING_PREFIX: &str = "Model metadata for `";
const FALLBACK_MODEL_METADATA_WARNING_SUFFIX: &str =
    "` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.";

#[derive(Default)]
pub(super) struct WarningDisplayState {
    fallback_model_metadata_slugs: HashSet<String>,
    /// Config warnings already shown at startup. The session repeats its
    /// config's startup warnings once; each repeat is shown only once (#419).
    shown_config_warnings: HashSet<String>,
    /// Config warnings that arrived before the session header; shown right
    /// after it so a startup redraw cannot hide them.
    pending_config_warnings: Vec<(String, String)>,
}

impl WarningDisplayState {
    pub(super) fn should_display(&mut self, message: &str) -> bool {
        if self.shown_config_warnings.remove(message) {
            return false;
        }
        fallback_model_metadata_warning_slug(message)
            .is_none_or(|slug| self.fallback_model_metadata_slugs.insert(slug.to_string()))
    }

    pub(super) fn config_warning_shown(&mut self, summary: String) {
        self.shown_config_warnings.insert(summary);
    }

    pub(super) fn defer_config_warning(&mut self, summary: String, message: String) {
        self.pending_config_warnings.push((summary, message));
    }

    pub(super) fn take_pending_config_warnings(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.pending_config_warnings)
    }
}

fn fallback_model_metadata_warning_slug(message: &str) -> Option<&str> {
    message
        .strip_prefix(FALLBACK_MODEL_METADATA_WARNING_PREFIX)?
        .strip_suffix(FALLBACK_MODEL_METADATA_WARNING_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_repeat_of_a_config_warning_is_shown_once() {
        let mut state = WarningDisplayState::default();
        let warning = "`[provider_accounts]` is ignored";
        assert!(state.should_display(warning));
        state.config_warning_shown(warning.to_string());
        assert!(!state.should_display(warning));
        assert!(state.should_display(warning));
    }
}
