set -e
# usage: build-rtx.sh acct|default
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
R=<rtx-dir>
cd $R/src/codex-rs
if [ "$1" = default ]; then
  export CARGO_TARGET_DIR=$R/target-default
  cargo build --locked -j16 -p codex-cli --bin corbanu
  cp $CARGO_TARGET_DIR/debug/corbanu $R/corbanu-default
else
  export CARGO_TARGET_DIR=$R/target
  cargo build --locked -j16 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
  cp $CARGO_TARGET_DIR/debug/corbanu $R/corbanu-acct
fi
