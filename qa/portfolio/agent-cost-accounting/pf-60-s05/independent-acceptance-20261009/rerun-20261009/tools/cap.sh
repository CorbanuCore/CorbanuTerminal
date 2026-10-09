#!/bin/bash
# usage: cap.sh <session> <name> [command...]: optionally type a slash command (closing an open popup first, never
# double-Esc), then scroll the popup with Down and save (a) cap/<name>.txt: union of lines in first-seen order,
# identical lines collapsed (S03 method), and (b) cap/raw/<name>.frames.txt: every distinct screen verbatim.
T="tmux -L pf60rerun"; C=<scratch>/cap; mkdir -p $C/raw
s=$1; o=$2; shift 2
if [ -n "$*" ]; then
  for i in 1 2 3 4; do $T capture-pane -p -t $s | grep -q 'Esc back/close' || break; $T send-keys -t $s Escape; sleep 0.4; done
  $T send-keys -t $s -l "$*"; sleep 0.3; $T send-keys -t $s Enter; sleep ${WAIT:-3}
fi
: > $C/raw/$o.frames.txt; prev=""
for i in $(seq 0 ${STEPS:-60}); do
  f=$($T capture-pane -p -J -t $s | sed 's/[[:space:]]*$//' | awk 'NF{p=1} p')
  if [ "$f" != "$prev" ]; then printf -- '--- frame %d\n%s\n' $i "$f" >> $C/raw/$o.frames.txt; prev="$f"; fi
  [ $i -lt ${STEPS:-60} ] && $T send-keys -t $s Down; sleep 0.12
done
{ echo "\$ $*"; grep -v '^--- frame' $C/raw/$o.frames.txt | sed 's/^› /  /' | awk '!seen[$0]++'; } > $C/$o.txt
