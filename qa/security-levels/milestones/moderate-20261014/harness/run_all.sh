#!/bin/bash
# Host driver for the Moderate first run: groups one at a time, lanes rotating.
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-first-20261014
cd $W
log() { echo "$(date -u +%FT%TZ) $*" >> $W/runs/seq.log; }
log "RUN-ALL start $*"
bash h/run_seq.sh G01 G02 G03 G04 G05 G06 G07 G08 G09 G10 G11 G12 G13 G14 G15 G16 G17 G18 G19 G20 G21 G22 G23 G24 G25
log "RUN-ALL done"
