use anyhow::Context;
use codex_app_server_protocol::CommandExecutionApprovalDecision;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::CommandExecutionRequestApprovalResponse;
use codex_app_server_protocol::FileChangeApprovalDecision;
use codex_app_server_protocol::FileChangeRequestApprovalParams;
use codex_app_server_protocol::FileChangeRequestApprovalResponse;
use codex_app_server_protocol::GrantedPermissionProfile;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::PermissionGrantScope;
use codex_app_server_protocol::PermissionsRequestApprovalParams;
use codex_app_server_protocol::PermissionsRequestApprovalResponse;
use codex_app_server_protocol::RequestId;
use serde_json::Value;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;

use crate::render::escape_html;

const CALLBACK_PREFIX: &str = "tg";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalAction {
    Approve,
    ApproveForSession,
    Decline,
}

impl ApprovalAction {
    fn code(self) -> &'static str {
        match self {
            Self::Approve => "a",
            Self::ApproveForSession => "s",
            Self::Decline => "d",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code {
            "a" => Some(Self::Approve),
            "s" => Some(Self::ApproveForSession),
            "d" => Some(Self::Decline),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCallback {
    pub action: ApprovalAction,
    pub request_id: RequestId,
}

impl ApprovalCallback {
    pub fn encode(&self) -> String {
        let (kind, value) = match &self.request_id {
            RequestId::Integer(value) => ("i", value.to_string()),
            RequestId::String(value) => ("s", urlencoding::encode(value).into_owned()),
        };
        format!("{CALLBACK_PREFIX}:{}:{kind}:{value}", self.action.code())
    }

    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.splitn(4, ':');
        if parts.next()? != CALLBACK_PREFIX {
            return None;
        }
        let action = ApprovalAction::from_code(parts.next()?)?;
        let kind = parts.next()?;
        let value = parts.next()?;
        let request_id = match kind {
            "i" => RequestId::Integer(value.parse().ok()?),
            "s" => RequestId::String(urlencoding::decode(value).ok()?.into_owned()),
            _ => return None,
        };
        Some(Self { action, request_id })
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
        let approve = ApprovalCallback {
            action: ApprovalAction::Approve,
            request_id: self.request_id.clone(),
        }
        .encode();
        let session = ApprovalCallback {
            action: ApprovalAction::ApproveForSession,
            request_id: self.request_id.clone(),
        }
        .encode();
        let decline = ApprovalCallback {
            action: ApprovalAction::Decline,
            request_id: self.request_id.clone(),
        }
        .encode();
        InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Approve", approve),
            InlineKeyboardButton::callback("Approve for session", session),
            InlineKeyboardButton::callback("Decline", decline),
        ]])
    }

    pub fn resolve_value(&self, action: ApprovalAction) -> anyhow::Result<Option<Value>> {
        if action == ApprovalAction::Decline {
            return Ok(None);
        }

        let value = match &self.kind {
            PendingApprovalKind::Command(_) => {
                let decision = match action {
                    ApprovalAction::Approve => CommandExecutionApprovalDecision::Accept,
                    ApprovalAction::ApproveForSession => {
                        CommandExecutionApprovalDecision::AcceptForSession
                    }
                    ApprovalAction::Decline => unreachable!(),
                };
                serde_json::to_value(CommandExecutionRequestApprovalResponse { decision })
                    .context("serialize command approval response")?
            }
            PendingApprovalKind::FileChange(_) => {
                let decision = match action {
                    ApprovalAction::Approve => FileChangeApprovalDecision::Accept,
                    ApprovalAction::ApproveForSession => {
                        FileChangeApprovalDecision::AcceptForSession
                    }
                    ApprovalAction::Decline => unreachable!(),
                };
                serde_json::to_value(FileChangeRequestApprovalResponse { decision })
                    .context("serialize file approval response")?
            }
            PendingApprovalKind::Permissions(params) => {
                let scope = match action {
                    ApprovalAction::Approve => PermissionGrantScope::Turn,
                    ApprovalAction::ApproveForSession => PermissionGrantScope::Session,
                    ApprovalAction::Decline => unreachable!(),
                };
                serde_json::to_value(PermissionsRequestApprovalResponse {
                    permissions: GrantedPermissionProfile {
                        network: params.permissions.network.clone(),
                        file_system: params.permissions.file_system.clone(),
                    },
                    scope,
                    strict_auto_review: None,
                })
                .context("serialize permissions approval response")?
            }
        };
        Ok(Some(value))
    }
}

pub fn rejection_error() -> JSONRPCErrorError {
    JSONRPCErrorError {
        code: -32000,
        message: "declined from Telegram".to_string(),
        data: None,
    }
}
