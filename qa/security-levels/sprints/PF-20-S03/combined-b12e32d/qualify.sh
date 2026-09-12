#!/bin/bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8
export TMPDIR=$(mktemp -d /home/travis/security-round5/anchor-tmp/combined-b12e32d.XXXXXX)
evidence=/home/travis/security-round5/evidence/anchor/combined-b12e32d
mirror=/home/travis/worktrees/security-anchor-combined-b12e32d
mkdir -p "$evidence"
printf 'TMPDIR=%s\n' "$TMPDIR"
git -C /home/travis/repos/CorbanuTerminal-harness fetch origin integration/security-round5-20260904
test ! -e "$mirror"
git -C /home/travis/repos/CorbanuTerminal-harness worktree add --detach "$mirror" b12e32db398c83854271e2e70f29e5290278af8b
cd "$mirror"
test -z "$(git status --porcelain)"
git rev-parse HEAD
trap 'bazel shutdown' EXIT
find codex-rs -name '*.rs' -type f -exec touch {} +
just fix -p codex-protected-state > "$evidence/fix-leaf.log" 2>&1
just fix -p codex-core > "$evidence/fix-core.log" 2>&1
just fix -p codex-memories-write > "$evidence/fix-memory.log" 2>&1
just fmt > "$evidence/fmt.log" 2>&1
git diff --binary > "$evidence/formatter.patch"
if test -n "$(git status --porcelain)"; then
    git status --short
    echo 'STOP: fix/fmt produced source delta; coordinator must assess.'
    exit 10
fi
just bazel-lock-check > "$evidence/bazel-lock.log" 2>&1
bazel shutdown
just test -p codex-protected-state --retries 0 --test-threads 4 > "$evidence/leaf.log" 2>&1
just test -p codex-core -E 'test(pf20_s03) | test(authoritative_state_tests) | test(pf_30_s04) | test(pf_30_s01) | test(realtime_conversation) | test(broker_client) | test(network_proxy_credential)' --retries 0 --test-threads 4 > "$evidence/core.log" 2>&1
just test -p codex-security-audit --retries 0 --test-threads 4 > "$evidence/audit.log" 2>&1
just test -p codex-config --retries 0 --test-threads 4 > "$evidence/config.log" 2>&1
just test -p codex-memories-write --retries 0 --test-threads 4 > "$evidence/memories-write.log" 2>&1
just test -p codex-memories-read --retries 0 --test-threads 4 > "$evidence/memories-read.log" 2>&1
just codex --version > "$evidence/cli-build.log" 2>&1
mkdir -p "$evidence/candidate"
cp "$CARGO_TARGET_DIR/debug/codex" "$evidence/candidate/codex"
sha256sum "$evidence/candidate/codex" | tee "$evidence/candidate.sha256"
export CARGO_BIN_EXE_codex="$evidence/candidate/codex"
export CORBANU_TMUX_REQUIRED=1
export CORBANU_TMUX_ARTIFACT_DIR="$evidence/tmux-artifacts"
export CORBANU_SECURITY_UI_EVIDENCE="$evidence/security-tmux"
export CORBANU_MEMORY_EVIDENCE="$evidence/memory-tmux"
just test -p codex-tui --lib -E 'test(security_view) | test(status::tests) | test(slash_command)' --retries 0 --test-threads 4 > "$evidence/tui-units.log" 2>&1
just test -p codex-tui --test all -E 'test(tmux_memory_worker_policy) | test(security_profiles) | test(tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly)' --retries 0 --test-threads 1 > "$evidence/tmux.log" 2>&1
python3 qa/security-levels/sprints/PF-20-S03/tmux_restart.py --binary "$CARGO_BIN_EXE_codex" --repo "$mirror" --evidence "$evidence/anchor-restart" > "$evidence/restart.log" 2>&1
sha256sum -c "$evidence/candidate.sha256"
git diff --check
test -z "$(git status --porcelain)"
git rev-parse HEAD
echo 'PASS: combined native root, memory regressions, actual-key TUI and same-home restart; no source delta.'
