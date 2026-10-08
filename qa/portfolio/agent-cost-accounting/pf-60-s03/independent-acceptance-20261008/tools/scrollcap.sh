#!/bin/bash
# usage: scrollcap.sh <session> <steps> [header-regex]: scroll popup with Down; print union of popup lines (from header) in order
T="tmux -L pf60indep"; s=$1; n=${2:-60}; h=${3:-'^(› |  )(Cost —|Recorded request|Request$|Descendant|Root|Attempt|Day |Hour |ISO week|Week|Month|Calendar|Bucket|Unknown|Z\.AI|zai|Other)'}
{ for i in $(seq 0 $n); do $T capture-pane -p -J -t $s | awk -v h="$h" '$0 ~ h && !f {f=1} f' ; [ $i -lt $n ] && $T send-keys -t $s Down; sleep 0.15; done; } | sed "s/^› /  /; s/[[:space:]]*$//" | grep -v '^\s*$' | awk '!seen[$0]++'
