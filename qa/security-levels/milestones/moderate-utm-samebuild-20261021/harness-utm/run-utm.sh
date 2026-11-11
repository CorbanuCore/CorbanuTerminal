#!/bin/bash
# utm-par2 (D93): the four groups of the five cases on UTM "macOS 3", one at a time, rotating lanes, after one gate.
# usage: run-utm.sh [GROUP...]   (default: T09 T17 T18 T21; label "utm"; replays: PAR_LABEL=utm-replay run-utm.sh T..)
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par2-20261021
cd $Q
export PAR_COMMON=${PAR_COMMON:-common/utm}
LABEL=${PAR_LABEL:-utm}
LIST="${*:-T09 T17 T18 T21}"
lanes=(agmx1 agmx2 agmx3); i=${LANE_OFFSET:-0}
python3 $Q/h/gate_ok.py $Q/$PAR_COMMON >> runs/seq.log 2>&1 || { echo "$(date -u +%FT%TZ) gate not passed; nothing run" >> runs/seq.log; exit 1; }
for G in $LIST; do
  LANE=${lanes[$((i % 3))]}; i=$((i + 1))
  [ ! -e "$Q/STOP" ] || { echo "$(date -u +%FT%TZ) STOP file present; stopping before $G" >> runs/seq.log; exit 0; }
  while :; do f=$(memory_pressure -Q | awk '/free percentage/ {gsub("%","",$NF); print $NF}'); [ "${f:-0}" -ge 25 ] && break; echo "$(date -u +%FT%TZ) paused: memory free $f%" >> runs/seq.log; sleep 60; done
  echo "$(date -u +%FT%TZ) group $G on $LANE ($LABEL; memory free $f%)" >> runs/seq.log
  ./hu/utm-run.sh group $LANE $G $LABEL
  echo "$(date -u +%FT%TZ) group $G ($LABEL) done" >> runs/seq.log
done
echo "$(date -u +%FT%TZ) ALL-DONE $LIST ($LABEL)" >> runs/seq.log
