#!/usr/bin/env bash
# Correct final linker selection; keep native C toolchain and static PIE.
set -euo pipefail
repo=/home/travis/worktrees/security-broker-static-20260912
root=/home/travis/security-round5/evidence/pf27-static-probe-20260912
out=$root/linkage-retry
mkdir "$out"
exec > >(tee "$out/run.log") 2>&1
trap 'result=$?; printf "%s\n" "$result" > "$out/run.exit"' EXIT
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
cd "$repo"
test "$(git rev-parse HEAD)" = 140e094adc71da6e74e9017fce5d9b85b92ce956
test -z "$(git status --porcelain)"
git rev-parse HEAD HEAD:codex-rs > "$out/source.txt"
sha256sum codex-rs/Cargo.lock > "$out/lock-before.sha256"
export RUSTUP_HOME=$root/rustup RUSTUP_TOOLCHAIN=1.95.0
export CARGO_HOME=/home/travis/.cargo CARGO_TARGET_DIR=$root/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 TMPDIR=$root/tmp
export PATH=/home/travis/.cargo/bin:/usr/bin:/bin
export CC_x86_64_unknown_linux_musl=$root/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/bin/ar
export CFLAGS_x86_64_unknown_linux_musl=-I$root/uapi/include
export X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=$root/openssl-uapi
export X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=1
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS='-C linker-flavor=ld.lld'
"$CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER" -flavor gnu --version > "$out/linker-version.txt"
sha256sum "$CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER" > "$out/linker.sha256"
cp "$root/musl/lib/musl-gcc.specs" "$out/previous-compiler.specs"
cd "$repo/codex-rs"
printf '%s BUILD START\n' "$(date -Is)"
set +e
cargo build --locked --target x86_64-unknown-linux-musl -p codex-secret-broker-service --features synthetic-fixture --bin codex-protected-root-probe > "$out/build.log" 2>&1
build_result=$?
set -e
printf '%s\n' "$build_result" > "$out/build.exit"
printf '%s BUILD EXIT %s\n' "$(date -Is)" "$build_result"
cd "$repo"
git status --porcelain > "$out/source-status.txt"
sha256sum codex-rs/Cargo.lock > "$out/lock-after.sha256"
cmp "$out/lock-before.sha256" "$out/lock-after.sha256"
test ! -s "$out/source-status.txt"
if test "$build_result" != 0; then
    tail -65 "$out/build.log"
    exit "$build_result"
fi
mkdir "$out/candidate"
cp "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/debug/codex-protected-root-probe" "$out/candidate/"
python3 "$root/scripts/inspect-linkage.py" "$out"
