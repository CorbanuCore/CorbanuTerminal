//! Explicit sharing, campaign creation and human prompt-review flows.
use super::campaign_tracker::item;
use super::*;
use serde_json::Value;
use serde_json::json;

impl ChatWidget {
    pub(super) fn tracker_local_workflow(&mut self, path: &str, body: Option<&Value>) -> bool {
        let mut data = body.cloned().unwrap_or_else(|| json!({}));
        match path {
            "/_search" => {
                let source = data["path"].as_str().unwrap_or("/activity").to_string();
                let initial = url::Url::parse(&format!("https://tracker.invalid{source}"))
                    .ok()
                    .and_then(|url| {
                        url.query_pairs()
                            .find(|(key, _)| key == "search")
                            .map(|(_, value)| value.into_owned())
                    })
                    .unwrap_or_default();
                let tx = self.app_event_tx.clone();
                self.show_custom_prompt_view(
                    CustomPromptView::new(
                        "Search activity".to_string(),
                        "Search preserved prompts and summaries within your current access."
                            .to_string(),
                        initial,
                        None,
                        Box::new(move |input| {
                            if let Ok(mut url) =
                                url::Url::parse(&format!("https://tracker.invalid{source}"))
                            {
                                let pairs: Vec<(String, String)> = url
                                    .query_pairs()
                                    .filter(|(k, _)| {
                                        !["search", "before", "beforeId"].contains(&k.as_ref())
                                    })
                                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                                    .collect();
                                url.set_query(None);
                                url.query_pairs_mut()
                                    .extend_pairs(pairs)
                                    .append_pair("search", &input);
                                tx.send(AppEvent::CampaignTrackerOpen {
                                    path: format!("{}?{}", url.path(), url.query().unwrap_or("")),
                                    body: None,
                                });
                            }
                        }),
                    )
                    .with_submit_mode(CustomPromptSubmitMode::CtrlD),
                );
            }
            "/_mapping" => {
                let initial = if data["note"].is_string() || data["taskIds"].is_array() {
                    let tasks = data["taskIds"]
                        .as_array()
                        .map(|ids| {
                            ids.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .unwrap_or_default();
                    format!("{tasks}\n{}", data["note"].as_str().unwrap_or(""))
                } else {
                    String::new()
                };
                let tx = self.app_event_tx.clone();
                self.show_custom_prompt_view(CustomPromptView::new("Correct task mapping".to_string(), "First line: owned task IDs separated by spaces (empty clears mapping). Following lines: rationale.".to_string(), initial, None, Box::new(move |input| {
                    let mut lines=input.lines();
                    let tasks: Vec<&str> = lines.next().unwrap_or("").split_whitespace().collect();
                    let note=lines.collect::<Vec<_>>().join("\n");
                    let mut body=data.clone();body["kind"]=json!("mapping");body["taskIds"]=json!(tasks);body["note"]=json!(note);
                    tx.send(AppEvent::CampaignTrackerOpen {path:"/annotations".to_string(),body:Some(body)});
                })).with_submit_mode(CustomPromptSubmitMode::CtrlD));
            }
            "/_share" => {
                let handle = data["handle"].as_str().unwrap_or("").to_string();
                let mut items = Vec::new();
                for days in [0, 7, 30, 365] {
                    for prompts in [false, true] {
                        let from = (chrono::Utc::now() - chrono::Duration::days(days))
                            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                        let expires = (chrono::Utc::now() + chrono::Duration::days(365))
                            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                        let caps = if prompts {
                            json!(["summary", "prompt", "replay", "review"])
                        } else {
                            json!(["summary", "replay"])
                        };
                        let title = format!(
                            "{} · {}",
                            if prompts {
                                "Prompts, replay & reviews"
                            } else {
                                "Summaries & replay"
                            },
                            if days == 0 {
                                "from now".to_string()
                            } else {
                                format!("past {days} days + future")
                            }
                        );
                        items.push(item(&title,"Enter grants this access for one year. Downloads need separate permission.","/grants".to_string(),Some(json!({"handle":handle,"capabilities":caps,"historyFrom":from,"expiresAt":expires}))));
                    }
                }
                self.tracker_selection(SelectionViewParams {
                    view_id: Some("campaign-tracker"),
                    title: Some(format!("Share your activity with @{handle}")),
                    subtitle: Some(
                        "Choose exactly what this accepted collaborator may read. Esc cancels."
                            .to_string(),
                    ),
                    items,
                    allow_number_shortcuts: false,
                    ..Default::default()
                });
            }
            "/_new_campaign" => {
                let initial = if data["title"].is_string() {
                    format!(
                        "{}\n{}",
                        data["title"].as_str().unwrap_or(""),
                        data["objective"].as_str().unwrap_or("")
                    )
                } else {
                    String::new()
                };
                let tx = self.app_event_tx.clone();
                self.show_custom_prompt_view(CustomPromptView::new("Create campaign".to_string(),"First line: campaign name. Following lines: objective.".to_string(),initial,Some("Creates a personal campaign. Linking tasks never grants access to prompts.".to_string()),Box::new(move |input|{
                    let mut lines=input.lines();let title=lines.next().unwrap_or("").to_string();let objective=lines.collect::<Vec<_>>().join("\n");
                    tx.send(AppEvent::CampaignTrackerOpen {path:"/campaigns".to_string(),body:Some(json!({"title":title,"objective":if objective.is_empty(){title}else{objective},"memberHandles":[],"taskIds":[]}))});
                })).with_submit_mode(CustomPromptSubmitMode::CtrlD));
            }
            "/_review" => {
                let dimensions = [
                    ("clarity", "Objective clarity"),
                    ("context", "Useful context"),
                    ("constraints", "Explicit constraints"),
                    ("successCriteria", "Observable success criteria"),
                    ("iteration", "Effective iteration"),
                ];
                if !data["scores"].is_object() {
                    data["scores"] = json!({});
                }
                if let Some((key, label)) = dimensions
                    .iter()
                    .find(|(key, _)| data["scores"].get(*key).is_none())
                {
                    let mut items = Vec::new();
                    for score in [None, Some(1), Some(2), Some(3), Some(4), Some(5)] {
                        let mut next = data.clone();
                        next["scores"][*key] = json!(score);
                        items.push(item(
                            &score.map_or_else(
                                || "Not assessable".to_string(),
                                |s| format!("{s} / 5"),
                            ),
                            "Use only the context available to the employee at this point.",
                            "/_review".to_string(),
                            Some(next),
                        ));
                    }
                    self.tracker_selection(SelectionViewParams {
                        view_id: Some("campaign-tracker"),
                        title: Some(format!("Prompt review · {label}")),
                        subtitle: Some(
                            "1 needs improvement · 5 strong · Esc cancels the draft".to_string(),
                        ),
                        items,
                        allow_number_shortcuts: false,
                        ..Default::default()
                    });
                } else {
                    let tx = self.app_event_tx.clone();
                    data["kind"] = json!("review");
                    let initial = data["note"].as_str().unwrap_or("").to_string();
                    self.show_custom_prompt_view(
                        CustomPromptView::new(
                            "Prompt review rationale".to_string(),
                            "Cite the prompt and available context. Explain the scores."
                                .to_string(),
                            initial,
                            Some(
                                "Human review; versioned against this exact activity record."
                                    .to_string(),
                            ),
                            Box::new(move |note| {
                                let mut body = data.clone();
                                body["note"] = json!(note);
                                tx.send(AppEvent::CampaignTrackerOpen {
                                    path: "/annotations".to_string(),
                                    body: Some(body),
                                });
                            }),
                        )
                        .with_submit_mode(CustomPromptSubmitMode::CtrlD),
                    );
                }
            }
            "/_document" => {
                let text = data["text"].as_str().unwrap_or("").to_string();
                self.app_event_tx
                    .send(AppEvent::CampaignTrackerDocument { text });
            }
            _ => return false,
        }
        true
    }
}
