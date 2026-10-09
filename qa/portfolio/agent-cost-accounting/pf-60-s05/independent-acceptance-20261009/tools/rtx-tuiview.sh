#!/bin/bash
# usage: tuiview.sh <home> <tmux-session> [corbanu args...]: view-only TUI (PLACEHOLDER key, no vault) in tmux, 160x60
S=<rtx-scratch>; T="tmux -L pf60s05indep"
h=$1; s=$2; shift 2
B=$S/${CORBANU_BIN:-corbanu-acct}
grep -q 'work/repo' $h/config.toml 2>/dev/null || printf '\n[projects."%s/work/repo"]\ntrust_level = "trusted"\n' "$S" >> $h/config.toml
$T kill-session -t $s 2>/dev/null
$T new-session -d -s $s -x ${COLS:-160} -y ${ROWS:-60} "cd $S/work/repo && env ZAI_API_KEY=placeholder-not-a-key CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG=${RUST_LOG:-info} $B $* 2>>$h/tui-stderr.log"
sleep 8
