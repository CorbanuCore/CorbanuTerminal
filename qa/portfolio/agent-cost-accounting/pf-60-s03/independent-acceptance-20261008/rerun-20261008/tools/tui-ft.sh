#!/bin/bash
# usage: tui-ft.sh <home> <cwd> <faketime> [corbanu args...]; macOS view-only TUI on a faked clock (placeholder key, no model calls).
# Prefix assignments go straight onto the binary (no /usr/bin/env, which would strip DYLD_*).
S=<scratch>
B=<corbanu-root>/.codex-work/targets/pf60-s03-indep-rerun-20261008/debug/corbanu
h=$1; cd "$2"; ft=$3; shift 3
TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@$ft" DYLD_FORCE_FLAT_NAMESPACE=1 DYLD_INSERT_LIBRARIES=$S/libfaketime/src/libfaketime.1.dylib \
  CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h ZAI_API_KEY=view-only-placeholder \
  exec $B "$@" 2>>"$h/stderr.log"
