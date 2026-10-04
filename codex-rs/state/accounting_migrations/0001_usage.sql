CREATE TABLE draft_accounting_attempts (
    attempt_id TEXT PRIMARY KEY, request_id TEXT NOT NULL, payload TEXT NOT NULL);
CREATE INDEX draft_accounting_request ON draft_accounting_attempts(request_id);
CREATE TABLE draft_accounting_observations (
    attempt_id TEXT NOT NULL REFERENCES draft_accounting_attempts(attempt_id),
    revision INTEGER NOT NULL CHECK(revision > 0),
    source TEXT NOT NULL, sequence INTEGER NOT NULL CHECK(sequence >= 0),
    payload TEXT NOT NULL, PRIMARY KEY(attempt_id, revision), UNIQUE(source, sequence));
CREATE TABLE draft_accounting_price_snapshots (
    snapshot_id TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL);
CREATE TABLE draft_accounting_price_bindings (
    attempt_id TEXT PRIMARY KEY NOT NULL REFERENCES draft_accounting_attempts(attempt_id),
    snapshot_id TEXT REFERENCES draft_accounting_price_snapshots(snapshot_id));
CREATE TABLE draft_accounting_estimates (
    attempt_id TEXT NOT NULL REFERENCES draft_accounting_price_bindings(attempt_id),
    evidence TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(attempt_id, evidence));
CREATE TABLE draft_accounting_contributions (
    attempt_id TEXT PRIMARY KEY NOT NULL, thread_id TEXT NOT NULL,
    utc_day INTEGER NOT NULL CHECK(utc_day >= 0), evidence TEXT NOT NULL,
    FOREIGN KEY(attempt_id, evidence) REFERENCES draft_accounting_estimates(attempt_id, evidence));
CREATE INDEX draft_accounting_contribution_day ON draft_accounting_contributions(thread_id, utc_day);
CREATE TABLE draft_accounting_tombstones (
    attempt_id TEXT PRIMARY KEY NOT NULL,
    expires_at_ms INTEGER NOT NULL CHECK(expires_at_ms >= 0));
CREATE TABLE draft_accounting_compact_days (
    thread_id TEXT NOT NULL, utc_day INTEGER NOT NULL, payload TEXT NOT NULL,
    PRIMARY KEY(thread_id, utc_day));
CREATE TABLE draft_accounting_compact_snapshots (
    thread_id TEXT NOT NULL, utc_day INTEGER NOT NULL, snapshot_id TEXT NOT NULL,
    PRIMARY KEY(thread_id, utc_day, snapshot_id),
    FOREIGN KEY(thread_id, utc_day) REFERENCES draft_accounting_compact_days(thread_id, utc_day),
    FOREIGN KEY(snapshot_id) REFERENCES draft_accounting_price_snapshots(snapshot_id));
CREATE TABLE draft_accounting_retention_checkpoint (
    singleton INTEGER PRIMARY KEY, completed_as_of_ms INTEGER,
    admission_active INTEGER NOT NULL CHECK(admission_active IN (0, 1)));
INSERT INTO draft_accounting_retention_checkpoint VALUES (1, NULL, 0);
