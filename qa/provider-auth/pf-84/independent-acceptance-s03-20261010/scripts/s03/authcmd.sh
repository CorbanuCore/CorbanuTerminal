#!/bin/sh
# Mock auth.command: logs the account it was asked for and prints that account's canary (or fails for unknown).
S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84s03acc
echo "$(date +%s) pid=$PPID account=${CORBANU_PROVIDER_ACCOUNT-<unset>} argv=[$*]" >> $S/proxy/authcmd.log
exec python3 -c "import json,sys;d=json.load(open('$S/proxy/canaries.json'));a=sys.argv[1] or 'default';t=[k for k,v in d.items() if v==a];print(t[0]) if t else sys.exit(3)" "${CORBANU_PROVIDER_ACCOUNT:-}"
