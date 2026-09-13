#!/usr/bin/env bash
# Exact-source private preparation proof, invoked by actual TMUX keys.
set -euo pipefail
source "${PF27_PAIR_REPO:?}/qa/security-levels/sprints/PF-27-S04/descriptor-root-session-20260912/qualify-rtx.sh"
run system-preflight just test --locked -p codex-secret-broker-service --features synthetic-fixture -E 'test(pf27_system_preflight_)' --retries 0
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/preflight-locks-after.txt"
diff "$out/locks-before.txt" "$out/preflight-locks-after.txt"
git diff --exit-code
test -z "$(git status --porcelain)"
printf 'PF27_SYSTEM_PREFLIGHT_TMUX_COMPLETE\n'
