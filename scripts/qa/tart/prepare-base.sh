#!/bin/bash
# One-time: turn a public cirruslabs macOS image into the local QA base image.
#
#   TART_QA_WORK=/path/with/space scripts/qa/tart/prepare-base.sh [OCI_IMAGE]
#
# The base gets an SSH key (password SSH off), a random admin password nobody keeps (sudo stays
# key-only through the image's passwordless sudo), no auto-login (the guest waits at the login
# window like the UTM guest), Gatekeeper on, the Command Line Tools (python3, git) and the CPU/memory of UTM "macOS 3" (8 CPUs, 16 GB). No host share is ever
# configured. Clones of this base are the per-run machines (see cycle.sh).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=SCRIPTDIR/lib.sh
. "$here/lib.sh"

IMAGE="${1:-ghcr.io/cirruslabs/macos-tahoe-vanilla:26.6.2}"
CPUS="${TART_QA_CPUS:-8}"
MEMORY_MB="${TART_QA_MEMORY_MB:-16384}"
PREP="$TART_QA_BASE-prep"
FDA="${TART_QA_SSH_FDA:-on}"
case "$FDA" in on | off) ;; *) die "TART_QA_SSH_FDA must be on or off" ;; esac

"$TART" list --source local --format json | grep -q "\"Name\" *: *\"$TART_QA_BASE\"" &&
  die "$TART_QA_BASE already exists; delete it first to rebuild"
mkdir -p "$(dirname "$TART_QA_KEY")" && chmod 700 "$(dirname "$TART_QA_KEY")"
[ -f "$TART_QA_KEY" ] || ssh-keygen -q -t ed25519 -N '' -C tart-qa -f "$TART_QA_KEY"

log "pulling $IMAGE (cached after the first pull)"
"$TART" pull "$IMAGE"
"$TART" clone "$IMAGE" "$PREP"
# On failure, stop and delete the prep VM: it may still accept the public default login.
trap 'stop_vm "$PREP" 10; "$TART" delete "$PREP" 2> /dev/null || true' EXIT
"$TART" set "$PREP" --cpu "$CPUS" --memory "$MEMORY_MB"
start_vm "$PREP"
ip="$(wait_ip "$PREP" 300)"
log "prep guest at $ip"

# Pin the host key (first contact, host-only NAT network) under the alias every clone uses.
for _ in $(seq 1 120); do
  key="$(ssh-keyscan -t ed25519 "$ip" 2> /dev/null | awk '$1 !~ /^#/ && NF >= 3 {print $2, $3}')" && [ -n "$key" ] && break
  sleep 2
done
[ -n "${key:-}" ] || die "no SSH host key from $ip"
echo "tart-qa-base $key" > "$TART_QA_KNOWN_HOSTS"
ssh-keygen -l -f "$TART_QA_KNOWN_HOSTS" >&2

# Install the key with the image's documented default login (admin/admin), once.
/usr/bin/expect -f - "$ip" "$(cat "$TART_QA_KEY.pub")" "$TART_QA_KNOWN_HOSTS" << 'EXP'
set ip [lindex $argv 0]; set pub [lindex $argv 1]; set kh [lindex $argv 2]
set timeout 60
spawn ssh -o UserKnownHostsFile=$kh -o StrictHostKeyChecking=yes -o HostKeyAlias=tart-qa-base \
  -o PubkeyAuthentication=no -o PreferredAuthentications=password,keyboard-interactive admin@$ip \
  "umask 077; mkdir -p ~/.ssh; printf '%s\\n' '$pub' >> ~/.ssh/authorized_keys; echo KEY-INSTALLED"
expect -re "(?i)password:" { send "admin\r" }
expect "KEY-INSTALLED" {} timeout { exit 1 }
expect eof
EXP

gssh "$ip" "TART_QA_SSH_FDA=$FDA bash -s" << 'GUEST'
set -euo pipefail
sudo -n true || { echo "admin has no passwordless sudo" >&2; exit 1; }
# Key-only SSH (the first matching sshd option wins, so this file sorts first).
printf '%s\n' 'PasswordAuthentication no' 'KbdInteractiveAuthentication no' 'PermitRootLogin no' |
  sudo tee /etc/ssh/sshd_config.d/000-corbanu-qa.conf > /dev/null
# Replace the public default password with a random one that is never stored or printed
# (admin holds the SecureToken, so the old password is required).
dscl . -passwd /Users/admin admin "$(openssl rand -base64 30)"
# No auto-login: the guest waits at the login window, as UTM "macOS 3" does.
sudo defaults delete /Library/Preferences/com.apple.loginwindow autoLoginUser 2> /dev/null || true
sudo rm -f /etc/kcpassword
sudo softwareupdate --schedule off > /dev/null 2>&1 || true
# Gatekeeper: the public image turns it off; macOS (and a real Mac) has it on.
sudo spctl --global-enable
# SSH sessions keep Full Disk Access ("Allow full disk access for remote users", on by default with
# Remote Login, and evidently on in UTM "macOS 3", whose harness deletes accounts over SSH). With
# TART_QA_SSH_FDA=off, TCC also applies to everything started over SSH (for TCC-specific checks).
[ "${TART_QA_SSH_FDA:-on}" = on ] || sudo tccutil reset SystemPolicyAllFiles > /dev/null
spctl --status
# Command Line Tools: the harness needs python3 and git (the vanilla image has only the stubs).
if ! xcode-select -p > /dev/null 2>&1; then
  flag=/tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress
  sudo touch "$flag"
  label="$(softwareupdate -l 2> /dev/null | sed -n 's/^\* Label: \(Command Line Tools for Xcode.*\)$/\1/p' | sort -V | tail -1)"
  [ -n "$label" ] || { echo "no Command Line Tools update offered" >&2; exit 1; }
  echo "installing $label"
  sudo softwareupdate -i "$label" > /dev/null
  sudo rm -f "$flag"
fi
/usr/bin/python3 --version; /usr/bin/git --version
sw_vers; csrutil status; uname -m
GUEST

log "shutting the prep guest down"
gssh "$ip" 'sudo shutdown -h now' > /dev/null 2>&1 || true
# Let the guest finish its own shutdown (about 6 s) so every change reaches the disk.
for _ in $(seq 1 240); do kill -0 "$(cat "$LOGS/$PREP.pid")" 2> /dev/null || break; sleep 0.5; done
stop_vm "$PREP" 120
trap - EXIT
"$TART" rename "$PREP" "$TART_QA_BASE"
"$TART" get "$TART_QA_BASE" >&2
log "base ready: $TART_QA_BASE"
