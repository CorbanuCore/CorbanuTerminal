#!/bin/bash
# utm-par2 (D94/D95): one fresh Tart clone per group: up, seal, tunnel, stage, gate, group, down. One VM at a time.
# usage: run-cur.sh [GROUP...]   (default: T09 T17; label "cur")
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par2-20261021
cd $Q
LABEL=${PAR_LABEL:-cur}
LIST="${*:-T09 T17}"
lanes=(agmx1 agmx2 agmx3); i=0
/Applications/UTM.app/Contents/MacOS/utmctl list | grep -q 'started' && { echo "$(date -u +%FT%TZ) a UTM VM is running; not starting Tart" >> runs/seq.log; exit 1; }
for G in $LIST; do
  LANE=${lanes[$((i % 3))]}; i=$((i + 1))
  [ ! -e "$Q/STOP" ] || { echo "$(date -u +%FT%TZ) STOP file present; stopping before $G" >> runs/seq.log; exit 0; }
  export PAR_COMMON=common/$G-$LABEL-$(date -u +%Y%m%dT%H%M%SZ)
  while :; do f=$(memory_pressure -Q | awk '/free percentage/ {gsub("%","",$NF); print $NF}'); [ "${f:-0}" -ge 25 ] && break; echo "$(date -u +%FT%TZ) paused: memory free $f%" >> runs/seq.log; sleep 60; done
  echo "$(date -u +%FT%TZ) group $G on $LANE (Tart clone, $LABEL; memory free $f%)" >> runs/seq.log
  ./ht/tart-run.sh up > /dev/null 2>&1 || { echo "$(date -u +%FT%TZ) up failed for $G" >> runs/seq.log; ./ht/tart-run.sh down > /dev/null 2>&1; exit 1; }
  ./ht/tart-run.sh seal > /dev/null 2>&1
  ./ht/tart-run.sh tunnel > /dev/null 2>&1
  ./ht/tart-run.sh stage > /dev/null 2>&1 || { echo "$(date -u +%FT%TZ) stage failed for $G" >> runs/seq.log; ./ht/tart-run.sh down > /dev/null 2>&1; exit 1; }
  ./ht/tart-run.sh gate > $Q/$PAR_COMMON/gate-out.txt 2>&1
  if python3 $Q/ht/gate_ok.py $Q/$PAR_COMMON >> runs/seq.log 2>&1; then
    echo "$(date -u +%FT%TZ) gate ok for $G ($PAR_COMMON)" >> runs/seq.log
    ./ht/tart-run.sh group $LANE $G $LABEL
  else
    echo "$(date -u +%FT%TZ) gate FAILED for $G; group blocked" >> runs/seq.log
  fi
  ./ht/tart-run.sh down > /dev/null 2>&1
  echo "$(date -u +%FT%TZ) group $G ($LABEL) clone deleted (done)" >> runs/seq.log
done
echo "$(date -u +%FT%TZ) ALL-DONE $LIST ($LABEL)" >> runs/seq.log
