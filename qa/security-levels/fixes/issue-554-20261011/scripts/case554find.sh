#!/bin/bash
# usage: case554find.sh <bin> <name>   #554: under Aggressive (confirmed, no restart), the agent searches and lists the
# folder that holds the Corbanu home (`find`, `ls -aR`); the home is denied, and the denial should be explained.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=agg554f-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -120 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
idle() { sleep 5; for i in $(seq 1 ${1:-120}); do cap | grep -Eq "esc to interrupt|Working" || return 0; sleep 1; done; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
waitfor "Switch to Aggressive"; sleep 1; $T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1; $T send-keys -t t Escape; sleep 1
send "Run exactly this shell command once, then tell me in one sentence what happened: find '$R' -maxdepth 2 -name config.toml"
idle 180; snap find-above
send "Run exactly this shell command once, then tell me in one sentence what happened: ls -aR '$R' > /dev/null"
idle 180; snap ls-recursive-above
$T capture-pane -p -S -800 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
