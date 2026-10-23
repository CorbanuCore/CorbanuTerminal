set -e
# usage: build-mac.sh [default]  (developer-accounting build of the worktree checkout; `default` = no feature)
ROOT=<corbanu-root>
RUNTIME=$ROOT/.codex-work/corbanu-terminal
export PATH="$RUNTIME/rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CARGO_HOME="$RUNTIME/cargo" CARGO_TARGET_DIR=$ROOT/.codex-work/targets/pf60-s03-indep-rerun2-20261008
export CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=off
unset CORBANU_HOME PFTERMINAL_HOME CODEX_HOME
cd $ROOT/worktrees/pf60-s03-indep-rerun2-20261008/codex-rs
if [ "$1" = default ]; then cargo build --locked -j8 -p codex-cli --bin corbanu; else cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting; fi
