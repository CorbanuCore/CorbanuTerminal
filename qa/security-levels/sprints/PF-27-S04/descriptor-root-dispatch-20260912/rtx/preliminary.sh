#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:/home/travis/.local/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2
export CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0
export TMPDIR=/home/travis/security-round5/tmp/pf27-root-compat
cd /home/travis/worktrees/security-broker-root-dispatch-20260912
out=/home/travis/security-round5/evidence/pf27-root-dispatch-20260912
export PF27_DISPATCH_RELAY="$out/relay"
run() {
    local label=$1 result=0
    shift
    flock -o /home/travis/security-round5/locks/build.lock "$@" > "$out/$label.log" 2>&1 || result=$?
    printf '%s\n' "$result" > "$out/$label.exit"
    tail -40 "$out/$label.log"
    test "$result" = 0
}
run relay-build cc -nostdlib -static-pie -fPIE -fcf-protection=none -fno-stack-protector -fno-builtin -Os -Wl,--build-id=none,-z,now -o "$PF27_DISPATCH_RELAY" qa/security-levels/sprints/PF-27-S04/descriptor-root-dispatch-20260912/relay.c
sha256sum qa/security-levels/sprints/PF-27-S04/descriptor-root-dispatch-20260912/relay.c "$PF27_DISPATCH_RELAY" > "$out/relay-hashes.txt"
readelf -hlWd "$PF27_DISPATCH_RELAY" > "$out/relay-elf.txt"
export PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe
export PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe
run relay-profile env PF27_ELF_STATIC="$PF27_DISPATCH_RELAY" just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run dispatch-initial just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf27_root_dispatch_)' --retries 0
