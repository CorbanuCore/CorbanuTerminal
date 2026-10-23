S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; L=/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-indep-accept-20261010/scripts/dev/corbanu-launcher.sh
run() { label="$1"; shift
  out=$(cd $S/launcher/ws && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_LAUNCHER_WORK_DIR=$S/launcher/work CORBANU_LAUNCHER_WORKSPACE_DIR=$S/launcher/ws "$@" sh $L exec --skip-git-repo-check "Reply with exactly the word pong and nothing else." 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   caller env: $*" | sed "s#$S#\$S#g"
  printf '%s\n' "$out" | grep -E -i '^pong|error|warning: (CORBANU|PFTERMINAL|CODEX)' | grep -v call_metrics | sort -u | sed "s#$S#\$S#g; s/^/   | /"; }
echo "coordinator home (activate.sh stand-in) = \$S/launcher/coordhome holds a REAL OpenAI API key; homeB = \$S/s01/hB holds a FAKE OpenAI key"
run "R1 launcher, no caller home -> coordinator home (real key)"
run "R2 launcher, caller CORBANU_HOME=homeB -> must use homeB (401)" CORBANU_HOME=$S/s01/hB
run "R3 launcher, caller CODEX_HOME=homeB only -> must use homeB (401)" CODEX_HOME=$S/s01/hB
run "R4 launcher, tool-shell shape: CODEX_HOME=coordhome + CORBANU_HOME=homeB -> homeB (401) + one warning" CODEX_HOME=$S/launcher/coordhome CORBANU_HOME=$S/s01/hB
