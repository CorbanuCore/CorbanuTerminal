CREATE TABLE draft_accounting_ledger_format_6 (
    version INTEGER PRIMARY KEY NOT NULL CHECK(version = 6));
INSERT INTO draft_accounting_ledger_format_6 VALUES (6);
-- Ledger format 6: an observation may record when a realtime session ended
-- (#824). The session is one attempt admitted when it starts; its tokens are
-- not reported to this client, so they stay unknown. A build that knows only
-- formats 1 to 5 refuses this ledger instead of rejecting such an observation.
