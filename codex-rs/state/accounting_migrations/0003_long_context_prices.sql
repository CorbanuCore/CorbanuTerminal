CREATE TABLE draft_accounting_ledger_format_3 (
    version INTEGER PRIMARY KEY NOT NULL CHECK(version = 3));
INSERT INTO draft_accounting_ledger_format_3 VALUES (3);
-- Ledger format 3: a price record may state a long-context tier, rates that
-- price a whole attempt whose input is above a threshold. A build that knows
-- only formats 1 and 2 refuses this ledger instead of pricing those attempts
-- at the base rates.
