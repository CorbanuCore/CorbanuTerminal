#!/bin/bash
# Demo wrapper: puts the candidate first on PATH as `corbanu` (so agent
# commands run it), then starts it twice with the same arguments and profile
# so a video can show a real restart. Record with
#   DEMO_CANDIDATE=<corbanu> python3 scripts/demo_video.py record <spec> --bin qa/demos/wrappers/restart-once.sh
REAL="${DEMO_CANDIDATE:?set DEMO_CANDIDATE to the candidate corbanu binary}"
BIN_DIR="$(mktemp -d)"
ln -s "$REAL" "$BIN_DIR/corbanu"
export PATH="$BIN_DIR:$PATH"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
"$REAL" "$@"
status=$?
rm -r "$BIN_DIR"
exit $status
