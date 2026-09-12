#!/usr/bin/env bash
# One accepted native prerequisite retry; original evidence is never replaced.
set -euo pipefail
repo=/home/travis/worktrees/security-broker-static-20260912
root=/home/travis/security-round5/evidence/pf27-static-probe-20260912
out=$root/openssl-retry
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
export GNUPGHOME=$root/gnupg-openssl
export PATH=/home/travis/.cargo/bin:/usr/bin:/bin
mkdir -p "$GNUPGHOME" "$root/downloads-openssl"
chmod 700 "$GNUPGHOME"
cd "$root/downloads-openssl"
url=https://github.com/openssl/openssl/releases/download/openssl-3.5.8
for name in openssl-3.5.8.tar.gz openssl-3.5.8.tar.gz.asc openssl-3.5.8.tar.gz.sha256; do
    curl --fail --location --proto '=https' --tlsv1.2 --max-time 300 "$url/$name" -o "$name"
done
curl --fail --location --proto '=https' --tlsv1.2 --max-time 60 https://openssl-library.org/source/pubkeys.asc -o pubkeys.asc
curl --fail --location --proto '=https' --tlsv1.2 --max-time 60 https://openssl-library.org/source/ -o "$out/official-downloads.html"
expected=$(awk '{print $1; exit}' openssl-3.5.8.tar.gz.sha256)
test "${#expected}" = 64
test "$expected" = "$(sha256sum openssl-3.5.8.tar.gz | cut -d' ' -f1)"
gpg --batch --import pubkeys.asc > "$out/key-import.log" 2>&1
gpg --batch --status-fd 1 --verify openssl-3.5.8.tar.gz.asc openssl-3.5.8.tar.gz > "$out/signature-status.txt" 2> "$out/signature.log"
awk '$2=="VALIDSIG" && $NF=="B146647E45A7B33947AB226B2A2C87D161692D40" {valid=1} END {exit !valid}' "$out/signature-status.txt"
sha256sum openssl-3.5.8.tar.gz openssl-3.5.8.tar.gz.asc openssl-3.5.8.tar.gz.sha256 pubkeys.asc > "$out/downloads.sha256"
tar -xzf openssl-3.5.8.tar.gz -C "$root"
cd "$root/openssl-3.5.8"
# Configure/build/install tooling only; no OpenSSL tests or target apps are run.
CC="$root/musl/bin/musl-gcc" perl ./Configure linux-x86_64 no-shared no-module no-dso no-async no-tests --prefix="$root/openssl" --openssldir="$root/openssl/ssl" --libdir=lib > "$out/configure.log" 2>&1
perl configdata.pm --dump > "$out/configuration.txt"
make -j8 build_libs > "$out/openssl-build.log" 2>&1
make install_dev > "$out/install.log" 2>&1
sha256sum "$root/openssl/lib/libssl.a" "$root/openssl/lib/libcrypto.a" "$root/musl/bin/musl-gcc" /usr/bin/gcc > "$out/native-libraries.sha256"
export CC_x86_64_unknown_linux_musl=$root/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/bin/ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$root/musl/bin/musl-gcc
export X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=$root/openssl
export X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=1
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
