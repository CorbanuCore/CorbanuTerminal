#!/usr/bin/env bash
set -euo pipefail
repo=/home/travis/worktrees/security-broker-sealed-20260912
evidence=${PF27_EVIDENCE_DIR:-/home/travis/security-round5/evidence/pf27-sealed-20260912/initial}
mkdir -p "$evidence"
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1
export TMPDIR
TMPDIR=$(mktemp -d /home/travis/.pf27s.XXXXXX)
cd "$repo"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
run() {
    local label=$1
    shift
    printf '%s START %s\n' "$(date -Is)" "$label"
    if "$@" > "$evidence/$label.log" 2>&1; then
        printf '%s PASS %s\n' "$(date -Is)" "$label"
    else
        local result=$?
        tail -60 "$evidence/$label.log"
        return "$result"
    fi
}
run fix just fix -p codex-secret-broker-service --features synthetic-fixture
run fmt just fmt
run clippy-default-scoped just clippy -p codex-secret-broker-service --no-deps
run clippy-fixture-scoped just clippy -p codex-secret-broker-service --features synthetic-fixture --no-deps
run bazel-lock just bazel-lock-update
bazel shutdown > "$evidence/bazel-shutdown.log" 2>&1 || true
run default just test -p codex-secret-broker-service --retries 0 --test-threads 4
run fixture just test -p codex-secret-broker-service --features synthetic-fixture --retries 0 --test-threads 4
run affected just test -p codex-protected-state -p codex-secret-broker -p codex-vault -p codex-network-proxy --retries 0 --test-threads 4
run build bash -c 'cd codex-rs && cargo build --locked -p codex-secret-broker-service --features synthetic-fixture'
mkdir -p "$evidence/candidate"
cp "$CARGO_TARGET_DIR/debug/codex-secret-broker-service" "$evidence/candidate/"
cp "$CARGO_TARGET_DIR/debug/codex-secret-broker-service-fixture" "$evidence/candidate/"
cp "$CARGO_TARGET_DIR/debug/codex-protected-root-probe" "$evidence/candidate/"
sha256sum "$evidence/candidate/"* > "$evidence/binaries.sha256"
printf '%s COMPLETE\n' "$(date -Is)"
