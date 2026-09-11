use super::*;

const VIEW_ID: &str = "tasknode-team-context";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamContext {
    status: String,
    members: Vec<TeamMember>,
    #[serde(default)]
    overview: String,
    #[serde(default)]
    include_in_personal_context: bool,
    generated_at: Option<String>,
    #[serde(default)]
    showing_previous_report: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamMember {
    display_name: String,
    #[serde(default)]
    hive_handle: String,
    task_history_visible: bool,
    tasks_past_day: Option<u64>,
    tasks_past_week: Option<u64>,
    #[serde(default)]
    focus: String,
    #[serde(default)]
    completed_changes: Vec<String>,
    #[serde(default)]
    operational_effect: String,
    #[serde(default)]
    recent_work: String,
}

impl ChatWidget {
    pub(crate) fn open_tasknode_team_context(&mut self) {
        self.show_or_replace_tasknode_selection(VIEW_ID, || {
            tasknode_loading_selection_params(
                VIEW_ID,
                "Task Node Team Context".to_string(),
                "Loading shared collaborator context...".to_string(),
            )
        });
        self.spawn_tasknode_value_request(
            "team-context",
            |client| {
                client
                    .get_json("/api/terminal/tasknode/team/context")
                    .map_err(|err| err.to_string())
            },
            |result| AppEvent::OpenTaskNodeTeamContextResult { result },
        );
    }

    pub(crate) fn handle_tasknode_team_context_result(&mut self, result: Result<Value, String>) {
        let report = parse_tasknode_value::<TeamContext>(result, "Team Context");
        self.show_or_replace_tasknode_selection(VIEW_ID, || team_context_params(report));
    }
}

fn team_context_params(report: Result<TeamContext, String>) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Task Node Team Context".bold()));
    let mut items = Vec::new();
    match report {
        Ok(report) => {
            push_tasknode_wrapped_line(
                &mut header,
                format!(
                    "Status: {} · {} members",
                    report.status,
                    report.members.len()
                ),
            );
            if report.showing_previous_report {
                push_tasknode_wrapped_line(
                    &mut header,
                    "Showing the previous report while a new one is prepared.",
                );
            }
            let text = team_context_text(&report);
            items.push(SelectionItem {
                name: "Read full report".to_string(),
                description: Some(
                    "Scroll through shared work summaries and task counts".to_string(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::TaskNodeTeamContextDocument { text: text.clone() })
                })],
                dismiss_on_select: false,
                ..Default::default()
            });
        }
        Err(error) => push_tasknode_wrapped_line(
            &mut header,
            format!("Team Context could not be loaded: {error}"),
        ),
    }
    items.push(SelectionItem {
        name: "Refresh Team Context".to_string(),
        description: Some("Reload the report and current sharing permissions".to_string()),
        actions: vec![Box::new(|tx| tx.send(AppEvent::OpenTaskNodeTeamContext))],
        dismiss_on_select: false,
        ..Default::default()
    });
    SelectionViewParams {
        view_id: Some(VIEW_ID),
        header: Box::new(header),
        footer_hint: Some(standard_popup_hint_line()),
        items,
        ..Default::default()
    }
}

fn team_context_text(report: &TeamContext) -> String {
    let mut lines = vec![
        "Task Node Team Context".to_string(),
        format!("Status: {}", report.status),
        format!(
            "Updated: {}",
            report
                .generated_at
                .as_deref()
                .unwrap_or("Not generated yet")
        ),
        format!(
            "Use in personal context: {}",
            if report.include_in_personal_context {
                "on"
            } else {
                "off"
            }
        ),
    ];
    if report.showing_previous_report {
        lines.push(
            "Showing the previous report. Refresh to check for an updated report.".to_string(),
        );
    }
    match report.status.as_str() {
        "empty" => lines.push(
            "No teammates are available. Manage collaborators in Task Node's Team page."
                .to_string(),
        ),
        "pending" | "processing" | "completed" => {
            lines.push("Team Context is being prepared. Refresh to check its progress.".to_string())
        }
        "failed" => lines.push(
            "Team Context generation failed. Check Task Node's Team page for recovery.".to_string(),
        ),
        "unavailable" => lines.push("Team Context is currently unavailable.".to_string()),
        _ => {}
    }
    if !report.overview.is_empty() {
        lines.extend([String::new(), report.overview.clone()]);
    }
    for member in &report.members {
        lines.extend([
            String::new(),
            format!(
                "{}{}",
                member.display_name,
                if member.hive_handle.is_empty() {
                    String::new()
                } else {
                    format!(" (@{})", member.hive_handle)
                }
            ),
        ]);
        if !member.task_history_visible {
            lines.push("Task history has not been shared with you.".to_string());
            continue;
        }
        let count = |value: Option<u64>| {
            value.map_or_else(|| "unavailable".to_string(), |value| value.to_string())
        };
        lines.push(format!(
            "Rewarded tasks: {} in 24 hours · {} in 7 days",
            count(member.tasks_past_day),
            count(member.tasks_past_week)
        ));
        if member.focus.is_empty()
            && member.completed_changes.is_empty()
            && member.operational_effect.is_empty()
        {
            lines.push(member.recent_work.clone());
        } else {
            if !member.focus.is_empty() {
                lines.push(format!("Focus: {}", member.focus));
            }
            lines.extend(
                member
                    .completed_changes
                    .iter()
                    .map(|change| format!("• {change}")),
            );
            if !member.operational_effect.is_empty() {
                lines.push(format!("Effect: {}", member.operational_effect));
            }
        }
    }
    lines.join("\n")
}

#[cfg(test)]
#[path = "team_context_tests.rs"]
mod tests;
