## Fixes

Numeric settings under `desktop`, `telegram`, and `openrouter_provider` retain their TOML types when configuration layers are read or updated. Previously, updating a table through `config/value/write` could replace existing numbers with nested tables on disk (#2761).
