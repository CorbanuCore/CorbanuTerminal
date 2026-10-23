-- #482: the default account's throttle row was keyed by an unsalted sha256
-- prefix of its API key, the same value on every machine. Keys are now
-- fingerprinted with this per-home salt instead.
CREATE TABLE provider_request_key_salt (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    salt BLOB NOT NULL
);

INSERT INTO provider_request_key_salt (id, salt) VALUES (1, randomblob(32));

-- Unsalted rows that hold no active cooldown or lease carry nothing worth
-- keeping. Active ones move to their salted key the next time that key is used.
DELETE FROM provider_request_state
WHERE key_fingerprint LIKE 'env:%'
  AND cooldown_until_ms <= CAST(strftime('%s', 'now') AS INTEGER) * 1000
  AND lease_until_ms <= CAST(strftime('%s', 'now') AS INTEGER) * 1000;
