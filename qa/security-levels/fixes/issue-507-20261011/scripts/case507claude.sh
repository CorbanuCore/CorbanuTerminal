#!/bin/bash
# usage: case507claude.sh <bin> <name>   #507, Claude pane (contained, Claude Plan, disposable Claude config):
# a long reply is streaming when the person turns the kill switch on; then a new Claude turn; then the switch off.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=agg507c-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -80 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
close() { for i in 1 2 3 4; do cap | grep -Eq "Grants and kill switch|Security level|Kill switch o(n|ff) in" || return 0; $T send-keys -t t Escape; sleep 1; done; }
kill_switch() { # on|off
  send "/security"; waitfor "Security level"; sleep 1
  $T send-keys -t t g; waitfor "Grants and kill switch"; sleep 0.5
  $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
  if [ $1 = on ]; then waitfor "Turn the kill switch on\?"; sleep 1; $T send-keys -t t Enter; waitfor "Kill switch on" 30
  else waitfor "Turn the kill switch off\?"; sleep 0.5; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter; waitfor "Kill switch off" 30; fi
  sleep 1; close; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
# Confirm Aggressive (no restart): the kill switch lives in its grants view.
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; $T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1; $T send-keys -t t Escape; sleep 1
send "/panes"; waitfor "Create User Pane"; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Opus 5.5 Claude Plan"; for i in 1 2 3 4 5 6; do $T send-keys -t t Down; sleep 0.2; done; $T send-keys -t t Enter
waitfor "Name Claude pane"; $T send-keys -t t Enter; sleep 3
send "Write a detailed 1500-word essay on the history of tea, in plain text."
sleep 20; snap claude-streaming
kill_switch on
sleep 10; snap claude-after-switch
send "Reply with exactly the word pong and nothing else."; sleep 15; snap claude-turn-kill-on
kill_switch off
send "Reply with exactly the word pong and nothing else."; sleep 40; snap claude-turn-kill-off
$T capture-pane -p -S -800 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
