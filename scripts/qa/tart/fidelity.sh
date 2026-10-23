#!/bin/bash
# Check that a fresh clone behaves like a real Mac for the security cases: SIP, Seatbelt, keychain
# services and TCC. Clones the base, runs the checks over SSH, deletes the clone.
#
#   TART_QA_WORK=... CORBANU_BIN=/path/to/signed/corbanu scripts/qa/tart/fidelity.sh
#
# Keychain checks use a throwaway keychain file with an explicit path only; the login keychain is
# never queried. No credential enters the guest.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=SCRIPTDIR/lib.sh
. "$here/lib.sh"
: "${CORBANU_BIN:?set CORBANU_BIN to the signed corbanu binary (cycle.sh shows how it resolves it)}"

name="qa-fidelity-$(date -u +%Y%m%dT%H%M%SZ)"
out="$LOGS/$name.txt"
"$TART" clone "$TART_QA_BASE" "$name"
trap 'stop_vm "$name" 10; "$TART" delete "$name"' EXIT
start_vm "$name"
ip="$(wait_ip "$name" 180)"
wait_ssh "$ip" 180 || die "no SSH on $ip"
gssh "$ip" 'mkdir -p ~/qa/bin && cat > ~/qa/bin/corbanu && chmod 755 ~/qa/bin/corbanu' < "$CORBANU_BIN"

gssh "$ip" 'bash -s' > "$out" 2>&1 << 'GUEST' || true
set -u
check() { printf '\n## %s\n$ %s\n' "$1" "$2"; bash -c "$2" 2>&1; echo "rc=$?"; }
echo "# guest: $(sw_vers -productVersion) ($(sw_vers -buildVersion)) $(uname -m), $(sysctl -n hw.ncpu) CPUs, $(($(sysctl -n hw.memsize) / 1073741824)) GB"
check "SIP" "csrutil status"
check "Authenticated root / sealed system volume" "csrutil authenticated-root status; diskutil info / | grep -E 'Sealed|Snapshot:'"
check "Gatekeeper" "spctl --status"
check "Copied binary signature (no quarantine after SSH copy)" "codesign --verify --strict -v ~/qa/bin/corbanu; xattr ~/qa/bin/corbanu"

d=/private/tmp/qa-sbx; rm -rf "$d"; mkdir -p "$d/denied" "$d/allowed"
p="(version 1)(allow default)(deny file-write* (subpath \"$d/denied\"))(deny file-read* (literal \"$d/allowed/secret\"))"
echo synthetic > "$d/allowed/secret"
check "Seatbelt: denied write (expect Operation not permitted)" "sandbox-exec -p '$p' /usr/bin/touch $d/denied/x"
check "Seatbelt: allowed write (positive control)" "sandbox-exec -p '$p' /usr/bin/touch $d/allowed/x && ls $d/allowed/x"
check "Seatbelt: denied read (expect Operation not permitted)" "sandbox-exec -p '$p' /bin/cat $d/allowed/secret"
check "Seatbelt: sandbox denial logged by the kernel" "log show --last 2m --style compact --predicate 'sender == \"Sandbox\"' 2>/dev/null | grep -c deny"
check "Corbanu sandbox (default read-only policy): write outside the cwd (expect denied)" "cd $d && ~/qa/bin/corbanu sandbox -- /usr/bin/touch /Users/admin/qa-outside-write; ls /Users/admin/qa-outside-write"
check "Corbanu sandbox: network (expect blocked)" "cd $d && ~/qa/bin/corbanu sandbox -- /usr/bin/curl -sS -m 5 -o /dev/null https://example.com"
check "Corbanu sandbox: read inside the cwd (positive control)" "cd $d && ~/qa/bin/corbanu sandbox -- /bin/cat $d/allowed/secret"

k=/tmp/qa-kc/throwaway.keychain-db; rm -rf /tmp/qa-kc; mkdir -p -m 700 /tmp/qa-kc
KP="$(openssl rand -hex 16)"; export KP  # throwaway; the log shows only $KP
check "Keychain services: create + unlock a throwaway keychain (explicit path)" "security create-keychain -p \"\$KP\" $k && security unlock-keychain -p \"\$KP\" $k && security show-keychain-info $k"
check "Keychain services: store and read back a synthetic item" "security add-generic-password -a qa -s qa-tart-synthetic -w synthetic-not-a-secret $k && security find-generic-password -a qa -s qa-tart-synthetic -w $k"
check "Keychain services: delete the throwaway keychain" "security delete-keychain $k; ls /tmp/qa-kc"
check "Login keychain file present (not opened)" "ls -l ~/Library/Keychains/ | sed -n '1,5p'"

check "TCC daemons" "pgrep -lx tccd"
check "TCC: system TCC.db readable as root over SSH? (authorization denied = no Full Disk Access for SSH)" "sudo /usr/bin/sqlite3 '/Library/Application Support/com.apple.TCC/TCC.db' 'select service, client, auth_value from access' 2>&1 | head -8"
check "TCC: own protected folders over SSH (expect Operation not permitted without FDA)" "ls ~/Desktop ~/Documents ~/Downloads ~/Library/Mail 2>&1"
sudo dscl . -create /Users/qatcc >/dev/null 2>&1 && sudo dscl . -create /Users/qatcc UniqueID 650 && sudo dscl . -create /Users/qatcc PrimaryGroupID 20 \
  && sudo dscl . -create /Users/qatcc NFSHomeDirectory /Users/qatcc && sudo dscl . -create /Users/qatcc UserShell /bin/bash \
  && sudo mkdir -p /Users/qatcc/Documents && sudo chown -R qatcc:staff /Users/qatcc
check "TCC: another user's protected folder via sudo (unprivileged user)" "cd /tmp && sudo -u qatcc ls /Users/qatcc/Documents && echo listed"
check "TCC: user without consent reading admin's Desktop" "cd /tmp && sudo -u qatcc ls /Users/admin/Desktop"
check "TCC: Apple Events to Finder from SSH (expect -1743 not authorized, or a hang without a GUI session)" "perl -e 'alarm 20; exec @ARGV' osascript -e 'tell application \"Finder\" to get name of startup disk' || echo exit=\$?"
check "GUI session" "stat -f 'console owner: %Su' /dev/console; who"
sudo dscl . -delete /Users/qatcc; sudo rm -rf /Users/qatcc
check "Host shares (expect none)" "mount | grep -ci virtiofs; ls '/Volumes/My Shared Files' 2>&1"
# Other guests on the shared NAT bridge are deliberately not probed (they may be live UTM runs).
check "Network: guest address and the host's bridge (host SSH reachable?)" "ipconfig getifaddr en0; nc -z -G 3 192.168.64.1 22 && echo host-ssh-open || echo host-ssh-closed"
GUEST
log "fidelity results: $out"
cat "$out"
