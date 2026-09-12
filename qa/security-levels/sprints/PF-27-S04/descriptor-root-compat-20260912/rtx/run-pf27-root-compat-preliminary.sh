#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:/home/travis/.local/bin:$PATH
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2
export PF27_SYNTHETIC_HOLD=/home/travis/security-round5/evidence/pf27-adapter-20260912/profile-final/hold
export PF27_ROOT_RELAY=/home/travis/security-round5/evidence/pf27-root-compat-20260912/relay
export PF27_ELF_STATIC="$PF27_ROOT_RELAY"
export PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe
export PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe
out=/home/travis/security-round5/evidence/pf27-root-compat-20260912
cd /home/travis/worktrees/security-broker-root-compat-20260912
test "$(id -u)" = 1001
test "$(sha256sum "$PF27_ROOT_RELAY" | cut -d ' ' -f1)" = ac3f6dd19d64c504e5be9bd0cb0cd190ef62325c34a252ab49f597b23eaa8e5c
run() {
    local label=$1 result=0
    shift
    test ! -e "$out/$label.exit"
    flock -o /home/travis/security-round5/locks/build.lock "$@" > "$out/$label.log" 2>&1 || result=$?
    printf '%s\n' "$result" > "$out/$label.exit"
    tail -30 "$out/$label.log"
    test "$result" = 0
}
run fix just fix -p codex-protected-state -p codex-linux-pidfd-spawn --features synthetic-fixture --no-deps
run fmt-repaired just fmt
run relay-profile just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run identity-focused just test --locked -p codex-linux-pidfd-spawn --features synthetic-fixture --run-ignored all -E 'test(pf27_retained_identity_)' --retries 0
run transport-focused just test --locked -p codex-protected-state --features synthetic-fixture --run-ignored all -E 'test(pf27_root_compat_)' --retries 0
