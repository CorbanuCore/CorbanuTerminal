#!/usr/bin/env bash
# Build tooling may execute; neither probe artifact is ever invoked.
set -euo pipefail
repo=/home/travis/worktrees/security-broker-static-20260912
root=/home/travis/security-round5/evidence/pf27-static-probe-20260912
out=$root/initial
mkdir "$out"
exec > >(tee "$out/run.log") 2>&1
trap 'result=$?; printf "%s\n" "$result" > "$out/run.exit"' EXIT
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
cd "$repo"
test "$(git rev-parse HEAD)" = 140e094adc71da6e74e9017fce5d9b85b92ce956
test "$(git rev-parse HEAD:codex-rs)" = 0aa65bd04f5e30f3a21e1309aac7aa6b83336859
test -z "$(git status --porcelain)"
git rev-parse HEAD HEAD:codex-rs > "$out/source.txt"
sha256sum codex-rs/Cargo.lock > "$out/lock-before.sha256"
export RUSTUP_HOME=$root/rustup
export CARGO_HOME=/home/travis/.cargo
export CARGO_TARGET_DIR=$root/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608
export TMPDIR=$root/tmp
export GNUPGHOME=$root/gnupg
export PATH=/home/travis/.cargo/bin:/usr/bin:/bin
mkdir -p "$RUSTUP_HOME" "$TMPDIR" "$GNUPGHOME" "$root/downloads"
chmod 700 "$GNUPGHOME"
printf 'Installing isolated pinned Rust 1.95.0 and musl target\n'
rustup toolchain install 1.95.0 --profile minimal --target x86_64-unknown-linux-musl --no-self-update > "$out/rust-install.log" 2>&1
export RUSTUP_TOOLCHAIN=1.95.0
rustc -Vv > "$out/rustc.txt"
cargo -Vv > "$out/cargo.txt"
gcc -v > "$out/gcc.txt" 2>&1
readelf --version > "$out/readelf.txt"
cd "$root/downloads"
for name in musl-1.2.6.tar.gz musl-1.2.6.tar.gz.asc; do
    curl --fail --location --proto '=https' --tlsv1.2 --max-time 180 "https://musl.libc.org/releases/$name" -o "$name"
done
curl --fail --location --proto '=https' --tlsv1.2 --max-time 60 https://musl.libc.org/musl.pub -o musl.pub
gpg --batch --import musl.pub > "$out/musl-key-import.log" 2>&1
test "$(gpg --batch --with-colons --fingerprint 56BCDB593020450F | awk -F: '$1=="fpr" {print $10; exit}')" = 836489290BB6B70F99FFDA0556BCDB593020450F
gpg --batch --verify musl-1.2.6.tar.gz.asc musl-1.2.6.tar.gz > "$out/musl-signature.log" 2>&1
sha256sum musl-1.2.6.tar.gz musl-1.2.6.tar.gz.asc musl.pub > "$out/downloads.sha256"
tar -xzf musl-1.2.6.tar.gz -C "$root"
cd "$root/musl-1.2.6"
./configure --prefix="$root/musl" --disable-shared > "$out/musl-configure.log" 2>&1
make -j8 > "$out/musl-build.log" 2>&1
make install > "$out/musl-install.log" 2>&1
export CC_x86_64_unknown_linux_musl=$root/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/bin/ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$root/musl/bin/musl-gcc
export PATH=$root/musl/bin:$PATH
sha256sum "$root/musl/bin/musl-gcc" "$root/musl/lib/libc.a" /usr/bin/gcc > "$out/compiler.sha256"
sysroot=$(rustc --print sysroot)
find "$sysroot/lib/rustlib/x86_64-unknown-linux-musl/lib" -type f -print0 | sort -z | xargs -0 sha256sum > "$out/rust-target.sha256"
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
