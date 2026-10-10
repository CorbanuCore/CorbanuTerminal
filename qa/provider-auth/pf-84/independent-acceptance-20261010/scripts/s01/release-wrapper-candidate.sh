S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch
run() { w=$1; label="$2"; shift 2; out=$(env -i HOME=$S/inst/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 "$@" $S/inst/bin/$w doctor --json 2>$S/s01/err.txt); home=$(printf '%s' "$out" | python3 -c 'import sys,re; s=sys.stdin.read(); m=re.search(r"\"sqlite home\": \"([^\"]*)\"", s); print(m.group(1) if m else "?")')
 echo "== [$w -> candidate binary] $label"; echo "   caller env: $*" | sed "s#$S#\$S#g"; echo "   home used: ${home#$S/}; warnings: $(grep -c -i warn $S/s01/err.txt)"; sed "s#$S#\$S#g; s/^/   | /" $S/s01/err.txt; }
for w in corbanu corbanu-debug; do
run $w "V1 no caller home"; run $w "V2 caller CORBANU_HOME=homeB" CORBANU_HOME=$S/s01/hB; run $w "V3 caller CODEX_HOME=homeB" CODEX_HOME=$S/s01/hB; done
run corbanu "V4 caller CORBANU_HOME=homeA CODEX_HOME=homeB" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hB
run corbanu-debug "V4 caller CORBANU_HOME=homeA CODEX_HOME=homeB" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hB
run corbanu-debug "V5 caller CORBANU_DEBUG_HOME=homeD" CORBANU_DEBUG_HOME=$S/s01/hD
run corbanu-debug "V6 caller CORBANU_DEBUG_HOME=homeD + CORBANU_HOME=homeB" CORBANU_DEBUG_HOME=$S/s01/hD CORBANU_HOME=$S/s01/hB
