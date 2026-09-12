#!/usr/bin/env bash
# Explicitly accepted UAPI-only prerequisite completion, no artifact execution.
set -euo pipefail
repo=/home/travis/worktrees/security-broker-static-20260912
root=/home/travis/security-round5/evidence/pf27-static-probe-20260912
out=$root/uapi-retry
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
lists=/var/lib/apt/lists/us.archive.ubuntu.com_ubuntu_dists_resolute-updates
cp "${lists}_InRelease" "$out/InRelease"
gpgv --keyring /usr/share/keyrings/ubuntu-archive-keyring.gpg "$out/InRelease" > "$out/repository-signature.txt" 2>&1
sha256sum /usr/share/keyrings/ubuntu-archive-keyring.gpg "$out/InRelease" > "$out/repository-trust.sha256"
packages=${lists}_main_binary-amd64_Packages
expected=$(awk '/^SHA256:/{s=1;next} /^[^ ]/{s=0} s && $3=="main/binary-amd64/Packages" {print $1}' "$out/InRelease")
test "${#expected}" = 64
test "$expected" = "$(sha256sum "$packages" | cut -d' ' -f1)"
sha256sum "$packages" > "$out/packages.sha256"
awk 'BEGIN {RS=""} /(^|\n)Package: linux-libc-dev(\n|$)/ && /(^|\n)Version: 7.0.0-31.31(\n|$)/ {print}' "$packages" > "$out/package-metadata.txt"
pin=b26e3493c7180b0cc8b5e7c2bf819323deca8b7662f34ddffef936b08ffb1456
test "$(awk '$1=="SHA256:" {print $2}' "$out/package-metadata.txt")" = "$pin"
deb=$root/linux-libc-dev_7.0.0-31.31_amd64.deb
curl --fail --location --proto '=https' --tlsv1.2 --max-time 180 https://archive.ubuntu.com/ubuntu/pool/main/l/linux/linux-libc-dev_7.0.0-31.31_amd64.deb -o "$deb"
test "$pin" = "$(sha256sum "$deb" | cut -d' ' -f1)"
sha256sum "$deb" > "$out/package.sha256"
dpkg-deb --info "$deb" > "$out/package-info.txt"
dpkg-deb --extract "$deb" "$root/uapi-package"
mkdir -p "$root/uapi/include"
cp -a "$root/uapi-package/usr/include/linux" "$root/uapi-package/usr/include/asm-generic" "$root/uapi/include/"
cp -a "$root/uapi-package/usr/include/x86_64-linux-gnu/asm" "$root/uapi/include/"
find "$root/uapi/include" -type f -print0 | sort -z | xargs -0 sha256sum > "$out/headers.sha256"
find "$root/uapi/include" -type l -print > "$out/header-symlinks.txt"
test ! -s "$out/header-symlinks.txt"
printf '#include <linux/mman.h>\n' | "$root/musl/bin/musl-gcc" -I"$root/uapi/include" -H -E -x c - > "$out/include-preprocess.txt" 2> "$out/include-resolution.txt"
! grep -E '(^| )/usr/include/' "$out/include-resolution.txt"
tarball=$root/downloads-openssl/openssl-3.5.8.tar.gz
test "$(sha256sum "$tarball" | cut -d' ' -f1)" = a8f84a39918ec6415ce765d9b429d313ba97b8143169c172e734b9514464f5b2
mkdir "$root/openssl-uapi-src"
tar -xzf "$tarball" -C "$root/openssl-uapi-src"
cd "$root/openssl-uapi-src/openssl-3.5.8"
CC="$root/musl/bin/musl-gcc" CPPFLAGS="-I$root/uapi/include" perl ./Configure linux-x86_64 no-shared no-module no-dso no-async no-tests --prefix="$root/openssl-uapi" --openssldir="$root/openssl-uapi/ssl" --libdir=lib > "$out/configure.log" 2>&1
perl configdata.pm --dump > "$out/configuration.txt"
make -j8 build_libs > "$out/openssl-build.log" 2>&1
make install_dev > "$out/install.log" 2>&1
sha256sum "$root/openssl-uapi/lib/libssl.a" "$root/openssl-uapi/lib/libcrypto.a" "$root/musl/bin/musl-gcc" /usr/bin/gcc > "$out/native-libraries.sha256"
export CC_x86_64_unknown_linux_musl=$root/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/bin/ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$root/musl/bin/musl-gcc
export CFLAGS_x86_64_unknown_linux_musl=-I$root/uapi/include
export X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=$root/openssl-uapi
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
