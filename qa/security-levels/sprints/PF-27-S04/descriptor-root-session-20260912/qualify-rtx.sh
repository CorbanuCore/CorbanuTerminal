#!/usr/bin/env bash
# Exact-source private session proof; send command and Enter separately.
set -euo pipefail
source "${PF27_PAIR_REPO:?}/qa/security-levels/sprints/PF-27-S04/descriptor-root-dispatch-20260912/qualify-rtx.sh"
run root-session just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf27_root_session_)' --retries 0
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/session-locks-after.txt"
diff "$out/locks-before.txt" "$out/session-locks-after.txt"
git diff --exit-code
test -z "$(git status --porcelain)"
printf 'PF27_ROOT_SESSION_TMUX_COMPLETE\n'
