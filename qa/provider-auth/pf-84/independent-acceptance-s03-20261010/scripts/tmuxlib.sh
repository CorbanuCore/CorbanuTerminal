# Minimal tmux driver following docs/tmuxHarness.md: private socket, literal text and Enter sent
# separately, waits need the pattern plus two identical captures, bounded captures.
T="tmux -L pf84s03acc"
cap() { $T capture-pane -p -t "$1" -S -400; }
wait_for() { # session pattern [timeout_s]
  local s=$1 pat=$2 to=${3:-60} prev="" cur i=0
  while [ $i -lt $((to*2)) ]; do cur=$(cap "$s"); if printf '%s' "$cur" | grep -q -E -- "$pat" && [ "$cur" = "$prev" ]; then return 0; fi; prev=$cur; sleep 0.5; i=$((i+1)); done
  echo "TIMEOUT waiting for /$pat/ in $s"; return 1; }
send_text() { $T send-keys -t "$1" -l -- "$2"; }
send_enter() { $T send-keys -t "$1" Enter; }
type_and_enter() { send_text "$1" "$2"; wait_for "$1" "$(printf '%s' "$2" | cut -c1-20 | sed 's/[][\.*^$(){}|+?]/./g')" 15; send_enter "$1"; }
# tui_start <session> <home> [args...]: start the candidate TUI in a clean env
tui_start() { local s=$1 home=$2; shift 2; local args=""; local a; for a in "$@"; do args="$args '$a'"; done; mkdir -p $S/logs/$s
  $T new-session -d -s "$s" -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=$S/pathbin:/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home ${TUI_ENV:-} $B -c log_dir='\"$S/logs/$s\"' --no-alt-screen $args; echo TUI_EXITED rc=\$?; sleep 9999"
  wait_for "$s" 'Do you trust|model: |rror' 60 || true
  if cap "$s" | grep -q 'Do you trust'; then send_enter "$s"; sleep 1; fi
  wait_for "$s" 'model: |rror|TUI_EXITED' 30 || true; sleep 2; }
tui_stop() { local s=$1; type_and_enter "$s" "/exit"; wait_for "$s" 'TUI_EXITED' 30; cap "$s" > $S/caps/tui-$s.txt; $T kill-session -t "$s"; }
