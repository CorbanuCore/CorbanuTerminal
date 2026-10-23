#!/bin/bash
# Host: isolation gate before any executor (capture lane agmx0): probes, smoke turn, coordinator egress, one free
# stand-in positive control and the relay's view of it. Output -> common/gate/
W2=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016
S=$W2/h/vmssh.sh; G=agent@192.168.64.3
O=$W2/common/gate; mkdir -p $O
R=gate-agmx0-$(date -u +%Y%m%dT%H%M%SZ)
$S -n $G 'for h in github.com raw.githubusercontent.com codeload.github.com example.com; do curl -sS -m 8 -o /dev/null https://$h 2>&1; echo "rc=$? $h"; done' > $O/agent-egress.txt 2>&1
$S -n $G "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch probe $R" > $O/$R.summary 2> $O/$R.stderr
$S -n $G "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch smoke $R" > $O/smoke-$R.jsonl 2> $O/smoke-$R.stderr
cat $O/$R.summary; grep -o 'AGMS-SMOKE-OK' $O/smoke-$R.jsonl | head -1
$S -n $G 'tail -3 ~/agms/mediator/mediator.jsonl' > $O/mediator-after-gate.jsonl
