#!/bin/bash
# utm-par2: copy the frozen inputs of the D94/D95 Tart re-run into the record and write design/FROZEN-d94d95.sha256
# (it does not list itself; the D93 freeze, design/FROZEN.sha256, is not changed). Run before the first Tart clone.
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par2-20261021
REC=/Volumes/CorbanuDrive/Corbanu/worktrees/utm-par2/qa/security-levels/milestones/moderate-utm-samebuild-20261021
mkdir -p "$REC"/{design,executor-packet,harness-tart,tools,candidate-cur}
[ ! -e "$REC/executor-packet/v4c" ] || rm -r "$REC/executor-packet/v4c"
cp -Rp "$Q/packet-v4c" "$REC/executor-packet/v4c"
cp -p "$Q/design/rerun-method-d94d95.md" "$REC/design/"
cp -p "$Q"/tools/patch_nav_v4c.py "$Q"/tools/freeze-d94d95.sh "$REC/tools/"
for f in "$Q"/ht/*; do [ "$(basename "$f")" = fixture.tar ] || cp -p "$f" "$REC/harness-tart/"; done
(cd "$Q/ht" && shasum -a 256 fixture.tar) > "$REC/harness-tart/fixture.tar.sha256"
cp -p "$Q/candidate-cur/INSTALL.json" "$Q/candidate-cur/build-head.txt" "$Q/candidate-cur/package-files.txt" "$Q/candidate-cur/packet-files.txt" "$REC/candidate-cur/"
cd "$REC"
find design/rerun-method-d94d95.md executor-packet/v4c harness-tart tools/patch_nav_v4c.py tools/freeze-d94d95.sh candidate-cur -type f | LC_ALL=C sort | xargs shasum -a 256 > design/FROZEN-d94d95.sha256
wc -l design/FROZEN-d94d95.sha256
