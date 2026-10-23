#!/bin/bash
# usage: cmdcap.sh <session> <outfile> <command...>: close popup if open (never double-Esc), type slash command, capture full scroll
T="${TMUXCMD:-tmux -L pf60rerun3}"; C=${CAPDIR:-<scratch>/cap}; here=$(dirname "$0")
s=$1; o=$2; shift 2
for i in 1 2 3 4; do $T capture-pane -p -t $s | grep -q 'Esc back/close' || break; $T send-keys -t $s Escape; sleep 0.4; done
$T send-keys -t $s -l "$*"; sleep 0.3; $T send-keys -t $s Enter; sleep 3
{ echo "\$ $*"; $here/scrollcap.sh $s ${STEPS:-45}; echo "--- last screen:"; $T capture-pane -p -J -t $s | sed 's/[[:space:]]*$//' | grep -v '^\s*$' | tail -${TAILN:-6}; } > $C/$o.txt
