CREATE TABLE gpu_provision_steps (
    rental_id TEXT NOT NULL,
    step_id TEXT NOT NULL,
    command_digest TEXT NOT NULL,
    status TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    postcondition_json TEXT,
    sanitized_error TEXT,
    started_at_ms INTEGER,
    completed_at_ms INTEGER,
    updated_at_ms INTEGER NOT NULL,
    PRIMARY KEY(rental_id, step_id),
    FOREIGN KEY(rental_id) REFERENCES gpu_rentals(rental_id) ON DELETE CASCADE
);

CREATE TABLE gpu_runtime_providers (
    rental_id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL UNIQUE,
    base_url TEXT NOT NULL,
    model_id TEXT NOT NULL,
    wire_api TEXT NOT NULL,
    health TEXT NOT NULL,
    display_hourly_microusd INTEGER NOT NULL,
    catalog_sequence INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(rental_id) REFERENCES gpu_rentals(rental_id) ON DELETE CASCADE
);

CREATE INDEX gpu_runtime_providers_catalog_idx
    ON gpu_runtime_providers(health, catalog_sequence, updated_at_ms);
