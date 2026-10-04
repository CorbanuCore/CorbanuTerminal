#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:/home/travis/.local/bin:$PATH
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2
export TMPDIR=/home/travis/security-round5/tmp/pf27-root-compat
proof=/home/travis/security-round5/evidence/pf27-system-preflight-20260913/preliminary
test -n "${TMUX:-}"
mkdir "$proof"
cd /home/travis/worktrees/security-broker-system-preflight-20260913
test "$(git rev-parse HEAD)" = b65bf1e932f8fb6d9b24d945fe67b18a57bb0ac2
run() {
    local label=$1 result=0
    shift
    flock -o /home/travis/security-round5/locks/build.lock "$@" > "$proof/$label.log" 2>&1 || result=$?
    printf '%s\n' "$result" > "$proof/$label.exit"
    tail -35 "$proof/$label.log"
    test "$result" = 0
}
run check cargo check --manifest-path codex-rs/Cargo.toml --locked -p codex-secret-broker-service --features synthetic-fixture --tests
run fix just fix -p codex-secret-broker-service --features synthetic-fixture
run fmt just fmt
run cases just test --locked -p codex-secret-broker-service --features synthetic-fixture -E 'test(pf27_system_preflight_)' --retries 0
printf 'PF27_PREFLIGHT_PRELIMINARY_COMPLETE\n'
