#!/bin/bash
# Demo wrapper for contained Claude panes. Puts Claude Code from
# CLAUDE_BIN_DIR first on PATH, then stores the spec's ZAI_API_KEY in the demo
# profile's vault as `provider/zai_api_key` (the Z.AI pane's label) through
# the real `/vault credential add` popup in a private, unrecorded tmux server,
# because the vault has no non-interactive add; it stops unless each step of
# the popup is on screen. The candidate keeps ZAI_API_KEY: the spec's main
# model uses it. With DEMO_RESTART=1 it starts
# the candidate twice, so a video can show a restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
export PATH="${CLAUDE_BIN_DIR:?set CLAUDE_BIN_DIR to a Claude Code 2.1.29x bin folder}:$PATH"
: "${ZAI_API_KEY:?the spec must provide ZAI_API_KEY}"
SEED="demo-seed-$$"
T="tmux -L $SEED"
# Waits until the seeding session shows $1, or stops the demo.
wait_for() {
  for _ in $(seq 1 60); do
    $T capture-pane -p | grep -q "$1" && return 0
    sleep 0.5
  done
  echo "demo-launch: the vault seeding session never showed '$1'; stopping" >&2
  $T kill-server 2>/dev/null
  exit 1
}
$T new-session -d -x 140 -y 40 "$(printf '%q ' "$REAL" "$@")"
wait_for 'model: *glm'
$T send-keys '/vault credential add'; sleep 0.5; $T send-keys Enter
wait_for '1/2 — label'
$T send-keys 'provider/zai_api_key'; sleep 0.3; $T send-keys Enter
# Only paste the key into the masked field, never into the composer.
wait_for '2/2 — secret (masked)'
printf '%s' "$ZAI_API_KEY" | $T load-buffer -b k -
$T paste-buffer -d -b k; sleep 0.5; $T send-keys Enter
wait_for 'Added vault credential'
$T send-keys '/exit'; sleep 0.5; $T send-keys Enter; sleep 3
$T kill-server 2>/dev/null
if [ "${DEMO_RESTART:-0}" = 1 ]; then
  "$REAL" "$@"
  printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
  sleep 1
fi
exec "$REAL" "$@"
