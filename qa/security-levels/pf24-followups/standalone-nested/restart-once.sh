#!/bin/bash
# Demo wrapper: puts the candidate `corbanu` and the standalone `codex-exec`
# and `codex-tui` built next to it first on PATH, so agent commands run them,
# then starts the candidate twice with the same arguments and profile so a
# video can show a real restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
BIN_DIR="$(mktemp -d)"
ln -s "$REAL" "$BIN_DIR/corbanu"
for standalone in codex-exec codex-tui; do
  ln -s "$(dirname "$REAL")/$standalone" "$BIN_DIR/$standalone"
done
export PATH="$BIN_DIR:$PATH"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
