CREATE TABLE draft_accounting_ledger_format_5 (
    version INTEGER PRIMARY KEY NOT NULL CHECK(version = 5));
INSERT INTO draft_accounting_ledger_format_5 VALUES (5);
-- Ledger format 5: an observation may record that the provider refused the
-- attempt (HTTP 401 or 403) before any response, so it used no tokens and is
-- not charged (#475). A build that knows only formats 1 to 4 refuses this
-- ledger instead of reading such an attempt as one with incomplete usage.
