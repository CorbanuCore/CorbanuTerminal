#!/bin/bash
# Host: stage the candidate (the installed signed developer build) and the executor packet.
# usage: make_stage.sh  (reads INSTALL.json; refuses a build older than the freeze requirement)
set -euo pipefail
W2=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016
B=/Volumes/CorbanuDrive/Corbanu/.codex-work/integration-build-20260922a
P=$B/integration-package-dev
ST=$W2/stage; C=$W2/candidate
rm -rf "$ST" && mkdir -p "$ST/package/bin" "$C"
cp "$B/INSTALL.json" "$C/INSTALL.json"
commit=$(python3 -c "import json;print(json.load(open('$C/INSTALL.json'))['commit'])")
git -C /Volumes/CorbanuDrive/Corbanu/CorbanuTerminal merge-base --is-ancestor 395ac0adcd "$commit" || { echo "build $commit is older than 395ac0adcd"; exit 3; }
cp -p "$P/bin/corbanu" "$P/bin/codex-code-mode-host" "$ST/package/bin/"
cp -Rp "$P/codex-resources" "$ST/package/"
cp -p "$P/codex-package.json" "$ST/package/"
want=$(python3 -c "import json;print(json.load(open('$C/INSTALL.json'))['corbanu_sha256'])")
got=$(shasum -a 256 "$ST/package/bin/corbanu" | cut -d' ' -f1)
[ "$want" = "$got" ] || { echo "binary digest $got != INSTALL.json $want"; exit 4; }
(cd "$ST/package" && find . -type f | sort | xargs shasum -a 256) > "$ST/package-files.txt"
cp "$ST/package-files.txt" "$C/package-files.txt"
codesign -dv "$ST/package/bin/corbanu" > "$C/codesign.txt" 2>&1 || true
grep -c CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION "$ST/package/bin/corbanu" > "$C/developer-marker-count.txt" || true
version=$("$ST/package/bin/corbanu" --version | awk '{print $2}')
python3 $W2/tools/make_packet.py $W2 "$ST/packet" "$got" "$version" | tee "$C/packet-build.txt"
if grep -q '^D3 GRANTED' /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/pf60-s10.grant.md; then
  echo "D3 GRANTED (owner, via the coordinator's grant file): ACC-35 may run" > "$ST/packet/d3.txt"
else
  echo "D3 NOT GRANTED: do not run ACC-35; record it as not checked live; covered by test" > "$ST/packet/d3.txt"
fi
pkt=$(shasum -a 256 "$ST/packet/design-cases.md" | cut -d' ' -f1)
printf '{"binary_sha256": "%s", "packet_sha256": "%s"}\n' "$got" "$pkt" > $W2/h/expected.json
(cd "$ST/packet" && find . -type f | sort | xargs shasum -a 256) > "$C/packet-files.txt"
printf 'commit %s\nversion %s\ncorbanu sha256 %s\nstaged %s\n' "$commit" "$version" "$got" "$(date -u +%FT%TZ)" | tee "$C/build-head.txt"
cat "$ST/packet/d3.txt"
