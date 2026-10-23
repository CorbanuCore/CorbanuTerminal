#!/bin/bash
# usage: NEOPW="$(corbanu vault auth-helper neo-vm)" admin.sh local-script.sh
set -euo pipefail
S=/tmp/pf83ssh.sh
G=agent@192.168.64.3
name=pf83admin-$(date +%s)-$$.sh
$S $G "mkdir -p -m 700 ~/pf83-admin && cat > ~/pf83-admin/$name && chmod 755 ~/pf83-admin ~/pf83-admin/$name" < "$1"
$S $G "test -f ~/pf83-admin/su.exp" || $S $G "cat > ~/pf83-admin/su.exp" < /tmp/pf83cb/su.exp
# script lives in agent's home (700); root can read it.
printf '%s\n' "$NEOPW" | $S $G "/usr/bin/expect -f ~/pf83-admin/su.exp /Users/agent/pf83-admin/$name"
