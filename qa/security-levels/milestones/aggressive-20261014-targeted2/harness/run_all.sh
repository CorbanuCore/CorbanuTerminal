#!/bin/bash
# Host driver for the targeted re-run: groups one at a time, lanes rotating.
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted2-20261014
cd $W
log() { echo "$(date -u +%FT%TZ) $*" >> $W/runs/seq.log; }
log "RUN-ALL start"
bash h/run_seq.sh Q01 Q02 Q03 Q04 Q05 Q06 Q07 Q08 Q09 Q10 Q11 Q12 Q13 Q14
log "RUN-ALL done"
