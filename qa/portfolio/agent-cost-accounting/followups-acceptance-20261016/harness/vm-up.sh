#!/bin/bash
# Host: take the macOS 3 lock (only if free), start "macOS 3" (never macOS 2), wait for SSH. Output -> common/vm-start.txt
set -uo pipefail
W2=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016
LOCK=/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/vm-macos3.lock
U=/Applications/UTM.app/Contents/MacOS/utmctl
mkdir -p $W2/common
if [ -e "$LOCK" ]; then echo "LOCK HELD: $(cat $LOCK)"; exit 3; fi
[ -d /Volumes/FASTDRIVE2/z23 ] || { echo "shared folder /Volumes/FASTDRIVE2/z23 missing: stop and file a request"; exit 4; }
printf 'holder: pf60-s10 (PF-60-S10 isolated code-blind acceptance)\ntaken: %s\nvm: macOS 3 only\n' "$(date -u +%FT%TZ)" > "$LOCK"
{
  date -u +%FT%TZ; $U list
  $U start "macOS 3"; echo "start-rc=$?"
  for i in $(seq 1 60); do
    if $W2/h/vmssh.sh -n agent@192.168.64.3 true 2>/dev/null; then echo "ssh open $(date -u +%FT%TZ)"; break; fi
    sleep 5
  done
  $W2/h/vmssh.sh -n agent@192.168.64.3 'echo up; date -u +%FT%TZ; sw_vers -productVersion'
  $U list
} 2>&1 | tee $W2/common/vm-start.txt
