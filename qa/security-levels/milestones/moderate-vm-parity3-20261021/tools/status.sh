#!/bin/bash
# utm-par3 status: last seq.log lines, the running group's tester event count, host memory, the relay's last request.
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par3-20261021
tail -3 $Q/runs/seq.log
R=$(ls -td $Q/runs/agms-* 2>/dev/null | head -1)
[ -n "$R" ] && echo "$(basename $R): events $(wc -l < $R/executor-events.jsonl 2>/dev/null) exit $(cat $R/executor-exit.txt 2>/dev/null)"
memory_pressure -Q | tail -1
tail -1 $Q/common/host/host-zai-relay.jsonl 2>/dev/null | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("relay", d.get("ts"), d.get("status"), d.get("path"), d.get("model"))' 2>/dev/null
ls /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/utm-par3.grant.md 2>/dev/null && echo GRANT-PRESENT
python3 -c "import json;print('installed', json.load(open('/Volumes/CorbanuDrive/Corbanu/.codex-work/integration-build-20260922a/INSTALL.json'))['commit'])"
