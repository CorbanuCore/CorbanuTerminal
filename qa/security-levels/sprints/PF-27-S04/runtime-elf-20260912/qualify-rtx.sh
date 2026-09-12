#!/usr/bin/env bash
set -euo pipefail
repo=/home/travis/worktrees/security-broker-elf-20260912
evidence=${PF27_EVIDENCE_DIR:?fresh evidence directory required}
mkdir -p "$evidence"
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1
export TMPDIR
TMPDIR=$(mktemp -d /home/travis/.pf27elf.XXXXXX)
export PF27_ELF_STATIC=/home/travis/security-round5/evidence/pf27-static-probe-20260912/linkage-retry/candidate/codex-protected-root-probe
export PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe
export PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe
cd "$repo"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
run() {
    local label=$1
    shift
    if "$@" > "$evidence/$label.log" 2>&1; then
        printf '0\n' > "$evidence/$label.exit"
    else
        local result=$?
        printf '%s\n' "$result" > "$evidence/$label.exit"
        tail -60 "$evidence/$label.log"
        return "$result"
    fi
}
git rev-parse HEAD HEAD:codex-rs > "$evidence/source.txt"
sha256sum "$PF27_ELF_STATIC" "$PF27_ELF_GNU" "$PF27_ELF_INTERPRETER" > "$evidence/artifacts.sha256"
run fix just fix -p codex-secret-broker-service --features synthetic-fixture
run fmt just fmt
run lint just clippy -p codex-secret-broker-service --features synthetic-fixture --no-deps
run default just test -p codex-secret-broker-service --retries 0 --test-threads 4
run synthetic just test -p codex-secret-broker-service --features synthetic-fixture --retries 0 --test-threads 4
run artifacts just test -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run affected just test -p codex-protected-state -p codex-secret-broker -p codex-vault -p codex-network-proxy --retries 0 --test-threads 4
git diff --stat > "$evidence/post-format-stat.txt"
printf 'COMPLETE\n'
