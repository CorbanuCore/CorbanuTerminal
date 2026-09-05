//! Read-only profile copy. Configuration intent is never proof of protection.

use codex_protocol::security::SecurityLevel;

pub(crate) const PROFILES: [SecurityLevel; 3] = [
    SecurityLevel::Permissive,
    SecurityLevel::Moderate,
    SecurityLevel::Aggressive,
];

pub(crate) fn profile_name(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Permissive => "Permissive",
        SecurityLevel::Moderate => "Moderate",
        SecurityLevel::Aggressive => "Aggressive",
    }
}

pub(crate) fn profile_summary(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Permissive => {
            "No additional security controls. Existing approvals, sandbox, vault, wallet, tool, network and agent policies remain unchanged."
        }
        SecurityLevel::Moderate => {
            "Planned: isolate credentials, treat external content as untrusted, protect sensitive data, preview protected actions, and support audit and revocation. Not available yet."
        }
        SecurityLevel::Aggressive => {
            "Planned: all Moderate controls, plus deny sensitive access by default, narrow expiring grants, and exact human approval for every sign or broadcast. Not available yet."
        }
    }
}

pub(crate) fn requested_summary(requested: Option<SecurityLevel>) -> String {
    match requested {
        Some(level) => format!("{} (configuration only)", profile_name(level)),
        None => "Unknown — cannot verify configuration".to_string(),
    }
}

pub(crate) fn status_summary(requested: SecurityLevel) -> String {
    format!(
        "Requested {}; effective protection unverified (/security)",
        profile_name(requested)
    )
}
