#!/bin/bash
# usage: run_seq.sh GROUP... ; runs groups one at a time, rotating lanes agmx1..3
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-third-20261013
i=0; lanes=(agmx1 agmx2 agmx3)
for g in "$@"; do
  lane=${lanes[$((i % 3))]}; i=$((i+1))
  $W/h/run_group.sh $lane $g >> $W/runs/seq.log 2>&1
done
echo "SEQ-DONE $*" >> $W/runs/seq.log
