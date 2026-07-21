CREATE TABLE gpu_rentals (
    rental_id TEXT PRIMARY KEY,
    installation_id TEXT NOT NULL,
    client_operation_id TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    recipe_revision TEXT NOT NULL,
    offer_snapshot_json TEXT NOT NULL,
    quote_expires_at_ms INTEGER,
    max_hourly_microusd INTEGER NOT NULL,
    max_total_microusd INTEGER NOT NULL,
    terminate_at_ms INTEGER NOT NULL,
    enforcement_class TEXT NOT NULL,
    desired_state TEXT NOT NULL,
    observed_state TEXT NOT NULL,
    provider_resource_id TEXT,
    ownership_tag TEXT NOT NULL UNIQUE,
    state_sequence INTEGER NOT NULL DEFAULT 1,
    controller_lease_owner TEXT,
    controller_lease_until_ms INTEGER NOT NULL DEFAULT 0,
    provision_step TEXT,
    endpoint_base_url TEXT,
    endpoint_provider_id TEXT,
    last_error_code TEXT,
    last_error_message TEXT,
    diagnostic_ref TEXT,
    last_reconciled_at_ms INTEGER,
    next_retry_at_ms INTEGER NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    estimated_accrued_microusd INTEGER NOT NULL DEFAULT 0,
    provider_reported_cost_microusd INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    terminated_confirmed_at_ms INTEGER
);

CREATE INDEX gpu_rentals_due_idx
    ON gpu_rentals(next_retry_at_ms, controller_lease_until_ms);
CREATE INDEX gpu_rentals_provider_resource_idx
    ON gpu_rentals(provider, provider_resource_id);
CREATE INDEX gpu_rentals_observed_state_idx
    ON gpu_rentals(observed_state, updated_at_ms DESC);

CREATE TABLE gpu_rental_operations (
    operation_id TEXT PRIMARY KEY,
    rental_id TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    operation_sequence INTEGER NOT NULL,
    status TEXT NOT NULL,
    provider_request_id TEXT,
    provider_resource_id TEXT,
    sanitized_error TEXT,
    started_at_ms INTEGER NOT NULL,
    completed_at_ms INTEGER,
    UNIQUE(rental_id, operation_kind, operation_sequence),
    FOREIGN KEY(rental_id) REFERENCES gpu_rentals(rental_id) ON DELETE CASCADE
);

CREATE INDEX gpu_rental_operations_status_idx
    ON gpu_rental_operations(rental_id, status, started_at_ms);

CREATE TABLE gpu_rental_notifications (
    rental_id TEXT NOT NULL,
    state_sequence INTEGER NOT NULL,
    notification_kind TEXT NOT NULL,
    delivered_at_ms INTEGER NOT NULL,
    PRIMARY KEY(rental_id, state_sequence, notification_kind),
    FOREIGN KEY(rental_id) REFERENCES gpu_rentals(rental_id) ON DELETE CASCADE
);
