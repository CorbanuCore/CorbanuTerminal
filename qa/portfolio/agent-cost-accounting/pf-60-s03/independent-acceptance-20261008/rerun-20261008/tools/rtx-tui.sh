#!/bin/bash
# remote side. usage: rtx-tui.sh <home> <cwd> [args]; view-only: placeholder key, no model calls made from these sessions. BIN selects the binary.
R=<rtx-dir>; h=$1; cd "$2"; shift 2
export TZ=${TZ:-America/Phoenix} CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h ZAI_API_KEY=view-only-placeholder
exec ${BIN:-$R/corbanu-acct} "$@" 2>>$h/stderr.log
