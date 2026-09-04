//! Shared presentation contract for Claude account authentication hosts.

use crate::bottom_pane::SelectionViewParams;
use ratatui::style::Stylize;
use ratatui::text::Line;

pub(crate) const METHOD_TITLE: &str = "Claude Plan authentication";
pub(crate) const METHOD_SUBTITLE: &str =
    "Choose one source; Corbanu never falls back to another account.";
pub(crate) const METHOD_FOOTER: &str =
    "Your account and billing path change only after success. Esc keeps the current method.";

pub(crate) const MANAGED_TOKEN_METHOD_NAME: &str = "Long-lived subscription token (Recommended)";
pub(crate) const MANAGED_TOKEN_METHOD_DESCRIPTION: &str = "Run `claude setup-token` in a private terminal, then paste its approximately one-year token here (Pro, Max, Team, or Enterprise).";
pub(crate) const CLAUDE_CODE_LOGIN_METHOD_NAME: &str = "Claude Code login";
pub(crate) const CLAUDE_CODE_LOGIN_METHOD_DESCRIPTION: &str =
    "Use Claude Code's rotating login state; reauthorization may be needed more often.";

pub(crate) const MANAGED_TOKEN_ENTRY_TITLE: &str = "Save Claude subscription token";
pub(crate) const MANAGED_TOKEN_ENTRY_LABEL: &str = "Long-lived token — masked";
pub(crate) const MANAGED_TOKEN_ENTRY_GUIDANCE: &str = "In a separate private terminal, run `claude setup-token`. Paste its token here; Corbanu never captures the command output or adds the token to chat.";

pub(crate) fn apply_method_choice_copy(params: &mut SelectionViewParams) {
    params.title = Some(METHOD_TITLE.to_string());
    params.subtitle = Some(METHOD_SUBTITLE.to_string());
    params.footer_note = Some(Line::from(METHOD_FOOTER.dim()));
    params.initial_selected_idx = Some(0);
    params.allow_number_shortcuts = false;
}
