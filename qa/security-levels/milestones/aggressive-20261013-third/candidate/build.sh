set -e
RUNTIME=/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-third-20261013
export PATH="$RUNTIME/rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export CARGO_HOME="$RUNTIME/cargo" CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/aggressive-rerun-20261011
export CARGO_BUILD_JOBS=8
unset CORBANU_HOME PFTERMINAL_HOME CODEX_HOME
cd /Volumes/CorbanuDrive/Corbanu/worktrees/aggressive-rerun-20261011
git rev-parse HEAD; git status --porcelain
cargo --version; rustc --version
python3 scripts/build_codex_package.py --variant corbanu --cargo-profile release \
  --package-dir $W/candidate/package --archive-output $W/candidate/corbanu-aarch64-apple-darwin.tar.gz \
  --symbols-dir $W/candidate/symbols
echo BUILD-DONE
