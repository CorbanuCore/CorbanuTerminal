S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch
run() { w=$1; label="$2"; shift 2; out=$(env -i HOME=$S/inst/fakehome PATH=/usr/bin:/bin "$@" $S/inst/bin/$w 2>&1); echo "== [$w] $label"; echo "   caller env: $*" | sed "s#$S#\$S#g"; echo "   wrapper handed over: $out" | sed "s#$S#\$S#g"; }
for w in corbanu corbanu-debug; do
run $w "W1 no caller home"
run $w "W2 caller CORBANU_HOME=homeB" CORBANU_HOME=$S/s01/hB
run $w "W3 caller CODEX_HOME=homeB" CODEX_HOME=$S/s01/hB
run $w "W4 caller PFTERMINAL_HOME=homeP" PFTERMINAL_HOME=$S/s01/hP
run $w "W5 caller CORBANU_HOME=homeA CODEX_HOME=homeB" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hB
done
run corbanu-debug "W6 caller CORBANU_DEBUG_HOME=homeD" CORBANU_DEBUG_HOME=$S/s01/hD
run corbanu-debug "W7 caller CORBANU_DEBUG_HOME=homeD + CORBANU_HOME=homeB" CORBANU_DEBUG_HOME=$S/s01/hD CORBANU_HOME=$S/s01/hB
