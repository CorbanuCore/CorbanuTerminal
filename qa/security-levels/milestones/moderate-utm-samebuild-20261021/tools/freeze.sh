#!/bin/bash
# utm-par2: copy the frozen inputs of the UTM same-build check (D93) into the record and write
# design/FROZEN.sha256 (it does not list itself). Run before "macOS 3" is started.
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par2-20261021
REC=/Volumes/CorbanuDrive/Corbanu/worktrees/utm-par2/qa/security-levels/milestones/moderate-utm-samebuild-20261021
mkdir -p "$REC"/{design,executor-packet,harness,harness-utm,tools,candidate}
[ ! -e "$REC/executor-packet/v4u" ] || rm -r "$REC/executor-packet/v4u"
cp -Rp "$Q/packet-v4u" "$REC/executor-packet/v4u"
cp -p "$Q/design/parity-method-utm.md" "$REC/design/"
cp -p "$Q"/tools/make_packet_utm.py "$Q"/tools/freeze.sh "$Q"/tools/groups.json "$REC/tools/"
for f in "$Q"/h/*; do [ "$(basename "$f")" = fixture.tar ] || cp -p "$f" "$REC/harness/"; done
(cd "$Q/h" && shasum -a 256 fixture.tar) > "$REC/harness/fixture.tar.sha256"
cp -p "$Q"/hu/* "$REC/harness-utm/"
cp -p "$Q/candidate-8c/INSTALL.json" "$Q/candidate-8c/build-head.txt" "$Q/candidate-8c/package-files.txt" "$Q/candidate-8c/packet-files.txt" "$REC/candidate/"
cd "$REC"
find design executor-packet harness harness-utm tools candidate -type f ! -name FROZEN.sha256 | LC_ALL=C sort | xargs shasum -a 256 > design/FROZEN.sha256
wc -l design/FROZEN.sha256
