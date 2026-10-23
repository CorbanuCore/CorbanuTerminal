#!/bin/bash
# Host driver for the third run: groups one at a time, lanes rotating; G21 gets the /etc/codex fixture around it.
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-third-20261013
cd $W
log() { echo "$(date -u +%FT%TZ) $*" >> $W/runs/seq.log; }
log "RUN-ALL start $*"
bash h/run_seq.sh G01 G02 G03 G04 G05 G06 G07 G08 G09 G10 G11 G12 G13 G14 G15 G16 G17 G18 G19 G20
NEOPW="$(corbanu vault auth-helper neo-vm)" h/admin.sh h/etc-codex-on.sh > common/g21-fixture-on.txt 2>&1
bash h/run_seq.sh G21
NEOPW="$(corbanu vault auth-helper neo-vm)" h/admin.sh h/etc-codex-off.sh > common/g21-fixture-off.txt 2>&1
bash h/run_seq.sh G22 G23 G24 G25 G26 G27 G28 G29 G30 G31 G32
log "RUN-ALL done"
