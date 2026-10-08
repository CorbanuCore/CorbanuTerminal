set -e
RUNTIME=<corbanu-root>/.codex-work/corbanu-terminal
export PATH="$RUNTIME/rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CARGO_HOME="$RUNTIME/cargo" CARGO_TARGET_DIR=<corbanu-root>/.codex-work/targets/pf60-s03-indep-acceptance-20261008
export CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=off
unset CORBANU_HOME PFTERMINAL_HOME CODEX_HOME
cd <corbanu-root>/worktrees/pf60-s03-indep-acceptance-20261008/codex-rs
cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
