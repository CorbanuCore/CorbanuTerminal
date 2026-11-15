//! Corbanu list helpers kept after upstream removed this module's option rows.

/// Position of the selected row in a list that does not fit its window, such
/// as `3 / 17`, so the person can tell that more rows exist (#1288).
pub(crate) fn list_position_label(selected_idx: Option<usize>, total: usize) -> String {
    let position = selected_idx.map_or(0, |idx| idx.saturating_add(1));
    format!("{position} / {total}")
}
