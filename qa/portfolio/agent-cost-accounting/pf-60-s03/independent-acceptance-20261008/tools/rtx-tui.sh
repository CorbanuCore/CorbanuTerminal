#!/bin/bash
# usage: tui.sh <home> <cwd> [args]; view-only: placeholder key, no model calls made from these sessions
R=$HOME/corbanu-rtx/pf60-indep-20261008; h=$1; cd "$2"; shift 2
export TZ=${TZ:-America/Phoenix} CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h ZAI_API_KEY=view-only-placeholder
exec $R/corbanu-acct "$@" 2>>$h/stderr.log
