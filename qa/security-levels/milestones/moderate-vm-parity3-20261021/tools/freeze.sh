#!/bin/bash
# utm-par3: copy the frozen inputs of the D139/D141 runs into the record and write design/FROZEN.sha256 (it does not
# list itself). Run before "macOS 3" or any Tart clone starts. The v9 amendment's own freeze is in its record.
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par3-20261021
WT=/Volumes/CorbanuDrive/Corbanu/worktrees/utm-par3
REC=$WT/qa/security-levels/milestones/moderate-vm-parity3-20261021
mkdir -p "$REC"/{design,executor-packet,harness,harness-utm,harness-tart,tools,candidate}
[ ! -e "$REC/executor-packet/v4d" ] || rm -r "$REC/executor-packet/v4d"
cp -Rp "$Q/packet-v4d" "$REC/executor-packet/v4d"
cp -p "$Q/design/parity-method-d139.md" "$REC/design/"
for f in "$Q"/h/*; do [ "$(basename "$f")" = fixture.tar ] || cp -p "$f" "$REC/harness/"; done
(cd "$Q/h" && shasum -a 256 fixture.tar) > "$REC/harness/fixture.tar.sha256"
cp -p "$Q"/hu/* "$REC/harness-utm/"; cp -p "$Q"/ht/* "$REC/harness-tart/"
{ echo "# Seal assets that harness-tart/tart-run.sh 'seal' copies at run time (hashed at the freeze)."
  (cd "$WT/scripts/qa/tart" && shasum -a 256 seal-guest.sh dns-refuse.py)
  shasum -a 256 /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/tools/tmux-source/3.7c/bin/tmux /opt/homebrew/opt/libevent/lib/libevent_core-2.1.7.dylib | sed 's#  .*/#  #'
} > "$REC/harness-tart/seal-assets.sha256"
cp -p "$Q"/tools/make_packet_v4d.py "$Q"/tools/make_stage.sh "$Q"/tools/freeze.sh "$REC/tools/"
cp -p "$Q/candidate/INSTALL.json" "$Q/candidate/build-head.txt" "$Q/candidate/package-files.txt" "$Q/candidate/packet-files.txt" "$REC/candidate/"
cd "$REC"
find design/parity-method-d139.md executor-packet/v4d harness harness-utm harness-tart tools candidate -type f | LC_ALL=C sort | xargs shasum -a 256 > design/FROZEN.sha256
wc -l design/FROZEN.sha256
