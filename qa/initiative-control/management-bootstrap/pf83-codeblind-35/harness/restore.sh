#!/bin/bash
# Run as root (via admin.sh) after the campaign. Reverses provision.sh from restore-state/.
echo PF83BEGIN
set -uo pipefail
S=/Users/agent/pf83-cb
ST=$S/restore-state
LANES="pf83x1 pf83x2 pf83x3"
say() { echo "[restore] $*"; }

# 1. Stop campaign processes (executors, pf counter logger, mediator, canary session).
for u in $LANES; do pkill -9 -u "$u" 2>/dev/null; done
pkill -f "pfctl -a corbanu.pf83 -vsr" 2>/dev/null
pkill -f "while true; do date -u +%FT%TZ; pfctl" 2>/dev/null
pkill -u agent -f pf83_mediator.py 2>/dev/null
sudo -u agent /Users/agent/bin/tmux -S /private/tmp/pf83-canary-ipc/tmux.sock kill-server 2>/dev/null
sleep 2

# 2. Network policy: original anchor, then a main reload so 'set skip on lo0' is back.
cp -p "$ST/corbanu.pf83.orig" /etc/pf.anchors/corbanu.pf83
pfctl -f /etc/pf.conf 2>&1 | grep -v -e ALTQ -e "flushing of rules" -e "present in the main" -e "pf.conf for further" -e '^$'
say "lo0: $(pfctl -sI -v 2>/dev/null | grep lo0)"
say "anchor matches original: $(cmp -s "$ST/corbanu.pf83.orig" /etc/pf.anchors/corbanu.pf83 && echo yes || echo NO)"
pfctl -a corbanu.pf83 -sr 2>/dev/null | grep -c "user =" | sed 's/^/[restore] per-user rules left: /'

# 3. Accounts, sudoers, package tree, per-uid temp files.
for uid in 602 603 604; do find /private/var/folders /private/tmp /Users/Shared -xdev -uid "$uid" -delete 2>/dev/null; done
for u in $LANES; do dscl . -delete "/Users/$u" 2>/dev/null; find "/Users/$u" -delete 2>/dev/null; done
dscl . -delete /Groups/pf83exec 2>/dev/null
find /etc/sudoers.d/pf83-codeblind -delete 2>/dev/null
chmod -R u+w /opt/pf83 2>/dev/null; find /opt/pf83 -delete 2>/dev/null
say "accounts left: $(dscl . list /Users | grep -c '^pf83x')  group: $(dscl . -read /Groups/pf83exec >/dev/null 2>&1 && echo present || echo absent)"
say "sudoers.d: $(ls /etc/sudoers.d | tr '\n' ' ')  /opt/pf83: $(test -e /opt/pf83 && echo present || echo absent)"

# 4. Original modes, socket, share, moved file.
while read -r path mode _; do chmod "$mode" "$path"; done < "$ST/home-modes.txt"
if [ -s "$ST/pgsock-mode.txt" ] && [ -S /private/tmp/.s.PGSQL.5432 ]; then
  read -r path mode < "$ST/pgsock-mode.txt"; chmod "$mode" "$path"; fi
stat -f '[restore] %N %Lp' /Users/neo /Users/neo_1 /private/tmp/.s.PGSQL.5432
if [ -s "$ST/virtiofs-mount.txt" ] && ! mount | grep -qi virtiofs; then
  mkdir -p "/Volumes/My Shared Files"
  mount_virtiofs share "/Volumes/My Shared Files" 2>&1 | sed 's/^/[restore] mount_virtiofs: /'
fi
say "virtiofs mounted: $(mount | grep -ci virtiofs)"
[ -f "$ST/pf83-allow-zai.sh.moved" ] && mv "$ST/pf83-allow-zai.sh.moved" /private/tmp/pf83-allow-zai.sh
say "pf83-allow-zai.sh restored: $(test -f /private/tmp/pf83-allow-zai.sh && echo yes || echo no)"

# 5. Canaries and campaign files (logs were copied to the host first).
find /Users/agent/pf83-canary /private/tmp/pf83-canary-ipc /Users/Shared/pf83-canary-ipc.pid -delete 2>/dev/null
find "$S" -delete 2>/dev/null
find /Users/agent/pf83-admin -type f ! -name "$(basename "$0")" -delete 2>/dev/null
say "campaign dirs left: $(ls -d /Users/agent/pf83-cb /Users/agent/pf83-canary /private/tmp/pf83-canary-ipc 2>/dev/null | wc -l)"
say "processes for removed uids: $(ps -axo uid= | awk '$1>=602 && $1<=604' | wc -l)"
say done
