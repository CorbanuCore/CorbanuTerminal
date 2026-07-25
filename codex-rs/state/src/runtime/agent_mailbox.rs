use super::*;
use codex_protocol::protocol::InterAgentCommunication;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentMailboxPhase {
    Admitted,
    Ready,
    Submitting,
    Submitted,
    ProviderRunning,
    RetryableFailure,
    UnknownOutcome,
    Applied,
    Cancelled,
    TerminalFailure,
}

impl AgentMailboxPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Ready => "ready",
            Self::Submitting => "submitting",
            Self::Submitted => "submitted",
            Self::ProviderRunning => "provider_running",
            Self::RetryableFailure => "retryable_failure",
            Self::UnknownOutcome => "unknown_outcome",
            Self::Applied => "applied",
            Self::Cancelled => "cancelled",
            Self::TerminalFailure => "terminal_failure",
        }
    }
}

impl FromStr for AgentMailboxPhase {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admitted" => Ok(Self::Admitted),
            "ready" => Ok(Self::Ready),
            "submitting" => Ok(Self::Submitting),
            "submitted" => Ok(Self::Submitted),
            "provider_running" => Ok(Self::ProviderRunning),
            "retryable_failure" => Ok(Self::RetryableFailure),
            "unknown_outcome" => Ok(Self::UnknownOutcome),
            "applied" => Ok(Self::Applied),
            "cancelled" => Ok(Self::Cancelled),
            "terminal_failure" => Ok(Self::TerminalFailure),
            other => anyhow::bail!("unknown agent mailbox phase `{other}`"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentMailboxAdmission {
    Inserted,
    Existing(AgentMailboxPhase),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentMailboxMessage {
    pub recipient_thread_id: ThreadId,
    pub communication: InterAgentCommunication,
    pub phase: AgentMailboxPhase,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl StateRuntime {
    pub async fn admit_agent_message(
        &self,
        recipient_thread_id: ThreadId,
        communication: &InterAgentCommunication,
        now_ms: i64,
    ) -> anyhow::Result<AgentMailboxAdmission> {
        let message_id = communication
            .message_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("agent mailbox message has no message_id"))?;
        let created_at_ms = communication.created_at_ms.unwrap_or(now_ms);
        let communication_json = serde_json::to_string(communication)?;
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query(
            r#"
INSERT OR IGNORE INTO agent_mailbox_messages (
    message_id,
    recipient_thread_id,
    communication_json,
    phase,
    created_at_ms,
    updated_at_ms
) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(message_id)
        .bind(recipient_thread_id.to_string())
        .bind(communication_json.as_str())
        .bind(AgentMailboxPhase::Admitted.as_str())
        .bind(created_at_ms)
        .bind(now_ms)
        .execute(&mut *tx)
        .await?
        .rows_affected()
            == 1;
        let row = sqlx::query(
            r#"
SELECT recipient_thread_id, communication_json, phase
FROM agent_mailbox_messages
WHERE message_id = ?
            "#,
        )
        .bind(message_id)
        .fetch_one(&mut *tx)
        .await?;
        let stored_recipient: String = row.try_get("recipient_thread_id")?;
        let stored_communication: String = row.try_get("communication_json")?;
        if stored_recipient != recipient_thread_id.to_string()
            || stored_communication != communication_json
        {
            anyhow::bail!(
                "agent mailbox message_id `{message_id}` conflicts with a different message"
            );
        }
        let phase = AgentMailboxPhase::from_str(row.try_get("phase")?)?;
        tx.commit().await?;
        Ok(if inserted {
            AgentMailboxAdmission::Inserted
        } else {
            AgentMailboxAdmission::Existing(phase)
        })
    }

    pub async fn transition_agent_message(
        &self,
        message_id: &str,
        expected: AgentMailboxPhase,
        next: AgentMailboxPhase,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
UPDATE agent_mailbox_messages
SET phase = ?, updated_at_ms = ?
WHERE message_id = ? AND phase = ?
            "#,
        )
        .bind(next.as_str())
        .bind(now_ms)
        .bind(message_id)
        .bind(expected.as_str())
        .execute(self.pool.as_ref())
        .await?
        .rows_affected();
        Ok(rows_affected == 1)
    }

    pub async fn list_recoverable_agent_messages(
        &self,
        recipient_thread_id: ThreadId,
    ) -> anyhow::Result<Vec<AgentMailboxMessage>> {
        let rows = sqlx::query(
            r#"
SELECT communication_json, phase, created_at_ms, updated_at_ms
FROM agent_mailbox_messages
WHERE recipient_thread_id = ?
  AND phase NOT IN ('applied', 'cancelled', 'terminal_failure')
ORDER BY created_at_ms ASC, message_id ASC
            "#,
        )
        .bind(recipient_thread_id.to_string())
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(AgentMailboxMessage {
                    recipient_thread_id,
                    communication: serde_json::from_str(
                        row.try_get::<String, _>("communication_json")?.as_str(),
                    )?,
                    phase: AgentMailboxPhase::from_str(row.try_get("phase")?)?,
                    created_at_ms: row.try_get("created_at_ms")?,
                    updated_at_ms: row.try_get("updated_at_ms")?,
                })
            })
            .collect()
    }

    pub async fn mark_agent_message_applied(
        &self,
        message_id: &str,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
UPDATE agent_mailbox_messages
SET phase = ?, updated_at_ms = ?
WHERE message_id = ?
  AND phase NOT IN ('applied', 'cancelled', 'terminal_failure', 'unknown_outcome')
            "#,
        )
        .bind(AgentMailboxPhase::Applied.as_str())
        .bind(now_ms)
        .bind(message_id)
        .execute(self.pool.as_ref())
        .await?
        .rows_affected();
        Ok(rows_affected == 1)
    }
}

#[cfg(test)]
#[path = "agent_mailbox_tests.rs"]
mod tests;
