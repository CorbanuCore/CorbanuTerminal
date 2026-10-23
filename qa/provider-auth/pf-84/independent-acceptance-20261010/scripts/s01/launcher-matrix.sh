S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; L=/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-indep-accept-20261010/scripts/dev/corbanu-launcher.sh
run() {
  label="$1"; shift
  out=$(cd $S/launcher/ws && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_LAUNCHER_WORK_DIR=$S/launcher/work CORBANU_LAUNCHER_WORKSPACE_DIR=$S/launcher/ws "$@" sh $L doctor --json 2>$S/s01/err.txt)
  home=$(printf '%s' "$out" | python3 -c 'import sys,re; s=sys.stdin.read(); m=re.search(r"\"sqlite home\": \"([^\"]*)\"", s); print(m.group(1) if m else "?")')
  echo "== $label"; echo "   caller env: $*" | sed "s#$S#\$S#g"; echo "   home used: ${home#$S/}"; echo "   stderr warnings: $(grep -c -i warn $S/s01/err.txt)"; sed "s#$S#\$S#g; s/^/   | /" $S/s01/err.txt
}
echo "activate.sh stand-in exports CORBANU_HOME=CODEX_HOME=\$S/launcher/coordhome unconditionally"
run "L1 no caller home (coordinator default)"
run "L2 caller CORBANU_HOME=homeB" CORBANU_HOME=$S/s01/hB
run "L3 caller CODEX_HOME=homeB only" CODEX_HOME=$S/s01/hB
run "L4 caller PFTERMINAL_HOME=homeP only" PFTERMINAL_HOME=$S/s01/hP
run "L5 caller CORBANU_HOME=homeA CODEX_HOME=homeB" CORBANU_HOME=$S/s01/hA CODEX_HOME=$S/s01/hB
run "L6 tool-shell case: CODEX_HOME=coordhome + CORBANU_HOME=homeB" CODEX_HOME=$S/launcher/coordhome CORBANU_HOME=$S/s01/hB
run "L7 inherited CORBANU_HOME=coordhome, worker sets only CODEX_HOME=homeB" CORBANU_HOME=$S/launcher/coordhome CODEX_HOME=$S/s01/hB
