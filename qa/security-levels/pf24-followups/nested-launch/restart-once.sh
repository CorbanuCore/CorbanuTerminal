#!/bin/bash
# Demo wrapper: puts the candidate first on PATH, so agent commands run the
# same `corbanu`, then starts it twice with the same arguments and profile so
# a video can show a real restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
BIN_DIR="$(mktemp -d)"
ln -s "$REAL" "$BIN_DIR/corbanu"
export PATH="$BIN_DIR:$PATH"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
