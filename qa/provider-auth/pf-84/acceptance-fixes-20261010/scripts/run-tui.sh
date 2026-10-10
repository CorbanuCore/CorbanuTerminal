#!/bin/bash
# usage: run-tui.sh <bin> <home> <with-default-env:0|1> args...  (started under env -i; never prints keys)
bin=$1 home=$2 withenv=$3; shift 3
export HOME=<scratch>/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home
if [ "$withenv" = 1 ]; then
  export ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME HOME=/Users/Neo /Users/Neo/.local/bin/corbanu vault auth-helper provider/zai_api_key)"
fi
cd <scratch>/work
exec <scratch>/bin/corbanu-$bin "$@"
