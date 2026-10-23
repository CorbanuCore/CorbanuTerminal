#!/bin/bash
# usage: views.sh <home-name> <tag>...: for each exec tag, resume its thread in a view-only TUI and capture /cost
S=<scratch>; cd $S; h=$1; shift
for t in "$@"; do
  id=$(grep -o '"thread_id":"[^"]*' logs/$t.jsonl | head -1 | cut -d'"' -f4); echo "$t $id" >> data/threads-mac.txt
  tools/tuiview.sh $S/homes/$h v-$t resume $id && STEPS=${STEPS:-45} tools/cap.sh v-$t mac-$t-conv /cost
  tmux -L pf60s05indep kill-session -t v-$t
done
