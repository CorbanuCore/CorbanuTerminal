CREATE TABLE draft_accounting_ledger_format_4 (
    version INTEGER PRIMARY KEY NOT NULL CHECK(version = 4));
INSERT INTO draft_accounting_ledger_format_4 VALUES (4);
-- Ledger format 4: an attempt may name the provider account it was sent on
-- (PF-84). Attempts on the default account omit it and keep their bytes. A
-- build that knows only formats 1 to 3 refuses this ledger instead of
-- rejecting or misattributing those attempts.
