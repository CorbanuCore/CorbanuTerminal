#!/bin/bash
# tart-par1: one fresh Tart clone per group: up, seal, tunnel, stage, gate, group, down. Sequential, one VM at a time.
# usage: run-groups.sh [GROUP...]   (default: every group in tools/groups.json not yet marked done in runs/seq.log)
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-par1-20261020
cd $Q
GROUPS_ALL=$(python3 -c "import json;print(' '.join(json.load(open('$Q/tools/groups.json'))))")
LIST="${*:-$GROUPS_ALL}"
lanes=(agmx1 agmx2 agmx3); i=0
for G in $LIST; do
  LANE=${lanes[$((i % 3))]}; i=$((i + 1))
  if [ -z "$*" ] && grep -q "^.* group $G clone deleted (done)$" runs/seq.log 2>/dev/null; then continue; fi
  [ ! -e "$Q/STOP" ] || { echo "$(date -u +%FT%TZ) STOP file present; stopping before $G" >> runs/seq.log; exit 0; }
  export PAR_COMMON=common/$G${PAR_LABEL:+-$PAR_LABEL}-$(date -u +%Y%m%dT%H%M%SZ)   # unique per clone (review r2)
  while :; do f=$(memory_pressure -Q | awk '/free percentage/ {gsub("%","",$NF); print $NF}'); [ "${f:-0}" -ge 20 ] && break; echo "$(date -u +%FT%TZ) paused: memory free $f%" >> runs/seq.log; sleep 60; done
  echo "$(date -u +%FT%TZ) group $G on $LANE (clone; memory free $f%)" >> runs/seq.log
  ./h/tart-run.sh up > /dev/null 2>&1 || { echo "$(date -u +%FT%TZ) up failed for $G" >> runs/seq.log; ./h/tart-run.sh down > /dev/null 2>&1; exit 1; }
  ./h/tart-run.sh seal > /dev/null 2>&1
  ./h/tart-run.sh tunnel > /dev/null 2>&1
  ./h/tart-run.sh stage > /dev/null 2>&1 || { echo "$(date -u +%FT%TZ) stage failed for $G" >> runs/seq.log; ./h/tart-run.sh down > /dev/null 2>&1; exit 1; }
  ./h/tart-run.sh gate > $Q/$PAR_COMMON/gate-out.txt 2>&1
  if python3 $Q/h/gate_ok.py $Q/$PAR_COMMON >> runs/seq.log 2>&1; then
    echo "$(date -u +%FT%TZ) gate ok for $G ($PAR_COMMON)" >> runs/seq.log
    ./h/tart-run.sh group $LANE $G ${PAR_LABEL:-}
  else
    echo "$(date -u +%FT%TZ) gate FAILED for $G; group blocked" >> runs/seq.log
  fi
  ./h/tart-run.sh down > /dev/null 2>&1
  echo "$(date -u +%FT%TZ) group $G${PAR_LABEL:+ ($PAR_LABEL)} clone deleted (done)" >> runs/seq.log
done
echo "$(date -u +%FT%TZ) ALL-DONE" >> runs/seq.log
