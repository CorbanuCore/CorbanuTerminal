#!/usr/bin/env bash
set -euo pipefail
candidate=${1:?Pass the qualified candidate directory}
shift
for binary in codex codex-code-mode-host pfterminal-walletd; do
    [[ -x "$candidate/$binary" ]] || { echo "Incomplete candidate: $binary is missing" >&2; exit 1; }
done
export PATH=/home/travis/security-round5/tools:$PATH
command -v node >/dev/null || { echo 'Installed MCP plugins require Node; run install-rtx-node.sh.' >&2; exit 1; }
export CORBANU_HOME=/home/travis/.corbanu
export CODEX_HOME=/home/travis/.corbanu
export PFTERMINAL_HOME=/home/travis/.corbanu
exec "$candidate/codex" --no-alt-screen "$@"
