#!/bin/bash
# usage: select.sh <session> <grep-pattern> : move Down until selected (›) line matches, then Enter
T="${TMUXCMD:-tmux -L pf60rerun3}"; s=$1; p=$2
for i in $(seq 1 120); do
  if $T capture-pane -p -t $s | grep '^› ' | grep -q -- "$p"; then $T send-keys -t $s Enter; sleep 1.2; exit 0; fi
  $T send-keys -t $s Down; sleep 0.12
done; echo "NOT FOUND: $p"; exit 1
