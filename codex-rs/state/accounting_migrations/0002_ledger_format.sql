CREATE TABLE draft_accounting_ledger_format (
    version INTEGER PRIMARY KEY NOT NULL CHECK(version = 2));
INSERT INTO draft_accounting_ledger_format VALUES (2);
-- Ledger format 2: a price record may state a Local or Undeclared basis and
-- where its basis came from, and a compact day may count that work. A build
-- that knows only format 1 refuses this ledger instead of misreading it.
