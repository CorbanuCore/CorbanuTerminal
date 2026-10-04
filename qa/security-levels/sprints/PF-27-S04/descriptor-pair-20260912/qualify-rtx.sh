#!/usr/bin/env bash
# Synthetic lifecycle proof; invoke by real keys in a private TMUX pane.
set -euo pipefail
repo=${PF27_PAIR_REPO:?exact checkout required}
out=${PF27_EVIDENCE_DIR:?evidence directory required}
export CARGO_TARGET_DIR=${PF27_PAIR_TARGET:?exclusive target required}
test -n "${TMUX:-}"
test "$(id -u)" != 0
mkdir -p "$out"
export CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
test "$(sha256sum "${PF27_SYNTHETIC_HOLD:?}" | cut -d ' ' -f1)" = 59ba145dc8178ae9680914943e8ba12c23c37d17ac4cc4a05a423539cbda1a6f
test "$(sha256sum "${PF27_SYNTHETIC_PROBE:?}" | cut -d ' ' -f1)" = f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0
cd "$repo"
git rev-parse HEAD HEAD:codex-rs
git diff --exit-code -- codex-rs
uname -a
getconf GNU_LIBC_VERSION
run() {
    local label=$1 result=0
    shift
    flock -o "${PF27_BUILD_LOCK:?}" "$@" > "$out/$label.log" 2>&1 || result=$?
    tail -25 "$out/$label.log"
    printf '%s\n' "$result" > "$out/$label.exit"
    test "$result" = 0
}
run adapter just test --locked -p codex-linux-pidfd-spawn --features synthetic-fixture --retries 0
run adapter-os just test --locked -p codex-linux-pidfd-spawn --features synthetic-fixture --run-ignored only -E '!test(pf27_unqualified_libc_fails_closed)' --retries 0
run default just test --locked -p codex-secret-broker-service --retries 0
run pair just test --locked -p codex-secret-broker-service --features synthetic-fixture -E 'test(pf_27_s01_pair_)' --retries 0
run real-pair just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_pair_)' --retries 0
run owner just test --locked -p codex-secret-broker-service --features synthetic-fixture -E 'test(pf_27_s01_owner_)' --retries 0
run real-owner just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_owner_)' --retries 0
for fixture in "${PF27_PROFILE_HOLD:?}" "${PF27_PROFILE_INSPECT:?}"; do
    run "profile-$(basename "$fixture")" env PF27_ELF_STATIC="$fixture" just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
done
run service just test --locked -p codex-secret-broker-service --features synthetic-fixture --retries 0
printf 'PF27_PAIR_A_TMUX_COMPLETE\n'
