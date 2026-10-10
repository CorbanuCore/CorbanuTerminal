# minimal tmux driver following docs/tmuxHarness.md rules: private socket, literal text and Enter sent separately,
# stable waits (condition + two identical captures), bounded captures.
T="tmux -L pf84acc-tui"
cap() { $T capture-pane -p -t "$1" -S -200; }
wait_for() { # session pattern timeout_s
  local s=$1 pat=$2 to=${3:-60} prev="" cur i=0
  while [ $i -lt $((to*2)) ]; do cur=$(cap $s); if printf '%s' "$cur" | grep -q -E -- "$pat" && [ "$cur" = "$prev" ]; then return 0; fi; prev=$cur; sleep 0.5; i=$((i+1)); done
  echo "TIMEOUT waiting for /$pat/ in $s"; return 1; }
send_text() { $T send-keys -t "$1" -l -- "$2"; }
send_enter() { $T send-keys -t "$1" Enter; }
type_and_enter() { send_text "$1" "$2"; wait_for "$1" "$(printf '%s' "$2" | cut -c1-20 | sed 's/[][\.*^$(){}|+?]/./g')" 15; send_enter "$1"; }
