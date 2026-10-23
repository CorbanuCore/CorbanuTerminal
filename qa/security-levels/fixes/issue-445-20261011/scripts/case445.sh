#!/bin/bash
# usage: case445.sh <bin> <name> [restart]  AGG-22 steps: Aggressive confirmed (optionally restarted), kill switch on
# in /security (g), then "What is 2+2?"; kill switch off, ask again. GLM 5.3 Flash, disposable home, private tmux.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; MODE=$3; R=$D/runs/$N; S=agg445-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
close() { for i in 1 2 3 4; do cap | grep -Eq "Grants and kill switch|Security level|Kill switch o(n|ff) in" || return 0; $T send-keys -t t Escape; sleep 1; done; }
idle() { sleep 4; for i in $(seq 1 ${1:-120}); do cap | grep -q "esc to interrupt" || return 0; sleep 1; done; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; $T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1
if [ "$MODE" = restart ]; then
  $T send-keys -t t r; sleep 15; waitfor "Corbanu Terminal \(v" 60; sleep 3
  send "/security"; waitfor "Active in this session: Aggressive"; sleep 1
else
  $T send-keys -t t Escape; sleep 1
  send "/security"; waitfor "Security level"; sleep 1
fi
$T send-keys -t t g; waitfor "Grants and kill switch"; sleep 0.5
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Turn the kill switch on\?"; sleep 1; snap kill-on-review
$T send-keys -t t Enter; waitfor "Kill switch on" 30; sleep 1; snap kill-on-done
close
send "What is 2+2?"
idle 120; sleep 2; snap turn-kill-on
send "/security"; waitfor "Security level"; sleep 1; snap list-kill-on
$T send-keys -t t g; waitfor "Grants and kill switch"; sleep 0.5
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Turn the kill switch off\?"; sleep 0.5; $T send-keys -t t Down; sleep 0.3; snap kill-off-review
$T send-keys -t t Enter; waitfor "Kill switch off" 30; sleep 1
close
send "What is 3+3?"
idle 120; sleep 2; snap turn-kill-off
$T capture-pane -p -S -400 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
