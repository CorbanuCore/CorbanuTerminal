#!/bin/bash
# remote side. usage: panetui.sh <home> : TUI with the real claude-plan token (read from stdin, environment only) in tmux -L pf60pane
R=$HOME/pf60-s05-rerun; h=$1
IFS= read -r KEYVAL
export CLAUDE_CODE_OAUTH_TOKEN="$KEYVAL"; unset KEYVAL
export PATH="$R/pathbin:$R/claudehome/.local/bin:$PATH" CLAUDE_CONFIG_DIR=$R/claudecfg CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG=info
cd $R/work/repo
tmux -L pf60pane kill-server 2>/dev/null
tmux -L pf60pane new-session -d -s p -x 160 -y 60 "$R/corbanu-acct 2>>$h/tui-stderr.log"
echo started
