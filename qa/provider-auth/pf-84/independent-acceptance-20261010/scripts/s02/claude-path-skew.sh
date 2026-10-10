S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; C=$S/s02/hC; HELPER=/Users/Neo/.local/bin/corbanu.bak-20261010
X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HELPER vault auth-helper claude-plan-test-token)" bash -c '
S='$S'; C='$C'
for pb in pathbin pathbin-pre; do
  out=$(cd $S/fakehome && env -i HOME=$S/fakehome PATH=$S/shim:$S/$pb:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $S/bin/corbanu-main exec --skip-git-repo-check -c provider_accounts.claude-plan="\"fake\"" "Reply with exactly the word pong and nothing else." 2>&1); rc=$?
  echo "== session selects claude-plan account fake; CLAUDE_CODE_OAUTH_TOKEN=<real default token>; PATH corbanu = $( [ $pb = pathbin ] && echo candidate 0fa45b54f0 || echo pre-PF-84 build 44b527f11f ) (exit $rc)"
  printf "%s\n" "$out" | grep -E "^pong|^ERROR" | sort -u | head -3 | sed "s/^/   | /"
done'
echo "   auth-command calls via pre-PF-84 corbanu: $(sed 's/^[0-9]* //' $S/pathbin-pre/calls.log | sort | uniq -c)"
