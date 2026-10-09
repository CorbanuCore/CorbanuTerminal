#!/bin/bash
# remote side. usage: rtx-views.sh <home-name> <tag>...: day view, then resume each tag's thread and capture /cost
S=<rtx-scratch>; cd $S; h=$1; shift
./rtx-tuiview.sh $S/homes/$h v-day && STEPS=${STEPS:-45} ./rtx-cap.sh v-day rtx-$h-day /cost; tmux -L pf60s05indep kill-session -t v-day
for t in "$@"; do
  id=$(grep -o '"thread_id":"[^"]*' logs/$t.jsonl | head -1 | cut -d'"' -f4); echo "$t $id" >> data/threads-rtx.txt
  ./rtx-tuiview.sh $S/homes/$h v-$t resume $id && STEPS=${STEPS:-45} ./rtx-cap.sh v-$t rtx-$t-conv /cost
  tmux -L pf60s05indep kill-session -t v-$t
done
