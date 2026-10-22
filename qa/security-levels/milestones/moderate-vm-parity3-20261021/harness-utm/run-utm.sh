#!/bin/bash
# utm-par3 (D139): the six UTM groups on "macOS 3", one at a time, rotating lanes, after one gate (up/tunnel/stage/gate
# are run before this, down/verify-boot after).   usage: run-utm.sh [GROUP:LABEL...]
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par3-20261021
cd $Q
export PAR_COMMON=${PAR_COMMON:-common/utm}
LIST="${*:-T17:utm1 T18:utm1 T17:utm2 T18:utm2 T17:utm3 T18:utm3}"
lanes=(agmx1 agmx2 agmx3); i=${LANE_OFFSET:-0}
python3 $Q/h/gate_ok.py $Q/$PAR_COMMON >> runs/seq.log 2>&1 || { echo "$(date -u +%FT%TZ) gate not passed; nothing run" >> runs/seq.log; exit 1; }
for item in $LIST; do
  G=${item%%:*}; LABEL=${item#*:}
  LANE=${lanes[$((i % 3))]}; i=$((i + 1))
  [ ! -e "$Q/STOP" ] || { echo "$(date -u +%FT%TZ) STOP file present; stopping before $item" >> runs/seq.log; exit 0; }
  while :; do f=$(memory_pressure -Q | awk '/free percentage/ {gsub("%","",$NF); print $NF}'); [ "${f:-0}" -ge 25 ] && break; echo "$(date -u +%FT%TZ) paused: memory free $f%" >> runs/seq.log; sleep 60; done
  echo "$(date -u +%FT%TZ) group $G on $LANE ($LABEL; memory free $f%)" >> runs/seq.log
  ./hu/utm-run.sh group $LANE $G $LABEL
  echo "$(date -u +%FT%TZ) group $G ($LABEL) done" >> runs/seq.log
done
echo "$(date -u +%FT%TZ) ALL-DONE $LIST" >> runs/seq.log
