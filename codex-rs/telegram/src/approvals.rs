use anyhow::Context;
use codex_app_server_protocol::CommandExecutionApprovalDecision;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::CommandExecutionRequestApprovalResponse;
use codex_app_server_protocol::ExecPolicyAmendment;
use codex_app_server_protocol::FileChangeApprovalDecision;
use codex_app_server_protocol::FileChangeRequestApprovalParams;
use codex_app_server_protocol::FileChangeRequestApprovalResponse;
use codex_app_server_protocol::GrantedPermissionProfile;
use codex_app_server_protocol::NetworkApprovalContext;
use codex_app_server_protocol::NetworkPolicyAmendment;
use codex_app_server_protocol::NetworkPolicyRuleAction;
use codex_app_server_protocol::PermissionGrantScope;
use codex_app_server_protocol::PermissionsRequestApprovalParams;
use codex_app_server_protocol::PermissionsRequestApprovalResponse;
use codex_app_server_protocol::RequestId;
use serde_json::Value;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;

use crate::render::escape_html;

const CALLBACK_PREFIX: &str = "tg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCallback {
    pub decision_index: usize,
    pub request_id: RequestId,
}

impl ApprovalCallback {
    pub fn encode(&self) -> String {
        let (kind, value) = match &self.request_id {
            RequestId::Integer(value) => ("i", value.to_string()),
            RequestId::String(value) => ("s", urlencoding::encode(value).into_owned()),
        };
        format!("{CALLBACK_PREFIX}:{}:{kind}:{value}", self.decision_index)
    }

    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.splitn(4, ':');
        if parts.next()? != CALLBACK_PREFIX {
            return None;
        }
        let decision_index = parts.next()?.parse().ok()?;
        let kind = parts.next()?;
        let value = parts.next()?;
        let request_id = match kind {
            "i" => RequestId::Integer(value.parse().ok()?),
            "s" => RequestId::String(urlencoding::decode(value).ok()?.into_owned()),
            _ => return None,
        };
        Some(Self {
            decision_index,
            request_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingApprovalKind {
    Command(CommandExecutionRequestApprovalParams),
    FileChange(FileChangeRequestApprovalParams),
    Permissions(PermissionsRequestApprovalParams),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingApproval {
    pub request_id: RequestId,
    pub kind: PendingApprovalKind,
}

impl PendingApproval {
    pub fn thread_id(&self) -> &str {
        match &self.kind {
            PendingApprovalKind::Command(params) => &params.thread_id,
            PendingApprovalKind::FileChange(params) => &params.thread_id,
            PendingApprovalKind::Permissions(params) => &params.thread_id,
        }
    }

    pub fn message(&self) -> String {
        match &self.kind {
            PendingApprovalKind::Command(params) => {
                let mut lines = vec!["Approval requested for command execution.".to_string()];
                if let Some(command) = &params.command {
                    lines.push(format!("<pre><code>{}</code></pre>", escape_html(command)));
                }
                if let Some(reason) = &params.reason {
                    lines.push(format!("Reason: {}", escape_html(reason)));
                }
                lines.join("\n")
            }
            PendingApprovalKind::FileChange(params) => {
                let reason = params
                    .reason
                    .as_ref()
                    .map(|reason| format!("\nReason: {}", escape_html(reason)))
                    .unwrap_or_default();
                format!("Approval requested for file changes.{reason}")
            }
            PendingApprovalKind::Permissions(params) => {
                let reason = params
                    .reason
                    .as_ref()
                    .map(|reason| format!("\nReason: {}", escape_html(reason)))
                    .unwrap_or_default();
                format!("Approval requested for additional permissions.{reason}")
            }
        }
    }

    pub fn keyboard(&self) -> InlineKeyboardMarkup {
        let buttons = self
            .approval_options()
            .into_iter()
            .enumerate()
            .map(|(decision_index, option)| {
                let callback = ApprovalCallback {
                    decision_index,
                    request_id: self.request_id.clone(),
                }
                .encode();
                InlineKeyboardButton::callback(option.label, callback)
            })
            .collect::<Vec<_>>();
        InlineKeyboardMarkup::new(vec![buttons])
    }

    pub fn resolve_value(&self, decision_index: usize) -> anyhow::Result<Value> {
        let value = match self
            .approval_options()
            .get(decision_index)
            .map(|option| option.decision.clone())
            .with_context(|| format!("approval decision {decision_index} is not available"))?
        {
            ApprovalDecision::Command(decision) => {
                serde_json::to_value(CommandExecutionRequestApprovalResponse { decision })
                    .context("serialize command approval response")?
            }
            ApprovalDecision::FileChange(decision) => {
                serde_json::to_value(FileChangeRequestApprovalResponse { decision })
                    .context("serialize file approval response")?
            }
            ApprovalDecision::Permissions { permissions, scope } => {
                serde_json::to_value(PermissionsRequestApprovalResponse {
                    permissions,
                    scope,
                    strict_auto_review: None,
                })
                .context("serialize permissions approval response")?
            }
        };
        Ok(value)
    }

    pub fn response_text(&self, decision_index: usize) -> anyhow::Result<&'static str> {
        let options = self.approval_options();
        let option = options
            .get(decision_index)
            .with_context(|| format!("approval decision {decision_index} is not available"))?;
        Ok(option.response_text)
    }

    fn approval_options(&self) -> Vec<ApprovalOption> {
        match &self.kind {
            PendingApprovalKind::Command(params) => command_approval_options(params),
            PendingApprovalKind::FileChange(_) => file_change_approval_options(),
            PendingApprovalKind::Permissions(params) => permissions_approval_options(params),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ApprovalOption {
    label: String,
    decision: ApprovalDecision,
    response_text: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
enum ApprovalDecision {
    Command(CommandExecutionApprovalDecision),
    FileChange(FileChangeApprovalDecision),
    Permissions {
        permissions: GrantedPermissionProfile,
        scope: PermissionGrantScope,
    },
}

fn command_approval_options(params: &CommandExecutionRequestApprovalParams) -> Vec<ApprovalOption> {
    effective_command_decisions(params)
        .into_iter()
        .map(|decision| {
            let response_text = command_response_text(&decision);
            ApprovalOption {
                label: command_decision_label(&decision),
                decision: ApprovalDecision::Command(decision),
                response_text,
            }
        })
        .collect()
}

fn effective_command_decisions(
    params: &CommandExecutionRequestApprovalParams,
) -> Vec<CommandExecutionApprovalDecision> {
    params.available_decisions.clone().unwrap_or_else(|| {
        default_command_decisions(
            params.network_approval_context.as_ref(),
            params.proposed_execpolicy_amendment.as_ref(),
            params.proposed_network_policy_amendments.as_deref(),
            params.additional_permissions.as_ref(),
        )
    })
}

fn default_command_decisions(
    network_approval_context: Option<&NetworkApprovalContext>,
    proposed_execpolicy_amendment: Option<&ExecPolicyAmendment>,
    proposed_network_policy_amendments: Option<&[NetworkPolicyAmendment]>,
    additional_permissions: Option<&codex_app_server_protocol::AdditionalPermissionProfile>,
) -> Vec<CommandExecutionApprovalDecision> {
    if network_approval_context.is_some() {
        let mut decisions = vec![
            CommandExecutionApprovalDecision::Accept,
            CommandExecutionApprovalDecision::AcceptForSession,
        ];
        if let Some(amendment) = proposed_network_policy_amendments.and_then(|amendments| {
            amendments
                .iter()
                .find(|amendment| amendment.action == NetworkPolicyRuleAction::Allow)
        }) {
            decisions.push(
                CommandExecutionApprovalDecision::ApplyNetworkPolicyAmendment {
                    network_policy_amendment: amendment.clone(),
                },
            );
        }
        decisions.push(CommandExecutionApprovalDecision::Cancel);
        return decisions;
    }

    if additional_permissions.is_some() {
        return vec![
            CommandExecutionApprovalDecision::Accept,
            CommandExecutionApprovalDecision::Cancel,
        ];
    }

    let mut decisions = vec![CommandExecutionApprovalDecision::Accept];
    if let Some(amendment) = proposed_execpolicy_amendment {
        decisions.push(
            CommandExecutionApprovalDecision::AcceptWithExecpolicyAmendment {
                execpolicy_amendment: amendment.clone(),
            },
        );
    }
    decisions.push(CommandExecutionApprovalDecision::Cancel);
    decisions
}

fn command_decision_label(decision: &CommandExecutionApprovalDecision) -> String {
    match decision {
        CommandExecutionApprovalDecision::Accept => "Approve".to_string(),
        CommandExecutionApprovalDecision::AcceptForSession => "Approve for session".to_string(),
        CommandExecutionApprovalDecision::AcceptWithExecpolicyAmendment { .. } => {
            "Approve and remember command".to_string()
        }
        CommandExecutionApprovalDecision::ApplyNetworkPolicyAmendment {
            network_policy_amendment,
        } => match network_policy_amendment.action {
            NetworkPolicyRuleAction::Allow => "Approve and allow host".to_string(),
            NetworkPolicyRuleAction::Deny => "Decline and block host".to_string(),
        },
        CommandExecutionApprovalDecision::Decline => "Decline".to_string(),
        CommandExecutionApprovalDecision::Cancel => "Cancel".to_string(),
    }
}

fn command_response_text(decision: &CommandExecutionApprovalDecision) -> &'static str {
    match decision {
        CommandExecutionApprovalDecision::Accept
        | CommandExecutionApprovalDecision::AcceptForSession
        | CommandExecutionApprovalDecision::AcceptWithExecpolicyAmendment { .. } => "Approved.",
        CommandExecutionApprovalDecision::ApplyNetworkPolicyAmendment {
            network_policy_amendment,
        } => match network_policy_amendment.action {
            NetworkPolicyRuleAction::Allow => "Approved.",
            NetworkPolicyRuleAction::Deny => "Declined.",
        },
        CommandExecutionApprovalDecision::Decline => "Declined.",
        CommandExecutionApprovalDecision::Cancel => "Cancelled.",
    }
}

fn file_change_approval_options() -> Vec<ApprovalOption> {
    [
        (FileChangeApprovalDecision::Accept, "Approve", "Approved."),
        (
            FileChangeApprovalDecision::AcceptForSession,
            "Approve for session",
            "Approved.",
        ),
        (FileChangeApprovalDecision::Decline, "Decline", "Declined."),
        (FileChangeApprovalDecision::Cancel, "Cancel", "Cancelled."),
    ]
    .into_iter()
    .map(|(decision, label, response_text)| ApprovalOption {
        label: label.to_string(),
        decision: ApprovalDecision::FileChange(decision),
        response_text,
    })
    .collect()
}

fn permissions_approval_options(params: &PermissionsRequestApprovalParams) -> Vec<ApprovalOption> {
    let granted_permissions = GrantedPermissionProfile {
        network: params.permissions.network.clone(),
        file_system: params.permissions.file_system.clone(),
    };
    vec![
        ApprovalOption {
            label: "Approve".to_string(),
            decision: ApprovalDecision::Permissions {
                permissions: granted_permissions.clone(),
                scope: PermissionGrantScope::Turn,
            },
            response_text: "Approved.",
        },
        ApprovalOption {
            label: "Approve for session".to_string(),
            decision: ApprovalDecision::Permissions {
                permissions: granted_permissions,
                scope: PermissionGrantScope::Session,
            },
            response_text: "Approved.",
        },
        ApprovalOption {
            label: "Decline".to_string(),
            decision: ApprovalDecision::Permissions {
                permissions: GrantedPermissionProfile::default(),
                scope: PermissionGrantScope::Turn,
            },
            response_text: "Declined.",
        },
    ]
}
