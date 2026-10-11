# Home hS without any default key (no env): does the TUI reach the chat on a named account? (#417 context)
. "$(dirname "$0")/../env.sh"; . $S/scripts/tmuxlib.sh
H=$S/homes/hS; TUI_ENV=""
for spec in "n1-account-main|--account main" "n2-config-main|-c provider_accounts.zai=\"main\""; do
  s=${spec%%|*}; a=${spec#*|}
  eval "tui_start $s $H $a"
  if cap $s | grep -q 'model: '; then type_and_enter $s "$PROMPT"; wait_for $s '^• pong|401|rror' 90; fi
  echo "-- $s ($a):"; cap $s | grep -v '^\s*$' | grep -E 'pong|API key|Connect|401|rror|model:' | head -5 | sed 's/^/   | /'
  tui_stop $s >/dev/null 2>&1 || $T kill-session -t $s
done
