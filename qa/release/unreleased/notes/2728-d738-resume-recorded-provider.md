## Upstream Codex rust-v0.167.0

- A conversation keeps running on the provider it was recorded on when you
  resume it or keep typing after changing your default provider, as before
  this upstream update (#555). Upstream made such a conversation read-only.
  Corbanu now continues it when your Corbanu home still has that provider
  configured and active; it stays read-only when the provider is no longer
  configured, has been deactivated in `/providers`, or your organization
  requires a different provider.
  While such a conversation runs, `/mcp`, apps, the experimental-features list and
  config reloads keep its provider and model instead of reloading your new default
  (#2853).
