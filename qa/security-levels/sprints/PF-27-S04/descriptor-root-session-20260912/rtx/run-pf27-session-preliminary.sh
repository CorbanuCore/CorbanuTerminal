#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:/home/travis/.local/bin:$PATH
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2
export TMPDIR=/home/travis/security-round5/tmp/pf27-root-compat
export PF27_DISPATCH_RELAY=/home/travis/security-round5/evidence/pf27-dispatch-receiving-eb5855959/final-tmux/dispatch-relay
proof=/home/travis/security-round5/evidence/pf27-root-session-20260912/preliminary
test -n "${TMUX:-}"
mkdir "$proof"
cd /home/travis/worktrees/security-broker-root-session-20260912
test "$(git rev-parse HEAD)" = 4e1b57b69f623ad5d5eb2883fb039e4ef46e13c8
test "$(sha256sum "$PF27_DISPATCH_RELAY" | cut -d ' ' -f1)" = 36f1fa188f3e95faa5643ec20e6640504ce44a2cfb3093e7aa0f205ec3a32867
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
run cases just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf27_root_session_)' --retries 0
printf 'PF27_SESSION_PRELIMINARY_COMPLETE\n'
