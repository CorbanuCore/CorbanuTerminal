#!/bin/bash
# Demo wrapper: stores an unknown level in the disposable profile before launch.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
printf 'version = 1\nlevel = "max"\n' > "$CODEX_HOME/security_level.toml"
exec "$REAL" "$@"
