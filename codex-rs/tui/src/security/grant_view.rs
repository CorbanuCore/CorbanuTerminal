//! PF-25-S01: the grant review, opened from a command approval under
//! Aggressive.
//!
//! Core offers a grant while it waits for the answer to one exact command
//! (`legacy_core::security_grant`). The review shows every field of that
//! offer and what stays denied. Only Enter here confirms it, through
//! [`GrantReview::confirm`]; Esc goes back to the approval with nothing
//! granted. The offer is Core's data: an agent's reason or command text is
//! shown, never trusted, and nothing an agent sends reaches the confirm key.

use chrono::Local;
use chrono::TimeZone;
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::exec_command::strip_bash_lc_and_escape;
use crate::legacy_core::security_grant::ConfirmedGrant;
use crate::legacy_core::security_grant::GrantOffer;
use crate::legacy_core::security_grant::GrantUses;
use crate::legacy_core::security_grant::HeldGrant;
use crate::legacy_core::security_grant::confirm;

/// What the grant review is showing.
pub(crate) struct GrantReview {
    offer: GrantOffer,
    uses: GrantUses,
    error: Option<String>,
}

impl GrantReview {
    pub(crate) fn new(offer: GrantOffer) -> Self {
        Self {
            offer,
            uses: GrantUses::Once,
            error: None,
        }
    }

    /// `u`: one run, or any run of this exact command until it expires.
    pub(crate) fn toggle_uses(&mut self) {
        self.uses = match self.uses {
            GrantUses::Once => GrantUses::UntilExpiry,
            GrantUses::UntilExpiry => GrantUses::Once,
        };
        self.error = None;
    }

    /// The person pressed Enter: issue exactly the grant shown. On failure
    /// the review stays open with the reason and nothing is granted.
    pub(crate) fn confirm(&mut self) -> Option<ConfirmedGrant> {
        match confirm(&self.offer, self.uses) {
            Ok(confirmed) => Some(confirmed),
            Err(error) => {
                self.error = Some(format!("{}.", capitalized(&error.to_string())));
                None
            }
        }
    }

    pub(crate) fn command(&self) -> &[String] {
        &self.offer.command
    }

    pub(crate) fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let width = usize::from(width.max(1));
        let wrap = |text: &str| -> Vec<Line<'static>> {
            textwrap::wrap(text, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned()))
                .collect()
        };
        let offer = &self.offer;
        let mut lines: Vec<Line<'static>> = vec!["Grant this command?".bold().into()];
        lines.extend(wrap(
            "Under Aggressive an approval never lifts the protected-path rules (credential files, the Corbanu home, shell and git startup files). This grant lets exactly this command run without them.",
        ));
        lines.push(Line::default());
        let label_width = 12;
        let field = |name: &str, value: String| -> Vec<Line<'static>> {
            let indent = " ".repeat(label_width + 2);
            let text = format!("  {name:<label_width$}{value}");
            textwrap::wrap(
                &text,
                textwrap::Options::new(width.max(label_width + 10)).subsequent_indent(&indent),
            )
            .into_iter()
            .map(|line| Line::from(line.into_owned()))
            .collect()
        };
        lines.extend(field("Agent", offer.actor_chain.join(" → ")));
        lines.extend(field("Session", offer.thread.to_string()));
        lines.extend(field("Action", offer.action.clone()));
        lines.extend(field("Resource", offer.resource.clone()));
        lines.extend(field(
            "Command",
            format!("$ {}", strip_bash_lc_and_escape(&offer.command)),
        ));
        lines.extend(field("Folder", offer.cwd.clone()));
        lines.extend(field("Digest", offer.operation.clone()));
        lines.extend(field("Destination", "none".to_string()));
        lines.extend(field(
            "Limit",
            match self.uses {
                GrantUses::Once => "1 run (press u for any run until it expires)".to_string(),
                GrantUses::UntilExpiry => {
                    "any run of this exact command until it expires (press u for 1 run)".to_string()
                }
            },
        ));
        lines.extend(field(
            "Expires",
            format!(
                "{} minutes after you confirm, or at a level change, revocation, kill switch or restart",
                offer.lifetime_seconds / 60
            ),
        ));
        lines.push(Line::default());
        lines.extend(wrap(
            match self.uses {
                GrantUses::Once => "Still denied: any other command or folder, this command a second time, its parent, sibling and child agents, other sessions, and everything else Aggressive denies.",
                GrantUses::UntilExpiry => "Still denied: any other command or folder, its parent, sibling and child agents, other sessions, and everything else Aggressive denies. /security lists the grant until it expires.",
            },
        ));
        if let Some(error) = &self.error {
            lines.push(Line::default());
            lines.extend(wrap(error).into_iter().map(Stylize::red));
        }
        lines.push(Line::default());
        lines.push(
            "enter grant and run · u change limit · esc back to the approval, nothing granted"
                .dim()
                .into(),
        );
        lines
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// `/security` lines for the grants held now.
pub(crate) fn held_lines(grants: &[HeldGrant]) -> Vec<String> {
    grants
        .iter()
        .map(|grant| {
            let uses = match grant.uses_left {
                Some(1) => "1 run left".to_string(),
                Some(uses) => format!("{uses} runs left"),
                None => "any run".to_string(),
            };
            let expires = Local
                .timestamp_opt(grant.expires_at_unix_seconds, 0)
                .single()
                .map_or_else(
                    || grant.expires_at_unix_seconds.to_string(),
                    |time| time.format("%H:%M").to_string(),
                );
            format!(
                "Grant: `{}` without the protected-path rules · {uses} · expires {expires}",
                grant.label
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "grant_view_tests.rs"]
mod tests;
