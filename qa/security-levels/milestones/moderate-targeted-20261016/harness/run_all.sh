#!/bin/bash
# Host driver for the Moderate targeted re-run: groups one at a time, lanes rotating.
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016
cd $W
log() { echo "$(date -u +%FT%TZ) $*" >> $W/runs/seq.log; }
log "RUN-ALL start $*"
bash h/run_seq.sh "$@"
log "RUN-ALL done"
