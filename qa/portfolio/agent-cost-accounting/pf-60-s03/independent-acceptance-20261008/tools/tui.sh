#!/bin/bash
# usage: tui.sh <home> <cwd> [corbanu args...]
h=$1; cd "$2"; shift 2
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h
ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" RUST_LOG="${RUST_LOG:-codex_api=trace,info}" exec <corbanu-root>/.codex-work/targets/pf60-s03-indep-acceptance-20261008/debug/corbanu "$@" 2>>"$h/stderr.log"
