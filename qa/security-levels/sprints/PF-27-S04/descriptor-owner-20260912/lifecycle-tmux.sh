#!/usr/bin/env bash
# Invoke in a private TMUX pane: send this command as text, then Enter separately.
# Internal synthetic lifecycle proof, not interactive product acceptance.
set -euo pipefail
repo=${PF27_OWNER_REPO:?exact owner checkout required}
out=${PF27_EVIDENCE_DIR:?fresh evidence directory required}
test -n "${TMUX:-}"
test "$(id -u)" != 0
mkdir -p "$out"
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target CARGO_BUILD_JOBS=4
export PF27_SYNTHETIC_HOLD=/home/travis/security-round5/evidence/pf27-adapter-20260912/profile-final/hold
export PF27_SYNTHETIC_PROBE=/home/travis/security-round5/evidence/pf27-static-probe-20260912/linkage-retry/candidate/codex-protected-root-probe
test "$(sha256sum "$PF27_SYNTHETIC_HOLD" | cut -d ' ' -f1)" = 59ba145dc8178ae9680914943e8ba12c23c37d17ac4cc4a05a423539cbda1a6f
test "$(sha256sum "$PF27_SYNTHETIC_PROBE" | cut -d ' ' -f1)" = f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0
cd "$repo"
git rev-parse HEAD HEAD:codex-rs
git diff --exit-code -- codex-rs
uname -a
getconf GNU_LIBC_VERSION
sha256sum "$PF27_SYNTHETIC_HOLD" "$PF27_SYNTHETIC_PROBE"
run() {
    local label=$1 result=0
    shift
    flock -o /home/travis/security-round5/locks/build.lock "$@" > "$out/$label.log" 2>&1 || result=$?
    tail -25 "$out/$label.log"
    printf '%s\n' "$result" > "$out/$label.exit"
    test "$result" = 0
}
run focused just test -p codex-secret-broker-service --features synthetic-fixture -E 'test(pf_27_s01_owner_)' --retries 0
run real-owner just test -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_owner_)' --retries 0
for fixture in hold inspect; do
    run "profile-$fixture" env PF27_ELF_STATIC="/home/travis/security-round5/evidence/pf27-adapter-20260912/profile-final/$fixture" PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe just test -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
done
run service just test -p codex-secret-broker-service --features synthetic-fixture --retries 0
printf 'PF27_OWNER_TMUX_COMPLETE\n'
