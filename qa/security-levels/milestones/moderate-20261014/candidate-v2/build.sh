set -e
RUNTIME=/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-first-20261014
export PATH="$RUNTIME/rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export CARGO_HOME="$RUNTIME/cargo" CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/qual-mod1
export CARGO_BUILD_JOBS=8
unset CORBANU_HOME PFTERMINAL_HOME CODEX_HOME
cd /Volumes/CorbanuDrive/Corbanu/worktrees/qual-mod1-cand
git rev-parse HEAD; git status --porcelain
cargo --version; rustc --version
python3 scripts/build_codex_package.py --variant corbanu --cargo-profile release \
  --package-dir $W/candidate2/package --archive-output $W/candidate2/corbanu-aarch64-apple-darwin.tar.gz \
  --symbols-dir $W/candidate2/symbols
echo BUILD-DONE
