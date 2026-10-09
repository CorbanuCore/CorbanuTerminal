#!/bin/bash
# usage: fullcap.sh <session> <outfile> <command...>: type a slash command into a TALL pane and capture the whole screen once
# (verbatim, no scrolling, no de-duplication). Closes an open popup first (never double-Esc).
T="tmux -L pf60s05indep"; C=<scratch>/cap
s=$1; o=$2; shift 2
for i in 1 2 3 4; do $T capture-pane -p -t $s | grep -q 'Esc back/close' || break; $T send-keys -t $s Escape; sleep 0.4; done
if [ -n "$*" ]; then $T send-keys -t $s -l "$*"; sleep 0.3; $T send-keys -t $s Enter; sleep ${WAIT:-3}; fi
{ echo "\$ $*"; $T capture-pane -p -J -t $s | sed 's/[[:space:]]*$//' | awk 'NF{p=1} p' ; } > $C/$o.txt
