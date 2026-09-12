//! Recover the failed tracker operation without blindly repeating writes.
use super::campaign_tracker::item;
use super::*;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use serde_json::Value;
use serde_json::json;

impl ChatWidget {
    pub(super) fn tracker_request_failed(
        &mut self,
        path: &str,
        body: Option<Value>,
        error: String,
    ) {
        let route = path.split('?').next().unwrap_or(path);
        let operation = match (route, body.is_some()) {
            ("/enrollment", _) => "Update recording",
            ("/campaigns", true) => "Create campaign",
            ("/annotations", true) => "Save review or mapping",
            ("/grants", true) => "Share activity",
            ("/revoke", _) => "Revoke sharing",
            ("/delete", _) => "Delete activity",
            ("/activity", _) => "Load activity",
            ("/replay", _) => "Load replay",
            ("/campaigns", false) => "Load campaigns",
            ("/grants", false) => "Load sharing",
            _ => "Load Campaign Tracker",
        };
        let mut items = Vec::new();
        let is_write = body.is_some();
        if let Some(mut draft) = body {
            if let Some(fields) = draft.as_object_mut() {
                fields.remove("apiKey");
            }
            let edit_route = match route {
                "/campaigns" => Some("/_new_campaign"),
                "/annotations" if draft["kind"] == "mapping" => Some("/_mapping"),
                "/annotations" if draft["kind"] == "review" => Some("/_review"),
                "/grants" => Some("/_share"),
                _ => None,
            };
            if let Some(edit_route) = edit_route {
                items.push(item(
                    "Edit draft",
                    "Restore your input for review and correction",
                    edit_route.to_string(),
                    Some(draft),
                ));
            } else if route == "/enrollment" {
                // Setting enrollment is idempotent; credentials are resolved anew.
                items.push(item(
                    "Retry recording change",
                    "Retry this workspace's requested recording state",
                    path.to_string(),
                    Some(draft),
                ));
            }
        } else {
            items.push(item(
                "Retry",
                "Reload the failed view",
                path.to_string(),
                None,
            ));
            if route == "/activity" || route == "/replay" {
                items.push(item(
                    "Edit search",
                    "Correct the query while retaining its scope",
                    "/_search".to_string(),
                    Some(json!({"path":path})),
                ));
            }
        }
        items.push(item(
            "Overview",
            "Open Campaign Tracker status",
            "/status".to_string(),
            None,
        ));
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Campaign Tracker".bold()));
        header.push(
            Paragraph::new(format!("{operation} failed. {error}")).wrap(Wrap { trim: false }),
        );
        if is_write {
            header.push(
                Paragraph::new("Review saved items before submitting again.".dim())
                    .wrap(Wrap { trim: false }),
            );
        }
        self.tracker_selection(SelectionViewParams {
            view_id: Some("campaign-tracker"),
            header: Box::new(header),
            items,
            ..Default::default()
        });
    }
}
