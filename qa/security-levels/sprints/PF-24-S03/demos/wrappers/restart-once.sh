#!/bin/bash
# Demo wrapper: runs the candidate, then starts it again with the same
# arguments and profile so a video can show a real restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
