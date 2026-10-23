# Shared tmux helpers. Needs T (tmux command) and R (run folder).
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -120 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
key() { $T send-keys -t t "$@"; sleep 0.4; }
down() { for i in $(seq 1 $1); do key Down; done; }
close() { for i in 1 2 3 4; do cap | grep -Eq "Grants and kill switch|Security level|Kill switch o(n|ff) in|Spawn Status|Switch Pane" || return 0; key Escape; sleep 0.6; done; }
kill_switch() { # on|off
  send "/security"; waitfor "Security level"; sleep 1
  key g; waitfor "Grants and kill switch"; sleep 0.5
  key Down; key Enter
  if [ $1 = on ]; then waitfor "Turn the kill switch on\?"; sleep 1; key Enter; waitfor "Kill switch on" 30
  else waitfor "Turn the kill switch off\?"; sleep 0.5; key Down; key Enter; waitfor "Kill switch off" 30; fi
  sleep 1; close; }
aggressive() {
  send "/security"; waitfor "Active in this session: Permissive"
  down 2; key Enter; waitfor "Switch to Aggressive"; sleep 1; key Enter
  waitfor "Saved: Aggressive" 30; sleep 1; key Escape; sleep 1; }
