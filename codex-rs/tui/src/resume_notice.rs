//! Explain automatic resume fallbacks without changing session selection.

use crate::Cli;
use crate::resume_picker::SessionSelection;

/// A cancelled picker deliberately starts fresh; only `--last` falls back
/// automatically. Describe the actual lookup filters rather than guessing
/// whether a missing conversation was caused by a provider switch.
pub(crate) fn resume_fallback_notice(
    cli: &Cli,
    selection: &SessionSelection,
    model_provider: Option<&str>,
    cwd: Option<&std::path::Path>,
) -> Option<String> {
    if !cli.resume_last
        || cli.fork_last
        || cli.fork_picker
        || cli.fork_session_id.is_some()
        || !matches!(selection, SessionSelection::StartFresh)
    {
        return None;
    }
    let sources = if cli.resume_include_non_interactive {
        ""
    } else {
        "interactive "
    };
    let provider = model_provider
        .map(|provider| format!(" for model provider `{provider}`"))
        .unwrap_or_default();
    let directory = if cwd.is_some() {
        " in the current directory"
    } else {
        ""
    };
    Some(format!(
        "No resumable {sources}conversation found{provider}{directory}. Starting a new conversation."
    ))
}

#[cfg(test)]
#[path = "resume_notice_tests.rs"]
mod tests;
