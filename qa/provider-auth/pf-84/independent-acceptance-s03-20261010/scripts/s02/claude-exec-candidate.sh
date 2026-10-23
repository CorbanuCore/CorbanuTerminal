# S03 exec --account on claude-plan with the candidate first on PATH (real default token in env).
# Run as: X="$(helper claude-plan-test-token)" bash claude-exec-candidate.sh   (after recheck-claude.sh)
. "$(dirname "$0")/../env.sh"; C=$S/homes/hC; : > $S/pathbin/calls.log
echo; echo "### S03 exec --account on claude-plan (PATH corbanu = candidate, real default token in env)"
sid() { printf '%s\n' "$1" | grep -E '^session id:' | head -1 | sed 's/^session id: //'; }
for a in real fake ghost cfgacct default; do
  out=$(cd $S/work && env -i HOME=$S/fakehome PATH=$S/shim:$S/pathbin:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $B exec --skip-git-repo-check --account $a "$PROMPT" 2>&1); rc=$?
  [ $a = fake ] && SF=$(sid "$out")
  echo "== --account $a (exit $rc)"; printf "%s\n" "$out" | grep -E "^pong|^ERROR|^Error" | sort -u | head -3 | sed "s/^/   | /"
done
out=$(cd $S/work && env -i HOME=$S/fakehome PATH=$S/shim:$S/pathbin:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $B exec --skip-git-repo-check resume $SF "$PROMPT" 2>&1); rc=$?
echo "== resume the --account fake session without --account (exit $rc)"; printf "%s\n" "$out" | grep -E "^pong|^ERROR|^Error" | sort -u | head -3 | sed "s/^/   | /"
echo "   token-command argv seen by the PATH shim:"; sed 's/^[0-9]* //' $S/pathbin/calls.log | sort | uniq -c | sed 's/^/     /'
