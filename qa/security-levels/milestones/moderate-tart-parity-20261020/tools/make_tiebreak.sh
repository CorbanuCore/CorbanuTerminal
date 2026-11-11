#!/bin/bash
# Prepare a tie-break candidate (parity-method.md): the UTM baseline candidate's signed binary in the same helper
# package, and the same packet with only the prompts' candidate line changed.  usage: make_tiebreak.sh COMMIT
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-par1-20261020
W=/Volumes/CorbanuDrive/Corbanu/.codex-work
case "$1" in
  df3d8c5d1c) BIN=$W/moderate-first-20261014/candidate2/package/bin/corbanu; SHA=abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a ;;
  4ef3be8467) BIN=$W/integration-build-20260922a/integration-package-dev/bin/corbanu.prev-4ef3be8467; SHA=cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25 ;;
  8bf182de43) BIN=$W/integration-build-20260922a/integration-package-dev/bin/corbanu.prev-8bf182de43; SHA=95ee54cb7ce1bcd3566da613cb415e8677227cb5e11baac63d5a1bbfecf9e509 ;;
  *) echo "unknown baseline commit $1" >&2; exit 64 ;;
esac
D=$Q/candidate-tb-$1
[ "$(shasum -a 256 "$BIN" | cut -d' ' -f1)" = "$SHA" ] || { echo "baseline binary digest mismatch" >&2; exit 4; }
if [ ! -d "$D/package" ]; then
  mkdir -p "$D"; cp -Rp "$Q/candidate/package" "$D/package"; cp -p "$BIN" "$D/package/bin/corbanu"
fi
printf '{"commit": "%s", "corbanu_sha256": "%s", "source": "%s"}\n' "$1" "$SHA" "$BIN" > "$D/INSTALL.json"
[ -d "$Q/packet-v4p-tb-$1" ] || python3 "$Q/tools/make_packet_par.py" \
  /Volumes/CorbanuDrive/Corbanu/worktrees/tart-par1/qa/security-levels/milestones/moderate-targeted2-20261020/executor-packet/v4 \
  "$Q/tools" "$Q/packet-v4p-tb-$1" "$SHA"
echo "stage with: CAND=$D $Q/h/make_stage.sh $Q/packet-v4p-tb-$1"
