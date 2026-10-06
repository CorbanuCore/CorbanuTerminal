#!/bin/bash
# Demo wrapper: stores Aggressive before the first launch, then runs the
# candidate twice with the same profile so a video can show a real restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
printf 'version = 1\nlevel = "aggressive"\n' > "$CODEX_HOME/security_level.toml"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
