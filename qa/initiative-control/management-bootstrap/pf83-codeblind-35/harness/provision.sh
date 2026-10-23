#!/bin/bash
# Run as root (via admin.sh). Provisions campaign-35 code-blind executor isolation on the
# sealed PF-83 guest and records every original setting it changes under restore-state/.
echo PF83BEGIN
set -euo pipefail
S=/Users/agent/pf83-cb
ST=$S/restore-state
LANES="pf83x1 pf83x2 pf83x3"
mkdir -p "$ST"
say() { echo "[provision] $*"; }

# 0. Original state, written once.
if [ ! -f "$ST/recorded" ]; then
  stat -f '%N %Lp %Su:%Sg' /Users/neo /Users/neo_1 > "$ST/home-modes.txt"
  cp -p /etc/pf.anchors/corbanu.pf83 "$ST/corbanu.pf83.orig"
  (mount | grep -i virtiofs || true) > "$ST/virtiofs-mount.txt"
  (stat -f '%N %Lp' /private/tmp/.s.PGSQL.5432 2>/dev/null || true) > "$ST/pgsock-mode.txt"
  [ -f /private/tmp/pf83-allow-zai.sh ] && cp -p /private/tmp/pf83-allow-zai.sh "$ST/" || true
  date -u +%FT%TZ > "$ST/recorded"
fi

# 1. Executor group and accounts: no password, no admin, no SSH, hidden.
dscl . -read /Groups/pf83exec >/dev/null 2>&1 || {
  dscl . -create /Groups/pf83exec; dscl . -create /Groups/pf83exec PrimaryGroupID 601
  dscl . -create /Groups/pf83exec RealName "PF83 code-blind executors"; }
uid=602
for u in $LANES; do
  if ! dscl . -read "/Users/$u" >/dev/null 2>&1; then
    dscl . -create "/Users/$u"
    dscl . -create "/Users/$u" UserShell /bin/bash
    dscl . -create "/Users/$u" RealName "PF83 code-blind executor $u"
    dscl . -create "/Users/$u" UniqueID "$uid"
    dscl . -create "/Users/$u" PrimaryGroupID 601
    dscl . -create "/Users/$u" NFSHomeDirectory "/Users/$u"
    dscl . -create "/Users/$u" Password '*'
    dscl . -create "/Users/$u" IsHidden 1
  fi
  mkdir -p "/Users/$u"; chown "$u:pf83exec" "/Users/$u"; chmod 700 "/Users/$u"
  uid=$((uid + 1))
done
printf '%s\n' "agent ALL=(pf83x1,pf83x2,pf83x3) NOPASSWD: ALL" > /etc/sudoers.d/pf83-codeblind
chmod 440 /etc/sudoers.d/pf83-codeblind
visudo -cf /etc/sudoers.d/pf83-codeblind

# 2. Read-only package, tools and packet, root-owned.
P=/Users/agent/pf83-cases-29.Y0k1xE
mkdir -p /opt/pf83/pkg /opt/pf83/bin /opt/pf83/etc /opt/pf83/packet
chmod -R u+w /opt/pf83
cp "$P"/package/* /opt/pf83/pkg/
cp "$P/package-manifest.json" /opt/pf83/package-manifest.json
cp /Users/agent/bin/tmux /opt/pf83/bin/tmux
cp "$S"/opt/bin/* /opt/pf83/bin/
cp "$S"/opt/etc/* /opt/pf83/etc/
cp -R "$S"/opt/packet/. /opt/pf83/packet/
chown -R root:wheel /opt/pf83
find /opt/pf83 -type f -exec chmod 444 {} +
chmod 555 /opt/pf83/pkg/* /opt/pf83/bin/*
find /opt/pf83 -type d -exec chmod 555 {} +
chmod 755 /opt/pf83
( cd /opt/pf83 && shasum -a 256 package-manifest.json pkg/* bin/* etc/* ) > "$S/opt-digests.txt"
find /opt/pf83/packet -type f -exec shasum -a 256 {} + >> "$S/opt-digests.txt"

# 3. Lock other homes, the unrelated database socket and the host share for the campaign.
chmod 700 /Users/neo /Users/neo_1
[ -S /private/tmp/.s.PGSQL.5432 ] && chmod 700 /private/tmp/.s.PGSQL.5432 || true
if mount | grep -q "My Shared Files"; then umount "/Volumes/My Shared Files" || diskutil unmount force "/Volumes/My Shared Files"; fi
[ -f /private/tmp/pf83-allow-zai.sh ] && mv /private/tmp/pf83-allow-zai.sh "$ST/pf83-allow-zai.sh.moved" || true

# 4. Per-user egress: executors reach only the loopback mediator.
python3 - "$ST/corbanu.pf83.orig" /etc/pf.anchors/corbanu.pf83 <<'PY'
import sys
src, dst = sys.argv[1], sys.argv[2]
text = open(src).read()
marker = "# 1. inbound SSH from the host only."
block = """# 0. PF83 campaign-35 code-blind executors (2026-10-05): loopback inference mediator only.
pass out quick on lo0 proto tcp from any to 127.0.0.1 port 18443 user { pf83x1, pf83x2, pf83x3 } keep state
block drop out log quick proto { tcp, udp } user { pf83x1, pf83x2, pf83x3 }

"""
assert marker in text and "campaign-35" not in text
open(dst, "w").write(text.replace(marker, block + marker, 1))
PY
pfctl -a corbanu.pf83 -f /etc/pf.anchors/corbanu.pf83 2>&1 | grep -v "ALTQ" || true
pfctl -a corbanu.pf83 -sr

# 5. Report.
say "users: $(for u in $LANES; do id "$u"; done | tr '\n' ';')"
say "staff member check: $(dseditgroup -o checkmember -m pf83x1 staff 2>&1 || true)"
stat -f '%N %Lp %Su:%Sg' /Users/neo /Users/neo_1 /opt/pf83 /opt/pf83/pkg /opt/pf83/pkg/corbanu
mount | grep -ci virtiofs || true
cat "$S/opt-digests.txt"
say done
