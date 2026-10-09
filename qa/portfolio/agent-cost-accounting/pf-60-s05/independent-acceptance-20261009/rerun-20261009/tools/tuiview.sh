#!/bin/bash
# usage: tuiview.sh <home> <tmux-session> [corbanu args...]: view-only TUI with PLACEHOLDER keys (no vault), 160x60
S=<scratch>; T="tmux -L pf60rerun"
h=$1; s=$2; shift 2
B=$S/bin/${CORBANU_BIN:-corbanu-acct}
$T kill-session -t $s 2>/dev/null
$T new-session -d -s $s -x ${COLS:-160} -y ${ROWS:-60} "cd $S/work/repo && env CODEX_API_KEY=placeholder-not-a-key KIMI_API_KEY=placeholder-not-a-key CLAUDE_CODE_OAUTH_TOKEN=placeholder-not-a-token CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG=${RUST_LOG:-info} $B $* 2>>$h/tui-stderr.log"
sleep ${BOOT:-8}
