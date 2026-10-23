S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=${B:-$S/bin/corbanu-main}
run() { # label, env assignments...
  label="$1"; shift
  out=$(env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 "$@" $B doctor --json 2>$S/s01/err.txt)
  home=$(printf '%s' "$out" | python3 -c 'import json,sys,re; s=sys.stdin.read(); m=re.search(r"\"sqlite home\": \"([^\"]*)\"", s); print(m.group(1) if m else "?")')
  nwarn=$(grep -c -i 'warn' $S/s01/err.txt)
  echo "== $label"; echo "   env: $*" | sed "s#$S#\$S#g"; echo "   home used: ${home#$S/}"; echo "   stderr warnings: $nwarn"; sed "s#$S#\$S#g; s/^/   | /" $S/s01/err.txt
}
run "1 CORBANU_HOME only" CORBANU_HOME=$S/s01/hA
run "2 CODEX_HOME only" CODEX_HOME=$S/s01/hB
run "3 CORBANU_HOME=A CODEX_HOME=B" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hB
run "4 PFTERMINAL_HOME=P CODEX_HOME=B" PFTERMINAL_HOME=$S/s01/hP CODEX_HOME=$S/s01/hB
run "5 CORBANU_HOME=A PFTERMINAL_HOME=P" CORBANU_HOME=$S/s01/hA PFTERMINAL_HOME=$S/s01/hP
run "6 all three differ" CORBANU_HOME=$S/s01/hA PFTERMINAL_HOME=$S/s01/hP CODEX_HOME=$S/s01/hB
run "7 same path, trailing slash" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hA/
run "8 same path via symlink" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hA-link
run "9 CODEX_HOME names missing dir" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/does-not-exist
