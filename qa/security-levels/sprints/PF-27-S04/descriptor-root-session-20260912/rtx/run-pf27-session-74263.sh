#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:/home/travis/.local/bin:$PATH
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export PF27_PAIR_REPO=/home/travis/worktrees/security-broker-root-session-20260912
export PF27_PAIR_TARGET=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2
export PF27_BAZEL_TARGET=/home/travis/security-round5/targets/pf27-receiving-c5e606fa2-bazel
export PF27_BUILD_LOCK=/home/travis/security-round5/locks/build.lock
export TMPDIR=/home/travis/security-round5/tmp/pf27-root-compat
export PF27_SYNTHETIC_HOLD=/home/travis/security-round5/evidence/pf27-adapter-20260912/profile-final/hold
export PF27_SYNTHETIC_INSPECT=/home/travis/security-round5/evidence/pf27-adapter-20260912/profile-final/inspect
export PF27_SYNTHETIC_PROBE=/home/travis/security-round5/evidence/pf27-static-probe-20260912/linkage-retry/candidate/codex-protected-root-probe
export PF27_PROFILE_HOLD="$PF27_SYNTHETIC_HOLD" PF27_PROFILE_INSPECT="$PF27_SYNTHETIC_INSPECT"
export PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe
export PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe
proof=/home/travis/security-round5/evidence/pf27-root-session-20260912
export PF27_EVIDENCE_DIR="$proof/final-tmux"
test -n "${TMUX:-}"
test "$(id -u)" = 1001
mkdir "$proof/final-started-74263"
cd "$PF27_PAIR_REPO"
test "$(git rev-parse HEAD)" = 74263bbbc5bf2163f571947c6828fbf108866c3f
test "$(git rev-parse HEAD:codex-rs)" = b7275b0c0433f081d6423463993abc658141a6a7
test "$(git rev-parse HEAD:codex-rs/Cargo.lock)" = e2dcd5d25bd6f7749b66d30928f5574bcd134d5f
test "$(git rev-parse HEAD:MODULE.bazel.lock)" = 5ecff077dbbbf7887a61c03a8e9aa354f002e1ed
test -z "$(git status --porcelain)"
test "$(sha256sum "$PF27_SYNTHETIC_INSPECT" | cut -d ' ' -f1)" = cc821a2ee024b3f17a9073de108a6212e5f2c34f77818715b775c59f880477ca
test "$(sha256sum "$PF27_ELF_GNU" | cut -d ' ' -f1)" = 836d74a13c910359ab71abe90523b682211445d355e5c4851164423910d7d555
test "$(sha256sum "$PF27_ELF_INTERPRETER" | cut -d ' ' -f1)" = 37fb0d7c5fd7e09ca23e9a5b1901b77786b87dfe8a869e191452c571bda7081d
{
    date -u; id; uname -a; getconf GNU_LIBC_VERSION; rustc -Vv
    git rev-parse HEAD HEAD:codex-rs HEAD:codex-rs/Cargo.lock HEAD:MODULE.bazel.lock
    sha256sum codex-rs/Cargo.toml codex-rs/Cargo.lock MODULE.bazel MODULE.bazel.lock
    sha256sum "$PF27_SYNTHETIC_HOLD" "$PF27_SYNTHETIC_INSPECT" "$PF27_SYNTHETIC_PROBE" "$PF27_ELF_GNU" "$PF27_ELF_INTERPRETER"
    stat -f -c '%T' "$TMPDIR"
} > "$proof/provenance-before.txt"
sha256sum codex-rs/Cargo.toml codex-rs/Cargo.lock MODULE.bazel MODULE.bazel.lock > "$proof/modules-before.txt"
result=0
bash qa/security-levels/sprints/PF-27-S04/descriptor-root-session-20260912/qualify-rtx.sh > "$proof/suite.log" 2>&1 || result=$?
printf '%s\n' "$result" > "$proof/suite.exit"
tail -50 "$proof/suite.log"
sha256sum codex-rs/Cargo.toml codex-rs/Cargo.lock MODULE.bazel MODULE.bazel.lock > "$proof/modules-after.txt"
{
    date -u
    git rev-parse HEAD HEAD:codex-rs HEAD:codex-rs/Cargo.lock HEAD:MODULE.bazel.lock
    git status --porcelain
    sha256sum "$PF27_SYNTHETIC_HOLD" "$PF27_SYNTHETIC_INSPECT" "$PF27_SYNTHETIC_PROBE" "$PF27_ELF_GNU" "$PF27_ELF_INTERPRETER"
} > "$proof/provenance-after.txt"
diff "$proof/modules-before.txt" "$proof/modules-after.txt"
test -z "$(git status --porcelain)"
test "$result" = 0
printf 'PF27_ROOT_SESSION_74263_COMPLETE\n'
