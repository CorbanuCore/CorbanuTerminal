#!/bin/bash
# Run as root (via admin.sh) after the campaign. Reverses provision.sh from restore-state/.
echo PF83BEGIN
set -uo pipefail
S=/Users/agent/agms
ST=$S/restore-state
LANES="agmx0 agmx1 agmx2 agmx3"
say() { echo "[restore] $*"; }

# 1. Stop campaign processes (executors, guard loop, mediator, canary session).
for u in $LANES; do pkill -9 -u "$u" 2>/dev/null; done
pkill -u agent -f "SecurityAgent >/dev/null" 2>/dev/null
pkill -u agent -f acc_mediator.py 2>/dev/null
pkill -u agent -f acc_standin.py 2>/dev/null
pkill -u agent -f chatgpt_standin.py 2>/dev/null
sudo -u agent /Users/agent/bin/tmux -S /private/tmp/agms-canary-ipc/tmux.sock kill-server 2>/dev/null
sudo -u agent /Users/agent/bin/tmux -S /Users/agent/agms/coord.sock kill-server 2>/dev/null
sleep 2

# 2. Network policy: original anchor, then a main reload so 'set skip on lo0' is back.
cp -p "$ST/corbanu.pf83.orig" /etc/pf.anchors/corbanu.pf83
pfctl -f /etc/pf.conf 2>&1 | grep -v -e ALTQ -e "flushing of rules" -e "present in the main" -e "pf.conf for further" -e '^$'
say "lo0: $(pfctl -sI -v 2>/dev/null | grep lo0)"
say "anchor matches original: $(cmp -s "$ST/corbanu.pf83.orig" /etc/pf.anchors/corbanu.pf83 && echo yes || echo NO)"
say "per-user rules left: $(pfctl -a corbanu.pf83 -sr 2>/dev/null | grep -c 'user =')"

# 3. Accounts, sudoers, package tree, per-uid temp files.
for uid in 610 611 612 613; do find /private/var/folders /private/tmp /Users/Shared -xdev -uid "$uid" -delete 2>/dev/null; done
for u in $LANES; do dscl . -delete "/Users/$u" 2>/dev/null; find "/Users/$u" -delete 2>/dev/null; done
dscl . -delete /Groups/agmsexec 2>/dev/null
find /etc/sudoers.d/agms-codeblind -delete 2>/dev/null
chmod -R u+w /opt/agms 2>/dev/null; find /opt/agms -delete 2>/dev/null
say "accounts left: $(dscl . list /Users | grep -c '^agmx')  group: $(dscl . -read /Groups/agmsexec >/dev/null 2>&1 && echo present || echo absent)"
say "sudoers.d: $(ls /etc/sudoers.d | tr '\n' ' ')  /opt/agms: $(test -e /opt/agms && echo present || echo absent)"

# 4. Original modes, socket, share, system config layer.
while read -r path mode _; do chmod "$mode" "$path"; done < "$ST/modes.txt"
if [ -s "$ST/pgsock-mode.txt" ] && [ -S /private/tmp/.s.PGSQL.5432 ]; then
  read -r path mode < "$ST/pgsock-mode.txt"; chmod "$mode" "$path"; fi
stat -f '[restore] %N %Lp' /Users/neo /Users/neo_1 /opt/homebrew /opt/homebrew/bin /opt/homebrew/sbin
if [ -s "$ST/virtiofs-mount.txt" ] && ! mount | grep -qi virtiofs; then
  mkdir -p "/Volumes/My Shared Files"
  mount_virtiofs share "/Volumes/My Shared Files" 2>&1 | sed 's/^/[restore] mount_virtiofs: /'
fi
say "virtiofs mounted: $(mount | grep -ci virtiofs)"
if [ -f "$ST/etc-codex/config.toml" ] && [ ! -e /etc/codex/config.toml ]; then
  mkdir -p /etc/codex; cp -p "$ST/etc-codex/config.toml" /etc/codex/config.toml; fi
say "/etc/codex/config.toml restored: $(test -f /etc/codex/config.toml && echo yes || echo no)"
cp -p "$ST/hosts.orig" /etc/hosts
say "/etc/hosts restored to the original: $(cmp -s "$ST/hosts.orig" /etc/hosts && echo yes || echo NO)"

# 5. Canaries and campaign files (logs were copied to the host first).
find /Users/agent/agms-canary /private/tmp/agms-canary-ipc /Users/Shared/agms-canary-ipc.pid -delete 2>/dev/null
find "$S" -delete 2>/dev/null
# This campaign's transport keys (the Z.AI key file predates the campaign and stays).
find /Users/agent/.pf83-auth/openai_api_key /Users/agent/.pf83-auth/kimi_api_key -delete 2>/dev/null
say "campaign key files left: $(ls /Users/agent/.pf83-auth | grep -c -E 'openai_api_key|kimi_api_key')"
find /Users/agent/ag-admin -type f ! -name "$(basename "$0")" -delete 2>/dev/null
say "campaign dirs left: $(ls -d /Users/agent/agms /Users/agent/agms-canary /private/tmp/agms-canary-ipc 2>/dev/null | wc -l)"
say "processes for removed uids: $(ps -axo uid= | awk '$1>=610 && $1<=613' | wc -l)"
say "guest clock (left at host UTC): $(date -u +%FT%TZ)"
say done
