//! PF-29-S02 text for the `/security` credential migration preview and
//! result. Locations, labels and actions only; never a value.

use crate::legacy_core::protected_preflight::migration::MigrationOutcome;
use crate::legacy_core::protected_preflight::migration::MigrationPlan;

pub(crate) fn preview_lines(plan: &MigrationPlan) -> Vec<String> {
    let mut lines = vec![format!(
        "Move {} credential{} into the vault?",
        plan.moves.len(),
        if plan.moves.len() == 1 { "" } else { "s" }
    )];
    for planned in &plan.moves {
        lines.push(format!(
            "• {} → vault label {}; the line becomes {}={}",
            planned.location,
            planned.label,
            planned.name,
            planned.reference()
        ));
    }
    let files = plan
        .restricted_files()
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if !files.is_empty() {
        lines.push(format!(
            "Access: {} become readable only by you (0600). No plaintext copy or backup is kept.",
            files.join(", ")
        ));
    }
    lines.push(
        "Restart: open a new shell and restart Corbanu Terminal. The profile then reads each value from the vault when it loads."
            .to_string(),
    );
    if !plan.unsupported.is_empty() {
        lines.push("Not moved; do this yourself:".to_string());
        lines.extend(
            plan.unsupported
                .iter()
                .map(|item| format!("• {}: {}", item.location, item.action)),
        );
    }
    lines.push(
        "Rotate every value listed here at its provider afterwards: agents could read it in plain text before."
            .to_string(),
    );
    lines
}

pub(crate) fn result_lines(result: &Result<MigrationOutcome, String>) -> Vec<String> {
    match result {
        Ok(outcome) => {
            let mut lines = vec![format!(
                "Moved {} credential{} into the vault",
                outcome.moved.len(),
                if outcome.moved.len() == 1 { "" } else { "s" }
            )];
            lines.extend(
                outcome
                    .moved
                    .iter()
                    .map(|(location, label)| format!("• {location} → {label}")),
            );
            lines.extend(
                outcome
                    .skipped
                    .iter()
                    .map(|(location, reason)| format!("• {location}: not moved, {reason}")),
            );
            if !outcome.rotate.is_empty() {
                lines.push(format!(
                    "Rotate at the provider: {}. Recovery never restores plain text.",
                    outcome.rotate.join(", ")
                ));
            }
            lines
        }
        Err(error) => vec![
            "Migration stopped".to_string(),
            error.clone(),
            "Your security level is unchanged and Aggressive cannot be saved until the migration is finished."
                .to_string(),
        ],
    }
}
