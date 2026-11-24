## Fixes

Exported config lock files (`config_lock_export_dir`) write numbers under `desktop`, `telegram`, `openrouter_provider` and MITM hook bodies as TOML numbers. Previously they could appear as `$serde_json::private::Number` tables; reading the lock back was unaffected (#2872).
