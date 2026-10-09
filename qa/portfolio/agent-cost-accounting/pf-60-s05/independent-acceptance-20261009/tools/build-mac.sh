set -e
# usage: build-mac.sh [default]  developer-accounting debug build of the QA worktree; `default` = no feature
ROOT=<corbanu-root>
RUNTIME=$ROOT/.codex-work/corbanu-terminal
export PATH="$RUNTIME/rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CARGO_HOME="$RUNTIME/cargo" CARGO_TARGET_DIR=$ROOT/.codex-work/targets/pf60-s05-indep-20261009
export CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=off
unset CORBANU_HOME PFTERMINAL_HOME CODEX_HOME
cd $ROOT/worktrees/pf60-s05-indep-20261009/codex-rs
S=$ROOT/tmp/pf60-s05-indep
if [ "$1" = default ]; then
  cargo build --locked -j8 -p codex-cli --bin corbanu
  cp $CARGO_TARGET_DIR/debug/corbanu $S/bin/corbanu-default
else
  cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
  cp $CARGO_TARGET_DIR/debug/corbanu $S/bin/corbanu-acct
fi
git -C .. rev-parse HEAD
