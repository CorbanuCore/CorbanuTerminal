#!/bin/bash
# Demo wrapper: puts the candidate first on PATH as `corbanu`, so agent
# commands run it rather than an installed corbanu. Record with
#   DEMO_CANDIDATE=<corbanu> python3 scripts/demo_video.py record <spec> --bin qa/demos/wrappers/candidate-on-path.sh
REAL="${DEMO_CANDIDATE:?set DEMO_CANDIDATE to the candidate corbanu binary}"
BIN_DIR="$(mktemp -d)" || exit 1
trap 'rm -r "$BIN_DIR"' EXIT
ln -s "$REAL" "$BIN_DIR/corbanu"
export PATH="$BIN_DIR:$PATH"
"$REAL" "$@"
