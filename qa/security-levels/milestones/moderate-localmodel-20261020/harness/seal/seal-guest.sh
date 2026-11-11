#!/bin/bash
# Run as root inside a FRESH clone (case-run.sh sends it). Recreates the sealed-guest baseline that
# the UTM "macOS 3" milestone harness (qa/security-levels/milestones/*/harness) expects, so its
# guest-side files run unchanged:
#   - coordinator account `agent` (standard user, SSH key only) and stand-in homes /Users/neo and
#     /Users/neo_1 (mode 700) that the isolation probes must fail to list;
#   - synthetic placeholders where the probes expect protected coordinator files;
#   - /Users/agent/bin/tmux plus the libevent library it links, under /opt/homebrew as on that guest;
#   - a sealed network: pf allows only SSH from the host, DHCP and HTTPS to the pinned Z.AI
#     addresses; DNS resolves nothing (api.z.ai is pinned in /etc/hosts).
# Usage: seal-guest.sh ASSET_DIR   (ASSET_DIR holds coordinator.pub, tmux, libevent_core-2.1.7.dylib,
#        dns-refuse.py)
set -euo pipefail
A="$1"
[ "$(id -u)" = 0 ] || { echo "run as root" >&2; exit 1; }
[ ! -e /Users/agent ] || { echo "not a fresh clone: /Users/agent exists" >&2; exit 1; }
HOST_IP="$(route -n get default | awk '/gateway/ {print $2}')"
ZAI_IPS="47.251.69.60 47.251.169.173"

mkuser() { # name uid home realname
  dscl . -create "/Users/$1"
  dscl . -create "/Users/$1" UniqueID "$2"
  dscl . -create "/Users/$1" PrimaryGroupID 20
  dscl . -create "/Users/$1" NFSHomeDirectory "$3"
  dscl . -create "/Users/$1" UserShell /bin/bash
  dscl . -create "/Users/$1" RealName "$4"
  dscl . -create "/Users/$1" Password '*'
  mkdir -p "$3"; chown "$1:staff" "$3"; chmod 700 "$3"
}
mkuser agent 502 /Users/agent "QA coordinator"
mkuser neo 503 /Users/neo "Stand-in owner home (probe target)"
mkuser neo2 504 /Users/neo_1 "Stand-in owner home (probe target)"
dscl . -create /Users/neo IsHidden 1
dscl . -create /Users/neo2 IsHidden 1
# Remote Login: only the image's admin (root actions) and the coordinator.
dseditgroup -o read com.apple.access_ssh > /dev/null 2>&1 || dseditgroup -o create -q com.apple.access_ssh
dseditgroup -o edit -a admin -t user com.apple.access_ssh
dseditgroup -o edit -a agent -t user com.apple.access_ssh

as_agent() { sudo -u agent -H /bin/bash -c "umask 077; $1"; }
as_agent 'mkdir -p ~/.ssh ~/bin ~/.pf83-auth ~/.corbanu ~/pf83-cases-29.Y0k1xE ~/agms/mediator'
install -o agent -g staff -m 600 "$A/coordinator.pub" /Users/agent/.ssh/authorized_keys
install -o agent -g staff -m 755 "$A/tmux" /Users/agent/bin/tmux
# Placeholders only: no real subscription login or history exists in this guest.
as_agent 'echo "{\"synthetic\": \"placeholder, not a credential\"}" > ~/.pf83-auth/auth.json; : > ~/.corbanu/pfterminal_state_5.sqlite'
for d in /Users/neo /Users/neo_1; do echo "stand-in, not real data" > "$d/README"; done

mkdir -p /opt/homebrew/bin /opt/homebrew/sbin /opt/homebrew/opt/libevent/lib
install -m 644 "$A/libevent_core-2.1.7.dylib" /opt/homebrew/opt/libevent/lib/libevent_core-2.1.7.dylib
chown -R neo2:admin /opt/homebrew
sudo -u agent /Users/agent/bin/tmux -V

# Sealed network. lo0 is not skipped, so the harness's per-user loopback rules apply.
for ip in $ZAI_IPS; do echo "$ip api.z.ai" >> /etc/hosts; done
cat > /etc/pf.anchors/corbanu.pf83 << PF
inference = "{ ${ZAI_IPS// /, } }"
# 1. inbound SSH from the host only. (flags any: the session running this script survives the load.)
pass in quick on en0 inet proto tcp from $HOST_IP to (en0) port 22 flags any keep state
pass out quick on en0 inet proto tcp from (en0) port 22 to $HOST_IP flags any keep state
pass out quick on en0 inet proto udp from any port 68 to $HOST_IP port 67 keep state
pass in quick on en0 inet proto icmp from $HOST_IP to (en0) icmp-type echoreq keep state
# 2. inference only.
pass out quick on en0 inet proto tcp from (en0) to \$inference port 443 keep state
block drop log quick on en0 all
PF
grep -q 'corbanu.pf83' /etc/pf.conf || printf '%s\n' 'anchor "corbanu.pf83"' \
  'load anchor "corbanu.pf83" from "/etc/pf.anchors/corbanu.pf83"' >> /etc/pf.conf
pfctl -nf /etc/pf.conf
pfctl -f /etc/pf.conf 2>&1 | grep -v -e ALTQ -e 'flushing of rules' -e 'present in the main' -e 'pf.conf for further' -e '^$' || true
pfctl -E 2>&1 | grep -v -e ALTQ -e '^$' || true
install -d -m 755 /var/qa && install -m 755 "$A/dns-refuse.py" /var/qa/dns-refuse.py
cat > /Library/LaunchDaemons/com.corbanu.qa.dns-refuse.plist << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.corbanu.qa.dns-refuse</string>
  <key>ProgramArguments</key><array><string>/usr/bin/python3</string><string>/var/qa/dns-refuse.py</string></array>
  <key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
</dict></plist>
PLIST
launchctl bootstrap system /Library/LaunchDaemons/com.corbanu.qa.dns-refuse.plist
for _ in $(seq 1 20); do lsof -nP -iUDP@127.0.0.1:53 > /dev/null 2>&1 && break; sleep 0.5; done
networksetup -listallnetworkservices | tail -n +2 | sed 's/^\*//' | while IFS= read -r svc; do
  networksetup -setdnsservers "$svc" 127.0.0.1 2> /dev/null || true
done
dscacheutil -flushcache; killall -HUP mDNSResponder 2> /dev/null || true

echo "sealed: host $HOST_IP; pf $(pfctl -s info 2>/dev/null | awk '/Status/ {print $2}'); rules $(pfctl -a corbanu.pf83 -sr | wc -l | tr -d ' ')"
echo "api.z.ai -> $(dscacheutil -q host -a name api.z.ai | awk '/ip_address/ {print $2}' | tr '\n' ' ')"
python3 -c 'import socket
try:
    socket.getaddrinfo("github.com", 443); print("github.com RESOLVED (seal incomplete)")
except socket.gaierror as e: print("github.com unresolvable:", e)'
stat -f '%N %Lp %Su:%Sg' /Users/* /opt/homebrew/bin /Users/agent/.pf83-auth
