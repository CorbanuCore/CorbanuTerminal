#!/bin/bash
# Demo wrapper for contained Claude panes. Puts Claude Code from
# CLAUDE_BIN_DIR first on PATH and copies DEMO_VAULT_FROM (the `secrets`
# folder of a disposable test profile that holds `provider/zai_api_key`) into
# the demo profile, since the vault has no non-interactive add. With
# DEMO_RESTART=1 it starts the candidate twice, so a video can show a restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
export PATH="${CLAUDE_BIN_DIR:?set CLAUDE_BIN_DIR to a Claude Code 2.1.29x bin folder}:$PATH"
cp -R "${DEMO_VAULT_FROM:?set DEMO_VAULT_FROM to a test profile secrets folder}" "${CODEX_HOME:?}/secrets"
if [ "${DEMO_RESTART:-0}" = 1 ]; then
  "$REAL" "$@"
  printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
  sleep 1
fi
exec "$REAL" "$@"
