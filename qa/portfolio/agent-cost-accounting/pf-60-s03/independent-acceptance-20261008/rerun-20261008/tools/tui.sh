#!/bin/bash
# usage: tui.sh <home> <cwd> [corbanu args...]; macOS developer-accounting build in a disposable home
h=$1; cd "$2"; shift 2
B=<corbanu-root>/.codex-work/targets/pf60-s03-indep-rerun-20261008/debug/corbanu
ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" exec env CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG="${RUST_LOG:-codex_api=trace,info}" $B "$@" 2>>"$h/stderr.log"
