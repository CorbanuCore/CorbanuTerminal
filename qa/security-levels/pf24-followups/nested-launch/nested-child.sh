#!/bin/bash
# Demo wrapper: starts the candidate as an agent command under Aggressive
# with nested agents set to pass would. The origin home stores Aggressive
# (pass) and holds a vault canary; the child's own home stores nothing.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
ORIGIN="$(cd "$CODEX_HOME/.." && pwd)/origin"
mkdir -p "$ORIGIN/secrets"
printf 'version = 1\nlevel = "aggressive"\nnested_agents = "pass"\n' > "$ORIGIN/security_level.toml"
printf 'nested-canary-0001\n' > "$ORIGIN/secrets/canary.txt"
export CORBANU_SECURITY_ORIGIN="$ORIGIN"
exec "$REAL" "$@"
