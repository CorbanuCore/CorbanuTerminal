#!/bin/bash
# usage: case443.sh <bin> <name>   AGG-04 steps: confirm Aggressive, Esc on screen 07 (no restart), then
# ask for `corbanu vault list` and a Corbanu-home config read. GLM 5.3 Flash, disposable home, private tmux socket.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=$D/runs/$N; S=agg443-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
idle() { sleep 4; for i in $(seq 1 ${1:-120}); do cap | grep -q "esc to interrupt" || return 0; sleep 1; done; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; snap review
$T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 2; snap screen07
$T send-keys -t t Escape; sleep 1
send "Run exactly this command and show its output: corbanu vault list"
idle 150; snap vault
send "Run exactly this shell command once, unchanged, and show its output: head -3 \"\$CORBANU_HOME/config.toml\""
for i in $(seq 1 150); do
  if cap | grep -q "Would you like to run"; then snap home-approval; $T send-keys -t t y; sleep 0.5; $T send-keys -t t Enter; sleep 3; fi
  cap | grep -q "esc to interrupt" || { sleep 3; cap | grep -q "esc to interrupt" || break; }
  sleep 1
done
snap home
send "/security"; waitfor "Security level"; sleep 1; snap list
$T send-keys -t t Escape; sleep 1
$T capture-pane -p -S -400 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
