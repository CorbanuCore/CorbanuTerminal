#!/bin/bash
# usage: NEOPW="$(corbanu vault auth-helper neo-vm)" admin.sh local-script.sh
set -euo pipefail
S=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016/h/vmssh.sh
G=agent@192.168.64.3
name=agadmin-$(date +%s)-$$.sh
$S $G "mkdir -p -m 700 ~/ag-admin && cat > ~/ag-admin/$name && chmod 755 ~/ag-admin ~/ag-admin/$name" < "$1"
$S $G "cat > ~/ag-admin/su.exp" < /Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016/ref35/harness/su.exp
printf '%s\n' "$NEOPW" | $S $G "/usr/bin/expect -f ~/ag-admin/su.exp /Users/agent/ag-admin/$name"
