set -e
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
export CARGO_TARGET_DIR=$HOME/corbanu-rtx/pf60-indep-20261008/target
cd $HOME/corbanu-rtx/pf60-indep-20261008/src/codex-rs
cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
