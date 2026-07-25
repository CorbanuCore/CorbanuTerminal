CREATE TABLE agent_mailbox_messages (
    message_id TEXT PRIMARY KEY,
    recipient_thread_id TEXT NOT NULL,
    communication_json TEXT NOT NULL,
    phase TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

CREATE INDEX agent_mailbox_recipient_phase_created_idx
    ON agent_mailbox_messages(recipient_thread_id, phase, created_at_ms, message_id);
