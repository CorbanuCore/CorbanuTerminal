#!/bin/bash
S=<scratch>
for v in before after; do
  for case in "CORBANU_HOME=$S/homeB" "CORBANU_HOME=$S/homeB CODEX_HOME=$S/homeD" "CORBANU_DEBUG_HOME=$S/homeD CORBANU_HOME=$S/homeB" "(none)"; do
    vars=$case; [ "$case" = "(none)" ] && vars=
    home=$(cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 $vars $S/dbg-$v/corbanu-debug doctor --json 2>$S/dbg.err | grep -o '"daemon state dir": *"[^"]*"' | sed 's#.*: *"##; s#/app-server-daemon"##')
    echo "== $v corbanu-debug [${case//$S\//}]: home=${home//$S\//} warnings=$(grep -c '^warning:' $S/dbg.err)"
    grep '^warning:' $S/dbg.err | sed "s#$S/##g; s/^/   | /"
  done
done
