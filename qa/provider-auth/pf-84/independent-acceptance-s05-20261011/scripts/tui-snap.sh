#!/bin/bash
# tui-snap.sh <tag> <home> <bin> [tui args...]: start the TUI (real Z.AI default key in its env,
# never argv), optionally send one prompt (PROMPT=...), capture /status and the first /cost
# screen (or the slash-command reply), then /exit. Captures go to $EV/captures/tui-<tag>.txt.
cd "$(dirname "$0")" && . ./env.sh && . ./tmuxlib.sh
TAG=$1 H=$2; BIN=$3; shift 3
OUT=$EV/captures/tui-$TAG.txt SOCK=$S/tui-$TAG.sock
/opt/homebrew/bin/tmux -S "$SOCK" kill-server 2>/dev/null
start_keyed_tui "$SOCK" t "${LABEL:-provider/zai_api_key}" "${VAR:-ZAI_API_KEY}" "$H" "$@"
wait_for t 'TPS:' 20 || sleep 3
sleep 2
{ echo "# tui-$TAG: bin=${BIN//$S/\$S} home=${H//$S/\$S} args: $*"; } > "$OUT"
if [ -n "${PROMPT:-}" ]; then
  send_line t "$PROMPT"; sleep "${PROMPT_WAIT:-15}"
  echo "--- after prompt ---" >> "$OUT"; cap t -80 | grep -v '^\s*$' | tail -8 >> "$OUT"
fi
for cmd in /status /cost; do
  tx send-keys -t t C-l; sleep 0.5; send_line t "$cmd"; sleep 5
  echo "--- $cmd ---" >> "$OUT"
  cap t -120 | grep -v '^\s*$' | sed -n '/^╰─*╯$/,$p' | tail -n +2 >> "$OUT"
  tx send-keys -t t Escape; sleep 1
done
send_line t /exit; sleep 3
echo "--- exit ---" >> "$OUT"; cap t -20 | grep -v '^\s*$' | tail -3 >> "$OUT"
/opt/homebrew/bin/tmux -S "$SOCK" kill-server 2>/dev/null
sed -i '' "s#$S#\$S#g" "$OUT"; cat "$OUT"
