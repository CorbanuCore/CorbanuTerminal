# Re-check S02-3b (#414: Claude Plan token via a PATH corbanu of another version) and S02-9b (#415: flag off),
# exactly as the 2026-10-10 run did, plus S03 `exec --account` on claude-plan.
# Run as: X="$(helper claude-plan-test-token)" bash recheck-claude.sh   (X = real default Claude token, never printed)
. "$(dirname "$0")/../env.sh"
mkdir -p $S/shim $S/pathbin $S/pathbin-pre
printf '#!/bin/sh\necho "$(date +%%s) security $*" >> %s/shim/security.log\nexit 44\n' $S > $S/shim/security; : > $S/shim/security.log
printf '#!/bin/sh\necho "$(date +%%s) argv=[$*] CORBANU_PROVIDER_ACCOUNT=${CORBANU_PROVIDER_ACCOUNT-<unset>}" >> %s/pathbin/calls.log\nexec %s "$@"\n' $S $B > $S/pathbin/corbanu
printf '#!/bin/sh\necho "$(date +%%s) argv=[$*] CORBANU_PROVIDER_ACCOUNT=${CORBANU_PROVIDER_ACCOUNT-<unset>}" >> %s/pathbin-pre/calls.log\nexec %s "$@"\n' $S $PRE > $S/pathbin-pre/corbanu
chmod +x $S/shim/security $S/pathbin/corbanu $S/pathbin-pre/corbanu; : > $S/pathbin/calls.log; : > $S/pathbin-pre/calls.log
h() { python3 -c "import hashlib,sys; d=sys.stdin.buffer.read().strip(); print('sha12='+hashlib.sha256(d).hexdigest()[:12] if d else 'EMPTY')"; }

C=$S/homes/hC; mkdir -p $C
printf 'model = "claude-opus-5-5-plan"\nmodel_provider = "claude-plan"\nsuppress_unstable_features_warning = true\n\n[features]\nnamed_accounts = true\n' > $C/config.toml
printf 'sk-ant-oat01-fake-%s' "$(openssl rand -hex 16)" > $S/claude-fake-token.txt
addval $C "$(cat $S/claude-fake-token.txt)" claude-plan fake --kind claude-token >/dev/null
addreal claude-plan-test-token $C claude-plan real --kind claude-token >/dev/null
mkdir -p $S/claudecfg; printf '{"claudeAiOauth":{"accessToken":"%s%s","expiresAt":4102444800000}}' sk-ant- oat01-cfgdir-canary-s03 > $S/claudecfg/.credentials.json
cx $C account add claude-plan cfgacct --kind claude-config-dir --value $S/claudecfg >/dev/null 2>&1
echo "home hC accounts:"; cx $C account list | sed 's/^/  /'

echo; echo "### S02-3b re-check (#414): session selects claude-plan account fake; real default token in CLAUDE_CODE_OAUTH_TOKEN"
for pb in pathbin pathbin-pre; do
  out=$(cd $S/work && env -i HOME=$S/fakehome PATH=$S/shim:$S/$pb:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $B exec --skip-git-repo-check -c provider_accounts.claude-plan='"fake"' "$PROMPT" 2>&1); rc=$?
  echo "== PATH corbanu = $( [ $pb = pathbin ] && echo "candidate (origin/main)" || echo "pre-PF-84 build 44b527f11f" ) (exit $rc)"
  printf "%s\n" "$out" | grep -E "^pong|^ERROR|^Error" | sort -u | head -3 | sed "s/^/   | /"
done
echo "   token-command calls via the PATH candidate shim:  $(sed 's/^[0-9]* //' $S/pathbin/calls.log | sort | uniq -c | tr -s ' ')"
echo "   token-command calls via the PATH pre-PF-84 shim:  $(sed 's/^[0-9]* //' $S/pathbin-pre/calls.log | sort | uniq -c | tr -s ' ')"
: > $S/pathbin/calls.log; : > $S/pathbin-pre/calls.log

echo; echo "### S03 exec --account on claude-plan (PATH corbanu = pre-PF-84 build, real default token in env)"
for a in real fake ghost cfgacct; do
  out=$(cd $S/work && env -i HOME=$S/fakehome PATH=$S/shim:$S/pathbin-pre:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $B exec --skip-git-repo-check --account $a "$PROMPT" 2>&1); rc=$?
  echo "== --account $a (exit $rc)"; printf "%s\n" "$out" | grep -E "^pong|^ERROR|^Error" | sort -u | head -3 | sed "s/^/   | /"
done
echo "   token-command calls via the PATH pre-PF-84 shim: $(wc -l < $S/pathbin-pre/calls.log | tr -d ' ')"

echo; echo "### S02-9b re-check (#415): internal-claude-oauth-token, flag on vs off"
run() { local label="$1"; shift; local o rc; o=$(env -i HOME=$S/fakehome PATH=$S/shim:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C "$@" 2>$S/e.txt); rc=$?; echo "$label: exit=$rc stdout=$(printf '%s' "$o" | h) stderr=$(head -c 260 $S/e.txt | tr '\n' ' ')"; }
echo "expected: fake token $(h < $S/claude-fake-token.txt); cfgacct $(printf %s%s sk-ant- oat01-cfgdir-canary-s03 | h); env canary $(printf envtok-canary-s03 | h)"
echo "-- flag ON (config.toml named_accounts = true)"
run "--account fake" $B internal-claude-oauth-token --account fake
run "--account cfgacct" $B internal-claude-oauth-token --account cfgacct
run "--account ghost" $B internal-claude-oauth-token --account ghost
run "CORBANU_PROVIDER_ACCOUNT=fake" env CORBANU_PROVIDER_ACCOUNT=fake $B internal-claude-oauth-token
cp $C/config.toml $C/config.toml.on; printf 'model = "claude-opus-5-5-plan"\nmodel_provider = "claude-plan"\n\n[features]\nnamed_accounts = false\n' > $C/config.toml
echo "-- flag OFF (config.toml named_accounts = false)  [#415]"
run "--account fake" $B internal-claude-oauth-token --account fake
run "--account cfgacct" $B internal-claude-oauth-token --account cfgacct
run "CORBANU_PROVIDER_ACCOUNT=fake" env CORBANU_PROVIDER_ACCOUNT=fake $B internal-claude-oauth-token
run "CORBANU_PROVIDER_ACCOUNT=fake + env token" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 CORBANU_PROVIDER_ACCOUNT=fake $B internal-claude-oauth-token
run "--account fake + env token" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 $B internal-claude-oauth-token --account fake
out=$(cd $S/work && env -i HOME=$S/fakehome PATH=$S/shim:$S/pathbin:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$C CLAUDE_CODE_OAUTH_TOKEN="$X" $B exec --skip-git-repo-check -c provider_accounts.claude-plan='"fake"' "$PROMPT" 2>&1); rc=$?
echo "flag OFF exec with provider_accounts.claude-plan=fake and the real default token in env (exit $rc):"; printf "%s\n" "$out" | grep -E "^pong|^ERROR|^Error|ignored" | sort -u | head -3 | sed "s/^/   | /"
cp $C/config.toml.on $C/config.toml
echo "-- env exception stays default-only (flag on)"
run "no account + env token" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 $B internal-claude-oauth-token
run "CORBANU_PROVIDER_ACCOUNT=fake + env token (candidate)" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 CORBANU_PROVIDER_ACCOUNT=fake $B internal-claude-oauth-token
run "CORBANU_PROVIDER_ACCOUNT=fake + env token (pre-PF-84 build)" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 CORBANU_PROVIDER_ACCOUNT=fake $PRE internal-claude-oauth-token
run "--account fake + env token (pre-PF-84 build)" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-s03 $PRE internal-claude-oauth-token --account fake
echo "security(1) CLI shim calls: $(wc -l < $S/shim/security.log | tr -d ' ')"
