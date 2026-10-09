set -e
# remote side. usage: build-rtx.sh acct|default
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
R=<rtx-scratch>
cd $R/src/codex-rs
export CARGO_TARGET_DIR=$R/target
if [ "$1" = default ]; then
  cargo build --locked -j32 -p codex-cli --bin corbanu
  cp $CARGO_TARGET_DIR/debug/corbanu $R/corbanu-default
else
  cargo build --locked -j32 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
  cp $CARGO_TARGET_DIR/debug/corbanu $R/corbanu-acct
fi
git -C .. rev-parse HEAD
