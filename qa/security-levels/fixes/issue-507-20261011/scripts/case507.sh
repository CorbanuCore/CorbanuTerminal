#!/bin/bash
# usage: case507.sh <bin> <name>   #507, Codex pane: Aggressive confirmed (no restart), a slow command is running when the person turns the kill
# switch on in /security (g); then a new question; then the switch is turned off. GLM 5.3 Flash, disposable home.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=agg507-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -80 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
close() { for i in 1 2 3 4; do cap | grep -Eq "Grants and kill switch|Security level|Kill switch o(n|ff) in" || return 0; $T send-keys -t t Escape; sleep 1; done; }
idle() { sleep 4; for i in $(seq 1 ${1:-120}); do cap | grep -q "esc to interrupt" || return 0; sleep 1; done; }
kill_switch() { # on|off
  send "/security"; waitfor "Security level"; sleep 1
  $T send-keys -t t g; waitfor "Grants and kill switch"; sleep 0.5
  $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
  if [ $1 = on ]; then waitfor "Turn the kill switch on\?"; sleep 1; snap kill-on-review; $T send-keys -t t Enter; waitfor "Kill switch on" 30
  else waitfor "Turn the kill switch off\?"; sleep 0.5; $T send-keys -t t Down; sleep 0.3; snap kill-off-review; $T send-keys -t t Enter; waitfor "Kill switch off" 30; fi
  sleep 1; snap kill-$1-done; close; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
# Confirm Aggressive (no restart): the kill switch lives in its grants view.
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; $T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1; $T send-keys -t t Escape; sleep 1
send "Run exactly this shell command once, unchanged, and wait for it to finish: sleep 45 && touch ran-after-kill-switch.txt && echo FINISHED"
waitfor "sleep 45" 90; sleep 6; snap running
kill_switch on
sleep 8; snap after-switch
sleep 45; { ls $R/ws; [ -e $R/ws/ran-after-kill-switch.txt ] && echo "MARKER: written (the command ran)" || echo "MARKER: absent (the command was stopped)"; } > $R/marker.txt
send "What is 2+2?"; idle 60; sleep 2; snap turn-kill-on
kill_switch off
send "What is 3+3?"; idle 120; sleep 2; snap turn-kill-off
$T capture-pane -p -S -600 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
