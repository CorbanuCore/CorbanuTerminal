#!/bin/bash
# usage: tuiview.sh <home> <cwd> [corbanu args...]: view-only TUI in a disposable home with a PLACEHOLDER key (no vault)
h=$1; cd "$2"; shift 2
B=<scratch>/bin/${CORBANU_BIN:-corbanu-acct}
exec env ZAI_API_KEY=placeholder-not-a-key CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG="${RUST_LOG:-info}" $B "$@" 2>>"$h/stderr.log"
