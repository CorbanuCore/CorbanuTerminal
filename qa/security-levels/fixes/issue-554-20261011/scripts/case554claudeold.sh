#!/bin/bash
# usage: case554claudeold.sh <bin> <name>   #554: under Aggressive (contained Claude panes), a Claude pane with the
# Claude Code found on PATH (CLAUDE_BIN_DIR, here an old one) gets a task.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=agg554o-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -80 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; $T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1; $T send-keys -t t Escape; sleep 1
send "/panes"; waitfor "Create User Pane"; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Opus 5.5 Claude Plan"; sleep 1; snap claude-pane-picker
for i in 1 2 3 4 5 6; do $T send-keys -t t Down; sleep 0.2; done; $T send-keys -t t Enter
waitfor "Name Claude pane"; $T send-keys -t t Enter; sleep 3; snap claude-pane-created
send "Reply with exactly the word pong and nothing else."; sleep 25; snap claude-turn
$T capture-pane -p -S -800 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
