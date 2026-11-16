//! User-facing explanation for a resume lookup that starts a new thread.

use crate::cli::ResumeArgs;
use codex_core::config::Config;

pub(crate) fn fallback_notice(
    config: &Config,
    args: &ResumeArgs,
    model_override_requested: bool,
) -> String {
    let directory = if args.all {
        ""
    } else {
        " in the current directory"
    };
    let reason = if args.last {
        let providers =
            super::resume_lookup_model_providers(config, args, model_override_requested);
        let provider = providers
            .as_ref()
            .and_then(|providers| providers.first())
            .map(|provider| format!(" for model provider `{provider}`"))
            .unwrap_or_default();
        format!("No resumable conversation found{provider}{directory}")
    } else if let Some(name) = args.session_id.as_deref() {
        format!("No saved conversation named `{name}` found{directory}")
    } else {
        "No conversation was selected to resume".to_string()
    };
    format!("{reason}. Starting a new conversation.")
}
