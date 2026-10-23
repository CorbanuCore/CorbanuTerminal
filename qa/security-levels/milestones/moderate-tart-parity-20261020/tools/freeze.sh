#!/bin/bash
# Copy the frozen inputs into the record and write design/FROZEN.sha256 (it does not list itself).
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-par1-20261020
REC=/Volumes/CorbanuDrive/Corbanu/worktrees/tart-par1/qa/security-levels/milestones/moderate-tart-parity-20261020
mkdir -p "$REC"/{design,executor-packet,harness,tools,candidate}
rm -rf "$REC/executor-packet/v4p"; cp -Rp "$Q/packet-v4p" "$REC/executor-packet/v4p"
cp -p "$Q"/design/* "$REC/design/"
cp -p "$Q"/tools/*.py "$Q"/tools/*.json "$Q"/tools/freeze.sh "$REC/tools/"
cp -p "$Q/run-groups.sh" "$REC/tools/"
for f in "$Q"/h/*; do [ "$(basename "$f")" = fixture.tar ] || cp -p "$f" "$REC/harness/"; done
cp -p /Volumes/CorbanuDrive/Corbanu/worktrees/tart-par1/scripts/qa/tart/seal-guest.sh /Volumes/CorbanuDrive/Corbanu/worktrees/tart-par1/scripts/qa/tart/dns-refuse.py "$REC/harness/"
(cd "$Q/h" && shasum -a 256 fixture.tar) > "$REC/harness/fixture.tar.sha256"
for d in "$Q"/packet-v4p-tb-*; do c=${d##*-tb-}; mkdir -p "$REC/executor-packet/tiebreak-prompts/$c"; cp -p "$d"/prompts/*.md "$REC/executor-packet/tiebreak-prompts/$c/"; cmp -s "$d/navigation.md" "$Q/packet-v4p/navigation.md"; done
cp -p "$Q/candidate/INSTALL.json" "$Q/candidate/build-head.txt" "$Q/candidate/package-files.txt" "$Q/candidate/packet-files.txt" "$REC/candidate/"
cd "$REC"
find design executor-packet harness tools candidate -type f ! -name FROZEN.sha256 | LC_ALL=C sort | xargs shasum -a 256 > design/FROZEN.sha256
wc -l design/FROZEN.sha256
