-- Keep the provider's reset hint separate from the bounded local cooldown.
ALTER TABLE provider_request_state
ADD COLUMN provider_reset_at_ms INTEGER NOT NULL DEFAULT 0;
